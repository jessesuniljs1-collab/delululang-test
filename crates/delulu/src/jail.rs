//! PS-A-04: the OS jail around a sandbox guest.
//!
//! The channel already means the guest holds no DeluluLang authority. The jail is the second half:
//! the operating system refusing what the guest was never given, so a guest that escapes the
//! interpreter — foreign code, a bug, a deliberate exploit — still cannot act.
//!
//! The limits are the owner's ruling D-V2-25: 1 GiB of memory and 5 minutes of processor time, never
//! unlimited, and the operator may change them. Each platform enforces what it actually has, and
//! says so rather than claiming more: `delulu sandbox probe` reports per level from attempts, never
//! from a version string (PS-0-04).

/// What a guest may consume. Never unlimited: a default that cannot be exceeded is the point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub memory_bytes: u64,
    pub cpu_seconds: u64,
    /// `--limits wall=SECONDS`, the operator's wall-clock ceiling (SANDBOX-STOP-1). No default, as for
    /// an ordinary run (D-V2-25 ruled memory and processor time): a process guest gets it from the
    /// host's watchdog, a microVM as the tighter of it and the VM's own ceiling.
    pub wall_seconds: Option<u64>,
}

impl Default for Limits {
    /// D-V2-25 (owner, 2026-09-18): 1 GiB and 5 minutes of processor time.
    fn default() -> Self {
        Limits { memory_bytes: 1024 * 1024 * 1024, cpu_seconds: 300, wall_seconds: None }
    }
}

/// What the jail actually enforces on this host, in the words the run report and `doctor` use. Only
/// what was applied — a guarantee that was not measured is not a guarantee (PS-0-04's rule).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Enforced {
    pub guarantees: Vec<&'static str>,
}

// `Jail` is exported because it is `confine`'s return type, not because callers name it: they hold
// the value and let it drop, which is what fires kill-on-close.
#[cfg(windows)]
#[allow(unused_imports)]
pub use windows_jail::{confine, end_with_host, CpuProbe, Jail};

#[cfg(not(windows))]
#[allow(unused_imports)]
pub use other::{confine, Jail};

/// Harden the child BEFORE it is spawned, where the platform allows it.
///
/// On Linux this is the strongest moment there is: the hooks run in the forked child, before `exec`,
/// so the limits are in force from the guest's first instruction rather than a moment after it. The
/// kernel enforces them whatever the guest later does, including if it escapes the interpreter.
///
/// Returns what was applied, for the run's report. Nothing is claimed that was not requested of the
/// kernel, and `no_new_privs` is set first so a later seccomp filter cannot be side-stepped through
/// a setuid binary (PS-A2 adds Landlock and seccomp proper with the crates the owner approved).
#[cfg(target_os = "linux")]
pub fn harden(cmd: &mut std::process::Command, limits: Limits) -> Vec<&'static str> {
    use std::os::unix::process::CommandExt as _;
    // SAFETY: `pre_exec` runs in the forked child before `exec` and calls only async-signal-safe
    // functions (`prctl`, `setrlimit`). A failure leaves the guest less confined, never more, and the
    // report below says only what was asked of the kernel.
    unsafe {
        cmd.pre_exec(move || {
            // A guest must never gain privileges through a setuid binary, and this must be set
            // before any filter that a setuid exec could otherwise escape.
            libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
            // Killed with the host, as the Windows job's kill-on-close does.
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL as libc::c_ulong, 0, 0, 0);
            // The resource argument is unsigned on glibc and signed on musl, so it is cast at the
            // call rather than typed here. (Typed as `c_int`, this did not compile for Linux at all —
            // caught by type-checking the module in WSL instead of letting CI find it.)
            let set = |res, value: libc::rlim_t| {
                let lim = libc::rlimit { rlim_cur: value, rlim_max: value };
                libc::setrlimit(res, &lim);
            };
            // `RLIMIT_DATA`, not `RLIMIT_AS`. The memory ceiling is about memory the guest USES, and
            // `RLIMIT_AS` caps the address space it RESERVES — which this binary does by the
            // gigabyte before `main`, because the WebAssembly engine reserves guard regions up
            // front. A 1 GiB `RLIMIT_AS` therefore killed the guest at startup, silently, and the
            // host sat out its whole connect deadline (CI run 35391962354).
            set(libc::RLIMIT_DATA, limits.memory_bytes as libc::rlim_t);
            set(libc::RLIMIT_CPU, limits.cpu_seconds as libc::rlim_t);
            // No core dump: a crash must not spill the guest's memory onto the disk, where it would
            // outlive the run and the scope it was granted.
            set(libc::RLIMIT_CORE, 0);
            // NOT `RLIMIT_NPROC`. It counts every process AND thread belonging to the whole user,
            // not this process's children, so a value of 1 fails whatever else that user is already
            // running and stops the guest creating a thread at all — CI run 35390738394 red on both
            // Linux jobs. "No new processes" is seccomp's job (it refuses `fork`/`clone` of a new
            // process), and it arrives with that layer; claiming it here would have been a guarantee
            // this code does not make.
            Ok(())
        });
    }
    vec![
        "memory ceiling",
        "processor-time ceiling",
        "no privilege escalation",
        "no core dump",
        "killed with the host",
    ]
}

/// The macOS jail: the guest runs under a DENY-DEFAULT Seatbelt profile. Everything is refused
/// except the four things a guest provably needs, and each of those four was measured on a runner
/// rather than reasoned about.
///
/// This replaces an allow-default profile with targeted denies. That earlier shape existed because a
/// deny-default one aborted even a plain C program before it reached its socket (experiment run
/// 35391102215, exit 134 = SIGABRT). That was a real measurement of an allow list that was too
/// SHORT, not a limit of Seatbelt, and the missing clause turned out to be `(allow file-read*)`:
/// with it, the real guest — CPython, wasmtime, threads and all — starts and binds its channel under
/// `(deny default)` (run 35478590757, `RESULT real-reads-everything: PASS rc=142`, where 142 is
/// 128+SIGALRM, the probe's own deadline firing while the guest waited for a hello it would never
/// get; it was alive and listening).
///
/// Then every other clause was TRIMMED, one at a time, and only the ones whose removal broke the
/// guest are here (run 35479148216):
///
/// | clause | verdict |
/// |---|---|
/// | `process-exec` on itself | load-bearing — without it `execvp` is refused (rc 71) |
/// | the socket literal, read and write | load-bearing — without it the guest cannot open its channel |
/// | `sysctl-read` | load-bearing — Rust's stack-overflow guard page needs it, and without it the guest panics with "failed to allocate a guard page" |
/// | `file-read*` | load-bearing — without it, abort |
/// | `file-map-executable` | NOT needed — dropped |
/// | `mach-lookup` | NOT needed — dropped, so the guest reaches no Mach service at all |
/// | `signal`, `process-info*`, `ipc-posix-shm` on self | NOT needed — dropped |
/// | `file-read-metadata` | NOT needed — dropped |
/// | write access to the channel DIRECTORY | NOT needed — the socket literal alone is enough |
///
/// What is NOT narrowed, and is said rather than implied: **reads**. Confining them by subpath aborts
/// the guest, and it aborts with every additional root the experiment tried — `/private/var/db`,
/// `/private/var/folders`, all of `/private/var`, `/opt`, `/private/tmp`. So on macOS the guest can
/// still READ the filesystem, and only the other layers stop it doing anything with what it read: it
/// cannot write, cannot reach the network, and cannot start a program to carry anything out. Linux is
/// tighter here, because Landlock does confine reads (see [`confine_filesystem`]).
///
/// **One read IS refused, since PS-B-03: the state directory.** T14 asks that a guest cannot read the
/// broker's key or the state directory, and with `(allow file-read*)` alone it could. The narrowing
/// that aborted the guest was an ALLOW-list of readable roots; this is the opposite shape, one DENY
/// after the broad allow (Seatbelt applies the last rule that matches), naming a directory the guest
/// never needs. Both spellings are named, for the reason given below for the socket.
/// `macos_tests::the_state_directory_is_unreadable_to_a_seatbelted_guest` measures it with a control.
///
/// **This profile is fail-closed on purpose.** It was measured on macOS 26.6.2 (arm64). If a future
/// release needs an allowance that is not here, the guest will fail to start and the run will REFUSE,
/// which is the direction D-V2-25 requires — never a quiet fall back to a weaker profile. The fix is
/// one more clause, added with the run that proved it necessary.
///
/// Returns the command to spawn instead, or `None` when this host cannot apply a profile at all.
#[cfg(target_os = "macos")]
pub fn seatbelt_launcher(
    exe: &std::path::Path,
    dir: &std::path::Path,
    args: &[&std::ffi::OsStr],
    unreadable: &[std::path::PathBuf],
) -> Option<(std::process::Command, Vec<&'static str>)> {
    if !std::path::Path::new("/usr/bin/sandbox-exec").is_file() {
        return None;
    }
    // Seatbelt matches the RESOLVED path, and macOS hands out temp directories at `/var/folders/…`
    // which resolve to `/private/var/folders/…`. A profile naming the unresolved spelling matches
    // nothing, so the guest's own bind came back EPERM (CI run 35394515395) even though the rule
    // itself is right. This is the campaign's recurring shape — a security decision made on an
    // unnormalized representation — so both spellings of both paths are named.
    let raw_sock = dir.join("broker.sock");
    let resolved_sock =
        std::fs::canonicalize(dir).map(|d| d.join("broker.sock")).unwrap_or_else(|_| raw_sock.clone());
    let raw_exe = exe.to_path_buf();
    let resolved_exe = std::fs::canonicalize(exe).unwrap_or_else(|_| raw_exe.clone());
    let p = |x: &std::path::Path| x.display().to_string();
    let mut profile = format!(
        "(version 1)\n\
         (deny default)\n\
         (allow process-exec (literal \"{}\") (literal \"{}\"))\n\
         (allow network-bind network-outbound (literal \"{}\") (literal \"{}\"))\n\
         (allow file-read* file-write* (literal \"{}\") (literal \"{}\"))\n\
         (allow sysctl-read)\n\
         (allow file-read*)\n",
        p(&raw_exe),
        p(&resolved_exe),
        p(&raw_sock),
        p(&resolved_sock),
        p(&raw_sock),
        p(&resolved_sock),
    );
    // After the broad allow, so it wins. A path Seatbelt could misread (a quote or a backslash in
    // it) is not written into the profile at all; the guarantee is then not claimed.
    let mut denied_any = false;
    for d in unreadable {
        let resolved = std::fs::canonicalize(d).unwrap_or_else(|_| d.clone());
        for spelling in [d.clone(), resolved] {
            let s = p(&spelling);
            if s.contains('"') || s.contains('\\') {
                continue;
            }
            profile.push_str(&format!("(deny file-read* file-write* (subpath \"{s}\"))\n"));
            denied_any = true;
        }
    }
    let path = dir.join("guest.sb");
    std::fs::write(&path, profile).ok()?;
    let mut cmd = std::process::Command::new("/usr/bin/sandbox-exec");
    cmd.arg("-f").arg(&path).arg(exe).args(args);
    let mut applied = vec![
        "deny by default",
        "no file writes",
        "no network but the channel",
        "no new programs",
        "no Mach services",
        "no signals or process info beyond itself",
    ];
    if denied_any {
        applied.push("the state directory unreadable");
    }
    Some((cmd, applied))
}

/// Windows hardens at the spawn: the child is created SUSPENDED so its Job Object is in place before
/// its first instruction rather than a moment after it. `resume` starts it once `confine` has applied
/// the job, and the guarantees are reported from there.
#[cfg(windows)]
pub fn harden(cmd: &mut std::process::Command, _limits: Limits) -> Vec<&'static str> {
    use std::os::windows::process::CommandExt as _;
    const CREATE_SUSPENDED: u32 = 0x0000_0004;
    cmd.creation_flags(CREATE_SUSPENDED);
    Vec::new()
}

/// macOS had NO time bound at all, and that was a real gap rather than a cosmetic one.
///
/// A guest normally dies with its host because the channel closes under it — the next read fails and
/// it exits. But a guest that is COMPUTING and asking for nothing never notices the host is gone, and
/// the channel it would fail on is idle. On Linux `PR_SET_PDEATHSIG` and `RLIMIT_CPU` both cover that;
/// on Windows the Job Object's kill-on-close does. On macOS there is no `PDEATHSIG`, so the
/// processor-time ceiling WAS the bound on a spinning orphan — and without it the bound was "for ever".
/// Since PS-E-02 (D-V2-60) a watcher outside the guest ends it with its host ([`HostWatch`]); the ceiling
/// stays the bound should the watcher itself be gone.
///
/// Both claims are measured, and the one that is not claimed is measured too (experiment run
/// 35480762820, `macos-rlimit-enforcement`):
///
/// - `RLIMIT_CPU` **is** enforced: a spinning C program with a one-second limit was killed by SIGXCPU
///   after one second, against a control that ran unlimited for sixteen. So the processor-time ceiling
///   is a real bound on a spinning orphan, which is what it is here for.
/// - `RLIMIT_DATA` is **refused**: `setrlimit` returns EINVAL on macOS. So there is no memory ceiling
///   to claim, and no call left in the code pretending to ask for one — a call the OS rejects is worse
///   than an absent call, because the next reader assumes it worked.
///
/// The Linux half of this file carries the matching scar: `RLIMIT_AS` capped the wasm engine's
/// RESERVATIONS rather than its use and killed the guest before `main` (CI run 35391962354).
#[cfg(target_os = "macos")]
pub fn harden(cmd: &mut std::process::Command, limits: Limits) -> Vec<&'static str> {
    use std::os::unix::process::CommandExt as _;
    // SAFETY: `pre_exec` runs in the forked child before `exec` and calls only `setrlimit`, which is
    // async-signal-safe. A failure leaves the guest less confined, never more.
    unsafe {
        cmd.pre_exec(move || {
            let set = |res, value: libc::rlim_t| {
                let lim = libc::rlimit { rlim_cur: value, rlim_max: value };
                libc::setrlimit(res, &lim);
            };
            set(libc::RLIMIT_CPU, limits.cpu_seconds as libc::rlim_t);
            set(libc::RLIMIT_CORE, 0);
            // No `RLIMIT_DATA`. It is not "set and not claimed" — it is REFUSED: on macOS the call
            // returns EINVAL (`setrlimit data: Invalid argument`, experiment run 35480762820), so it
            // was a line that looked like a memory ceiling and was not one. A call the OS rejects is
            // worse than an absent call, because the next reader assumes it worked.
            Ok(())
        });
    }
    vec!["processor-time ceiling", "no core dump"]
}

/// PS-E-02 (`V2_OPENSHELL_STUDY.md` §4.2, D-V2-60): on macOS nothing in the kernel ends a guest when its
/// host dies — there is no `PR_SET_PDEATHSIG` and no job object — so a guest that computes and asks for
/// nothing outlived a killed host until its processor-time ceiling (witnessed red on a macOS runner,
/// `witness.yml`). A WATCHER takes the kernel's place: a small process of this binary, started by the
/// host right after the guest, that waits on two things with `kqueue` — its standard input, a pipe only
/// the host holds (end of file: the host is gone, by dying or by finishing), and the guest's exit
/// (`EVFILT_PROC`/`NOTE_EXIT`). The host gone first ends the guest with SIGKILL; the guest gone first ends
/// the watcher. Dropping the [`Jail`] that holds it closes the pipe, so — as the Windows job's
/// kill-on-close does — a guest the host is finished with never outlives it.
///
/// It runs OUTSIDE the guest, not as a thread inside it (the design's first shape): a watcher in the
/// guest is the guest's own word, and a guest that escapes the interpreter could stop it. Outside, the
/// guest cannot reach it — its Seatbelt profile denies every signal. The guest's exit is registered
/// while the host still holds the guest unreaped, so the pid it would kill is the guest's and no other.
/// The guarantee is claimed only once the watcher says it is armed ([`HostWatch::start`]).
#[cfg(target_os = "macos")]
pub const HOST_WATCH_SUBCOMMAND: &str = "__host_watch";

/// The byte a watcher writes once both of its waits are registered.
#[cfg(target_os = "macos")]
const WATCH_ARMED: u8 = 0x06;

/// The watcher's exit status when the process it was given had already ended (`ESRCH`): nothing to watch.
#[cfg(target_os = "macos")]
const WATCH_NOTHING: i32 = 3;

/// A running watcher, owned by the host for as long as it owns the guest.
#[cfg(target_os = "macos")]
pub struct HostWatch {
    pipe: Option<std::process::ChildStdin>,
    watcher: std::process::Child,
}

#[cfg(target_os = "macos")]
impl HostWatch {
    /// Start a watcher for `guest` and wait (within `deadline`) for it to say it is armed. On any failure
    /// the watcher is ended FIRST, then its pipe closed, so a watcher that armed late cannot end a guest
    /// the host goes on serving without the guarantee.
    pub fn start(exe: &std::path::Path, guest: &std::process::Child, deadline: std::time::Duration) -> Result<HostWatch, String> {
        use std::io::Read as _;
        use std::os::fd::AsRawFd as _;
        let mut cmd = std::process::Command::new(exe);
        cmd.arg(HOST_WATCH_SUBCOMMAND)
            .arg(guest.id().to_string())
            .env_clear()
            .current_dir("/")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null());
        for name in crate::guest::LOADER_ENV {
            if let Ok(v) = std::env::var(name) {
                cmd.env(name, v);
            }
        }
        let mut watcher = cmd.spawn().map_err(|e| format!("the watcher could not be started: {e}"))?;
        let pipe = watcher.stdin.take();
        let armed = watcher.stdout.take().is_some_and(|mut out| {
            let mut p = libc::pollfd { fd: out.as_raw_fd(), events: libc::POLLIN, revents: 0 };
            let ms = deadline.as_millis().min(i32::MAX as u128) as libc::c_int;
            // SAFETY: one `pollfd` on a descriptor this function owns, for the length given.
            let ready = unsafe { libc::poll(&mut p, 1, ms) } == 1;
            let mut byte = [0u8; 1];
            ready && matches!(out.read(&mut byte), Ok(1)) && byte[0] == WATCH_ARMED
        });
        if !armed {
            // Say which: a watcher that EXITED found nothing to watch or could not arm; one still running
            // did not answer in time.
            let why = match watcher.try_wait() {
                Ok(Some(st)) if st.code() == Some(WATCH_NOTHING) => "the process it was to watch had already ended".to_string(),
                Ok(Some(st)) => format!("the watcher could not arm ({st})"),
                _ => format!("the watcher did not say it was armed within {deadline:?}"),
            };
            let _ = watcher.kill();
            let _ = watcher.wait();
            drop(pipe);
            return Err(why);
        }
        Ok(HostWatch { pipe, watcher })
    }
}

#[cfg(target_os = "macos")]
impl Drop for HostWatch {
    fn drop(&mut self) {
        // The host is done with the guest: the pipe's end of file tells the watcher so. A guest already
        // reaped was seen exiting first, and the watcher has gone or goes without acting.
        drop(self.pipe.take());
        let until = std::time::Instant::now() + std::time::Duration::from_secs(2);
        loop {
            match self.watcher.try_wait() {
                Ok(Some(_)) | Err(_) => return,
                Ok(None) if std::time::Instant::now() >= until => {
                    let _ = self.watcher.kill();
                    let _ = self.watcher.wait();
                    return;
                }
                Ok(None) => std::thread::sleep(std::time::Duration::from_millis(5)),
            }
        }
    }
}

/// `__host_watch <guest pid>` — the watcher's side of [`HostWatch`]. Exit 0 when it has done its work
/// (the guest gone, or ended because the host was), 1 when it could not arm, 2 on a bad invocation, and
/// [`WATCH_NOTHING`] when the process it was given had already ended.
#[cfg(target_os = "macos")]
pub fn run_host_watch(args: &[String]) -> i32 {
    let [guest] = args else { return 2 };
    let Ok(guest) = guest.parse::<libc::pid_t>() else { return 2 };
    if guest <= 1 {
        return 2;
    }
    let event = |ident: usize, filter: i16, fflags: u32| libc::kevent {
        ident,
        filter,
        flags: libc::EV_ADD,
        fflags,
        data: 0,
        udata: std::ptr::null_mut(),
    };
    // SAFETY: a kqueue this process owns, its own standard input and a plain `kill`; every buffer passed
    // is a local array of the length given.
    unsafe {
        let kq = libc::kqueue();
        if kq < 0 {
            return 1;
        }
        let host = [event(0, libc::EVFILT_READ, 0)];
        if libc::kevent(kq, host.as_ptr(), 1, std::ptr::null_mut(), 0, std::ptr::null()) != 0 {
            return 1;
        }
        let exit = [event(guest as usize, libc::EVFILT_PROC, libc::NOTE_EXIT)];
        if libc::kevent(kq, exit.as_ptr(), 1, std::ptr::null_mut(), 0, std::ptr::null()) != 0 {
            // ESRCH: the guest is already gone — a launcher that exits at once — and there is nothing to watch.
            return if std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) { WATCH_NOTHING } else { 1 };
        }
        if libc::write(1, [WATCH_ARMED].as_ptr().cast(), 1) != 1 {
            return 1;
        }
        libc::close(1);
        let mut got: [libc::kevent; 2] = std::mem::zeroed();
        loop {
            let n = libc::kevent(kq, std::ptr::null(), 0, got.as_mut_ptr(), 2, std::ptr::null());
            if n < 0 {
                if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                // The kqueue failed under the watcher: the host is still here to end its guest itself.
                return 1;
            }
            let seen = &got[..n as usize];
            // The guest's exit first, whatever else arrived with it: then there is no guest to end.
            if seen.iter().any(|e| e.filter == libc::EVFILT_PROC) {
                return 0;
            }
            if seen.iter().any(|e| e.filter == libc::EVFILT_READ && e.flags & libc::EV_EOF != 0) {
                // One last look, without waiting, for an exit that raced the end of file.
                let zero = libc::timespec { tv_sec: 0, tv_nsec: 0 };
                let late = libc::kevent(kq, std::ptr::null(), 0, got.as_mut_ptr(), 2, &zero);
                if late > 0 && got[..late as usize].iter().any(|e| e.filter == libc::EVFILT_PROC) {
                    return 0;
                }
                libc::kill(guest, libc::SIGKILL);
                return 0;
            }
            // Bytes on the pipe: the host writes none, so anything there is read and dropped.
            let mut sink = [0u8; 64];
            if libc::read(0, sink.as_mut_ptr().cast(), sink.len()) == 0 {
                libc::kill(guest, libc::SIGKILL);
                return 0;
            }
        }
    }
}

/// Anywhere else the guest is confined only by the channel, and the run says so rather than implying
/// a boundary this build does not have.
#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub fn harden(_cmd: &mut std::process::Command, _limits: Limits) -> Vec<&'static str> {
    Vec::new()
}

/// PS-A2: the guest narrows its own VIEW OF THE FILESYSTEM, with Landlock, before the program runs.
///
/// seccomp says which syscalls a guest may make; Landlock says which files those syscalls may
/// reach. They answer different questions, and the filesystem one is the one that matters most
/// here: the guest performs no effects, so it needs to open no file of the operator's at all. Every
/// read and every write the program asks for is performed by the HOST, inside the scope the
/// operator granted, and arrives back over the channel.
///
/// So the ruleset is: nothing writable anywhere, reads only from the system paths a process needs in
/// order to keep running, and no TCP. A guest that escapes the interpreter entirely still cannot
/// open the operator's home directory, cannot truncate a file it can no longer open, and cannot
/// carry what it read out over a socket of its own.
///
/// Two things are deliberately NOT claimed. Landlock mediates TCP only (ABI v4), so UDP and raw
/// sockets are outside it and the guarantee says `TCP` rather than `network`. And what the kernel
/// actually supports is read back from the syscall — never from a version string — so an older
/// kernel yields a shorter list rather than the same list with less behind it (PS-0-04's rule).
///
/// The channel socket is already connected by the time this runs, and an open file descriptor is
/// not re-checked, so the guest keeps talking to its host with nothing writable beneath it.
///
/// Returns what was applied, and `None` only when this kernel has no Landlock at all — in which
/// case the caller SAYS so, because "the sandbox quietly did not apply" is the failure this phase
/// exists to prevent. It is not fatal: seccomp, the rlimits and the channel are unaffected.
#[cfg(target_os = "linux")]
pub fn confine_filesystem(channel_dir: Option<&std::path::Path>) -> Option<Vec<&'static str>> {
    use landlock::{
        path_beneath_rules, Access, AccessFs, AccessNet, LandlockStatus, Ruleset, RulesetAttr,
        RulesetCreatedAttr, ABI,
    };

    // Read-only, and only what a running process needs: its loader and libraries, the system
    // configuration a libc call may consult, `/sys`, its OWN `/proc` entry, and the null, zero and
    // random devices. Nothing under `/home`, `/root`, `/tmp` or a workspace — that is the operator's
    // data, and the guest has no business reading any of it. Paths that do not exist on this host are
    // skipped by `path_beneath_rules`, which is why `/lib64` may be named on an architecture that has
    // no such directory.
    //
    // PS-E-03 (`V2_OPENSHELL_STUDY.md` §4.3), each witnessed by the escaped guest (`escaped_tests`)
    // before it was narrowed:
    // - H3 (GUEST-PROC-1): this named `/proc`, and an escaped guest read another process of the same
    //   user's `environ` and `cmdline` — an API key in a shell's environment — because the read-mode
    //   ptrace check passes for the same uid. `/proc/self` is resolved when the rule is made, in the
    //   guest itself, so the rule is the guest's own entry and no other. Traced: after it locks itself
    //   down, a guest running a program opens no file at all, so nothing global under `/proc` is needed.
    // - H7 (GUEST-DEV-1, found while building H3): this named `/dev`, so an escaped same-user guest
    //   could open the operator's terminal (`/dev/pts/N`) for reading — keystrokes. The devices a
    //   process may reasonably read are named one by one.
    const SYSTEM_READ: &[&str] = &[
        "/usr", "/lib", "/lib64", "/lib32", "/bin", "/sbin", "/etc", "/sys", "/proc/self", "/dev/null", "/dev/zero",
        "/dev/full", "/dev/random", "/dev/urandom",
    ];

    // The rights are requested at ABI v3 for the filesystem, because v3 is where `truncate` became
    // mediated: without it a guest could still empty a file it can no longer open, and "no file
    // writes" would be a claim this code does not make. Network rights arrive at v4. Both are
    // requested best-effort — the default — so an older kernel drops what it lacks, and the
    // guarantees below are chosen from what it actually reported.
    let fs_abi = ABI::V3;
    let net_abi = ABI::V4;
    let ruleset = Ruleset::default()
        .handle_access(AccessFs::from_all(fs_abi))
        .and_then(|r| r.handle_access(AccessNet::from_all(net_abi)))
        .and_then(|r| r.create())
        // No writable rule at all: the whole point is that this list is empty.
        .and_then(|r| r.add_rules(path_beneath_rules(SYSTEM_READ, AccessFs::from_read(fs_abi))))
        // The channel directory, read-only, so a guest can still stat the socket it is already
        // talking through. It is named rather than assumed, as the Seatbelt profile names it. A guest
        // born holding its socket (PS-B-03b) has no directory, and gets no rule.
        .and_then(|r| r.add_rules(path_beneath_rules(channel_dir, AccessFs::from_read(fs_abi))))
        // No TCP port is ever added, so every bind and every connect is refused.
        .and_then(|r| r.restrict_self());
    let status = match ruleset {
        Ok(s) => s,
        // A ruleset that could not be built or installed confines nothing. Say nothing, claim
        // nothing: the caller reports the absence.
        Err(_) => return None,
    };
    let effective = match status.landlock {
        LandlockStatus::Available { effective_abi, .. } => effective_abi,
        // The kernel has no Landlock, or it is not enabled in `CONFIG_LSM`.
        LandlockStatus::NotEnabled | LandlockStatus::NotImplemented => return None,
    };
    if effective < ABI::V1 {
        return None;
    }
    let mut applied = vec![
        // True from v1 up: no rule grants any write right, so nothing anywhere can be created,
        // opened for writing, renamed, linked or removed.
        if effective >= ABI::V3 { "no file writes" } else { "no file writes but truncation" },
        "reads only from the system paths",
    ];
    if effective >= ABI::V4 {
        applied.push("no TCP bind or connect");
    }
    Some(applied)
}

/// The guest locks ITSELF down once it has connected and been told what to run (PS-A-04).
///
/// This is the moment the filter can be strictest: the guest has its channel, and from here it needs
/// no new program, no debugger and no namespace of its own — it interprets, and asks the host for
/// every effect. Applying it earlier is impossible, because the syscalls denied here are exactly the
/// ones a process needs in order to become the guest at all.
///
/// A denylist rather than an allowlist, deliberately: an allowlist of everything a Rust program may
/// call is long, host-dependent, and fails closed in the worst way — by killing ordinary runs on a
/// libc version nobody tested. What is denied here is what a guest has no business doing at all.
///
/// Fails closed: a host that cannot install the filter refuses the run and says so, because
/// "the sandbox quietly did not apply" is the failure this phase exists to prevent.
#[cfg(target_os = "linux")]
pub fn lock_down_self() -> Result<Vec<&'static str>, String> {
    use seccompiler::{SeccompAction, SeccompFilter};
    use std::collections::BTreeMap;

    // Each of these is a capability a guest never legitimately needs: starting another program,
    // attaching a debugger to one, rearranging namespaces or mounts, loading kernel code, or reading
    // and writing another process's memory.
    // 64-bit ARM has no `fork` or `vfork` syscall at all — everything goes through `clone` there —
    // so naming them unconditionally does not compile for that architecture (the arm64 job caught
    // it). Denying what does not exist is not a boundary anyway.
    #[cfg(target_arch = "x86_64")]
    const ARCH_DENIED: &[libc::c_long] = &[libc::SYS_fork, libc::SYS_vfork];
    #[cfg(not(target_arch = "x86_64"))]
    const ARCH_DENIED: &[libc::c_long] = &[];

    let denied: Vec<libc::c_long> = [
        // PS-E-03 (`V2_OPENSHELL_STUDY.md` §4.3), each witnessed by the escaped guest (`escaped_tests`):
        // H2 (GUEST-SOCKET-1) — no new socket of any family. Landlock mediates TCP only, and an escaped
        // guest opened UDP, netlink and Unix sockets and CONNECTED to the operator's own socket (an SSH
        // agent's, a session bus's). The guest's channel is connected before this runs, and it needs no
        // other socket: every effect is the host's.
        libc::SYS_socket,
        libc::SYS_socketpair,
        // H1 (GUEST-SYSCALL-1) — calls the filter did not name: anonymous memory files, io_uring (its own
        // path to the kernel's operations), userfaultfd, process file descriptors, the new mount API and
        // the other `kexec`. None is needed to interpret a program.
        libc::SYS_memfd_create,
        libc::SYS_io_uring_setup,
        libc::SYS_io_uring_enter,
        libc::SYS_io_uring_register,
        libc::SYS_userfaultfd,
        libc::SYS_pidfd_open,
        libc::SYS_pidfd_getfd,
        libc::SYS_pidfd_send_signal,
        libc::SYS_fsopen,
        libc::SYS_fsconfig,
        libc::SYS_fsmount,
        libc::SYS_fspick,
        libc::SYS_move_mount,
        libc::SYS_open_tree,
        libc::SYS_mount_setattr,
        libc::SYS_kexec_file_load,
        // H10 (GUEST-PROCESS-1): the calls that ACT on another process of the same user and name it by pid —
        // its priority, CPUs, scheduling class, I/O priority, memory advice. An escaped guest set each one
        // on the operator's process; a guest sets none of them even on itself. (`prlimit64` is ruled below:
        // the C library reads its own limits through it.)
        libc::SYS_setpriority,
        libc::SYS_sched_setaffinity,
        libc::SYS_sched_setscheduler,
        libc::SYS_sched_setparam,
        libc::SYS_sched_setattr,
        libc::SYS_ioprio_set,
        libc::SYS_process_madvise,
        libc::SYS_execve,
        libc::SYS_execveat,
        libc::SYS_ptrace,
        libc::SYS_unshare,
        libc::SYS_setns,
        libc::SYS_mount,
        libc::SYS_umount2,
        libc::SYS_pivot_root,
        libc::SYS_chroot,
        libc::SYS_init_module,
        libc::SYS_finit_module,
        libc::SYS_delete_module,
        libc::SYS_kexec_load,
        libc::SYS_bpf,
        libc::SYS_perf_event_open,
        libc::SYS_process_vm_readv,
        libc::SYS_process_vm_writev,
        libc::SYS_open_by_handle_at,
    ]
    .into_iter()
    .chain(ARCH_DENIED.iter().copied())
    .collect();
    // `c_long` IS `i64` on every Linux target this builds for, so a cast here is not just redundant,
    // it is a lint error under `-D warnings`.
    let mut rules: BTreeMap<i64, Vec<seccompiler::SeccompRule>> =
        denied.iter().map(|s| (*s, Vec::new())).collect();
    // H1: `unshare` was denied, and `clone` with a namespace flag did the same thing — an escaped guest
    // made a user namespace with it. `clone` itself stays, because threads are made with it: only its
    // namespace flags are refused, one rule per flag (a call matches if ANY rule does). `CLONE_NEWTIME`
    // is not among them: in `clone` its bit is part of the exit signal.
    let namespaces = [
        libc::CLONE_NEWNS,
        libc::CLONE_NEWCGROUP,
        libc::CLONE_NEWUTS,
        libc::CLONE_NEWIPC,
        libc::CLONE_NEWUSER,
        libc::CLONE_NEWPID,
        libc::CLONE_NEWNET,
    ];
    let clone_rules = namespaces
        .iter()
        .map(|f| {
            let flag = *f as u64;
            seccompiler::SeccompCondition::new(
                0,
                seccompiler::SeccompCmpArgLen::Qword,
                seccompiler::SeccompCmpOp::MaskedEq(flag),
                flag,
            )
            .and_then(|c| seccompiler::SeccompRule::new(vec![c]))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("the syscall filter could not be built: {e}"))?;
    rules.insert(libc::SYS_clone, clone_rules);
    // H8 (GUEST-TIOCSTI-1, found with the escaped guest): a guest shares its host's session, so the
    // operator's terminal is its controlling terminal and its standard error, and `TIOCSTI` pushed a
    // keystroke into that terminal's input — for the operator's shell to read and RUN once the guest is
    // gone (`dev.tty.legacy_tiocsti` is 1 here, as on many hosts). `TIOCLINUX` can paste a selection on
    // a Linux console the same way. Compared on the low 32 bits, because the kernel truncates the command
    // to them: a caller setting the high bits must not walk around the rule.
    // `libc::Ioctl` is `c_ulong` on glibc and `c_int` on musl — the microVM's static guest — so the cast is
    // needed there and redundant here.
    #[allow(clippy::unnecessary_cast)]
    let ioctl_rules = [libc::TIOCSTI as u64, libc::TIOCLINUX as u64]
        .iter()
        .map(|cmd| {
            seccompiler::SeccompCondition::new(1, seccompiler::SeccompCmpArgLen::Dword, seccompiler::SeccompCmpOp::Eq, *cmd)
                .and_then(|c| seccompiler::SeccompRule::new(vec![c]))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("the syscall filter could not be built: {e}"))?;
    rules.insert(libc::SYS_ioctl, ioctl_rules);
    // H9 (GUEST-SIGNAL-1, found with the escaped guest): nothing stopped a signal to another process of the
    // same user — one process, the guest's whole process group (its host and the terminal's foreground
    // job among them), or a queued signal; `kill(-1, SIGKILL)` would end every process the operator has.
    // A guest signals only itself (`abort` is a signal to itself), so every call that names a process is
    // refused unless it names THIS one — its pid, fixed when the filter is made, compared on 32 bits as
    // `pid_t` is. `tkill`, which names a thread by id alone, is refused outright: `raise` uses `tgkill`.
    // SAFETY: `getpid` cannot fail.
    let own = unsafe { libc::getpid() } as u64;
    let not_self = || {
        seccompiler::SeccompCondition::new(0, seccompiler::SeccompCmpArgLen::Dword, seccompiler::SeccompCmpOp::Ne, own)
            .and_then(|c| seccompiler::SeccompRule::new(vec![c]))
            .map_err(|e| format!("the syscall filter could not be built: {e}"))
    };
    for call in [libc::SYS_kill, libc::SYS_tgkill, libc::SYS_rt_sigqueueinfo, libc::SYS_rt_tgsigqueueinfo] {
        rules.insert(call, vec![not_self()?]);
    }
    rules.insert(libc::SYS_tkill, Vec::new());
    // H10: `prlimit64` on a process other than itself (0 or its own pid) is refused — an escaped guest lowered
    // the operator's process's limits through it; its own, the C library still reads.
    let not_self_nor_zero = [0, own]
        .iter()
        .map(|v| seccompiler::SeccompCondition::new(0, seccompiler::SeccompCmpArgLen::Dword, seccompiler::SeccompCmpOp::Ne, *v))
        .collect::<Result<Vec<_>, _>>()
        .and_then(seccompiler::SeccompRule::new)
        .map_err(|e| format!("the syscall filter could not be built: {e}"))?;
    rules.insert(libc::SYS_prlimit64, vec![not_self_nor_zero]);
    // Everything else runs; a denied call fails with EPERM rather than killing the process, so the
    // guest reports a refusal instead of vanishing and leaving the host to guess.
    let filter = SeccompFilter::new(
        rules,
        SeccompAction::Allow,
        SeccompAction::Errno(libc::EPERM as u32),
        std::env::consts::ARCH.try_into().map_err(|e| format!("this architecture has no seccomp backend: {e:?}"))?,
    )
    .map_err(|e| format!("the syscall filter could not be built: {e}"))?;
    let program: seccompiler::BpfProgram =
        filter.try_into().map_err(|e| format!("the syscall filter could not be compiled: {e}"))?;
    seccompiler::apply_filter(&program).map_err(|e| format!("the syscall filter could not be installed: {e}"))?;
    // H1: `clone3` carries its flags in memory, where no filter can read them, so it is answered as if
    // the kernel had no such call — ENOSYS — and the C library makes its threads with `clone`, whose
    // flags the rule above does read. A second filter, because its answer differs from the first's.
    let clone3 = SeccompFilter::new(
        [(libc::SYS_clone3, Vec::new())].into_iter().collect(),
        SeccompAction::Allow,
        SeccompAction::Errno(libc::ENOSYS as u32),
        std::env::consts::ARCH.try_into().map_err(|e| format!("this architecture has no seccomp backend: {e:?}"))?,
    )
    .map_err(|e| format!("the syscall filter could not be built: {e}"))?;
    let clone3: seccompiler::BpfProgram =
        clone3.try_into().map_err(|e| format!("the syscall filter could not be compiled: {e}"))?;
    seccompiler::apply_filter(&clone3).map_err(|e| format!("the syscall filter could not be installed: {e}"))?;
    Ok(vec!["no new programs", "no debugger", "no namespace or module tricks", "no sockets but the channel"])
}

/// Elsewhere the guest's confinement is entirely the host's doing (the Job Object, the Seatbelt
/// profile), so there is nothing for it to apply to itself.
#[cfg(not(target_os = "linux"))]
pub fn lock_down_self() -> Result<Vec<&'static str>, String> {
    Ok(Vec::new())
}

/// Start a guest that [`harden`] created suspended. On platforms that do not suspend, this is a
/// no-op and the guest has been running since `spawn`.
///
/// Returns whether the guest is now running: a guest that cannot be resumed must not be waited on.
#[cfg(windows)]
pub fn resume(child: &std::process::Child) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
    };
    use windows_sys::Win32::System::Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME};

    // `std::process::Child` does not hand out the thread handle, so the process's own threads are
    // found the documented way. A suspended process has exactly one.
    let pid = child.id();
    let mut resumed = false;
    // SAFETY: the snapshot and every thread handle opened from it are closed on both paths.
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
        if snap.is_null() {
            return false;
        }
        let mut entry: THREADENTRY32 = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
        let mut ok = Thread32First(snap, &mut entry);
        while ok != 0 {
            if entry.th32OwnerProcessID == pid {
                let th = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                if !th.is_null() {
                    // -1 means the call failed; anything else is the previous suspend count.
                    if ResumeThread(th) != u32::MAX {
                        resumed = true;
                    }
                    CloseHandle(th);
                }
            }
            ok = Thread32Next(snap, &mut entry);
        }
        CloseHandle(snap);
    }
    resumed
}

/// Elsewhere the guest was never suspended, so it is already running.
#[cfg(not(windows))]
pub fn resume(_child: &std::process::Child) -> bool {
    true
}

/// The Windows jail: a Job Object carrying every limit this host will accept.
///
/// `ActiveProcessLimit = 1` is the one that matters most and the one that was actually measured: the
/// PS-0-08 experiment (run 35378619727) showed a grandchild refused with 1816, not enough quota. The
/// first run of that experiment appeared to show the opposite, and it was the probe that was wrong —
/// which is why this comment names the run that proved it.
#[cfg(windows)]
mod windows_jail {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicAccountingInformation, JobObjectBasicUIRestrictions,
        JobObjectExtendedLimitInformation, QueryInformationJobObject, SetInformationJobObject,
        JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_BASIC_UI_RESTRICTIONS,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
        JOB_OBJECT_LIMIT_JOB_TIME, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOB_OBJECT_UILIMIT_DESKTOP, JOB_OBJECT_UILIMIT_EXITWINDOWS,
        JOB_OBJECT_UILIMIT_GLOBALATOMS, JOB_OBJECT_UILIMIT_HANDLES, JOB_OBJECT_UILIMIT_READCLIPBOARD,
        JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS, JOB_OBJECT_UILIMIT_WRITECLIPBOARD,
    };

    /// A held Job Object. Dropping it fires kill-on-close for every process still assigned, so a
    /// guest never outlives the host that was serving it. The first handle is a jailed guest's job, with
    /// its limits and its accounting; the second an external launcher's (PS-E-02, [`end_with_host`]),
    /// whose only limit is kill-on-close and whose accounting nothing reads — a level-3 run claims none.
    pub struct Jail(HANDLE, HANDLE);

    impl Drop for Jail {
        fn drop(&mut self) {
            for h in [self.0, self.1] {
                if !h.is_null() {
                    // SAFETY: a job handle this value owns, closed once.
                    unsafe { CloseHandle(h) };
                }
            }
        }
    }

    impl Jail {
        /// No jail: the guest is confined by something DeluluLang did not apply (PS-D-01, L3).
        pub fn none() -> Jail {
            Jail(std::ptr::null_mut(), std::ptr::null_mut())
        }

        /// The jailed guest's job, for the tests that read its limits back.
        #[cfg(test)]
        pub(super) fn handle(&self) -> HANDLE {
            self.0
        }

        /// What the job measured: processor time (user + kernel) and the peak memory any process in
        /// it committed. The job's own accounting, read after the guest ended — so a stop is named
        /// from the OS's measurement, never guessed from an exit code that does not say
        /// (SANDBOX-STOP-1).
        pub fn usage(&self) -> Option<(std::time::Duration, u64)> {
            if self.0.is_null() {
                return None;
            }
            // SAFETY: structures zeroed and sized with `size_of`; the handle is this value's.
            unsafe {
                let mut acct: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = std::mem::zeroed();
                let mut ext: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                let ok = QueryInformationJobObject(
                    self.0,
                    JobObjectBasicAccountingInformation,
                    &mut acct as *mut _ as *mut core::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                    std::ptr::null_mut(),
                ) != 0
                    && QueryInformationJobObject(
                        self.0,
                        JobObjectExtendedLimitInformation,
                        &mut ext as *mut _ as *mut core::ffi::c_void,
                        std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                        std::ptr::null_mut(),
                    ) != 0;
                if !ok {
                    return None;
                }
                let ticks = (acct.TotalUserTime.max(0) + acct.TotalKernelTime.max(0)) as u64;
                Some((std::time::Duration::from_nanos(ticks.saturating_mul(100)), ext.PeakProcessMemoryUsed as u64))
            }
        }

        /// The job's processor-time accounting, for the host's watchdog thread while the guest runs.
        /// The job's own `PerJobUserTimeLimit` is checked late — a 3 s budget measured at 5–8 s, a 6 s
        /// one at 12.9 s (SANDBOX-CPU-LATE-1) — so the host reads the same accounting and ends the guest
        /// at the budget; the job's limit stays as the backstop.
        pub fn cpu_probe(&self) -> Option<CpuProbe> {
            (!self.0.is_null()).then_some(CpuProbe(self.0 as usize))
        }
    }

    /// A read-only view of a job's processor time from another thread. Valid while the [`Jail`] it
    /// came from is alive: the watchdog holding it is joined before the guest is reaped, and the jail
    /// outlives both.
    #[derive(Clone, Copy)]
    pub struct CpuProbe(usize);

    impl CpuProbe {
        /// User plus kernel time of every process the job has held, as the job accounts it.
        pub fn used(self) -> Option<std::time::Duration> {
            // SAFETY: the structure is zeroed and sized with `size_of`; the handle is the live jail's.
            unsafe {
                let mut acct: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = std::mem::zeroed();
                if QueryInformationJobObject(
                    self.0 as HANDLE,
                    JobObjectBasicAccountingInformation,
                    &mut acct as *mut _ as *mut core::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                    std::ptr::null_mut(),
                ) == 0
                {
                    return None;
                }
                let ticks = (acct.TotalUserTime.max(0) + acct.TotalKernelTime.max(0)) as u64;
                Some(std::time::Duration::from_nanos(ticks.saturating_mul(100)))
            }
        }
    }

    /// Put `child` in a job with `limits`. Returns what was enforced, so the caller reports the
    /// truth rather than the intention.
    pub fn confine(child: &impl std::os::windows::io::AsRawHandle, limits: super::Limits) -> (Jail, super::Enforced) {
        let mut enforced = super::Enforced::default();
        // SAFETY: every call below takes handles this function owns or the child's own handle, and
        // the structures are zeroed and sized with `size_of`.
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return (Jail::none(), enforced);
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
                | JOB_OBJECT_LIMIT_PROCESS_MEMORY
                | JOB_OBJECT_LIMIT_JOB_TIME;
            // One process: the guest runs the interpreter and starts nothing else.
            info.BasicLimitInformation.ActiveProcessLimit = 1;
            info.ProcessMemoryLimit = limits.memory_bytes as usize;
            // Job time is in 100-nanosecond units.
            info.BasicLimitInformation.PerJobUserTimeLimit = (limits.cpu_seconds as i64) * 10_000_000;
            if SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ) == 0
            {
                CloseHandle(job);
                return (Jail::none(), enforced);
            }
            enforced.guarantees.push("one process only");
            enforced.guarantees.push("memory ceiling");
            enforced.guarantees.push("processor-time ceiling");
            enforced.guarantees.push("killed with the host");

            // The desktop, the clipboard and the other user-interface surfaces are not effects the
            // language grants, so a guest has no business reaching them.
            let mut ui: JOBOBJECT_BASIC_UI_RESTRICTIONS = std::mem::zeroed();
            ui.UIRestrictionsClass = JOB_OBJECT_UILIMIT_DESKTOP
                | JOB_OBJECT_UILIMIT_EXITWINDOWS
                | JOB_OBJECT_UILIMIT_GLOBALATOMS
                | JOB_OBJECT_UILIMIT_HANDLES
                | JOB_OBJECT_UILIMIT_READCLIPBOARD
                | JOB_OBJECT_UILIMIT_WRITECLIPBOARD
                | JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS;
            if SetInformationJobObject(
                job,
                JobObjectBasicUIRestrictions,
                &ui as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_BASIC_UI_RESTRICTIONS>() as u32,
            ) != 0
            {
                enforced.guarantees.push("no desktop, clipboard or global atoms");
            }

            if AssignProcessToJobObject(job, child.as_raw_handle() as HANDLE) == 0 {
                // An outer job that forbids nesting, most likely. Nothing is enforced, and the
                // caller must not be told otherwise.
                CloseHandle(job);
                return (Jail::none(), super::Enforced::default());
            }
            (Jail(job, std::ptr::null_mut()), enforced)
        }
    }

    /// PS-E-02 (`V2_OPENSHELL_STUDY.md` §4.2): an external launcher's job — kill-on-close and nothing else.
    /// The launcher, and whatever it starts inside the job, ends when the host is gone, because the job's
    /// only handle closes with the host. Its limits stay the launcher's own (told to it in its
    /// environment), so none is imposed here and none is claimed. The launcher is assigned while it is
    /// still suspended, so nothing it starts can race the assignment.
    pub fn end_with_host(child: &impl std::os::windows::io::AsRawHandle) -> Result<Jail, String> {
        // SAFETY: a job this function creates and either returns or closes; the child's own handle; the
        // structure zeroed and sized with `size_of`.
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return Err(format!("no job object could be created: {}", std::io::Error::last_os_error()));
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ) == 0
            {
                let e = std::io::Error::last_os_error();
                CloseHandle(job);
                return Err(format!("the job refused kill-on-close: {e}"));
            }
            if AssignProcessToJobObject(job, child.as_raw_handle() as HANDLE) == 0 {
                let e = std::io::Error::last_os_error();
                CloseHandle(job);
                return Err(format!("the launcher could not join its job: {e}"));
            }
            Ok(Jail(std::ptr::null_mut(), job))
        }
    }
}

#[cfg(all(test, windows))]
mod windows_tests {
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::JobObjects::{
        IsProcessInJob, QueryInformationJobObject, JobObjectExtendedLimitInformation,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
        JOB_OBJECT_LIMIT_JOB_TIME, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
    };

    /// A child that waits, so the jail is asked about a process that is certainly still alive.
    fn waiting_child() -> std::process::Child {
        std::process::Command::new("cmd.exe")
            .args(["/c", "pause"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .expect("cmd.exe starts")
    }

    /// The jail is not a good intention: ask the OS what it actually applied.
    #[test]
    fn a_confined_child_is_in_a_job_carrying_every_limit() {
        use std::os::windows::io::AsRawHandle as _;
        let mut child = waiting_child();
        let limits = super::Limits::default();
        let (jail, enforced) = super::confine(&child, limits);
        assert!(!enforced.guarantees.is_empty(), "the job was not applied at all");

        let mut in_job: i32 = 0;
        // SAFETY: the child is alive and its handle is valid until `wait`.
        let ok = unsafe { IsProcessInJob(child.as_raw_handle() as HANDLE, std::ptr::null_mut(), &mut in_job) };
        assert_ne!(ok, 0, "IsProcessInJob failed");
        assert_ne!(in_job, 0, "the guest is not in any job");

        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        let mut returned: u32 = 0;
        // SAFETY: querying the job this process created, into a correctly sized structure.
        let ok = unsafe {
            QueryInformationJobObject(
                jail_handle(&jail),
                JobObjectExtendedLimitInformation,
                &mut info as *mut _ as *mut core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                &mut returned,
            )
        };
        assert_ne!(ok, 0, "QueryInformationJobObject failed");
        let flags = info.BasicLimitInformation.LimitFlags;
        for (bit, name) in [
            (JOB_OBJECT_LIMIT_ACTIVE_PROCESS, "one process only"),
            (JOB_OBJECT_LIMIT_PROCESS_MEMORY, "memory ceiling"),
            (JOB_OBJECT_LIMIT_JOB_TIME, "processor-time ceiling"),
            (JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, "killed with the host"),
        ] {
            assert_ne!(flags & bit, 0, "the job does not carry `{name}`");
        }
        assert_eq!(info.BasicLimitInformation.ActiveProcessLimit, 1);
        assert_eq!(info.ProcessMemoryLimit as u64, limits.memory_bytes);
        assert_eq!(info.BasicLimitInformation.PerJobUserTimeLimit, (limits.cpu_seconds as i64) * 10_000_000);
        assert!(enforced.guarantees.contains(&"one process only"), "{enforced:?}");

        let _ = child.kill();
        let _ = child.wait();
    }

    /// Reach the handle for the query above without widening the public surface.
    fn jail_handle(jail: &super::Jail) -> HANDLE {
        jail.handle()
    }
}

/// Linux and macOS jails arrive with the rest of PS-A2 (Landlock, seccomp and rlimits on Linux; a
/// generated Seatbelt profile on macOS — both measured on CI in run 35378619727). Until then this
/// host enforces nothing at the OS level and says so, which is the honest report: the channel still
/// means the guest holds no DeluluLang authority.
#[cfg(not(windows))]
mod other {
    /// Nothing to hold on Linux (the kernel's death signal and limits need no handle); on macOS the
    /// watcher that ends the guest with its host (PS-E-02, [`super::HostWatch`]), for as long as the host
    /// holds the guest.
    #[derive(Default)]
    pub struct Jail {
        #[cfg(target_os = "macos")]
        watch: Option<super::HostWatch>,
    }

    impl Jail {
        /// No jail: the guest is confined by something DeluluLang did not apply (PS-D-01, L3).
        pub fn none() -> Jail {
            Jail::default()
        }

        /// This jail, now also holding the watcher that ends the guest with its host.
        #[cfg(target_os = "macos")]
        pub fn watched_by(mut self, watch: super::HostWatch) -> Jail {
            self.watch = Some(watch);
            self
        }
    }

    pub fn confine<T>(_child: &T, _limits: super::Limits) -> (Jail, super::Enforced) {
        (Jail::default(), super::Enforced::default())
    }
}

/// PS-A2: the Landlock ruleset, measured rather than asserted.
///
/// `restrict_self` cannot be undone, so the measurement runs in a CHILD process — this same test
/// binary, re-entered with one environment variable — and every run is done twice: once WITHOUT the
/// ruleset and once with it. The unconfined half is the falsification: if it cannot write the file
/// either, the confined half's refusal proves nothing about Landlock, and the test says so instead
/// of passing.
#[cfg(all(test, target_os = "linux"))]
mod linux_tests {
    const MODE: &str = "DELULU_JAIL_TEST_MODE";
    const CHAN: &str = "DELULU_JAIL_TEST_CHAN";
    const SECRET: &str = "DELULU_JAIL_TEST_SECRET";

    /// The child half. Without the environment variable this is not a test of anything and returns
    /// at once; the parent below is what drives it.
    #[test]
    fn jail_probe_child() {
        let Ok(mode) = std::env::var(MODE) else { return };
        let chan = std::path::PathBuf::from(std::env::var(CHAN).expect("the channel directory"));
        let secret = std::path::PathBuf::from(std::env::var(SECRET).expect("the secret directory"));
        if mode == "confined" {
            match super::confine_filesystem(Some(&chan)) {
                Some(applied) => println!("APPLIED={}", applied.join(",")),
                None => println!("APPLIED=none"),
            }
        } else {
            println!("APPLIED=skipped");
        }
        // Exactly the same four attempts in both halves, so the only difference is the ruleset.
        println!("WRITE={}", std::fs::write(chan.join("escaped.txt"), b"x").is_ok());
        println!("SECRET={}", std::fs::read(secret.join("data")).is_ok());
        println!("SYSTEM={}", std::fs::read("/proc/self/status").is_ok());
        println!("BIND={}", std::net::TcpListener::bind("127.0.0.1:0").is_ok());
    }

    /// Run the child half in `mode` and return everything it printed.
    fn probe(mode: &str, chan: &std::path::Path, secret: &std::path::Path) -> String {
        let out = std::process::Command::new(std::env::current_exe().expect("this test binary"))
            .args(["--exact", "jail::linux_tests::jail_probe_child", "--nocapture"])
            .env(MODE, mode)
            .env(CHAN, chan)
            .env(SECRET, secret)
            .output()
            .expect("the child test process runs");
        assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    #[test]
    fn landlock_leaves_the_guest_nothing_writable_and_nothing_of_the_operator_to_read() {
        let base = std::env::temp_dir().join(format!("delulu-jail-{}", std::process::id()));
        let (chan, secret) = (base.join("chan"), base.join("secret"));
        std::fs::create_dir_all(&chan).expect("a channel directory");
        std::fs::create_dir_all(&secret).expect("a secret directory");
        std::fs::write(secret.join("data"), b"the operator's").expect("a secret to read");

        // The control. Without it, every assertion below could be passing for the wrong reason —
        // a path that does not exist, a read-only mount, a container that forbids binding.
        let free = probe("free", &chan, &secret);
        for expected in ["WRITE=true", "SECRET=true", "SYSTEM=true", "BIND=true"] {
            assert!(free.contains(expected), "the unconfined control could not `{expected}`, so this test proves nothing:
{free}");
        }

        let confined = probe("confined", &chan, &secret);
        if confined.contains("APPLIED=none") {
            // Honest, and visible: this host measured nothing, and the run said so rather than
            // reporting a boundary it does not have.
            eprintln!("this kernel has no Landlock, so there was nothing to measure:
{confined}");
            let _ = std::fs::remove_dir_all(&base);
            return;
        }
        assert!(confined.contains("WRITE=false"), "the guest could still write:
{confined}");
        assert!(confined.contains("SECRET=false"), "the guest could still read the operator's file:
{confined}");
        assert!(confined.contains("SYSTEM=true"), "the guest could not read `/proc`, so it cannot run at all:
{confined}");
        if confined.contains("no TCP bind or connect") {
            assert!(confined.contains("BIND=false"), "`no TCP bind or connect` was claimed and the guest bound anyway:
{confined}");
        }
        let _ = std::fs::remove_dir_all(&base);
    }
}

/// PS-E-03 (`V2_OPENSHELL_STUDY.md` §4.3): the ESCAPED GUEST. A guest whose interpreter an attacker has
/// taken over is a process that has applied everything the guest applies to itself — Landlock, then the
/// syscall filter, as `serve_as_guest` does — and then makes whatever system calls it likes. This child is
/// exactly that, with no interpreter in the way: the guest's own lock-down, then raw attempts at what
/// §4.3's hypotheses say may still be open, each printed as `NAME=true|false`. The parent runs it twice —
/// FREE, the control (an attempt that fails unconfined proves nothing when it fails confined, and is
/// reported as unmeasurable here) and ESCAPED — and a hypothesis is a finding only where the escaped
/// attempt succeeds. What a jailed guest's HOST adds on top (the rlimits, a separate identity) is not
/// applied here: this measures the guest's own layers, which every Linux guest has.
#[cfg(all(test, target_os = "linux"))]
mod escaped_tests {
    use std::collections::BTreeMap;

    const MODE: &str = "DELULU_ESCAPED_TEST_MODE";
    const CHAN: &str = "DELULU_ESCAPED_TEST_CHAN";
    const SOCK: &str = "DELULU_ESCAPED_TEST_SOCK";
    const PID: &str = "DELULU_ESCAPED_TEST_PID";
    const TTY: &str = "DELULU_ESCAPED_TEST_TTY";
    /// What the operator's other process holds in its environment — an API key, in life.
    const SENTINEL: &str = "DELULU_ESCAPED_SENTINEL=the-operators-key";

    /// The child half; without the environment variable it is not a test of anything and returns.
    #[test]
    fn escaped_guest_child() {
        let Ok(mode) = std::env::var(MODE) else { return };
        let chan = std::path::PathBuf::from(std::env::var(CHAN).expect("the channel directory"));
        let sock = std::path::PathBuf::from(std::env::var(SOCK).expect("the operator's socket"));
        let pid: i32 = std::env::var(PID).expect("the operator's process").parse().expect("a pid");
        let tty = std::env::var(TTY).expect("the operator's terminal");
        // libtest printed this test's name without a newline; the first `NAME=` line must not join it.
        println!();
        // H8: a guest shares its host's session, so the operator's terminal is its CONTROLLING terminal and
        // its standard error. This child takes the pseudo-terminal the same way, before its lock-down.
        // SAFETY: `setsid` and one `ioctl` on a descriptor this child opens and keeps.
        let controlling = unsafe {
            let fd = std::ffi::CString::new(tty.clone()).map(|p| libc::open(p.as_ptr(), libc::O_RDWR)).unwrap_or(-1);
            libc::setsid();
            if fd >= 0 && libc::ioctl(fd, libc::TIOCSCTTY, 0) == 0 {
                fd
            } else {
                -1
            }
        };
        if mode == "escaped" {
            let fs = super::confine_filesystem(Some(&chan));
            println!("LANDLOCK={}", fs.map(|a| a.join(",")).unwrap_or_else(|| "none".into()));
            println!("SECCOMP={}", super::lock_down_self().is_ok());
        }
        let ok = |name: &str, fd: libc::c_long| {
            println!("{name}={}", fd >= 0);
            if fd >= 0 {
                // SAFETY: a descriptor this child just made.
                unsafe { libc::close(fd as libc::c_int) };
            }
        };
        // SAFETY: each is one raw system call on this child's own behalf; every descriptor it returns is
        // closed at once, and the clone's child does nothing but `_exit`.
        unsafe {
            // H2 — sockets. Landlock mediates TCP only.
            ok("UDP", libc::socket(libc::AF_INET, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC, 0) as libc::c_long);
            ok("NETLINK", libc::socket(libc::AF_NETLINK, libc::SOCK_RAW | libc::SOCK_CLOEXEC, libc::NETLINK_ROUTE) as libc::c_long);
            ok("UNIX_SOCKET", libc::socket(libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0) as libc::c_long);
            println!("UNIX_CONNECT={}", std::os::unix::net::UnixStream::connect(&sock).is_ok());
            // H1 — system calls the filter does not name.
            ok("MEMFD", libc::syscall(libc::SYS_memfd_create, c"escaped".as_ptr(), 0));
            let mut params = [0u8; 120];
            ok("IO_URING", libc::syscall(libc::SYS_io_uring_setup, 1, params.as_mut_ptr()));
            ok("USERFAULTFD", libc::syscall(libc::SYS_userfaultfd, libc::O_CLOEXEC));
            ok("PIDFD_OPEN", libc::syscall(libc::SYS_pidfd_open, pid, 0));
            ok("FSOPEN", libc::syscall(libc::SYS_fsopen, c"tmpfs".as_ptr(), 0));
            let child = libc::syscall(libc::SYS_clone, (libc::CLONE_NEWUSER | libc::SIGCHLD) as libc::c_ulong, 0, 0, 0, 0);
            if child == 0 {
                libc::_exit(0);
            }
            if child > 0 {
                libc::waitpid(child as libc::pid_t, std::ptr::null_mut(), 0);
            }
            println!("CLONE_NEWUSER={}", child > 0);
            // `clone3` carries its flags in memory, out of any filter's reach: the same namespace, asked
            // for through it (`struct clone_args`, its first version: flags … tls, eight words).
            let mut args = [0u64; 8];
            args[0] = libc::CLONE_NEWUSER as u64;
            args[4] = libc::SIGCHLD as u64;
            let child = libc::syscall(libc::SYS_clone3, args.as_mut_ptr(), std::mem::size_of_val(&args));
            if child == 0 {
                libc::_exit(0);
            }
            if child > 0 {
                libc::waitpid(child as libc::pid_t, std::ptr::null_mut(), 0);
            }
            println!("CLONE3={}", child > 0);
        }
        // H3 — another process of the same user, through `/proc`.
        let environ = std::fs::read(format!("/proc/{pid}/environ"))
            .map(|b| String::from_utf8_lossy(&b).contains(SENTINEL))
            .unwrap_or(false);
        println!("PROC_ENVIRON={environ}");
        println!("PROC_CMDLINE={}", std::fs::read(format!("/proc/{pid}/cmdline")).is_ok());
        // H7 — the operator's terminal, opened for reading: what is typed there.
        println!("TERMINAL={}", std::fs::File::open(&tty).is_ok());
        // H8 — a keystroke pushed into the controlling terminal's input (`TIOCSTI`): what the operator's
        // shell reads next, and runs, once the guest is gone.
        // SAFETY: one `ioctl` with a one-byte buffer on the descriptor made above.
        let injected = controlling >= 0 && unsafe { libc::ioctl(controlling, libc::TIOCSTI, c"x".as_ptr()) } == 0;
        println!("TIOCSTI={injected}");
        // The same command with its high 32 bits set: the kernel reads only the low 32, so a filter that
        // compared all 64 would let this spelling through.
        // SAFETY: as above, through the raw system call so the high bits reach the kernel.
        #[allow(clippy::unnecessary_cast)] // `c_int` on musl, `c_ulong` on glibc
        let high = libc::TIOCSTI as u64 | (1u64 << 32);
        let injected = controlling >= 0 && unsafe { libc::syscall(libc::SYS_ioctl, controlling, high, c"x".as_ptr()) } == 0;
        println!("TIOCSTI_HIGH={injected}");
        // H9 — a signal to another process of the same user. Signal 0 asks only whether it WOULD be
        // delivered, so nothing is harmed; `kill(-1, SIGKILL)` would end every process the operator has.
        // SAFETY: plain signal-0 probes; nothing is delivered.
        unsafe {
            println!("SIGNAL_OTHER={}", libc::kill(pid, 0) == 0);
            println!("SIGNAL_GROUP={}", libc::kill(0, 0) == 0);
            println!("SIGNAL_QUEUE={}", {
                let mut info: libc::siginfo_t = std::mem::zeroed();
                info.si_signo = 0;
                info.si_code = -1; // SI_QUEUE
                libc::syscall(libc::SYS_rt_sigqueueinfo, pid, 0, &mut info as *mut libc::siginfo_t) == 0
            });
            // The guest must still be able to signal ITSELF: `abort` is a signal to itself.
            println!("SIGNAL_SELF={}", libc::kill(libc::getpid(), 0) == 0);
        }
        // H10 — the other calls that ACT on another process of the same user: its resource limits, its
        // priority, its CPUs, its scheduling class, its I/O priority. Each is set to the value it already
        // has, read first, so nothing is changed even unconfined — and a refused read sets nothing.
        // SAFETY: each call is given buffers of the size it names; values are only ever written back as read.
        unsafe {
            let mut lim: libc::rlimit64 = std::mem::zeroed();
            let read = libc::prlimit64(pid, libc::RLIMIT_CORE, std::ptr::null(), &mut lim) == 0;
            println!("PRLIMIT_OTHER={}", read && libc::prlimit64(pid, libc::RLIMIT_CORE, &lim, std::ptr::null_mut()) == 0);
            *libc::__errno_location() = 0;
            let nice = libc::getpriority(libc::PRIO_PROCESS, pid as libc::id_t);
            let read = *libc::__errno_location() == 0;
            println!("PRIORITY_OTHER={}", read && libc::setpriority(libc::PRIO_PROCESS, pid as libc::id_t, nice) == 0);
            let mut cpus: libc::cpu_set_t = std::mem::zeroed();
            let size = std::mem::size_of::<libc::cpu_set_t>();
            let read = libc::sched_getaffinity(pid, size, &mut cpus) == 0;
            println!("AFFINITY_OTHER={}", read && libc::sched_setaffinity(pid, size, &cpus) == 0);
            // Zeroed rather than spelled: musl's `sched_param` has fields glibc's has not.
            let param: libc::sched_param = std::mem::zeroed();
            let policy = libc::sched_getscheduler(pid);
            println!("SCHEDULER_OTHER={}", policy == libc::SCHED_OTHER && libc::sched_setscheduler(pid, libc::SCHED_OTHER, &param) == 0);
            // `ioprio_get`/`ioprio_set` for a process (`IOPRIO_WHO_PROCESS` is 1).
            let prio = libc::syscall(libc::SYS_ioprio_get, 1, pid);
            println!("IOPRIO_OTHER={}", prio >= 0 && libc::syscall(libc::SYS_ioprio_set, 1, pid, prio) == 0);
            // The guest's own limits stay readable: the C library asks for its stack limit.
            let mut own: libc::rlimit64 = std::mem::zeroed();
            println!("PRLIMIT_SELF={}", libc::prlimit64(0, libc::RLIMIT_STACK, std::ptr::null(), &mut own) == 0);
        }
        // What the guest itself must keep, or it cannot run: the control of any narrowing.
        println!("PROC_SELF={}", std::fs::read("/proc/self/status").is_ok());
        println!("DEV_NULL={}", std::fs::File::open("/dev/null").is_ok());
    }

    /// The operator's world beside the guest: a Unix socket outside anything granted (an SSH agent, a
    /// session bus), and a process of the same user holding a secret in its environment.
    struct World {
        base: std::path::PathBuf,
        _listener: std::os::unix::net::UnixListener,
        operator: std::process::Child,
        /// The operator's terminal: a pseudo-terminal this test holds open, and the path of its other end.
        terminal: (libc::c_int, String),
    }

    impl Drop for World {
        fn drop(&mut self) {
            // SAFETY: the terminal's descriptor this world opened, closed once.
            unsafe { libc::close(self.terminal.0) };
            let _ = self.operator.kill();
            let _ = self.operator.wait();
            let _ = std::fs::remove_dir_all(&self.base);
        }
    }

    fn world(tag: &str) -> World {
        let base = std::env::temp_dir().join(format!("delulu-escaped-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(base.join("chan")).expect("a channel directory");
        std::fs::create_dir_all(base.join("operator")).expect("the operator's directory");
        let listener = std::os::unix::net::UnixListener::bind(base.join("operator/agent.sock")).expect("the operator's socket");
        let (k, v) = SENTINEL.split_once('=').unwrap();
        let operator = std::process::Command::new("sleep").arg("60").env(k, v).spawn().expect("the operator's process");
        // SAFETY: the documented sequence for a new pseudo-terminal; the name is copied out at once.
        let terminal = unsafe {
            let fd = libc::posix_openpt(libc::O_RDWR | libc::O_NOCTTY | libc::O_CLOEXEC);
            assert!(fd >= 0 && libc::grantpt(fd) == 0 && libc::unlockpt(fd) == 0, "a pseudo-terminal");
            let mut name = [0 as libc::c_char; 128];
            assert_eq!(libc::ptsname_r(fd, name.as_mut_ptr(), name.len()), 0, "its name");
            (fd, std::ffi::CStr::from_ptr(name.as_ptr()).to_string_lossy().into_owned())
        };
        World { base, _listener: listener, operator, terminal }
    }

    /// Run the child half in `mode`; every `NAME=true|false` it printed.
    fn probe(w: &World, mode: &str) -> BTreeMap<String, String> {
        let out = std::process::Command::new(std::env::current_exe().expect("this test binary"))
            .args(["--exact", "jail::escaped_tests::escaped_guest_child", "--nocapture", "--test-threads=1"])
            .env(MODE, mode)
            .env(CHAN, w.base.join("chan"))
            .env(SOCK, w.base.join("operator/agent.sock"))
            .env(PID, w.operator.id().to_string())
            .env(TTY, &w.terminal.1)
            .output()
            .expect("the child test process runs");
        assert!(out.status.success(), "the child failed: {}", String::from_utf8_lossy(&out.stderr));
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| l.split_once('=').map(|(k, v)| (k.to_string(), v.to_string())))
            .filter(|(k, _)| k.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
            .collect()
    }

    /// The attempts in `names` that an escaped guest still succeeds at, where the free control succeeded
    /// too; `None` when this kernel has no Landlock (nothing of the guest's own confinement to measure).
    fn still_open(tag: &str, names: &[&str]) -> Option<Vec<String>> {
        let w = world(tag);
        let free = probe(&w, "free");
        let escaped = probe(&w, "escaped");
        if escaped.get("LANDLOCK").map(String::as_str) == Some("none") {
            eprintln!("this kernel has no Landlock; nothing of the guest's own confinement to measure: {escaped:?}");
            return None;
        }
        assert_eq!(escaped.get("SECCOMP").map(String::as_str), Some("true"), "the filter was not installed: {escaped:?}");
        assert_eq!(escaped.get("PROC_SELF").map(String::as_str), Some("true"), "the guest cannot read itself: {escaped:?}");
        assert_eq!(escaped.get("DEV_NULL").map(String::as_str), Some("true"), "the guest cannot open /dev/null: {escaped:?}");
        assert_eq!(escaped.get("SIGNAL_SELF").map(String::as_str), Some("true"), "the guest cannot signal itself: {escaped:?}");
        assert_eq!(escaped.get("PRLIMIT_SELF").map(String::as_str), Some("true"), "the guest cannot read its own limits: {escaped:?}");
        let mut open = Vec::new();
        for n in names {
            match (free.get(*n).map(String::as_str), escaped.get(*n).map(String::as_str)) {
                (Some("true"), Some("true")) => open.push(n.to_string()),
                (Some("true"), Some("false")) => {}
                // An attempt the child never reported is the harness's fault, not the host's: fail.
                (None, _) | (_, None) => panic!("{n} was not reported: free {free:?}, escaped {escaped:?}"),
                (f, e) => eprintln!("{n}: unmeasurable on this host (free {f:?}, escaped {e:?})"),
            }
        }
        eprintln!("free: {free:?}\nescaped: {escaped:?}");
        Some(open)
    }

    /// H2: after lock-down an escaped guest opens no socket — no UDP (a DNS exfiltration channel where the
    /// host has a network), no netlink, and no Unix socket to reach the operator's agent or session bus.
    #[test]
    fn h2_an_escaped_guest_opens_no_socket() {
        let Some(open) = still_open("h2", &["UDP", "NETLINK", "UNIX_SOCKET", "UNIX_CONNECT"]) else { return };
        assert!(open.is_empty(), "an escaped guest still reached: {open:?}");
    }

    /// H3: an escaped guest reads nothing of the operator's other processes through `/proc` — not the
    /// environment (an API key), not the command line.
    #[test]
    fn h3_an_escaped_guest_reads_no_other_process() {
        let Some(open) = still_open("h3", &["PROC_ENVIRON", "PROC_CMDLINE"]) else { return };
        assert!(open.is_empty(), "an escaped guest still reached: {open:?}");
    }

    /// H7: an escaped guest cannot open the operator's terminal to read what is typed there.
    #[test]
    fn h7_an_escaped_guest_opens_no_terminal() {
        let Some(open) = still_open("h7", &["TERMINAL"]) else { return };
        assert!(open.is_empty(), "an escaped guest still reached: {open:?}");
    }

    /// H8: an escaped guest cannot push keystrokes into the operator's terminal — its controlling terminal,
    /// shared with the host's session — for the operator's shell to run after the guest is gone.
    #[test]
    fn h8_an_escaped_guest_types_nothing_into_the_operators_terminal() {
        let Some(open) = still_open("h8", &["TIOCSTI", "TIOCSTI_HIGH"]) else { return };
        assert!(open.is_empty(), "an escaped guest still reached: {open:?}");
    }

    /// H9: an escaped guest cannot signal another process of the operator's — not one (`kill`), not its
    /// process group, not by a queued signal. `kill(-1, SIGKILL)` would end every process the user has.
    #[test]
    fn h9_an_escaped_guest_signals_no_other_process() {
        let Some(open) = still_open("h9", &["SIGNAL_OTHER", "SIGNAL_GROUP", "SIGNAL_QUEUE"]) else { return };
        assert!(open.is_empty(), "an escaped guest still reached: {open:?}");
    }

    /// H10: an escaped guest cannot act on another process of the operator's by any other call either —
    /// lower its limits, its priority or its I/O priority, pin it to one CPU, or change its scheduling.
    #[test]
    fn h10_an_escaped_guest_changes_no_other_process() {
        let names = ["PRLIMIT_OTHER", "PRIORITY_OTHER", "AFFINITY_OTHER", "SCHEDULER_OTHER", "IOPRIO_OTHER"];
        let Some(open) = still_open("h10", &names) else { return };
        assert!(open.is_empty(), "an escaped guest still reached: {open:?}");
    }

    /// H1: the system calls the filter did not name are refused after lock-down.
    #[test]
    fn h1_an_escaped_guest_makes_none_of_the_unnamed_calls() {
        let Some(open) = still_open("h1", &["MEMFD", "IO_URING", "USERFAULTFD", "PIDFD_OPEN", "FSOPEN", "CLONE_NEWUSER", "CLONE3"]) else {
            return;
        };
        assert!(open.is_empty(), "an escaped guest still reached: {open:?}");
    }
}

/// PS-B-03 / T14 on macOS, measured rather than asserted: the same child, run once as the operator
/// (the control) and once under the guest's Seatbelt profile, tries to read a key in a state
/// directory and a file outside it. The control must read both, or the refusal proves nothing.
#[cfg(all(test, target_os = "macos"))]
mod macos_tests {
    const MODE: &str = "DELULU_SEATBELT_TEST_MODE";

    #[test]
    fn seatbelt_probe_child() {
        let Ok(paths) = std::env::var(MODE) else { return };
        let (key, elsewhere) = paths.split_once('|').expect("two paths");
        println!("STATE={}", std::fs::read(key).is_ok());
        println!("ELSEWHERE={}", std::fs::read(elsewhere).is_ok());
    }

    #[test]
    fn the_state_directory_is_unreadable_to_a_seatbelted_guest() {
        let base = std::env::temp_dir().join(format!("dsb{}", std::process::id()));
        let (chan, state, other) = (base.join("c"), base.join("state"), base.join("other"));
        for d in [&chan, &state, &other] {
            std::fs::create_dir_all(d).unwrap();
        }
        std::fs::write(state.join("broker.key"), b"a key").unwrap();
        std::fs::write(other.join("plain.txt"), b"plain").unwrap();
        let paths = format!("{}|{}", state.join("broker.key").display(), other.join("plain.txt").display());
        let exe = std::env::current_exe().unwrap();
        let args: Vec<&std::ffi::OsStr> =
            ["--exact", "jail::macos_tests::seatbelt_probe_child", "--nocapture"].iter().map(std::ffi::OsStr::new).collect();

        let free = std::process::Command::new(&exe).args(&args).env(MODE, &paths).output().unwrap();
        let free = String::from_utf8_lossy(&free.stdout).to_string();
        assert!(free.contains("STATE=true") && free.contains("ELSEWHERE=true"), "the control must read both:\n{free}");

        let (mut cmd, applied) =
            super::seatbelt_launcher(&exe, &chan, &args, std::slice::from_ref(&state)).expect("sandbox-exec exists on macOS");
        assert!(applied.contains(&"the state directory unreadable"), "{applied:?}");
        let out = cmd.env(MODE, &paths).output().unwrap();
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        assert!(text.contains("ELSEWHERE=true"), "the profiled child ran and could read what it may:\n{text}\n{}", String::from_utf8_lossy(&out.stderr));
        assert!(text.contains("STATE=false"), "T14: the profiled child read the state directory:\n{text}");
        let _ = std::fs::remove_dir_all(&base);
    }
}
