//! The sandbox guest (PS-A-03): `delulu __guest`, the child that runs a program while holding no
//! authority of its own.
//!
//! The host sends one [`Hello`] frame — the program, the hash it must match, the seed and the clock
//! — and then answers the guest's requests on `delulu-sandbox-channel/2` until the guest says it is
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

/// The guest argument that means "your channel is your standard input and output" (PS-B-03): a guest
/// started under a separate identity cannot reach a channel by name, so it is born holding one. On
/// Linux (PS-B-03b) the channel is one socket, inherited as standard input.
#[cfg(any(windows, target_os = "linux"))]
pub const STDIO_FLAG: &str = "--stdio";

/// The first byte a Linux `--stdio` guest writes, before it reads anything. The host waits for it
/// before committing to the launch: a stranger to the operator's account can fail to LOAD — a library
/// under a directory only the operator may walk — and without this byte that death would surface as a
/// run that failed, instead of a launch the host could still make the ordinary way.
#[cfg(target_os = "linux")]
pub const STDIO_READY: u8 = 0x06;

/// Run as the guest. Returns the process exit status.
pub fn run_guest(args: &[String]) -> i32 {
    #[cfg(any(windows, target_os = "linux"))]
    if args.first().map(String::as_str) == Some(STDIO_FLAG) {
        let mut conn = match stdio_channel() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("error: the sandbox guest cannot open its channel: {e}");
                return 2;
            }
        };
        if conn.set_read_timeout(Some(CHANNEL_DEADLINE)).is_err() {
            eprintln!("error: the sandbox guest cannot set its channel deadline — refusing to run unbounded");
            return 2;
        }
        #[cfg(target_os = "linux")]
        if io::Write::write_all(&mut conn, &[STDIO_READY]).is_err() {
            eprintln!("error: the sandbox guest cannot reach its host over its channel");
            return 2;
        }
        return serve_as_guest(conn, None);
    }
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
    serve_as_guest(conn, Some(&dir))
}

/// The channel of a guest started with [`STDIO_FLAG`]: the pipes it inherited as standard input and
/// output.
///
/// From here on standard output IS the channel, so its slot is pointed at standard error before
/// anything else can print: a stray `println!` then lands on the operator's terminal instead of
/// inside a frame, where it would desynchronize the conversation. (Rust's standard streams look the
/// handle up on every write, which is what makes redirecting the slot sufficient.)
///
/// Both are handles on one end of a duplex pipe, read directly; the deadline is kept by a watchdog
/// that ends this guest if its host falls silent (`pipe_channel.rs`).
#[cfg(windows)]
fn stdio_channel() -> Result<crate::pipe_channel::GuestChannel<std::fs::File, std::fs::File>, String> {
    use std::os::windows::io::FromRawHandle as _;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::System::Console::{
        GetStdHandle, SetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };
    // SAFETY: the two handles are this process's own standard handles, taken over exactly once here
    // (their slots are re-pointed before the `File`s own them, so nothing else closes them).
    unsafe {
        let (input, output) = (GetStdHandle(STD_INPUT_HANDLE), GetStdHandle(STD_OUTPUT_HANDLE));
        if input.is_null() || output.is_null() || input == INVALID_HANDLE_VALUE || output == INVALID_HANDLE_VALUE {
            return Err("it was started without the pipes a `--stdio` guest is given".into());
        }
        SetStdHandle(STD_OUTPUT_HANDLE, GetStdHandle(STD_ERROR_HANDLE));
        SetStdHandle(STD_INPUT_HANDLE, std::ptr::null_mut());
        let reader = std::fs::File::from_raw_handle(input as _);
        let writer = std::fs::File::from_raw_handle(output as _);
        crate::pipe_channel::GuestChannel::new(reader, writer, || {
            eprintln!("error: the sandbox guest heard nothing from its host within {CHANNEL_DEADLINE:?} — ending");
            std::process::exit(2);
        })
        .map_err(|e| e.to_string())
    }
}

/// The channel of a Linux guest started with [`STDIO_FLAG`]: the socket it inherited as standard
/// input. Standard output was pointed at nothing by the host, so a stray `println!` cannot reach it.
#[cfg(target_os = "linux")]
fn stdio_channel() -> Result<std::os::unix::net::UnixStream, String> {
    use std::os::fd::FromRawFd as _;
    // SAFETY: descriptor 0 is taken over exactly once, here; nothing else in the guest reads stdin.
    let conn = unsafe { std::os::unix::net::UnixStream::from_raw_fd(0) };
    // A socket answers this; the null device a plain launch gives a guest does not.
    conn.local_addr().map_err(|_| "it was started without the socket a `--stdio` guest is given".to_string())?;
    Ok(conn)
}

/// Everything a guest does once it holds a channel, however it came by it.
fn serve_as_guest<C: std::io::Read + std::io::Write + 'static>(mut conn: C, dir: Option<&std::path::Path>) -> i32 {
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
    // The filesystem first, because it is the one a guest needs none of — every read and write the
    // program asks for is performed by the HOST — and because the syscall filter below says nothing
    // about WHICH files a permitted syscall may reach.
    // What the guest applies to itself, for the host's report (RW 4.23).
    let mut own: Vec<&'static str> = Vec::new();
    #[cfg(target_os = "linux")]
    match crate::jail::confine_filesystem(dir) {
        Some(applied) => {
            eprintln!("sandbox: the guest narrowed its own view — {}", applied.join("; "));
            own.extend(applied);
        }
        // Never silence this: a host whose kernel has no Landlock must not read as one that applied
        // it. The run continues — the channel, the rlimits and the syscall filter are untouched —
        // but nothing here is claimed. (A guest born holding its socket has no channel directory;
        // the `None` arm covers the kernel, not the directory.)
        None => eprintln!("sandbox: this kernel has no Landlock — the guest's view of the filesystem was NOT narrowed"),
    }
    #[cfg(not(target_os = "linux"))]
    let _ = dir;

    // Fail closed — a guest that cannot be locked down does not run the program.
    match crate::jail::lock_down_self() {
        Ok(applied) if !applied.is_empty() => {
            eprintln!("sandbox: the guest locked itself down — {}", applied.join("; "));
            own.extend(applied);
        }
        Ok(_) => {}
        Err(why) => {
            eprintln!("error: the sandbox guest could not lock itself down ({why}) — nothing ran");
            return 2;
        }
    }

    let sink = Rc::new(ChannelSink::new(conn));
    // Told to the host now — after the lock-down, before the program's first line — so the report
    // counts what this guest really applied, and the words come from the toolchain, not the program.
    if let Err(e) = sink.confined(&own) {
        eprintln!("error: the host would not take this guest's confinement report ({e}) — nothing ran");
        return 2;
    }
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
/// request kind on `delulu-sandbox-channel/2`, and a handle cannot stand in for a thread or a
/// library. They arrive with the rest of PS-A.
///
/// `Http` joined at PS-B-02, and needed no new request kind to do it: `get` is an ordinary
/// capability method, the host performs it with the egress client every L0 run uses, and what
/// crosses back is a `Str` or a `NetErr` — plain values. The guest never holds a socket, an address or
/// a resolver; its own jail still denies it every network call but the channel. Until the client
/// existed this refusal was the honest answer, because the host had nothing to perform the request
/// with; now the refusal would be the dishonest one.
const CARRIED: &[delulu_check::ResourceKind] = &[
    delulu_check::ResourceKind::Console,
    delulu_check::ResourceKind::FsRead,
    delulu_check::ResourceKind::FsWrite,
    delulu_check::ResourceKind::Clock,
    delulu_check::ResourceKind::Rand,
    delulu_check::ResourceKind::Http,
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

/// The `run` flags a sandboxed run actually applies. Every other flag `run` knows is refused under
/// `--sandbox`, before anything runs.
///
/// Found during PS-B-05 by following the Survey's blast radius into this file: the sandboxed path
/// read six of `run`'s options and silently dropped the rest. `--sandbox --lease TOKEN` never redeemed
/// the token — the guest ran holding nothing, and the resulting DL0703 told a lease holder to "pass
/// `--grant console`", which is exactly how a holder would step outside its delegation.
/// `--broker daemon` got embedded custody. `--locked` and `--deny-advisories` stopped gating, so a CI
/// line that gained `--sandbox` lost its supply-chain checks without a word. It is the rule PS-A-10
/// wrote for the opposite direction: a flag that is silently ignored reads exactly like a flag that
/// was applied. An ALLOWLIST rather than a list of refusals, because every flag added to `run` after
/// a refusal list is written falls through it (the lease-run device grants were lost that way, 10g).
const APPLIED_UNDER_SANDBOX: &[&str] =
    &["--sandbox", "--sandbox-profile", "--mode", "--limits", "--grant", "--json", "--report-out", "--no-prompt"];

fn refuse_what_the_guest_does_not_apply(opts: &crate::cli::Opts) -> Option<i32> {
    // The ordinary path's own two refusals first. They sat in `cmd_run_inner`, AFTER the dispatch to
    // this function, so under `--sandbox` a misspelled flag (`--totally-bogus`), a flag with no value
    // and a second file were all accepted in silence while the same command line without
    // `--sandbox` refused them (P1-F3). Same functions, same words, so the two paths cannot drift.
    if let Some(code) = crate::cli::refuse_extra_positionals("run", opts) {
        return Some(code);
    }
    if let Some(code) = crate::cli::refuse_unknown_flags("run", opts) {
        return Some(code);
    }
    let dropped: Vec<&str> =
        opts.seen_flags.iter().map(String::as_str).filter(|f| !APPLIED_UNDER_SANDBOX.contains(f)).collect();
    if dropped.is_empty() {
        return None;
    }
    eprintln!(
        "error: a sandboxed run does not apply {} yet. Nothing ran, because a flag that is silently \
         ignored reads exactly like a flag that was applied. Drop it, or run without `--sandbox`.",
        dropped.iter().map(|f| format!("`{f}`")).collect::<Vec<_>>().join(", ")
    );
    if dropped.iter().any(|f| *f == "--lease" || *f == "--broker") {
        eprintln!(
            "  `--lease` and `--broker`: a guest's effects are performed by the host in embedded \
             custody, and the channel does not carry broker custody yet. A lease runs without \
             `--sandbox`, under exactly its delegated authority and budget."
        );
    }
    Some(2)
}

/// `delulu run <file> --sandbox …` (PS-A-07): run the program as a jailed guest.
///
/// Refusals come first and are explicit, because the one thing a sandbox flag must never do is
/// quietly not apply: an unknown profile, limits that cannot be read, a surface the channel does not
/// carry, or a host with no jail at all are each refused with a reason and a way forward.
pub fn cmd_run_sandboxed(file: Option<&str>, opts: &crate::cli::Opts, _rest: &[String]) -> i32 {
    if let Some(code) = refuse_what_the_guest_does_not_apply(opts) {
        return code;
    }
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
    let served = spawn_and_serve_with(&program, root, 0xDE1, None, limits, profile, opts.report_out.as_deref());
    let egress = delulu_runtime::egress::take_log();
    if !opts.json {
        crate::run_cmd::print_egress_notes(&egress);
    }
    match served {
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

/// The host half: launch a guest, hand it the program, and serve its effects under `root`, with the
/// default limits and the `contained` profile.
///
/// The guest is jailed by `spawn_and_serve_with` (PS-A: a Job Object, Seatbelt, or rlimits with
/// Landlock and seccomp, per platform) and holds no capability of its own — it can only ask. (This
/// comment said the guest had "no OS jail yet, which PS-A2 adds" until 2026-09-25, a phase after
/// PS-A2 added it.)
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
    let (mut child, jail, applied) = match launch(&exe, &dir, limits, false) {
        Ok(l) => l,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dir);
            return Err(e);
        }
    };
    if applied.is_empty() {
        // Never claim a boundary that was not applied: PS-0-04's rule, in the place it matters most.
        eprintln!("sandbox: no OS jail on this host yet — the guest still holds no authority of its own");
    } else {
        eprintln!("sandbox: the guest is confined — {}", applied.join("; "));
    }
    let _ = &jail;
    // PS-A-08: the launch is recorded before the guest is told what to run, so the evidence exists
    // even if everything after it fails.
    let policy = crate::policy::SandboxPolicy::derive(
        if applied.is_empty() { 0 } else { 1 },
        profile,
        Some(limits),
        crate::policy::Mode::Strict,
    );
    let backend = if applied.is_empty() { "inproc" } else { "process" };
    audit_sandbox(
        "sandbox-launch",
        "allow",
        Some(blake3::hash(program.as_bytes()).to_hex().to_string()),
        Some(policy.to_json_with(backend, &applied, &[], 0)),
    );

    let mut evidence = Evidence::default();
    let served = converse(&mut child, &dir, program, root, seed, fixed_clock_ms, &mut evidence);
    let denied = evidence.denied;
    // RW 4.23: the layers the guest applied to itself count as applied, in the report and in the
    // chain — they were in force before the program's first line, and the host checked the words.
    let mut applied = applied;
    for w in evidence.own {
        if !applied.contains(&w) {
            applied.push(w);
        }
    }
    // Whatever happened on the channel, the child is not left running and the channel is removed.
    let status = child.wait();
    let _ = std::fs::remove_dir_all(&dir);
    // The run report (D-V2-21): written by the RUNTIME to the file the operator named, never on the
    // program's own output, which the program could forge. `granted` and `host_guarantees` carry
    // what this host actually applied, so a report never claims a boundary that was not there.
    // PS-A-08: the refusals as their own record, not only as a count on the death record. A run
    // report can be deleted; this is the copy an operator cannot quietly lose, and a guest that was
    // refused fifty things is the single most interesting fact about a program nobody wrote. It is
    // written only when there WAS a refusal, so an ordinary run does not grow the chain.
    if denied.1 > 0 {
        audit_sandbox(
            "channel-violation",
            "deny",
            Some(format!("{} refusal(s) on the channel", denied.1)),
            Some(serde_json::json!({
                "denied_total": denied.1,
                // The same bound the report keeps, for the same reason: a guest refused in a loop must
                // not be able to make the host write without limit.
                "denied": denied.0,
                "policy_hash": policy.hash(),
            })),
        );
    }
    // A ceiling that FIRED is a different fact from a program that failed, and only the OS knows which
    // happened. What can be said honestly is what the exit status says: on Unix a guest killed by a
    // signal names it (SIGXCPU is the processor-time ceiling, SIGKILL is the usual memory or
    // pdeathsig kill); on Windows the job's limits terminate the process and the code is what the OS
    // set. Nothing is inferred beyond that — the record says which status was observed, and does not
    // claim WHICH limit fired when the status cannot tell.
    if let Ok(st) = &status {
        if !st.success() {
            if let Some(why) = limit_kill_reason(st) {
                audit_sandbox(
                    "sandbox-limit",
                    "deny",
                    Some(why),
                    Some(serde_json::json!({
                        "limits": { "memory_bytes": limits.memory_bytes, "cpu_seconds": limits.cpu_seconds },
                        "policy_hash": policy.hash(),
                    })),
                );
            }
        }
    }
    // And the end of the guest's life, with the reason it ended: an exit, or a channel that failed.
    audit_sandbox(
        "sandbox-death",
        if matches!(&served, Ok(0)) { "allow" } else { "deny" },
        Some(match &served {
            Ok(code) => format!("exit {code}"),
            Err(e) => format!("channel failed: {e}"),
        }),
        // How many times the host said no, in the chain as well as the report: a run report can be
        // discarded, and the audit chain is the copy an operator cannot quietly lose.
        Some(serde_json::json!({ "denied_total": denied.1 })),
    );

    // PS-B-02: the guest's network requests were performed HERE, by the host, so their record is in
    // this process — the same record an L0 run reports, in the same shape. A snapshot: the caller
    // takes the log afterwards, to tell the operator on stderr.
    let egress = delulu_runtime::egress::snapshot();
    if let Some(path) = report_out {
        let exit = served.as_ref().copied().unwrap_or(1);
        let report = serde_json::json!({
            "command": "run",
            "schema": 1,
            "delulu_version": env!("CARGO_PKG_VERSION"),
            "diagnostics": [],
            "summary": { "errors": if exit == 0 { 0 } else { 1 }, "warnings": 0 },
            "sandbox": policy.to_json_with(backend, &applied, &denied.0, denied.1),
            "outcome": { "ran": true, "exit": exit },
            "egress": egress.to_json(),
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

/// A launched guest: PS-A's, reached through a named channel in its directory, or — on Windows since
/// PS-B-03 and on Linux since PS-B-03b — one started as a separate identity, holding its channel from
/// birth.
enum Guest {
    Plain(std::process::Child),
    #[cfg(any(windows, target_os = "linux"))]
    Contained(crate::identity::ContainedGuest),
}

impl Guest {
    fn try_wait(&mut self) -> io::Result<Option<std::process::ExitStatus>> {
        match self {
            Guest::Plain(c) => c.try_wait(),
            #[cfg(any(windows, target_os = "linux"))]
            Guest::Contained(c) => c.try_wait(),
        }
    }

    fn wait(&mut self) -> io::Result<std::process::ExitStatus> {
        match self {
            Guest::Plain(c) => c.wait(),
            #[cfg(any(windows, target_os = "linux"))]
            Guest::Contained(c) => c.wait(),
        }
    }

    fn kill(&mut self) -> io::Result<()> {
        match self {
            Guest::Plain(c) => c.kill(),
            #[cfg(any(windows, target_os = "linux"))]
            Guest::Contained(c) => c.kill(),
        }
    }

    /// Standard error, when the launch captured it (the probe does; a run shares the host's).
    fn take_stderr(&mut self) -> Option<Box<dyn io::Read>> {
        match self {
            Guest::Plain(c) => c.stderr.take().map(|e| Box::new(e) as Box<dyn io::Read>),
            #[cfg(any(windows, target_os = "linux"))]
            Guest::Contained(c) => c.stderr.take().map(|e| Box::new(e) as Box<dyn io::Read>),
        }
    }
}

/// Why the last launch could not give its guest a separate identity, for `doctor` to repeat. Read
/// from the attempt rather than guessed from the host's configuration.
static IDENTITY_REFUSAL: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

pub fn identity_refusal() -> Option<String> {
    IDENTITY_REFUSAL.lock().ok().and_then(|r| r.clone())
}

/// Say why the identity was not applied: always to `doctor`, and to the operator's terminal when this
/// is a run (a probe's caller reports it in its own words instead of an interleaved line).
#[cfg(any(windows, target_os = "linux"))]
fn identity_refused(why: String, quiet: bool) {
    if !quiet {
        eprintln!(
            "sandbox: the guest could not be given a separate identity on this host ({why}); it runs as \
             this OS user, under the jail below"
        );
    }
    if let Ok(mut r) = IDENTITY_REFUSAL.lock() {
        *r = Some(why);
    }
}

/// Wait for a Linux `--stdio` guest's first byte, which it sends once it has loaded and holds its
/// channel. A guest that dies first — a library the stranger may not read — is reported with what it
/// managed to say, so the launch can fall back instead of failing the run.
#[cfg(target_os = "linux")]
fn await_ready(g: &mut crate::identity::ContainedGuest) -> Result<(), String> {
    use std::io::Read as _;
    let Some(chan) = g.channel.as_mut() else { return Err("its channel was already taken".into()) };
    chan.set_read_timeout(Some(CONNECT_DEADLINE)).map_err(|e| format!("no deadline on its channel: {e}"))?;
    let mut first = [0u8; 1];
    let got = chan.read(&mut first);
    let _ = chan.set_read_timeout(None);
    match got {
        Ok(1) if first[0] == STDIO_READY => Ok(()),
        Ok(1) => Err("its first byte was not the one a guest sends".into()),
        Ok(_) => {
            let status = g.wait().map(|s| s.to_string()).unwrap_or_else(|e| e.to_string());
            let mut said = String::new();
            if let Some(mut e) = g.stderr.take() {
                let _ = e.read_to_string(&mut said);
            }
            let said = said.lines().next().map(|l| format!(": {l}")).unwrap_or_default();
            Err(format!("the guest exited before it was ready ({status}){said}"))
        }
        Err(e) => Err(format!("the guest was not ready within {CONNECT_DEADLINE:?}: {e}")),
    }
}

/// Start a guest under its jail and return it running, with what was applied.
///
/// The guest is first tried as a SEPARATE IDENTITY (`identity.rs`): on Windows a per-run AppContainer
/// with no capabilities, its channel on inherited pipes (PS-B-03); on Linux a subordinate uid in its
/// own user namespace, its channel an inherited socket (PS-B-03b). A host that cannot give it one
/// runs it as PS-A did, and says so on standard error — never silently: the identity is named among
/// the applied guarantees only when it was applied, so the run report cannot claim it either.
fn launch(
    exe: &std::path::Path,
    dir: &std::path::Path,
    limits: crate::jail::Limits,
    capture_stderr: bool,
) -> io::Result<(Guest, crate::jail::Jail, Vec<&'static str>)> {
    #[cfg(windows)]
    {
        let env: Vec<(String, String)> =
            LOADER_ENV.iter().filter_map(|k| std::env::var(k).ok().map(|v| (k.to_string(), v))).collect();
        let args: [&std::ffi::OsStr; 2] = [GUEST_SUBCOMMAND.as_ref(), STDIO_FLAG.as_ref()];
        match crate::identity::spawn_contained(&args, &env, capture_stderr) {
            Ok(mut g) => {
                // The same Job Object as a plain guest, applied while it is still suspended.
                let (jail, enforced) = crate::jail::confine(&g, limits);
                let mut applied: Vec<&'static str> = enforced.guarantees.to_vec();
                applied.push(crate::identity::GUARANTEE);
                if !g.resume() {
                    let _ = g.kill();
                    let _ = g.wait();
                    return Err(io::Error::other("the sandbox guest could not be started under its jail"));
                }
                return Ok((Guest::Contained(g), jail, applied));
            }
            Err(why) => identity_refused(why, capture_stderr),
        }
    }
    #[cfg(target_os = "linux")]
    {
        let env: Vec<(String, String)> =
            LOADER_ENV.iter().filter_map(|k| std::env::var(k).ok().map(|v| (k.to_string(), v))).collect();
        let args: [&std::ffi::OsStr; 2] = [GUEST_SUBCOMMAND.as_ref(), STDIO_FLAG.as_ref()];
        let mut hardened: Vec<&'static str> = Vec::new();
        // The jail's own pre-`exec` steps run after the identity change, inside `spawn_contained`.
        let spawned = crate::identity::spawn_contained(exe, &args, &env, capture_stderr, |cmd| {
            hardened = crate::jail::harden(cmd, limits);
        });
        match spawned.and_then(|mut g| match await_ready(&mut g) {
            Ok(()) => Ok(g),
            Err(why) => {
                let _ = g.kill();
                let _ = g.wait();
                Err(why)
            }
        }) {
            Ok(g) => {
                let (jail, enforced) = crate::jail::confine(&g, limits);
                let mut applied = hardened;
                applied.extend(enforced.guarantees.iter().copied());
                applied.push(crate::identity::GUARANTEE);
                return Ok((Guest::Contained(g), jail, applied));
            }
            Err(why) => identity_refused(why, capture_stderr),
        }
    }
    let (mut cmd, launched) = guest_command(exe, dir);
    if capture_stderr {
        cmd.stderr(std::process::Stdio::piped());
    }
    // Where the platform allows it, the limits are in force from the guest's first instruction.
    let mut applied = crate::jail::harden(&mut cmd, limits);
    applied.extend(launched);
    let mut child = cmd.spawn()?;
    // PS-A-04: the OS jail, applied before the guest has been told what to run — it is still waiting
    // for a hello at this point, so it has executed no program bytes yet.
    let (jail, enforced) = crate::jail::confine(&child, limits);
    applied.extend(enforced.guarantees.iter().copied());
    // Only now does the guest run: on Windows it was created suspended, so it meets its jail before
    // its first instruction rather than a moment after.
    if !crate::jail::resume(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(io::Error::other("the sandbox guest could not be started under its jail"));
    }
    Ok((Guest::Plain(child), jail, applied))
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
        // T14: the state directory holds the broker's key and the root policy; the guest never needs
        // it, so the profile refuses it even though every other read has to stay allowed.
        let unreadable: Vec<std::path::PathBuf> = crate::brokerd::resolve_state_dir(None).into_iter().collect();
        match crate::jail::seatbelt_launcher(exe, dir, &args, &unreadable) {
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

/// What a conversation leaves for the report, whichever way it ended.
#[derive(Default)]
struct Evidence {
    /// Every refusal the host gave (bounded), and how many there were.
    denied: (Vec<String>, u64),
    /// What the guest reported applying to itself before its program ran (RW 4.23).
    own: Vec<&'static str>,
}

/// Connect to the guest, tell it what to run, and serve it until it is done.
fn converse(
    child: &mut Guest,
    dir: &std::path::Path,
    program: &str,
    root: Rc<RootVal>,
    seed: u64,
    fixed_clock_ms: Option<i64>,
    // Filled in on EVERY path, including the failing ones: this phase has already lost a diagnosis
    // three CI runs in a row to a value that was only reported in the success branch.
    evidence: &mut Evidence,
) -> io::Result<i32> {
    let mut conn = open_channel(child, dir, CHANNEL_DEADLINE)?;
    let hello = Hello {
        version: CHANNEL_VERSION.to_string(),
        program: program.to_string(),
        hash: blake3::hash(program.as_bytes()).to_hex().to_string(),
        seed,
        fixed_clock_ms,
    };
    write_frame(&mut conn, &hello)?;
    let mut host = HostChannel::new(LocalSink).with_root(root);
    // The refusals come back with the exit code, because a report that lists only what was allowed
    // says nothing about what the program TRIED — which is the interesting half when the program is
    // one nobody wrote. They are read after `serve` returns, on both paths, so a guest that died
    // mid-conversation still reports what it had been refused up to then.
    let served = host.serve(&mut conn);
    // Read on both paths, before the result is returned: a guest that died mid-conversation still
    // reports what it had been refused up to then.
    let (list, total) = host.denied();
    evidence.denied = (list.to_vec(), total);
    evidence.own = host.self_applied().to_vec();
    served
}

/// Either kind of channel, as one reader-and-writer.
trait Channel: io::Read + io::Write {}
impl<T: io::Read + io::Write> Channel for T {}

/// The host's end of the guest's channel, with `deadline` on every read. A contained guest already
/// holds its pipes; a plain one is connected to by name within the connect deadline.
fn open_channel(child: &mut Guest, dir: &std::path::Path, deadline: std::time::Duration) -> io::Result<Box<dyn Channel>> {
    #[cfg(windows)]
    if let Guest::Contained(g) = child {
        let mut conn = g.channel.take().ok_or_else(|| io::Error::other("the contained guest's channel was already taken"))?;
        conn.set_read_timeout(Some(deadline))?;
        return Ok(Box::new(conn));
    }
    #[cfg(target_os = "linux")]
    if let Guest::Contained(g) = child {
        let conn = g.channel.take().ok_or_else(|| io::Error::other("the contained guest's channel was already taken"))?;
        conn.set_read_timeout(Some(deadline))?;
        return Ok(Box::new(conn));
    }
    let mut conn = connect_by_name(child, dir)?;
    conn.set_read_timeout(Some(deadline))?;
    Ok(Box::new(conn))
}

fn connect_by_name(child: &mut Guest, dir: &std::path::Path) -> io::Result<crate::broker_transport::Connection> {
    let deadline = std::time::Instant::now() + CONNECT_DEADLINE;
    let conn = loop {
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
    Ok(conn)
}

/// PS-A-07: ATTEMPT the L1 launcher, so `sandbox probe` and `sandbox status` answer from a launch
/// rather than from a sentence in the source.
///
/// The probe's rule is that every line is an attempt. This line was the exception: it said "the L1
/// guest launcher: not in this build" as a hard-coded `false`, and it went on saying it after PS-A
/// built the launcher — so `probe` reported L1 ABSENT on a host where `run --sandbox` works, and
/// `doctor` printed the same. A hard-coded verdict is not an attempt, and it was wrong in the safe
/// direction only by luck.
///
/// What is attempted is the launcher and nothing else: the jail is applied, a guest is spawned under
/// it, and the channel it opens is waited for. It deliberately does NOT run a program, because a run
/// writes `sandbox-launch` and `sandbox-death` records, and a diagnostic that appends to a
/// single-writer audit chain is the defect this phase already fixed once — `doctor` refused its own
/// machine after every sandboxed run became a writer.
///
/// The guest is killed and its channel directory removed on every path out.
pub fn attempt_launch() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| format!("this executable cannot be located: {e}"))?;
    let dir = std::env::temp_dir().join(format!("delulu-probe-{}-{}", std::process::id(), channel_tag()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("no channel directory: {e}"))?;
    use std::io::Read as _;
    let finish = |r: Result<String, String>, child: Option<&mut Guest>| {
        if let Some(c) = child {
            let _ = c.kill();
            let _ = c.wait();
        }
        let _ = std::fs::remove_dir_all(&dir);
        r
    };
    // Whatever the guest managed to say, since it is the only witness to its own death.
    let said = |child: &mut Guest, how: String| -> String {
        let Some(mut err) = child.take_stderr() else { return how };
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        let s = s.trim();
        if s.is_empty() { how } else { format!("{how}: {}", s.lines().next().unwrap_or(s)) }
    };

    // The probe's guest is killed on purpose, and a killed guest says so on its standard error. That
    // belongs in this function's answer, not on the operator's terminal, where "error: the sandbox
    // guest was started without a hello frame" from a successful PROBE reads as a broken host.
    let (mut child, jail, applied) = match launch(&exe, &dir, crate::jail::Limits::default(), true) {
        Ok(l) => l,
        Err(e) => return finish(Err(format!("the guest could not be started: {e}")), None),
    };
    let _ = &jail;
    let confined = |applied: &[&str]| {
        if applied.is_empty() {
            "a guest started and opened its channel; this host applied no OS boundary to it".to_string()
        } else {
            format!("a guest started under its jail and opened its channel ({})", applied.join("; "))
        }
    };

    // A contained guest (PS-B-03) holds its channel from birth, so "it opened its channel" is shown by
    // a conversation instead of a connection: a hello carrying a program that does nothing, answered
    // with its exit. Nothing is performed and nothing is recorded — the probe still writes no audit
    // record — but it proves more than the plain probe does: the guest loaded, ran under its identity,
    // and spoke the protocol both ways.
    #[cfg(any(windows, target_os = "linux"))]
    if matches!(child, Guest::Contained(_)) {
        let answered = (|| -> io::Result<i32> {
            let mut conn = open_channel(&mut child, &dir, std::time::Duration::from_secs(3))?;
            write_frame(
                &mut conn,
                &Hello {
                    version: CHANNEL_VERSION.to_string(),
                    program: PROBE_PROGRAM.to_string(),
                    hash: blake3::hash(PROBE_PROGRAM.as_bytes()).to_hex().to_string(),
                    seed: 0,
                    fixed_clock_ms: None,
                },
            )?;
            HostChannel::new(LocalSink).with_root(Rc::new(RootVal::default())).serve(&mut conn)
        })();
        return match answered {
            Ok(0) => finish(Ok(confined(&applied)), Some(&mut child)),
            Ok(code) => {
                let how = said(&mut child, format!("a contained guest ran the probe's empty program and exited {code}"));
                finish(Err(how), Some(&mut child))
            }
            Err(e) => {
                let how = said(&mut child, format!("a contained guest never answered on its channel ({e})"));
                finish(Err(how), Some(&mut child))
            }
        };
    }

    // A short deadline: this is a probe, and an unreachable guest is an answer, not something to wait
    // ten seconds for.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        if crate::broker_transport::connect(&dir).is_ok() {
            return finish(Ok(confined(&applied)), Some(&mut child));
        }
        if std::time::Instant::now() >= deadline {
            // Say WHY, as `converse` learned to: a guest killed by its own jail otherwise reads as an
            // unexplained timeout.
            let how = match child.try_wait().ok().flatten() {
                Some(status) => format!("the guest exited before opening its channel ({status})"),
                None => "the guest is running but never opened its channel".to_string(),
            };
            let how = said(&mut child, how);
            return finish(Err(how), Some(&mut child));
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// The program the probe hands a contained guest: it asks for nothing and does nothing.
#[cfg(any(windows, target_os = "linux"))]
const PROBE_PROGRAM: &str = "module probe\n\nfn main(root: Root) {\n}\n";

/// Did the OS kill this guest for exceeding a ceiling, and can we say WHICH?
///
/// Only what the exit status actually carries. On Unix a signal names itself, and `SIGXCPU` is
/// unambiguous — it exists for exactly this. `SIGKILL` is not: it is what the processor-time hard
/// limit escalates to, what `PR_SET_PDEATHSIG` sends when the host dies, and what an operator's
/// `kill -9` sends, so the record says that rather than picking one. Elsewhere `None`, because a
/// record that guesses which limit fired is worse than no record — it would be read as measurement.
fn limit_kill_reason(status: &std::process::ExitStatus) -> Option<String> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt as _;
        if let Some(sig) = status.signal() {
            return Some(match sig {
                libc::SIGXCPU => "killed by SIGXCPU — the processor-time ceiling fired".to_string(),
                libc::SIGKILL => {
                    "killed by SIGKILL — a hard ceiling, the host's death signal, or an operator; the \
                     status cannot tell which"
                        .to_string()
                }
                other => format!("killed by signal {other}"),
            });
        }
        None
    }
    #[cfg(not(unix))]
    {
        // Windows: the Job Object terminates the process when a limit is exceeded, and the code it
        // leaves is the OS's, not the program's. There is no status bit that says "a limit did this",
        // so nothing is claimed — the death record already carries the code, and `denied_total` and
        // the guarantees say what was in force.
        let _ = status;
        None
    }
}

/// PS-A-08: record a sandbox lifecycle event in the audit chain, when this machine has one.
///
/// The chain is the project's existing one — the same file, the same hashes, verified by the same
/// `audit verify` — because a sandbox with its own private log would be evidence nobody checks. Best
/// effort by design: a run must not fail because the machine has no broker state directory, and the
/// record carries only what was applied, never what was intended.
fn audit_sandbox(action: &str, decision: &str, target: Option<String>, authority: Option<serde_json::Value>) {
    let Some(state) = crate::brokerd::resolve_state_dir(None) else { return };
    if !state.join("audit").exists() {
        return;
    }
    let _ = append_audit(&state, action, decision, target, authority);
}

/// PS-B-06: the same append, for a record that MUST exist — a break-glass use, a refused ticket, a
/// change to the host policy. The chain is created if this machine has none (a break-glass record is
/// worth starting one for), and any failure is returned, so the caller can refuse to do what it
/// could not record.
pub(crate) fn audit_required(
    action: &str,
    decision: &str,
    target: Option<String>,
    authority: Option<serde_json::Value>,
) -> Result<(), String> {
    let state = crate::brokerd::resolve_state_dir(None).ok_or("no state directory (no HOME/USERPROFILE)")?;
    append_audit(&state, action, decision, target, authority)
}

fn append_audit(
    state: &std::path::Path,
    action: &str,
    decision: &str,
    target: Option<String>,
    authority: Option<serde_json::Value>,
) -> Result<(), String> {
    use delulu_broker::audit::{AuditEntry, AuditLog, AuditSink};
    let dir = state.join("audit");
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create the audit chain at `{}`: {e}", dir.display()))?;
    // ONE writer at a time. The chain is single-writer by design — the broker daemon owns it — and
    // making every sandboxed run a writer broke that immediately: two runs in parallel each read the
    // same head, and their records landed on ONE line, `}{` in the middle, which `audit verify` then
    // reported as malformed. Found locally, by `doctor` refusing its own machine.
    //
    // The lock is an atomic create: whoever makes the file owns the append, and the head is read
    // AFTER it is held, so no writer chains onto a head that another has already moved.
    let _lock = AppendLock::take(&dir).ok_or("the audit chain's append lock could not be taken")?;
    let mut log = AuditLog::open(&dir).map_err(|e| format!("the audit chain cannot be opened: {e:?}"))?;
    // Continue the chain's numbering: the last record's seq plus one, or 1 for an empty log.
    let seq = delulu_broker::audit::tail(&dir, 1).ok().and_then(|r| r.last().map(|x| x.seq + 1)).unwrap_or(1);
    let entry = AuditEntry {
        seq,
        ts: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or_default(),
        actor_node: None,
        action: action.to_string(),
        target,
        authority,
        span: None,
        decision: decision.to_string(),
    };
    log.append(entry).map(|_| ()).map_err(|e| format!("the audit record could not be written: {e:?}"))
}

/// Exclusive access to an audit directory for the length of one append.
///
/// `create_new` is the whole mechanism: it succeeds for exactly one process. A stale lock from a
/// killed run is taken over after a short wait rather than blocking for ever, because an audit record
/// is evidence and must not be able to hang a run.
struct AppendLock(std::path::PathBuf);

impl AppendLock {
    fn take(dir: &std::path::Path) -> Option<AppendLock> {
        let path = dir.join("append.lock");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Some(AppendLock(path)),
                Err(_) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                // A lock nobody released: take it over rather than lose the record entirely.
                Err(_) => {
                    let _ = std::fs::remove_file(&path);
                    return std::fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&path)
                        .ok()
                        .map(|_| AppendLock(path));
                }
            }
        }
    }
}

impl Drop for AppendLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
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
