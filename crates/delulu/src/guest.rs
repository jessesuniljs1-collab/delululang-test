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
    let exe = std::env::current_exe()?;
    let dir = std::env::temp_dir().join(format!("delulu-guest-{}-{}", std::process::id(), channel_tag()));
    std::fs::create_dir_all(&dir)?;
    let limits = crate::jail::Limits::default();
    let mut cmd = guest_command(&exe, &dir);
    // Where the platform allows it, the limits are in force from the guest's first instruction.
    let before_exec = crate::jail::harden(&mut cmd, limits);
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
    let served = converse(&dir, program, root, seed, fixed_clock_ms);
    // Whatever happened on the channel, the child is not left running and the channel is removed.
    let status = child.wait();
    let _ = std::fs::remove_dir_all(&dir);
    let status = status?;
    match served {
        Ok(exit) => Ok(exit),
        // A guest that dies without saying goodbye is a failure, never a silent success.
        Err(e) if status.success() => Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("the guest stopped mid-conversation: {e}"),
        )),
        Err(_) => Ok(status.code().unwrap_or(1)),
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
fn guest_command(exe: &std::path::Path, dir: &std::path::Path) -> std::process::Command {
    let mut cmd = std::process::Command::new(exe);
    cmd.arg(GUEST_SUBCOMMAND).arg(dir);
    cmd.env_clear();
    // Windows loads a process's DLLs through `PATH` and the system directories: with an entirely
    // empty environment the guest dies at 0xC0000135, DLL not found, before it runs a line. These
    // four are what the loader needs, and none of them is authority.
    #[cfg(windows)]
    for name in ["SystemRoot", "SystemDrive", "WINDIR", "PATH"] {
        if let Ok(v) = std::env::var(name) {
            cmd.env(name, v);
        }
    }
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::null());
    cmd
}

/// Connect to the guest, tell it what to run, and serve it until it is done.
fn converse(
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
            Err(e) if std::time::Instant::now() >= deadline => return Err(e),
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
        let cmd = guest_command(std::path::Path::new("delulu"), std::path::Path::new("chan"));
        let names: Vec<String> =
            cmd.get_envs().map(|(k, _)| k.to_string_lossy().to_string()).collect();
        let allowed: &[&str] =
            if cfg!(windows) { &["SystemRoot", "SystemDrive", "WINDIR", "PATH"] } else { &[] };
        for n in &names {
            assert!(allowed.contains(&n.as_str()), "the guest would inherit `{n}`");
        }
        // The arguments carry the channel directory and nothing else: no secret is ever on a command
        // line, where every process on the machine can read it.
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert_eq!(args, vec![GUEST_SUBCOMMAND.to_string(), "chan".to_string()]);
    }
}
