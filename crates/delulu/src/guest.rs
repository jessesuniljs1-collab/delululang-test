//! The sandbox guest (PS-A-03): `delulu __guest`, the child that runs a program while holding no
//! authority of its own.
//!
//! The host opens with an [`Open`] frame — this run's generation, and no program. The guest locks
//! itself down and reports what it applied; only once the host has accepted that report
//! (`boundary.rs`, PS-E-01) does it send the [`Program`] frame — the program, the hash it must match,
//! the seed and the clock — and then it answers the guest's requests on `delulu-sandbox-channel/3`
//! until the guest says it is done. The guest's own root is EMPTY: every capability it uses is a handle the host minted, so
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

use delulu_runtime::channel::{read_frame, ChannelSink, HostChannel, Open, Program, CHANNEL_VERSION};
use delulu_runtime::interp::Interp;
use delulu_runtime::sink::LocalSink;
use delulu_runtime::value::{RootVal, Value};

/// How long either side waits for the other before giving up. A channel with no deadline is the
/// IPC-1 shape: one stalled peer hangs the other for ever (PS-0-07 fixed the same hole for the
/// foreign worker, which is why the guest borrows its transport rather than using pipes).
pub(crate) const CHANNEL_DEADLINE: std::time::Duration = std::time::Duration::from_secs(60);

/// How long the host waits for the guest to come up at all.
pub(crate) const CONNECT_DEADLINE: std::time::Duration = std::time::Duration::from_secs(10);

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

/// PS-D-01: the guest argument that means "your channel is your standard input and output, as two
/// ordinary PIPES" — what an operator's external launcher (L3) gives a guest it runs in its own
/// environment: `docker run -i`, `ssh`, a cloud sandbox's exec. Unlike [`STDIO_FLAG`] it needs no
/// inherited socket or duplex pipe, on any platform. The host tells a launcher the words to run in
/// `DELULU_GUEST_ARGS`.
pub const STDIO_PIPES_FLAG: &str = "--stdio-pipes";

/// PS-E-05 (b), D-V2-83: an external launcher's declaration, after [`STDIO_PIPES_FLAG`], that it starts the
/// guest inside an outer wall whose syscall filter forbids the guest adding its own — NVIDIA OpenShell's
/// sandbox does (routine run 8 read `seccomp` answered with EPERM there). Declared, a guest whose own filter
/// is refused with EPERM while a filter is in force runs under that one and reports it as such
/// ([`delulu_runtime::channel::OUTER_FILTER`]); undeclared, it fails closed as always. The launcher's word,
/// like its wall: the host reports it as the guest's (`guest_reported`) at level 3 and measures none of it.
pub const OUTER_FILTER_FLAG: &str = "--outer-syscall-filter";

/// The channel of a guest started with [`STDIO_PIPES_FLAG`]: standard input for reading, and a copy of
/// standard output for writing, with standard output itself pointed at standard error, so a stray
/// print can never reach the channel. The read watchdog ends a guest whose host fell silent.
fn stdio_pipes_channel() -> Result<crate::pipe_channel::GuestChannel<std::fs::File, std::fs::File>, String> {
    #[cfg(windows)]
    {
        stdio_channel()
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::os::fd::FromRawFd as _;
        // SAFETY: descriptors 0, 1 and 2 are this process's own; each is taken over exactly once here.
        unsafe {
            let out = libc::dup(1);
            if out < 0 {
                return Err(format!("its standard output cannot be duplicated: {}", io::Error::last_os_error()));
            }
            if libc::dup2(2, 1) < 0 {
                return Err(format!("its standard output cannot be redirected: {}", io::Error::last_os_error()));
            }
            let reader = std::fs::File::from_raw_fd(0);
            let writer = std::fs::File::from_raw_fd(out);
            crate::pipe_channel::GuestChannel::new(reader, writer, || {
                eprintln!("error: the sandbox guest heard nothing from its host within {CHANNEL_DEADLINE:?} — ending");
                std::process::exit(2);
            })
            .map_err(|e| e.to_string())
        }
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        Err("a pipes guest is not built for this platform".to_string())
    }
}

/// PS-D-01: start the operator's launcher. Its words are the program and its arguments (no shell). It
/// is told, in its environment, the guest's own words and the limits the run asked for, so a recipe can
/// map them (`docker --memory`, `--cpus`); enforcing them is the launcher's — DeluluLang measures none of
/// it, and the report says so.
///
/// PS-D-02: where this run requires attestation, the launcher is also told the run's nonce and where to
/// write its attester's document ([`crate::attest::ENV_NONCE`], [`crate::attest::ENV_OUT`]).
///
/// PS-E-04: `launcher` is the command's first word, already resolved and hashed
/// ([`crate::launcher::Launcher`]); it is started by that path — on Linux, as the very file hashed.
fn launch_external(
    cmd: &str,
    launcher: &crate::launcher::Launcher,
    limits: crate::jail::Limits,
    attest: Option<(&str, &std::path::Path)>,
) -> io::Result<(Guest, crate::jail::Jail, Vec<&'static str>)> {
    let mut words = cmd.split_whitespace();
    let program = words.next().ok_or_else(|| io::Error::other("the external launcher's command is empty"))?;
    let mut c = std::process::Command::new(&launcher.path);
    c.args(words)
        .env("DELULU_GUEST_ARGS", format!("{GUEST_SUBCOMMAND} {STDIO_PIPES_FLAG}"))
        .env("DELULU_LIMIT_MEMORY_BYTES", limits.memory_bytes.to_string())
        .env("DELULU_LIMIT_CPU_SECONDS", limits.cpu_seconds.to_string())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        // RW 4.32: relayed by the host, escaped and marked, never the operator's terminal itself.
        .stderr(std::process::Stdio::piped());
    if let Some(w) = limits.wall_seconds {
        c.env("DELULU_LIMIT_WALL_SECONDS", w.to_string());
    }
    if let Some((nonce, out)) = attest {
        c.env(crate::attest::ENV_NONCE, nonce).env(crate::attest::ENV_OUT, out);
    }
    // PS-E-02 (§4.2): the launcher ends with its host, as a jailed guest always has. It was started with
    // no death signal, so a host killed with SIGKILL left it running (witnessed on `71221d3`). What the
    // launcher itself started — a container — is the launcher's to end; the report does not claim it.
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt as _;
        let host = std::process::id();
        // SAFETY: only async-signal-safe calls (`prctl`, `getppid`, `_exit`) between fork and exec.
        unsafe {
            c.pre_exec(move || {
                if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL as libc::c_ulong, 0, 0, 0) != 0 {
                    return Err(io::Error::last_os_error());
                }
                // The host may have died between the fork and the prctl: then no signal will come.
                if libc::getppid() as u32 != host {
                    libc::_exit(1);
                }
                Ok(())
            });
        }
    }
    // PS-E-02 on Windows: created suspended, so the launcher joins its job before its first instruction
    // and nothing it starts can race the assignment.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_SUSPENDED: u32 = 0x0000_0004;
        c.creation_flags(CREATE_SUSPENDED);
    }
    // PS-E-04: the file that was hashed is the file that starts. Registered last, so every step above runs
    // first in the child.
    launcher.start_as_hashed(&mut c)?;
    let child = c.spawn().map_err(|e| {
        let why = match e.kind() {
            io::ErrorKind::NotFound => "there is no such program".to_string(),
            io::ErrorKind::PermissionDenied => "it is not executable by this user".to_string(),
            _ => crate::cli::broker_unreachable_detail(&e),
        };
        io::Error::other(format!("the external launcher `{program}` could not be started: {why}"))
    })?;
    // PS-E-02 on macOS, which has no death signal: the watcher that ends a jailed guest with its host
    // ends the launcher (D-V2-60). Nothing is claimed for it either way — the level stays 3.
    #[cfg(target_os = "macos")]
    let jail = match std::env::current_exe()
        .map_err(|e| e.to_string())
        .and_then(|exe| crate::jail::HostWatch::start(&exe, &child, CONNECT_DEADLINE))
    {
        Ok(watch) => crate::jail::Jail::none().watched_by(watch),
        Err(why) => {
            eprintln!("sandbox: the external launcher will not be ended if this host dies ({why})");
            crate::jail::Jail::none()
        }
    };
    // PS-E-02 on Windows: a job with kill-on-close and nothing else, which ends the launcher — and what
    // it started inside the job — when the host is gone, as the jailed guest's job always has. Nothing is
    // claimed for it: the level stays 3, and a container the launcher asked a service for is the service's.
    #[cfg(windows)]
    let jail = {
        let jail = crate::jail::end_with_host(&child).unwrap_or_else(|why| {
            eprintln!("sandbox: the external launcher will not be ended if this host dies ({why})");
            crate::jail::Jail::none()
        });
        if !crate::jail::resume(&child) {
            let mut child = child;
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::other(format!("the external launcher `{program}` could not be started: it did not resume")));
        }
        jail
    };
    #[cfg(not(any(windows, target_os = "macos")))]
    let jail = crate::jail::Jail::none();
    Ok((Guest::External(child), jail, Vec::new()))
}

/// Run as the guest. Returns the process exit status.
pub fn run_guest(args: &[String]) -> i32 {
    // SANDBOX-STOP-1: from here a refused allocation ends this process with a status the host names.
    crate::ceiling::enter_guest_mode();
    // D-V2-83: the outer-wall declaration is an external launcher's alone. A guest the HOST starts is
    // started with fixed words and never needs it; a word a guest does not act on is refused, not ignored.
    if args.iter().any(|a| a == OUTER_FILTER_FLAG) && args.first().map(String::as_str) != Some(STDIO_PIPES_FLAG) {
        eprintln!(
            "error: `{OUTER_FILTER_FLAG}` declares an external launcher's wall — only a guest an external launcher starts \
             (`__guest {STDIO_PIPES_FLAG}`) takes it"
        );
        return 2;
    }
    // PS-D-01: a guest an external launcher started, its channel two ordinary pipes.
    if args.first().map(String::as_str) == Some(STDIO_PIPES_FLAG) {
        let outer = match &args[1..] {
            [] => false,
            [flag] if flag == OUTER_FILTER_FLAG => true,
            rest => {
                let shown: Vec<String> = rest.iter().map(|a| delulu_runtime::channel::shown(a, 64)).collect();
                eprintln!(
                    "error: the sandbox guest does not know `{}` — after `{STDIO_PIPES_FLAG}` it takes only \
                     `{OUTER_FILTER_FLAG}`, once",
                    shown.join(" ")
                );
                return 2;
            }
        };
        // Where the guest applies no syscall filter of its own, an outer one has nothing to stand in for.
        #[cfg(not(target_os = "linux"))]
        if outer {
            eprintln!(
                "error: `{OUTER_FILTER_FLAG}`: on this operating system the guest applies no syscall filter of its own, \
                 so there is nothing for an outer one to stand in for"
            );
            return 2;
        }
        return match stdio_pipes_channel() {
            Ok(conn) => serve_as_guest(conn, None, &[], outer),
            Err(e) => {
                eprintln!("error: the sandbox guest cannot open its channel: {e}");
                2
            }
        };
    }
    // PS-C-03: the microVM guest, run by its own kernel as PID 1, whose channel is a vsock stream to
    // the host it dials itself.
    #[cfg(target_os = "linux")]
    if args.first().map(String::as_str) == Some(crate::microvm::VSOCK_FLAG) {
        return crate::microvm::run_vm_guest(&args[1..]);
    }
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
        return serve_as_guest(conn, None, &[], false);
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
    serve_as_guest(conn, Some(&dir), &[], false)
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
///
/// `measured` is what the guest found true of its own boundary by ATTEMPTING it before this was
/// called — the microVM guest's "no network stack in its kernel" (PS-C-03) — reported with the layers
/// it applies below, in the same one confinement report.
pub(crate) fn serve_as_guest<C: std::io::Read + std::io::Write + 'static>(
    mut conn: C,
    dir: Option<&std::path::Path>,
    measured: &[&'static str],
    outer_filter: bool,
) -> i32 {
    // PS-E-01: the host opens with this run's generation and NO program. The program comes only after
    // this guest has locked itself down and the host has accepted its report of what it applied.
    let open: Open = match read_frame(&mut conn) {
        Ok(o) => o,
        // No opening, no run: a guest reached by anything other than its host does nothing at all.
        Err(e) => {
            eprintln!("error: the sandbox guest was started without an opening frame from its host ({e})");
            return 2;
        }
    };
    if open.version != CHANNEL_VERSION {
        eprintln!("error: the host speaks `{}`, this guest speaks `{CHANNEL_VERSION}`", open.version);
        return 2;
    }

    // The guest narrows itself to what interpreting needs — before it has been sent a program at all.
    // The filesystem first, because it is the one a guest needs none of — every read and write the
    // program asks for is performed by the HOST — and because the syscall filter below says nothing
    // about WHICH files a permitted syscall may reach.
    // What the guest applies to itself, for the host's report (RW 4.23).
    let mut own: Vec<&'static str> = measured.to_vec();
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

    // Fail closed — a guest that cannot be locked down does not run the program. (D-V2-83: the one
    // exception is an outer wall's filter its launcher declared, which `lock_down_self` checks is in force.)
    match crate::jail::lock_down_self(outer_filter) {
        Ok(applied) if applied == [delulu_runtime::channel::OUTER_FILTER] => {
            eprintln!(
                "sandbox: the guest's own syscall filter was refused by a filter already in force on it — the \
                 outer wall its launcher declared (`{OUTER_FILTER_FLAG}`) stands in for it; the guest's own \
                 filter is NOT installed"
            );
            own.extend(applied);
        }
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
    // Told to the host now — after the lock-down, before the guest holds any program — so the report
    // counts what this guest really applied, and the words come from the toolchain, not the program.
    if let Err(e) = sink.confined(&own, &open.generation) {
        eprintln!("error: the host would not take this guest's confinement report ({e}) — nothing ran");
        return 2;
    }
    // Confirmed: only now does the host send what to run.
    let hello: Program = match sink.receive() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: the host confirmed this guest but sent no program ({e}) — nothing ran");
            return 2;
        }
    };
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

    let interp = Interp::new(&checked.module).with_effect_sink(sink.clone());
    // The guest's root grants NOTHING. Every capability the program obtains is minted by the host,
    // over the channel, from the root the operator actually granted.
    let exit = match interp.run_main(Value::Root(Rc::new(RootVal::default()))) {
        Ok(_) => 0,
        Err(f) => {
            // TERMINAL-TEXT-1: a fault can quote the program's own strings (an assertion's values).
            eprintln!("error[{}]: {}", f.code, delulu_diag::terminal_line(&f.message));
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
/// request kind on `delulu-sandbox-channel/3`, and a handle cannot stand in for a thread or a
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
const APPLIED_UNDER_SANDBOX: &[&str] = &[
    "--sandbox",
    "--sandbox-profile",
    "--mode",
    "--limits",
    "--grant",
    "--json",
    "--report-out",
    "--no-prompt",
    // PS-C: `--isolation microvm` chooses the boundary a sandboxed run gets; any other value is refused
    // by `isolation_of` before anything runs.
    "--isolation",
    // REMAINING_WORK 4.20, closed: broker custody for guests. Under `--broker daemon` or `--lease` the
    // host authorizes every operation the guest asks for through the broker before performing it —
    // revocation, expiry, the Guard's tiers and permits — as it does for a program it interprets.
    "--broker",
    "--lease",
    "--epoch-ms",
    // PS-D-01: `external:CMD` chooses an L3 launcher for the guest.
    "--sandbox-backend",
    // PS-D-02: an external launcher's attester must vouch for the guest before the program is sent.
    "--require-attestation",
    // PS-E-04: the external launcher's file must have this BLAKE3 digest, or nothing starts.
    "--launcher-digest",
];

/// The largest program a sandboxed run sends: the channel's frame bound, less room for the frame's other
/// fields (the hash, the seed, the clock and the encoding).
const MAX_PROGRAM_BYTES: usize = delulu_runtime::channel::MAX_FRAME as usize - 64 * 1024;

/// Which boundary a sandboxed run asked for: the jailed guest process (L1, `--sandbox`) or the
/// microVM (L2, `--isolation microvm`, PS-C). The guest and the channel are the same in both; what
/// differs is what stands between the guest and the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Isolation {
    Process,
    MicroVm,
    /// PS-D-01 (L3): an operator-supplied launcher, the command (its words, no shell) as given; (PS-D-02)
    /// the key its attester must sign with, when the run requires attestation; and (PS-E-04) the BLAKE3
    /// digest the launcher's file must have, when the run pins it.
    External(&'static str, Option<&'static str>, Option<&'static str>),
}

impl Isolation {
    pub fn level(self) -> u8 {
        match self {
            Isolation::Process => 1,
            Isolation::MicroVm => 2,
            Isolation::External(..) => 3,
        }
    }

    fn backend(self) -> &'static str {
        match self {
            Isolation::Process => "process",
            Isolation::MicroVm => "microvm",
            Isolation::External(..) => "external",
        }
    }
}

/// The boundary this command line asked for. `--isolation none` or `process` beside `--sandbox` names
/// a weaker boundary than the one `--sandbox` gives, and which of the two was meant is not something
/// to guess, so it is refused.
fn isolation_of(opts: &crate::cli::Opts) -> Result<Isolation, String> {
    // PS-D-01: `--sandbox-backend external:CMD`. The command is the operator's, run WITHOUT a shell: its
    // words are the program and its arguments (a launcher that needs quoting belongs in a script).
    if let Some(b) = opts.sandbox_backend.as_deref() {
        let cmd = match b.strip_prefix("external:") {
            Some(cmd) if !cmd.trim().is_empty() => cmd.trim(),
            Some(_) => {
                return Err("`--sandbox-backend external:` needs the launcher's command after the colon — e.g. \
                            `external:docker run -i --rm --network none IMAGE delulu __guest --stdio-pipes`. Nothing ran."
                    .to_string())
            }
            None => {
                return Err(format!(
                    "`--sandbox-backend {b}` is not a backend this command knows: `external:CMD` (L3) is the one. The \
                     jailed process is `--sandbox` alone, the microVM `--isolation microvm`. Nothing ran."
                ))
            }
        };
        if let Some(i) = opts.isolation.as_deref() {
            return Err(format!(
                "`--isolation {i}` and `--sandbox-backend external:…` name two different boundaries. Nothing ran: say which one you mean."
            ));
        }
        let pinned = match opts.require_attestation.as_deref() {
            None => None,
            Some(given) => {
                let key = crate::attest::pinned_key(given).map_err(|why| format!("{why}. Nothing ran."))?;
                Some(&*Box::leak(key.into_boxed_str()))
            }
        };
        let launcher_pin = match opts.launcher_digest.as_deref() {
            None => None,
            Some(given) => {
                let pin = crate::launcher::pinned_digest(given).map_err(|why| format!("{why}. Nothing ran."))?;
                Some(&*Box::leak(pin.into_boxed_str()))
            }
        };
        return Ok(Isolation::External(Box::leak(cmd.to_string().into_boxed_str()), pinned, launcher_pin));
    }
    // PS-E-04: the launcher's digest pins a file DeluluLang starts; the jailed process and the microVM
    // start this binary, which a pin would not be checked against.
    if opts.launcher_digest.is_some() {
        return Err("`--launcher-digest` is for an external launcher (`--sandbox-backend external:CMD`, level 3): it \
                    pins the file that launcher's word resolves to. Nothing ran."
            .to_string());
    }
    // PS-D-02: an attester vouches for a boundary this host did NOT measure. L1 and L2 are measured here,
    // and nothing an attester says would be checked against them, so the flag is refused rather than
    // accepted and ignored.
    if opts.require_attestation.is_some() {
        return Err("`--require-attestation` is for an external launcher (`--sandbox-backend external:CMD`, level 3): \
                    this host measures the jailed process and the microVM itself, and the report says what it \
                    measured. Nothing ran."
            .to_string());
    }
    match opts.isolation.as_deref() {
        None => Ok(Isolation::Process),
        Some("microvm") => Ok(Isolation::MicroVm),
        Some(other) => Err(format!(
            "`--isolation {other}` and `--sandbox` name two different boundaries. Nothing ran. `--sandbox` \
             alone is the jailed guest process; `--isolation microvm` is the microVM; `--isolation {other}` \
             without `--sandbox` is the weaker profile it names."
        )),
    }
}

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
    Some(2)
}

/// PS-D-02: a dry run launches nothing, so an attestation it requires is reported as required and not
/// verified — never left out, which would read as a run that asked for none.
fn with_attestation_required(mut sandbox: serde_json::Value, isolation: Isolation) -> serde_json::Value {
    if let Isolation::External(_, Some(key), _) = isolation {
        sandbox["attestation"] = crate::attest::required_json(key);
    }
    sandbox
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
    let isolation = match isolation_of(opts) {
        Ok(i) => i,
        Err(why) => {
            eprintln!("error: {why}");
            return 2;
        }
    };
    // L2 is refused, with the fallbacks named, on a host that cannot give it — before the program is
    // read, as the ordinary path always refused it. Never quietly run at L1 instead (trap 8).
    if isolation == Isolation::MicroVm {
        if let Err(detail) = crate::cli::microvm_unavailable() {
            return crate::run_cmd::refuse_microvm(file, &detail, opts.json);
        }
    }
    // P5 (D-V2-42): a package directory runs sandboxed exactly as it runs without the sandbox —
    // resolved, checked authoritatively and flattened by `load_package_for_run`, the ordinary run's
    // own loader — and the guest is handed the flattened text. It used to be read as a file, so
    // `delulu run . --sandbox`, the command a new package's own output teaches, died on the raw OS
    // error (`Access is denied. (os error 5)` on Windows, `Is a directory` on Linux): C27's defect,
    // back in the one path C27's fix had not reached.
    let is_package = std::path::Path::new(file).is_dir();
    let (program, loaded) = if is_package {
        match crate::run_cmd::load_package_for_run(file, opts) {
            Ok((map, checked, flat)) => (flat, Some((map, checked))),
            Err(c) => return c,
        }
    } else {
        match std::fs::read_to_string(file) {
            Ok(s) => (s, None),
            Err(e) => {
                eprintln!("error: {}", crate::cli::unreadable(&file, &e));
                return 2;
            }
        }
    };
    // The red-team pass on `/3` (F1): a program the channel cannot carry in one frame is refused here,
    // before a guest exists — it used to launch a guest, confirm it, and then fail to send.
    if program.len() > MAX_PROGRAM_BYTES {
        eprintln!(
            "error: `{file}` is {} bytes, larger than the sandbox channel carries in one frame ({} bytes). \
             Nothing ran: split the program into modules, or run it without `--sandbox` if you accept no \
             confinement.",
            program.len(),
            MAX_PROGRAM_BYTES
        );
        return 2;
    }
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
    let mut limits = match parse_limits(opts.limits.as_deref(), profile) {
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
        let policy =
            crate::policy::SandboxPolicy::derive(isolation.level(), profile, Some(limits), mode).requesting(isolation.level());
        let checked = delulu_check::check_source(0, &program);
        let required = crate::cli::required_grants_of(file, &checked);
        let carried = unsupported_surface(&program);
        let report = serde_json::json!({
            "command": "run",
            "schema": 1,
            "delulu_version": env!("CARGO_PKG_VERSION"),
            "diagnostics": [],
            "summary": { "errors": 0, "warnings": 0 },
            "sandbox": with_attestation_required(policy.to_json("none", &[]), isolation),
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
    // Checked here, as the ordinary run checks it, before a guest exists: a program that does not
    // check is refused with its diagnostics, where the guest could only say "the host sent a program
    // that does not check" and hang up — the reader got no line, no code, no fix. The guest still
    // checks what it is sent; this is the reader's copy, not the gate's only one. (A package was
    // checked by its loader.) Then the manifest binds the sandboxed run as it binds the ordinary one:
    // a package's own, or the one beside a single file.
    let (map, checked) = match loaded {
        Some(x) => x,
        None => {
            let mut map = delulu_diag::SourceMap::new();
            let fid = map.add_file(file.to_string(), program.clone());
            let checked = delulu_check::check_source(fid, &program);
            if checked.diagnostics.iter().any(|d| d.is_error()) {
                crate::cli::print_diagnostics("run", &checked.diagnostics, &map, None, opts.json);
                return 1;
            }
            (map, checked)
        }
    };
    if let Err(c) = crate::run_cmd::manifest_gate(file, is_package, &checked, &map, opts.json) {
        return c;
    }
    let mut grants = delulu_runtime::broker::Grants::default();
    for g in &opts.grants {
        if let Err(e) = grants.add(g) {
            eprintln!("error: {e}");
            return 2;
        }
    }
    // Broker custody for the guest (REMAINING_WORK 4.20). The same two doors as the ordinary run and
    // the same code behind the first: `--lease` redeems the token with `run_cmd::redeem_lease` — the
    // grants become the node's own, a local grant beside it is refused, the Guard's status line is
    // printed — and a node's budget narrows the guest's limits (never widens them); `--broker daemon`
    // issues a root from the grants, recording the guest's limits as its budget. Decided after the
    // audit branch above, because redeeming a lease is not a dry run.
    let lease_mode = opts.lease.is_some();
    let daemon_mode = match opts.broker.as_deref() {
        None | Some("embedded") => lease_mode,
        Some("daemon") => true,
        Some(other) => {
            eprintln!("error: unknown --broker mode `{other}` (embedded | daemon)");
            return 2;
        }
    };
    let mut custody: Option<crate::broker_client::BrokerClientCustody> = None;
    if let Some(token) = &opts.lease {
        let json = opts.json;
        let mut hold = |ceiling: &delulu_broker::BudgetScope| -> Result<(), i32> {
            limits.memory_bytes = limits.memory_bytes.min(ceiling.memory_bytes);
            limits.cpu_seconds = limits.cpu_seconds.min(ceiling.cpu_seconds);
            if !json {
                eprintln!(
                    "lease: the guest held to the delegated budget, mem={} bytes, cpu={} s",
                    limits.memory_bytes, limits.cpu_seconds
                );
            }
            Ok(())
        };
        match crate::run_cmd::redeem_lease(token, &mut grants, opts, &map, &mut hold) {
            Ok(c) => custody = Some(c),
            Err(code) => return code,
        }
    }
    if daemon_mode && !opts.json {
        eprintln!("custody: daemon");
    }
    if daemon_mode && custody.is_none() {
        let Some(state_dir) = crate::brokerd::resolve_state_dir(None) else {
            eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE)");
            return 2;
        };
        let mut spec = crate::cli::authority_spec_from_grants(&grants, file);
        spec.budget = Some(
            delulu_broker::BudgetScope { memory_bytes: limits.memory_bytes, cpu_seconds: limits.cpu_seconds }
                .to_grant_string(),
        );
        match crate::broker_client::BrokerClientCustody::issue_root(state_dir, spec, opts.epoch_ms) {
            Ok(c) => custody = Some(c),
            Err(d) => {
                let diag = delulu_diag::Diagnostic::error(d.code, d.message);
                crate::cli::print_diagnostics("run", &[diag], &map, None, opts.json);
                return 1;
            }
        }
    }
    let custody_record = custody.as_ref().map(|c| serde_json::json!({ "mode": "daemon", "node": c.node().to_string() }));
    let root = Rc::new(crate::cli::build_root(&grants));
    let served = spawn_and_serve_custody(
        &program,
        root,
        limits,
        profile,
        opts.report_out.as_deref(),
        isolation,
        custody.map(|c| Box::new(c) as Box<dyn delulu_runtime::Custody>),
        custody_record,
    );
    let egress = delulu_runtime::egress::take_log();
    if !opts.json {
        crate::run_cmd::print_egress_notes(&egress);
    }
    match served {
        Ok(exit) => exit,
        // D-V2-59: the profile's refusal, before the program was sent.
        Err(e) if crate::boundary::is_refusal(&e) => {
            eprintln!("error[DL1408]: {e}");
            2
        }
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
            .ok_or_else(|| format!("`--limits {part}` needs the form mem=BYTES, cpu=SECONDS or wall=SECONDS"))?;
        let n: u64 = value.parse().map_err(|_| format!("`{value}` is not a number in `--limits {part}` — memory is a count of bytes (1 GiB is 1073741824), processor and wall time are seconds"))?;
        match key.trim() {
            "mem" => limits.memory_bytes = n.min(limits.memory_bytes),
            "cpu" => limits.cpu_seconds = n.min(limits.cpu_seconds),
            // SANDBOX-STOP-1: accepted under the sandbox as it is without it (the agent pass found it
            // refused while the report listed a wall-clock ceiling). Narrowing only, like the others.
            "wall" if n == 0 => return Err("`--limits wall=0` would stop the run before it started".to_string()),
            "wall" => limits.wall_seconds = Some(limits.wall_seconds.map_or(n, |w| w.min(n))),
            other => return Err(format!("`--limits {other}=…` is not a limit this command knows (mem, cpu, wall)")),
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
            eprintln!("error: {}", crate::cli::unreadable(&file, &e));
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
    spawn_and_serve_with(
        program,
        root,
        seed,
        fixed_clock_ms,
        crate::jail::Limits::default(),
        crate::policy::Profile::Contained,
        None,
        Isolation::Process,
    )
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
    isolation: Isolation,
) -> io::Result<i32> {
    serve_under(program, root, seed, fixed_clock_ms, limits, profile, report_out, isolation, None, None)
}

/// `run --sandbox` itself: the same, under the custody the run chose — the broker's, when there is
/// one, which then authorizes every operation the guest asks for.
#[allow(clippy::too_many_arguments)]
fn spawn_and_serve_custody(
    program: &str,
    root: Rc<RootVal>,
    limits: crate::jail::Limits,
    profile: crate::policy::Profile,
    report_out: Option<&str>,
    isolation: Isolation,
    custody: Option<Box<dyn delulu_runtime::Custody>>,
    custody_record: Option<serde_json::Value>,
) -> io::Result<i32> {
    serve_under(program, root, 0xDE1, None, limits, profile, report_out, isolation, custody, custody_record)
}

#[allow(clippy::too_many_arguments)]
fn serve_under(
    program: &str,
    root: Rc<RootVal>,
    seed: u64,
    fixed_clock_ms: Option<i64>,
    limits: crate::jail::Limits,
    profile: crate::policy::Profile,
    report_out: Option<&str>,
    isolation: Isolation,
    custody: Option<Box<dyn delulu_runtime::Custody>>,
    custody_record: Option<serde_json::Value>,
) -> io::Result<i32> {
    let exe = std::env::current_exe()?;
    // SANDBOX-STOP-1: the processor time of this process's reaped children, before the guest exists
    // — the guest's own use is the difference after it is reaped.
    let children_before = children_cpu();
    let dir = std::env::temp_dir().join(format!("delulu-guest-{}-{}", std::process::id(), channel_tag()));
    make_run_dir(&dir)?;
    // PS-E-01: every run has a generation — 32 bytes of the OS's randomness — at every level. The guest
    // must echo it to confirm its boundary, the launch record and the report name it, and (PS-D-02) an
    // attester signs over it: an attested run's nonce IS its generation.
    let generation = match crate::attest::fresh_nonce() {
        Ok(g) => g,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dir);
            return Err(io::Error::other(e));
        }
    };
    // PS-D-02: the place in this run's own directory where the attester's document must appear.
    let attest_plan = match isolation {
        Isolation::External(_, Some(key), _) => Some((key, generation.clone(), dir.join(crate::attest::FILE_NAME))),
        _ => None,
    };
    // PS-E-04: the launcher is resolved once and its bytes hashed, and a pinned run refuses any other file
    // before anything starts — recorded, because a launcher that changed under a pin is news to someone.
    let resolved = match isolation {
        Isolation::External(cmd, _, pin) => {
            let word = cmd.split_whitespace().next().unwrap_or_default();
            let l = match crate::launcher::Launcher::resolve(word) {
                Ok(l) => l,
                Err(e) => {
                    let _ = std::fs::remove_dir_all(&dir);
                    return Err(e);
                }
            };
            if let Err(why) = l.check_pin(pin) {
                audit_sandbox(
                    "sandbox-launcher",
                    "deny",
                    Some(l.blake3.clone()),
                    Some(serde_json::json!({
                        "launcher_path": l.path.display().to_string(),
                        "launcher_blake3": l.blake3,
                        "pinned": pin,
                        "generation": generation,
                    })),
                );
                let _ = std::fs::remove_dir_all(&dir);
                return Err(io::Error::other(why));
            }
            Some(l)
        }
        _ => None,
    };
    let launched = match (isolation, &resolved) {
        (Isolation::Process, _) => launch(&exe, &dir, limits, true, false),
        (Isolation::MicroVm, _) => launch_vm(limits, false),
        (Isolation::External(cmd, ..), Some(l)) => {
            launch_external(cmd, l, limits, attest_plan.as_ref().map(|(_, nonce, path)| (nonce.as_str(), path.as_path())))
        }
        (Isolation::External(..), None) => Err(io::Error::other("the external launcher was not resolved")),
    };
    let external = matches!(isolation, Isolation::External(..));
    let launcher = match isolation {
        // The program only: an argument can carry an operator's token, and a report is shared.
        Isolation::External(cmd, ..) => cmd.split_whitespace().next().map(str::to_string),
        _ => None,
    };
    let (mut child, jail, mut applied) = match launched {
        Ok(l) => l,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&dir);
            return Err(e);
        }
    };
    // RW 4.32: the guest's standard error (at L3, the launcher's) reaches the operator through this host,
    // escaped and marked. A microVM's console has its own relay (`microvm.rs`).
    let stderr_relay = match isolation {
        Isolation::MicroVm => None,
        _ => child.take_stderr().map(|from| relay_stderr(from, if external { "launcher" } else { "guest" })),
    };
    let relayed = |relay: &Option<std::sync::mpsc::Receiver<()>>| {
        if let Some(done) = relay {
            let _ = done.recv_timeout(STDERR_RELAY_GRACE);
        }
    };
    // PS-E-03 H4 (HOST-DUMPABLE-1): from here this host holds the run's custody — lease tokens, a secret's
    // bytes on their way to `expose` — and a process of the same user could read its environment and its
    // memory through `/proc` (witnessed as a non-root user: the host's `environ`, sentinel and all).
    // Non-dumpable, its `/proc` entries are root's and no same-user process may read them or attach.
    // Set only now, after the launch: a child forked before it would inherit the flag until its `exec`,
    // and the Linux identity path writes that child's `/proc` uid map in exactly that window. Never
    // undone — a core dump of this process would spill the same bytes.
    #[cfg(target_os = "linux")]
    // SAFETY: a flag on this process alone.
    unsafe {
        libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0);
    }
    // SANDBOX-STOP-1: a process guest's wall-clock ceiling is the host's watchdog, started below. Not
    // claimed for an external launcher: ending the launcher's process need not end what it started (a
    // container can outlive its client), so it is attempted there and never counted.
    if limits.wall_seconds.is_some() && child.killer().is_some() && !external && !applied.contains(&"wall-clock ceiling") {
        applied.push("wall-clock ceiling");
    }
    // D-V2-90 (PS-E-01): macOS gives a process no memory ceiling to be handed (`RLIMIT_DATA` is refused,
    // there is no job object), so a process guest's ceiling there is the host's sampler of its peak
    // footprint (`jail::peak_footprint`), started below with the watchdog — claimed only where the kernel
    // answered the first reading, and never for an external launcher, whose wall is the operator's.
    #[cfg(target_os = "macos")]
    let memory_watch: Option<(MemReader, u64)> = match child.killer() {
        Some(Killer(pid)) if !external && crate::jail::peak_footprint(pid as u32).is_some() => {
            applied.push(crate::jail::SAMPLED_MEMORY_CEILING);
            Some((Box::new(move || crate::jail::peak_footprint(pid as u32)) as MemReader, limits.memory_bytes))
        }
        _ => None,
    };
    #[cfg(not(target_os = "macos"))]
    let memory_watch: Option<(MemReader, u64)> = None;
    if let Some(program) = &launcher {
        eprintln!(
            "sandbox: an external launcher (`{program}`) runs the guest — the boundary is the operator's, and \
             DeluluLang measured none of it (level 3, guarantees unknown). The guest still holds no authority of \
             its own: every effect it asks for is performed here, under the same checks."
        );
    } else if applied.is_empty() {
        // Never claim a boundary that was not applied: PS-0-04's rule, in the place it matters most.
        eprintln!("sandbox: no OS jail on this host yet — the guest still holds no authority of its own");
    } else {
        eprintln!("sandbox: the guest is confined — {}", applied.join("; "));
    }
    let _ = &jail;
    // PS-A-08: the launch is recorded before the guest is told what to run, so the evidence exists
    // even if everything after it fails.
    let policy = crate::policy::SandboxPolicy::derive(
        if external {
            isolation.level()
        } else if applied.is_empty() {
            0
        } else {
            isolation.level()
        },
        profile,
        Some(limits),
        crate::policy::Mode::Strict,
    )
    .requesting(isolation.level());
    let backend = if external {
        isolation.backend()
    } else if applied.is_empty() {
        "inproc"
    } else {
        isolation.backend()
    };
    // PS-C-06: a microVM's launch record names the image it booted, by the hashes of the copies that
    // were checked and booted.
    let image = child.image();
    let with_image = |mut v: serde_json::Value| {
        if let Some(obj) = v.as_object_mut() {
            obj.insert("generation".to_string(), serde_json::json!(generation));
        }
        if let (Some(img), Some(obj)) = (&image, v.as_object_mut()) {
            obj.insert("image".to_string(), img.clone());
        }
        if let (Some(program), Some(obj)) = (&launcher, v.as_object_mut()) {
            obj.insert("launcher".to_string(), serde_json::json!(program));
        }
        // PS-E-04: the file that word resolved to, and the digest of the bytes that were started.
        if let (Some(l), Some(obj)) = (&resolved, v.as_object_mut()) {
            obj.insert("launcher_path".to_string(), serde_json::json!(l.path.display().to_string()));
            obj.insert("launcher_blake3".to_string(), serde_json::json!(l.blake3));
        }
        v
    };
    audit_sandbox(
        "sandbox-launch",
        "allow",
        Some(blake3::hash(program.as_bytes()).to_hex().to_string()),
        Some(with_image(policy.to_json_with(backend, &applied, &[], 0))),
    );
    // PS-D-02: the attester's document is read and checked BEFORE the program is sent. On a refusal the
    // guest is ended having been told nothing, so the program never ran and no effect was performed
    // under the grants or a lease; the refusal is in the audit chain, beside the launch it ends.
    let attested = match &attest_plan {
        None => None,
        Some((key, nonce, path)) => {
            // D-V2-89: a statement that binds a launcher is checked against the one this host started.
            let started = resolved.as_ref().map(|l| l.blake3.as_str());
            let checked = crate::attest::await_and_verify(path, key, nonce, started, CONNECT_DEADLINE, || {
                child.try_wait().ok().flatten().map(|st| st.to_string())
            });
            match checked {
                Ok(a) => {
                    let bound = match &a.launcher_blake3 {
                        Some(d) => format!(", for the launcher it measured ({}…, the one started)", &d[..16]),
                        None => String::new(),
                    };
                    eprintln!(
                        "sandbox: attested by `{}`, signed with the pinned key {}… for this run{bound}: {} — the \
                         attester's word, not a measurement; the host's own guarantees are unchanged",
                        a.attester,
                        &a.key[..16],
                        a.guarantees.join("; ")
                    );
                    audit_sandbox("sandbox-attestation", "allow", Some(a.attester.clone()), Some(a.to_json()));
                    Some(a)
                }
                Err(refusal) => {
                    let why = refusal.explain();
                    audit_sandbox(
                        "sandbox-attestation",
                        "deny",
                        Some(refusal.code().to_string()),
                        Some(serde_json::json!({ "key": key, "reason": why })),
                    );
                    let _ = child.kill();
                    let _ = child.wait();
                    relayed(&stderr_relay);
                    let _ = std::fs::remove_dir_all(&dir);
                    audit_sandbox(
                        "sandbox-death",
                        "deny",
                        Some(format!("attestation refused ({})", refusal.code())),
                        // The red-team pass on `/3` (F10): the same fields as every other death record.
                        Some(serde_json::json!({
                            "denied_total": 0,
                            "confirmed": false,
                            "sent": false,
                            "ended_by_host": true,
                            "generation": generation,
                            "guest_words": [],
                        })),
                    );
                    return Err(io::Error::other(format!("the guest was not served: {why}")));
                }
            }
        }
    };

    let mut evidence = Evidence::default();
    // SANDBOX-STOP-1: the operator's wall-clock ceiling for a process guest (a microVM's VMM carries
    // its own watchdog, which honours the same number) and, on Windows, the processor-time ceiling
    // read from the job's accounting: the job's own limit fires late (SANDBOX-CPU-LATE-1).
    #[cfg(windows)]
    let cpu_watch: Option<(CpuReader, std::time::Duration)> = jail
        .cpu_probe()
        .map(|p| (Box::new(move || p.used()) as CpuReader, std::time::Duration::from_secs(limits.cpu_seconds)));
    #[cfg(not(windows))]
    let cpu_watch: Option<(CpuReader, std::time::Duration)> = None;
    let watch = match (limits.wall_seconds, cpu_watch, memory_watch, child.killer()) {
        (None, None, None, _) | (_, _, _, None) => None,
        (w, c, m, Some(k)) => Some(Watchdog::start(k, w.map(std::time::Duration::from_secs), c, m)),
    };
    let launch_words = applied.clone();
    // D-V2-87: the statement checked above, so at level 3 a claim that names a property can meet it.
    let need = crate::boundary::Requirement { profile, launch: &launch_words, measured_by_host: !external, attested: attested.as_ref() };
    let served = converse(&mut child, &dir, program, &generation, &need, root, seed, fixed_clock_ms, custody, &mut evidence)
        // The red-team pass on `/3` (F2, F3): a channel error can quote what the guest sent — a refused
        // word, a decoder's quotation of a frame — and this text reaches the terminal, the report and the
        // chain. Escaped and bounded here, once, before any of them. (A profile's refusal is the host's
        // own words, and keeps its type.)
        .map_err(|e| {
            if crate::boundary::is_refusal(&e) {
                e
            } else {
                io::Error::new(e.kind(), delulu_runtime::channel::shown(&e.to_string(), 1024))
            }
        });
    // D-V2-59: refused before the program was sent, because the boundary lacks what the profile requires.
    let refused = served.as_ref().err().is_some_and(crate::boundary::is_refusal);
    // The red-team pass on `/3` (F4): a conversation that failed ENDS the guest, and one that said
    // goodbye gets a short grace to exit. `wait()` alone let a guest — an external launcher above all —
    // hold the host for as long as it liked after its channel had failed: "the channel's deadline ended
    // the run" was true only in words. Still before the watchdog is stopped and the guest reaped, so a
    // process id cannot have been reused.
    let host_ended = end_guest(&mut child, served.is_err());
    // Stopped, and joined, BEFORE the guest is reaped below: until then its process id cannot have
    // been reused, so the watchdog can only ever have ended this guest.
    let fired = watch.and_then(Watchdog::stop);
    let wall_fired = match (fired == Some(Fired::Wall), child.vm_wall_fired()) {
        (true, _) => Some("the host's wall-clock watchdog"),
        (_, true) => Some("the microVM's wall-clock watchdog"),
        _ => None,
    };
    let denied = evidence.denied;
    // PS-E-01: a guest that never confirmed its boundary was never sent the program, so it never ran —
    // and (the red-team pass, F1) neither did one that confirmed and then never took the program.
    let confirmed = evidence.confirmed;
    let sent = evidence.sent;
    // RW 4.23: the layers the guest applied to itself count as applied, in the report and in the
    // chain — they were in force before the program's first line, and the host checked the words.
    // RW 4.31 (the red-team pass's F7): but only where the HOST started the guest. At L3 the guest is a
    // binary the operator's launcher chose, and its words are its own: reported as `guest_reported`,
    // never as a host guarantee or in the posture (a lying guest made its report say "writes denied"
    // beside properties that said `unknown`).
    let mut guest_reported: Vec<&'static str> = Vec::new();
    // RW 4.32: the words the host accepted when the guest confirmed, kept for the death record — the
    // chain's own copy of what the guest claimed to have applied to itself.
    let guest_words = evidence.own.clone();
    for w in evidence.own {
        if external {
            guest_reported.push(w);
        } else if !applied.contains(&w) {
            applied.push(w);
        }
    }
    // Whatever happened on the channel, the child is not left running and the channel is removed.
    let status = child.wait();
    relayed(&stderr_relay);
    let _ = std::fs::remove_dir_all(&dir);
    // SANDBOX-STOP-1: a guest that ended without saying goodbye may have been STOPPED by a ceiling —
    // named here only from evidence (the watchdog, the guest's allocator status, the OS's signal, the
    // job's accounting), as the ordinary run names its stops.
    let evidence_of_stop = StopEvidence {
        wall_fired,
        cpu_fired: fired == Some(Fired::Cpu),
        memory_fired: match fired {
            Some(Fired::Memory(peak)) => Some(peak),
            _ => None,
        },
        guest_cpu: children_cpu().zip(children_before).map(|(after, before)| after.saturating_sub(before)),
        vm_memory: child.vm_memory_ceiling(),
    };
    // A guest the HOST ended (F4) was stopped by nothing a report should name as a ceiling.
    let stop = match (&served, &status) {
        (Err(_), Ok(st)) if !host_ended => stop_reason(st, limits, evidence_of_stop, &jail),
        _ => None,
    };
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
        if !st.success() && !host_ended {
            let why = stop.as_ref().map(|s| s.message.clone()).or_else(|| limit_kill_reason(st));
            if let Some(why) = why {
                audit_sandbox(
                    "sandbox-limit",
                    "deny",
                    Some(why),
                    Some(serde_json::json!({
                        "limits": { "memory_bytes": limits.memory_bytes, "cpu_seconds": limits.cpu_seconds, "wall_seconds": limits.wall_seconds },
                        "stopped_by": stop.as_ref().map(|s| s.json.clone()),
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
            Err(e) if refused => format!("refused (DL1408): {e}"),
            Err(e) => format!("channel failed: {e}"),
        }),
        // How many times the host said no, in the chain as well as the report: a run report can be
        // discarded, and the audit chain is the copy an operator cannot quietly lose. And (PS-E-01)
        // whether the guest confirmed its boundary, for which generation — so a guest that never did is
        // on the record as one that was never sent the program.
        Some(serde_json::json!({
            "denied_total": denied.1,
            "confirmed": confirmed,
            "sent": sent,
            "ended_by_host": host_ended,
            "generation": generation,
            "guest_words": guest_words,
        })),
    );

    // PS-B-02: the guest's network requests were performed HERE, by the host, so their record is in
    // this process — the same record an L0 run reports, in the same shape. A snapshot: the caller
    // takes the log afterwards, to tell the operator on stderr.
    let egress = delulu_runtime::egress::snapshot();
    if let Some(s) = &stop {
        eprintln!("error: {}", s.message);
    }
    if let Some(path) = report_out {
        let exit = if refused { 2 } else { served.as_ref().copied().unwrap_or(1) };
        let mut outcome = serde_json::json!({ "ran": sent, "exit": exit });
        if let Some(s) = &stop {
            outcome["stopped_by"] = s.json.clone();
        }
        let report = serde_json::json!({
            "command": "run",
            "schema": 1,
            "delulu_version": env!("CARGO_PKG_VERSION"),
            "diagnostics": [],
            "summary": { "errors": if exit == 0 { 0 } else { 1 }, "warnings": 0 },
            "sandbox": with_image(policy.to_json_with(backend, &applied, &denied.0, denied.1)),
            "outcome": outcome,
            "egress": egress.to_json(),
            // Who decided each use: the broker's node when the run was under it, else embedded.
            "custody": custody_record.clone().unwrap_or_else(|| serde_json::json!({ "mode": "embedded", "node": null })),
        });
        // PS-D-02: the attester's claims, as the attester's — beside `host_guarantees`, never in them.
        let mut report = report;
        // PS-E-01: the five properties this boundary has, answered from what was applied — the same words
        // as `host_guarantees` and the posture, so the three cannot disagree. An external launcher's are
        // `unknown`: DeluluLang measured none of its wall — and (D-V2-87) one its pinned attester named
        // carries that claim beside the `unknown`, as the attester's, the same answer the profile was held to.
        report["sandbox"]["properties"] = crate::boundary::properties(&applied, !external, attested.as_ref());
        if external && !guest_reported.is_empty() {
            report["sandbox"]["guest_reported"] = serde_json::json!(guest_reported);
        }
        if let Some(a) = &attested {
            report["sandbox"]["attestation"] = a.to_json();
        }
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
        // D-V2-59: a profile's refusal is returned as it is, for the caller to print as DL1408.
        Err(e) if refused => Err(e),
        Err(e) => {
            if stop.is_none() {
                eprintln!("sandbox: {e}");
            }
            if status.success() && stop.is_none() {
                Err(io::Error::new(io::ErrorKind::UnexpectedEof, format!("the guest stopped mid-conversation: {e}")))
            } else {
                // Exit 1, as the report says (SANDBOX-STOP-1): the guest's own status — 68, 9, a
                // signal — is the OS's word about the guest, recorded in the audit chain, and a
                // process exit that disagreed with its own report was the agent pass's finding.
                Ok(1)
            }
        }
    }
}

/// A launched guest: PS-A's, reached through a named channel in its directory, or — on Windows since
/// PS-B-03 and on Linux since PS-B-03b — one started as a separate identity, holding its channel from
/// birth.
enum Guest {
    Plain(std::process::Child),
    /// PS-D-01 (L3): the operator's launcher; the channel is its standard input and output.
    External(std::process::Child),
    #[cfg(any(windows, target_os = "linux"))]
    Contained(crate::identity::ContainedGuest),
    /// PS-C: the guest is PID 1 of its own kernel, and this is its VMM.
    #[cfg(target_os = "linux")]
    Vm(Box<crate::microvm::Vm>),
}

impl Guest {
    fn try_wait(&mut self) -> io::Result<Option<std::process::ExitStatus>> {
        match self {
            Guest::Plain(c) | Guest::External(c) => c.try_wait(),
            #[cfg(any(windows, target_os = "linux"))]
            Guest::Contained(c) => c.try_wait(),
            #[cfg(target_os = "linux")]
            Guest::Vm(v) => v.try_wait(),
        }
    }

    fn wait(&mut self) -> io::Result<std::process::ExitStatus> {
        match self {
            Guest::Plain(c) | Guest::External(c) => c.wait(),
            #[cfg(any(windows, target_os = "linux"))]
            Guest::Contained(c) => c.wait(),
            #[cfg(target_os = "linux")]
            Guest::Vm(v) => v.wait(),
        }
    }

    fn kill(&mut self) -> io::Result<()> {
        match self {
            Guest::Plain(c) | Guest::External(c) => c.kill(),
            #[cfg(any(windows, target_os = "linux"))]
            Guest::Contained(c) => c.kill(),
            #[cfg(target_os = "linux")]
            Guest::Vm(v) => v.kill(),
        }
    }

    /// What the wall-clock watchdog needs to end this guest from its own thread: the process id on
    /// Unix, the process handle on Windows — both valid until the guest is reaped, which happens only
    /// after the watchdog has been stopped. `None` for a microVM, whose VMM has its own watchdog.
    fn killer(&self) -> Option<Killer> {
        match self {
            #[cfg(unix)]
            Guest::Plain(c) | Guest::External(c) => Some(Killer(c.id() as usize)),
            #[cfg(windows)]
            Guest::Plain(c) | Guest::External(c) => Some(Killer(std::os::windows::io::AsRawHandle::as_raw_handle(c) as usize)),
            #[cfg(target_os = "linux")]
            Guest::Contained(c) => Some(Killer(c.id() as usize)),
            #[cfg(windows)]
            Guest::Contained(c) => Some(Killer(c.raw_process() as usize)),
            #[cfg(target_os = "linux")]
            Guest::Vm(_) => None,
        }
    }

    /// Whether a microVM's guest said it reached its memory ceiling.
    fn vm_memory_ceiling(&self) -> bool {
        match self {
            #[cfg(target_os = "linux")]
            Guest::Vm(v) => v.memory_ceiling_reached(),
            _ => false,
        }
    }

    /// Whether a microVM's own watchdog ended it.
    fn vm_wall_fired(&self) -> bool {
        match self {
            #[cfg(target_os = "linux")]
            Guest::Vm(v) => v.wall_fired(),
            _ => false,
        }
    }

    /// The image a microVM guest booted, for the launch record and the report; nothing for a process.
    fn image(&self) -> Option<serde_json::Value> {
        match self {
            #[cfg(target_os = "linux")]
            Guest::Vm(v) => v.image.as_ref().map(|i| i.to_json()),
            _ => None,
        }
    }

    /// Standard error, when the launch captured it: the probe keeps it, a run relays it (RW 4.32).
    fn take_stderr(&mut self) -> Option<Box<dyn io::Read + Send>> {
        match self {
            Guest::Plain(c) | Guest::External(c) => c.stderr.take().map(|e| Box::new(e) as Box<dyn io::Read + Send>),
            #[cfg(any(windows, target_os = "linux"))]
            Guest::Contained(c) => c.stderr.take().map(|e| Box::new(e) as Box<dyn io::Read + Send>),
            #[cfg(target_os = "linux")]
            Guest::Vm(v) => v.take_console(),
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
///
/// `capture_stderr`: the guest's standard error is a pipe the caller takes — a run relays it (RW 4.32), the
/// probe keeps it. `quiet`: the launch says nothing on the host's own standard error (the probe reports
/// in its own words).
fn launch(
    exe: &std::path::Path,
    dir: &std::path::Path,
    limits: crate::jail::Limits,
    capture_stderr: bool,
    quiet: bool,
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
            Err(why) => identity_refused(why, quiet),
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
            Err(why) => identity_refused(why, quiet),
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
    // PS-E-02 (D-V2-60): on macOS nothing in the kernel ends the guest with its host; a watcher outside
    // it does, and "killed with the host" is claimed only once the watcher says it is armed. The guest is
    // still waiting for its channel here, so no program byte exists in it yet.
    #[cfg(target_os = "macos")]
    let jail = match crate::jail::HostWatch::start(exe, &child, CONNECT_DEADLINE) {
        Ok(watch) => {
            applied.push("killed with the host");
            jail.watched_by(watch)
        }
        Err(why) => {
            if !quiet {
                eprintln!("sandbox: nothing will end the guest if this host dies ({why})");
            }
            jail
        }
    };
    Ok((Guest::Plain(child), jail, applied))
}

/// Boot the microVM (PS-C-03). What the host applied comes back in the same shape as a process
/// launch's, so everything after it — the policy, the audit records, the report — is shared.
#[cfg(target_os = "linux")]
fn launch_vm(limits: crate::jail::Limits, keep_console: bool) -> io::Result<(Guest, crate::jail::Jail, Vec<&'static str>)> {
    let (vm, applied) = crate::microvm::boot(limits, keep_console)?;
    let (jail, _) = crate::jail::confine(&(), limits);
    Ok((Guest::Vm(Box::new(vm)), jail, applied))
}

/// Elsewhere there is no microVM; `cmd_run_sandboxed` refuses with DL1408 before reaching this.
#[cfg(not(target_os = "linux"))]
fn launch_vm(_limits: crate::jail::Limits, _keep_console: bool) -> io::Result<(Guest, crate::jail::Jail, Vec<&'static str>)> {
    Err(io::Error::other(crate::cli::microvm_unavailable().err().unwrap_or_default()))
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

/// How long a guest that said goodbye may take to exit before the host ends it (F4): a launcher that
/// cleans up — `docker run --rm` removing its container — takes a moment; one that lingers is ended.
const EXIT_GRACE: std::time::Duration = std::time::Duration::from_secs(10);

/// After a conversation: a guest whose channel FAILED is ended at once (after a moment, so a guest that
/// is already exiting — a ceiling fired — finishes and its status still says why); one that said goodbye
/// gets [`EXIT_GRACE`]. True when the host ended it.
fn end_guest(child: &mut Guest, failed: bool) -> bool {
    let grace = if failed { std::time::Duration::from_millis(250) } else { EXIT_GRACE };
    let until = std::time::Instant::now() + grace;
    loop {
        match child.try_wait() {
            Ok(Some(_)) | Err(_) => return false,
            Ok(None) if std::time::Instant::now() >= until => return child.kill().is_ok(),
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(20)),
        }
    }
}

/// What a conversation leaves for the report, whichever way it ended.
#[derive(Default)]
struct Evidence {
    /// PS-E-01: whether the guest confirmed its boundary.
    confirmed: bool,
    /// Whether the program was then SENT — written whole to the channel (the red-team pass, F1).
    sent: bool,
    /// Every refusal the host gave (bounded), and how many there were.
    denied: (Vec<String>, u64),
    /// What the guest reported applying to itself before its program ran (RW 4.23).
    own: Vec<&'static str>,
}

/// Connect to the guest, have it confirm its boundary, tell it what to run, and serve it until it is
/// done.
#[allow(clippy::too_many_arguments)]
fn converse(
    child: &mut Guest,
    dir: &std::path::Path,
    program: &str,
    generation: &str,
    need: &crate::boundary::Requirement<'_>,
    root: Rc<RootVal>,
    seed: u64,
    fixed_clock_ms: Option<i64>,
    custody: Option<Box<dyn delulu_runtime::Custody>>,
    // Filled in on EVERY path, including the failing ones: this phase has already lost a diagnosis
    // three CI runs in a row to a value that was only reported in the success branch.
    evidence: &mut Evidence,
) -> io::Result<i32> {
    let conn = open_channel(child, dir, CHANNEL_DEADLINE)?;
    let mut host = HostChannel::new(LocalSink).with_root(root).with_generation(generation);
    if let Some(c) = custody {
        host = host.with_custody(c);
    }
    // PS-E-01: the boundary first. `boundary::open` sends the generation and no program; only the
    // `Confirmed` that `confirm` builds from the guest's accepted report can send the program.
    let confirmed = match crate::boundary::open(conn, generation).and_then(|o| o.confirm(&mut host, need)) {
        Ok(c) => c,
        Err(e) => {
            let (list, total) = host.denied();
            evidence.denied = (list.to_vec(), total);
            return Err(e);
        }
    };
    evidence.confirmed = true;
    // RW 4.23: what the guest applied to itself, as the host accepted it.
    evidence.own = confirmed.applied().to_vec();
    let mut conn = confirmed.send_program(&Program {
        program: program.to_string(),
        hash: blake3::hash(program.as_bytes()).to_hex().to_string(),
        seed,
        fixed_clock_ms,
    })?;
    evidence.sent = true;
    // The refusals come back with the exit code, because a report that lists only what was allowed
    // says nothing about what the program TRIED — which is the interesting half when the program is
    // one nobody wrote. They are read after `serve` returns, on both paths, so a guest that died
    // mid-conversation still reports what it had been refused up to then.
    let served = host.serve(&mut conn).map_err(in_words);
    // Read on both paths, before the result is returned: a guest that died mid-conversation still
    // reports what it had been refused up to then.
    let (list, total) = host.denied();
    evidence.denied = (list.to_vec(), total);
    served
}

/// A channel failure in the words an operator can act on. Two of them reached the operator as the
/// operating system's own text — "failed to fill whole buffer" for a guest that hung up, and "Resource
/// temporarily unavailable (os error 11)" for one that went silent until the deadline — found by the
/// red team's hostile guests (PS-C-06). Every other error keeps its words: they already say what
/// happened ("frame length exceeds the channel's bound").
fn in_words(e: io::Error) -> io::Error {
    match e.kind() {
        io::ErrorKind::UnexpectedEof => {
            io::Error::new(io::ErrorKind::UnexpectedEof, "the guest closed the channel without saying goodbye")
        }
        // RW 4.37: not silence either — the guest stopped taking the host's answer.
        _ if delulu_runtime::channel::FrameNotTaken::is(&e) => {
            let within = e.get_ref().and_then(|x| x.downcast_ref::<delulu_runtime::channel::FrameNotTaken>()).map(|f| f.within);
            io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "the guest did not take the host's answer whole within {:?}, and the channel's deadline ended the run",
                    within.unwrap_or(delulu_runtime::channel::FRAME_DEADLINE)
                ),
            )
        }
        // RW 4.32: not silence — a frame begun and not finished within the frame deadline.
        _ if delulu_runtime::channel::FrameTooSlow::is(&e) => io::Error::new(
            io::ErrorKind::TimedOut,
            format!("the guest {}, and the channel's deadline ended the run", e.get_ref().map_or_else(String::new, |x| x.to_string())),
        ),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => io::Error::new(
            io::ErrorKind::TimedOut,
            format!("the guest said nothing for {CHANNEL_DEADLINE:?}, and the channel's deadline ended the run"),
        ),
        _ => e,
    }
}

/// Either kind of channel, as one reader-and-writer.
trait Channel: io::Read + io::Write {}
impl<T: io::Read + io::Write> Channel for T {}

/// The host's end of the guest's channel, with `deadline` on every read. A contained guest already
/// holds its pipes; a plain one is connected to by name within the connect deadline.
fn open_channel(child: &mut Guest, dir: &std::path::Path, deadline: std::time::Duration) -> io::Result<Box<dyn Channel>> {
    // PS-D-01: an external launcher's channel is its standard output and input, read with a deadline.
    if let Guest::External(c) = child {
        let (Some(input), Some(output)) = (c.stdin.take(), c.stdout.take()) else {
            return Err(io::Error::other("the external launcher's standard input and output were already taken"));
        };
        return Ok(Box::new(crate::pipe_channel::HostPipes::new(output, input, deadline)));
    }
    #[cfg(windows)]
    if let Guest::Contained(g) = child {
        let mut conn = g.channel.take().ok_or_else(|| io::Error::other("the contained guest's channel was already taken"))?;
        // RW 4.37: this pipe's every operation, a write included, is waited for within this deadline.
        conn.set_read_timeout(Some(deadline))?;
        return Ok(Box::new(conn));
    }
    #[cfg(target_os = "linux")]
    if let Guest::Contained(g) = child {
        let conn = g.channel.take().ok_or_else(|| io::Error::other("the contained guest's channel was already taken"))?;
        return Ok(Box::new(bounded(conn, deadline)?));
    }
    // A microVM's guest dialled in during the boot; its stream is the channel.
    #[cfg(target_os = "linux")]
    if let Guest::Vm(vm) = child {
        let conn = vm.channel.take().ok_or_else(|| io::Error::other("the microVM guest's channel was already taken"))?;
        return Ok(Box::new(bounded(conn, deadline)?));
    }
    Ok(Box::new(bounded_by_name(connect_by_name(child, dir)?, deadline)?))
}

/// The host's end of a guest's channel reached by name, with `deadline` on every read and every write
/// (RW 4.37, as [`bounded`]): a Unix socket's `SO_SNDTIMEO`, or a Windows pipe's write that waits for room
/// within the bound (RW 4.35).
fn bounded_by_name(
    mut conn: crate::broker_transport::Connection,
    deadline: std::time::Duration,
) -> io::Result<crate::broker_transport::Connection> {
    conn.set_read_timeout(Some(deadline))?;
    conn.set_write_timeout(Some(deadline))?;
    Ok(conn)
}

/// The host's end of a guest's socket, with `deadline` on every read AND every write (RW 4.37): a guest
/// that stops reading an answer holds the host's write as surely as a silent one holds a read. The whole
/// answer's bound is `HostChannel::serve`'s (`channel::Within`); this one is each write's, without which a
/// write to a guest that reads nothing never returns to be checked.
#[cfg(any(target_os = "linux", all(test, unix)))]
fn bounded(conn: std::os::unix::net::UnixStream, deadline: std::time::Duration) -> io::Result<std::os::unix::net::UnixStream> {
    conn.set_read_timeout(Some(deadline))?;
    conn.set_write_timeout(Some(deadline))?;
    Ok(conn)
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
    make_run_dir(&dir).map_err(|e| format!("no channel directory: {e}"))?;
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
    let (mut child, jail, applied) = match launch(&exe, &dir, crate::jail::Limits::default(), true, true) {
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
            let conn = open_channel(&mut child, &dir, std::time::Duration::from_secs(3))?;
            // The same order as a run (PS-E-01): the guest confirms its boundary before it is sent even
            // the probe's empty program.
            let generation = crate::attest::fresh_nonce().map_err(io::Error::other)?;
            let mut host = HostChannel::new(LocalSink).with_root(Rc::new(RootVal::default())).with_generation(&generation);
            let need = crate::boundary::Requirement {
                profile: crate::policy::Profile::Dev,
                launch: &[],
                measured_by_host: true,
                attested: None,
            };
            let mut conn = crate::boundary::open(conn, &generation)?.confirm(&mut host, &need)?.send_program(&Program {
                program: PROBE_PROGRAM.to_string(),
                hash: blake3::hash(PROBE_PROGRAM.as_bytes()).to_hex().to_string(),
                seed: 0,
                fixed_clock_ms: None,
            })?;
            host.serve(&mut conn)
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

/// A guest's process id (Unix) or process handle (Windows), for ending it from another thread.
#[derive(Clone, Copy)]
struct Killer(usize);

impl Killer {
    fn kill(self) {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        // SAFETY: the guest has not been reaped (the watchdog is stopped first), so the id is its.
        unsafe {
            libc::kill(self.0 as libc::pid_t, libc::SIGKILL);
        }
        #[cfg(windows)]
        // SAFETY: the handle is the guest's own, alive for as long as the guest value is.
        unsafe {
            windows_sys::Win32::System::Threading::TerminateProcess(self.0 as windows_sys::Win32::Foundation::HANDLE, 1);
        }
    }
}

/// Which of the host's watchdogs ended a process guest.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fired {
    Wall,
    Cpu,
    /// The host's memory sampler (macOS, D-V2-90), with the peak it read.
    Memory(u64),
}

/// Reads a running guest's processor time, where the host can (Windows: the job's accounting).
type CpuReader = Box<dyn Fn() -> Option<std::time::Duration> + Send>;

/// Reads a running guest's peak memory, where the host is its memory ceiling (macOS, D-V2-90).
type MemReader = Box<dyn Fn() -> Option<u64> + Send>;

/// The host's watchdog for a process guest (SANDBOX-STOP-1): ends the guest when the operator's
/// `--limits wall=` passes or, where the host can read the guest's processor time while it runs, when
/// `cpu=` is spent — or, where the host is the guest's memory ceiling (macOS, D-V2-90), when its peak
/// memory reaches `mem=` — unless stopped first. `stop` says which fired, if one did.
struct Watchdog {
    cancel: std::sync::mpsc::Sender<()>,
    handle: std::thread::JoinHandle<Option<Fired>>,
}

impl Watchdog {
    /// How often processor time is read: about the most a guest can spend past its budget.
    const TICK: std::time::Duration = std::time::Duration::from_millis(100);

    /// How often a guest's peak memory is read where the host is its ceiling (macOS, D-V2-90). What a guest
    /// allocates in one interval is what it can pass its budget by: at the ordinary run's 25 ms a guest
    /// doubling strings reached 247 MB against a 64 MiB budget on a macOS runner (`witness.yml`
    /// `37268383827`), so the host reads five times as often — a `proc_pid_rusage` call, a few microseconds.
    const MEMORY_TICK: std::time::Duration = std::time::Duration::from_millis(5);

    fn start(
        k: Killer,
        wall: Option<std::time::Duration>,
        cpu: Option<(CpuReader, std::time::Duration)>,
        memory: Option<(MemReader, u64)>,
    ) -> Watchdog {
        let (cancel, rx) = std::sync::mpsc::channel::<()>();
        let started = std::time::Instant::now();
        // What a guest allocates in one interval is what it can pass its budget by, so the memory interval
        // is the ceiling's resolution ([`Self::MEMORY_TICK`]).
        let tick = if memory.is_some() { Self::MEMORY_TICK } else { Self::TICK };
        let sampling = cpu.is_some() || memory.is_some();
        let handle = std::thread::spawn(move || loop {
            let left = wall.map(|w| w.saturating_sub(started.elapsed()));
            let wait = match (left, sampling) {
                (Some(l), false) => l,
                (Some(l), true) => l.min(tick),
                (None, _) => tick,
            };
            match rx.recv_timeout(wait) {
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if wall.is_some_and(|w| started.elapsed() >= w) {
                        k.kill();
                        return Some(Fired::Wall);
                    }
                    if let Some((read, budget)) = &cpu {
                        if read().is_some_and(|used| used >= *budget) {
                            k.kill();
                            return Some(Fired::Cpu);
                        }
                    }
                    if let Some((read, budget)) = &memory {
                        if let Some(peak) = read().filter(|peak| peak >= budget) {
                            k.kill();
                            return Some(Fired::Memory(peak));
                        }
                    }
                }
                _ => return None,
            }
        });
        Watchdog { cancel, handle }
    }

    fn stop(self) -> Option<Fired> {
        drop(self.cancel);
        self.handle.join().unwrap_or(None)
    }
}

/// The processor time (user + system) of this process's REAPED children, as the OS accounts it.
fn children_cpu() -> Option<std::time::Duration> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // SAFETY: `getrusage` writes one `rusage`.
        let r = unsafe {
            let mut r: libc::rusage = std::mem::zeroed();
            if libc::getrusage(libc::RUSAGE_CHILDREN, &mut r) != 0 {
                return None;
            }
            r
        };
        let tv = |t: libc::timeval| std::time::Duration::new(t.tv_sec as u64, (t.tv_usec as u32).saturating_mul(1000));
        Some(tv(r.ru_utime) + tv(r.ru_stime))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

/// What the host knows about how a guest ended, beyond its exit status.
struct StopEvidence {
    /// Which watchdog ended it, if one did.
    wall_fired: Option<&'static str>,
    /// The host's processor-time watchdog ended it (Windows).
    #[cfg_attr(not(windows), allow(dead_code))]
    cpu_fired: bool,
    /// The host's memory sampler ended it, at this peak (macOS, D-V2-90).
    memory_fired: Option<u64>,
    /// Its processor time, measured by the OS once it was reaped (Unix).
    #[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
    guest_cpu: Option<std::time::Duration>,
    /// A microVM guest's own console line: its memory ceiling was reached.
    vm_memory: bool,
}

/// A named stop: the operator's sentence and the report's `stopped_by`.
struct Stop {
    message: String,
    json: serde_json::Value,
}

/// Why a guest that ended without saying goodbye was stopped — ONLY from evidence: the host's own
/// watchdog; the guest's allocator status ([`crate::ceiling::GUEST_MEMORY_EXIT`], which the program
/// cannot choose); `SIGXCPU`, which exists for exactly this; on Windows the job's own accounting of
/// processor time and peak memory. Anything else stays unnamed ("closed the channel without saying
/// goodbye"), because a record that guesses which limit fired would be read as a measurement.
#[allow(unused_variables)]
fn stop_reason(st: &std::process::ExitStatus, limits: crate::jail::Limits, ev: StopEvidence, jail: &crate::jail::Jail) -> Option<Stop> {
    let budgets = "Budgets are the operator's, set per run with `--limits";
    if let Some(source) = ev.wall_fired {
        if let Some(w) = limits.wall_seconds {
            return Some(Stop {
                message: format!("the run was stopped: it ran for {w} s of wall-clock time, its budget. {budgets} wall=SECONDS`"),
                json: serde_json::json!({ "dimension": "wall", "budget_seconds": w, "source": source }),
            });
        }
        return Some(Stop {
            message: "the run was stopped: the microVM passed its wall-clock ceiling (twice its processor time, at least a minute)".to_string(),
            json: serde_json::json!({ "dimension": "wall", "source": source }),
        });
    }
    let memory = |observed: Option<u64>, source: &str| {
        let mut json = serde_json::json!({ "dimension": "memory", "budget_bytes": limits.memory_bytes, "source": source });
        if let Some(o) = observed {
            json["observed_bytes"] = serde_json::json!(o);
        }
        // Who refused it more: the operating system at its own ceiling, or this host, which read the guest
        // past its budget and ended it (macOS, D-V2-90).
        let by = if source.starts_with("the host's") { "this host ended it" } else { "the operating system refused it more" };
        Stop {
            message: format!(
                "the run was stopped: the guest reached its memory ceiling of {} (D-V2-25), and {by}. {budgets} \
                 mem=BYTES`",
                crate::budget::human_bytes(limits.memory_bytes)
            ),
            json,
        }
    };
    if let Some(peak) = ev.memory_fired {
        return Some(memory(Some(peak), "the host's memory sampler, reading the guest's peak footprint every 5 ms"));
    }
    if st.code() == Some(crate::ceiling::GUEST_MEMORY_EXIT) {
        return Some(memory(None, "the guest's allocator, refused at the ceiling"));
    }
    if ev.vm_memory {
        return Some(memory(None, "the microVM guest's allocator, refused at the VM's memory"));
    }
    let cpu = |observed: Option<std::time::Duration>, source: &str| {
        let mut json = serde_json::json!({ "dimension": "cpu", "budget_seconds": limits.cpu_seconds, "source": source });
        if let Some(o) = observed {
            json["observed_seconds"] = serde_json::json!(o.as_secs_f64());
        }
        Stop {
            message: format!(
                "the run was stopped: the guest used its {} s of processor time (D-V2-25). {budgets} cpu=SECONDS`",
                limits.cpu_seconds
            ),
            json,
        }
    };
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::os::unix::process::ExitStatusExt as _;
        if st.signal() == Some(libc::SIGXCPU) {
            return Some(cpu(ev.guest_cpu, "SIGXCPU from the operating system"));
        }
        // With equal soft and hard processor-time limits the kernel's answer is SIGKILL, which is also
        // what a host's death or an operator sends — so it is named a processor-time stop only when the
        // OS's own accounting of the reaped guest shows the budget spent.
        if st.signal() == Some(libc::SIGKILL) {
            if let Some(used) = ev.guest_cpu.filter(|u| u.as_secs_f64() >= limits.cpu_seconds as f64 * 0.95) {
                return Some(cpu(Some(used), "the operating system's processor-time limit (SIGKILL), with the time the OS accounted"));
            }
        }
    }
    #[cfg(windows)]
    if let Some((used, peak)) = jail.usage() {
        if ev.cpu_fired {
            return Some(cpu(Some(used), "the host's processor-time watchdog, reading the job's accounting"));
        }
        // The job ends a process at its limit, so a measurement at (or within 5% of) the budget IS
        // the limit firing; one well below it is some other death, and stays unnamed.
        if used.as_secs_f64() >= limits.cpu_seconds as f64 * 0.95 {
            return Some(cpu(Some(used), "the job's processor-time accounting"));
        }
        if peak as f64 >= limits.memory_bytes as f64 * 0.95 {
            return Some(memory(Some(peak), "the job's peak-memory accounting"));
        }
    }
    let _ = cpu;
    None
}

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

/// RW 4.32: how much of a guest's standard error reaches the operator, and the longest line relayed whole.
const STDERR_RELAY_CAP: usize = 1024 * 1024;
const STDERR_RELAY_LINE: usize = 8 * 1024;

/// How long the host waits, once the guest is gone, for the relay to print its last line.
const STDERR_RELAY_GRACE: std::time::Duration = std::time::Duration::from_secs(2);

/// RW 4.32: the guest's standard error — at L3 the launcher's, which carries the guest's inside it — read
/// by the host and printed on the host's a line at a time, each escaped (TERMINAL-TEXT-1) and marked with
/// whose it is. It used to be the operator's terminal itself, so a guest that escaped its interpreter
/// wrote control sequences, and lines that read like the host's, straight onto it. Bounded: past
/// [`STDERR_RELAY_CAP`] bytes the rest is still read — a full pipe would stall the writer — and discarded,
/// and that is said once. The receiver hears when the last line is out.
fn relay_stderr(mut from: Box<dyn io::Read + Send>, who: &'static str) -> std::sync::mpsc::Receiver<()> {
    struct Relay {
        who: &'static str,
        passed: usize,
        told: bool,
    }
    impl Relay {
        fn line(&mut self, bytes: &[u8]) {
            if self.passed >= STDERR_RELAY_CAP {
                if !self.told {
                    self.told = true;
                    eprintln!(
                        "sandbox: the {}'s standard error passed {} KiB; the rest was discarded",
                        self.who,
                        STDERR_RELAY_CAP / 1024
                    );
                }
                return;
            }
            self.passed += bytes.len() + 1;
            let text = String::from_utf8_lossy(bytes);
            let text = text.strip_suffix('\r').unwrap_or(&text);
            eprintln!("{}: {}", self.who, delulu_diag::terminal_line(text));
        }
    }
    let (done, finished) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut relay = Relay { who, passed: 0, told: false };
        let mut line: Vec<u8> = Vec::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = match from.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            };
            for &b in &buf[..n] {
                if b == b'\n' {
                    relay.line(&line);
                    line.clear();
                } else {
                    line.push(b);
                    if line.len() >= STDERR_RELAY_LINE {
                        relay.line(&line);
                        line.clear();
                    }
                }
            }
        }
        if !line.is_empty() {
            relay.line(&line);
        }
        let _ = done.send(());
    });
    finished
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
    // ONE writer at a time, and never onto a stale head. Making every sandboxed run a writer broke
    // the chain's single-writer design twice: two runs in parallel landed their records on ONE line
    // (found by `doctor` refusing its own machine), and a run beside the broker daemon made the
    // daemon's next record chain onto a head it had cached before the run's (AUDIT-WRITERS-1). The
    // log itself now takes the append lock and catches up under it, for every writer.
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

/// A per-call channel name: the clock alone collides when runs start together.
/// RUNDIR-PERM-1: a run's own directory — the guest's channel socket, a macOS guest's Seatbelt profile, an
/// attester's document — is made by the host as its user's alone (0700 on Unix), and only if nothing is
/// there yet: a directory someone else made under this name is not this run's. Before, it was made with
/// the process's default mode (0755) and the guest was left to narrow it — which a macOS guest cannot do,
/// its profile refusing it every write but its socket, so every macOS run printed the guest's "not
/// owner-only … this filesystem does not enforce POSIX permissions": a false alarm, blaming the filesystem.
fn make_run_dir(dir: &std::path::Path) -> io::Result<()> {
    let mut b = std::fs::DirBuilder::new();
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut b, 0o700);
    b.create(dir)
}

pub(crate) fn channel_tag() -> String {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    format!("{t}-{n}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RUNDIR-PERM-1: the run's own directory is made its user's alone by the host, and a directory already
    /// under its name — someone else's — is refused rather than used.
    #[cfg(unix)]
    #[test]
    fn a_runs_own_directory_is_made_owner_only_and_never_adopted() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = std::env::temp_dir().join(format!("delulu-rundir-{}-{}", std::process::id(), channel_tag()));
        make_run_dir(&dir).expect("a fresh run directory");
        let mode = std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700, "the run's directory is {mode:04o}");
        assert!(make_run_dir(&dir).is_err(), "a directory already there was adopted as this run's");
        let _ = std::fs::remove_dir(&dir);
    }

    /// RW 4.32: a guest that dripped a frame past the frame deadline is not told of as a SILENT one —
    /// the operator reads what it did, and a silence keeps its own words.
    #[test]
    fn a_slow_frame_and_a_silence_are_told_apart() {
        let slow = io::Error::new(
            io::ErrorKind::TimedOut,
            delulu_runtime::channel::FrameTooSlow { within: delulu_runtime::channel::FRAME_DEADLINE },
        );
        let said = in_words(slow).to_string();
        assert_eq!(said, "the guest did not send one whole frame within 60s, and the channel's deadline ended the run");
        let quiet = in_words(io::Error::from(io::ErrorKind::TimedOut)).to_string();
        assert!(quiet.contains("said nothing for 60s"), "{quiet}");
        // RW 4.37: nor is a guest that stopped taking the host's answer.
        let untaken = io::Error::new(
            io::ErrorKind::TimedOut,
            delulu_runtime::channel::FrameNotTaken { within: delulu_runtime::channel::FRAME_DEADLINE },
        );
        let said = in_words(untaken).to_string();
        assert_eq!(said, "the guest did not take the host's answer whole within 60s, and the channel's deadline ended the run");
    }

    /// RW 4.37: a guest that stops reading cannot hold its host. A guest asks for a large file's text — an
    /// 8 MiB answer, far more than a socket holds — and then never reads it, or reads it a little at a time.
    /// The host's end is configured as `open_channel` configures it: a socket pair as a Linux guest's and a
    /// microVM's channel is (`bounded`), and a connection made by name as a guest's is elsewhere — macOS's,
    /// and Windows' named pipe (`bounded_by_name`). The whole answer is owed within the frame deadline: past
    /// it the host ends the conversation in words, rather than waiting on a guest that has stopped. No default
    /// wall-clock limit would have ended it — a guest that is not reading uses no processor time. Red on
    /// `5bd76aa`: the host wrote until the guest hung up, 15 s later.
    #[test]
    fn a_guest_that_stops_reading_an_answer_cannot_hold_its_host() {
        use delulu_runtime::channel::{read_frame, write_frame, HostChannel, ReqBody, Request, Response, WireValue, CHANNEL_VERSION};
        use std::sync::mpsc;
        let base = std::env::temp_dir().join(format!("dl-hold-{}-{}", std::process::id(), channel_tag()));
        let dir = base.join("data");
        std::fs::create_dir_all(&dir).unwrap();
        #[cfg(unix)]
        let dir = dir.canonicalize().unwrap();
        std::fs::write(dir.join("big.txt"), "x".repeat(8 * 1024 * 1024)).unwrap();
        let within = std::time::Duration::from_millis(800);

        // The guest: it asks for a read capability, then for the big file's text, and then — until it is told
        // to stop, or for 20 s — reads nothing, or a little at a time.
        fn guest<C: io::Read + io::Write>(mut g: C, dir: String, sip: Option<usize>, stop: mpsc::Receiver<()>) {
            let ask = |g: &mut C, seq: u64, body: ReqBody| write_frame(g, &Request { version: CHANNEL_VERSION.into(), seq, body }).unwrap();
            ask(&mut g, 1, ReqBody::RootMethod { method: "fs_read".into(), args: vec![WireValue::Str(dir)], file: 0, start: 0, end: 1 });
            let handle = match read_frame::<Response>(&mut g).unwrap() {
                Response::Ok(WireValue::Cap { handle, .. }) => handle,
                other => panic!("the host minted no capability: {other:?}"),
            };
            let args = vec![WireValue::Str("big.txt".into())];
            ask(&mut g, 2, ReqBody::CapMethod { cap: handle, method: "read_text".into(), args, file: 0, start: 0, end: 1 });
            let until = std::time::Instant::now() + std::time::Duration::from_secs(20);
            let mut buf = vec![0u8; sip.unwrap_or(1)];
            while std::time::Instant::now() < until && stop.try_recv().is_err() {
                if sip.is_some() && matches!(g.read(&mut buf), Ok(0)) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }

        // Serve one such guest on `conn`; the guest runs on its own thread and is stopped once the host is done.
        let check = |what: &str, mut conn: Box<dyn Channel>, run_guest: Box<dyn FnOnce(mpsc::Receiver<()>) + Send>| {
            let (stop, stopped) = mpsc::channel();
            let g = std::thread::spawn(move || run_guest(stopped));
            let mut grants = delulu_runtime::broker::Grants::default();
            grants.add(&format!("fs.read={}", dir.display())).unwrap();
            let mut host = HostChannel::new(delulu_runtime::sink::LocalSink)
                .with_root(std::rc::Rc::new(crate::cli::build_root(&grants)))
                .with_frame_deadline(within);
            let t = std::time::Instant::now();
            let served = host.serve(&mut conn);
            let took = t.elapsed();
            let _ = stop.send(());
            drop(conn);
            g.join().unwrap();
            let said = in_words(served.expect_err("the conversation cannot have ended well")).to_string();
            assert!(took < std::time::Duration::from_secs(8), "{what}: the guest held its host for {took:?} ({said})");
            assert!(said.contains("did not take the host's answer whole within 800ms"), "{what}: in words that say what the guest did: {said}");
        };

        for (name, sip) in [("never reads", None), ("reads a little at a time", Some(4096usize))] {
            #[cfg(unix)]
            {
                let (host_end, guest_end) = std::os::unix::net::UnixStream::pair().unwrap();
                let shown = dir.display().to_string();
                check(&format!("{name}, a socket pair"), Box::new(bounded(host_end, within).unwrap()), Box::new(move |stop| guest(guest_end, shown, sip, stop)));
            }
            let state = base.join(if sip.is_some() { "s" } else { "n" });
            std::fs::create_dir_all(&state).unwrap();
            let (ready, listening) = mpsc::channel();
            let (shown, at) = (dir.display().to_string(), state.clone());
            let run_guest: Box<dyn FnOnce(mpsc::Receiver<()>) + Send> = Box::new(move |stop| {
                let listener = crate::broker_transport::Listener::bind(&at).unwrap();
                ready.send(()).unwrap();
                guest(listener.accept().unwrap(), shown, sip, stop)
            });
            let (stop_fwd, stop_rx) = mpsc::channel::<()>();
            let g = std::thread::spawn(move || run_guest(stop_rx));
            listening.recv().unwrap();
            let conn = bounded_by_name(crate::broker_transport::connect(&state).unwrap(), within).unwrap();
            check(&format!("{name}, by name"), Box::new(conn), Box::new(move |stop: mpsc::Receiver<()>| {
                let _ = stop.recv();
                let _ = stop_fwd.send(());
                g.join().unwrap();
            }));
        }
        let _ = std::fs::remove_dir_all(&base);
    }

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
