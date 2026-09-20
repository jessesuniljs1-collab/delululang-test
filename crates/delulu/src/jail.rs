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
}

impl Default for Limits {
    /// D-V2-25 (owner, 2026-09-18): 1 GiB and 5 minutes of processor time.
    fn default() -> Self {
        Limits { memory_bytes: 1024 * 1024 * 1024, cpu_seconds: 300 }
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
pub use windows_jail::{confine, Jail};

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
    let profile = format!(
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
    let path = dir.join("guest.sb");
    std::fs::write(&path, profile).ok()?;
    let mut cmd = std::process::Command::new("/usr/bin/sandbox-exec");
    cmd.arg("-f").arg(&path).arg(exe).args(args);
    Some((
        cmd,
        vec![
            "deny by default",
            "no file writes",
            "no network but the channel",
            "no new programs",
            "no Mach services",
            "no signals or process info beyond itself",
        ],
    ))
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
/// processor-time ceiling IS the bound on a spinning orphan — and without it the bound was "for ever".
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
pub fn confine_filesystem(channel_dir: &std::path::Path) -> Option<Vec<&'static str>> {
    use landlock::{
        path_beneath_rules, Access, AccessFs, AccessNet, LandlockStatus, Ruleset, RulesetAttr,
        RulesetCreatedAttr, ABI,
    };

    // Read-only, and only what a running process needs: its loader and libraries, the system
    // configuration a libc call may consult, `/proc` and `/sys` for the process's own facts, and
    // `/dev` for the random and null devices. Nothing under `/home`, `/root`, `/tmp` or a
    // workspace — that is the operator's data, and the guest has no business reading any of it.
    // Paths that do not exist on this host are skipped by `path_beneath_rules`, which is why
    // `/lib64` may be named on an architecture that has no such directory.
    const SYSTEM_READ: &[&str] =
        &["/usr", "/lib", "/lib64", "/lib32", "/bin", "/sbin", "/etc", "/proc", "/sys", "/dev"];

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
        // talking through. It is named rather than assumed, as the Seatbelt profile names it.
        .and_then(|r| r.add_rules(path_beneath_rules([channel_dir], AccessFs::from_read(fs_abi))))
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
    let rules: BTreeMap<i64, Vec<seccompiler::SeccompRule>> =
        denied.iter().map(|s| (*s, Vec::new())).collect();
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
    Ok(vec!["no new programs", "no debugger", "no namespace or module tricks"])
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
    use std::os::windows::io::AsRawHandle as _;

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicUIRestrictions,
        JobObjectExtendedLimitInformation, SetInformationJobObject, JOBOBJECT_BASIC_UI_RESTRICTIONS,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
        JOB_OBJECT_LIMIT_JOB_TIME, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOB_OBJECT_UILIMIT_DESKTOP, JOB_OBJECT_UILIMIT_EXITWINDOWS,
        JOB_OBJECT_UILIMIT_GLOBALATOMS, JOB_OBJECT_UILIMIT_HANDLES, JOB_OBJECT_UILIMIT_READCLIPBOARD,
        JOB_OBJECT_UILIMIT_SYSTEMPARAMETERS, JOB_OBJECT_UILIMIT_WRITECLIPBOARD,
    };

    /// A held Job Object. Dropping it fires kill-on-close for every process still assigned, so a
    /// guest never outlives the host that was serving it.
    pub struct Jail(HANDLE);

    impl Drop for Jail {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CloseHandle(self.0) };
            }
        }
    }

    /// Put `child` in a job with `limits`. Returns what was enforced, so the caller reports the
    /// truth rather than the intention.
    pub fn confine(child: &std::process::Child, limits: super::Limits) -> (Jail, super::Enforced) {
        let mut enforced = super::Enforced::default();
        // SAFETY: every call below takes handles this function owns or the child's own handle, and
        // the structures are zeroed and sized with `size_of`.
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return (Jail(std::ptr::null_mut()), enforced);
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
                return (Jail(std::ptr::null_mut()), enforced);
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
                return (Jail(std::ptr::null_mut()), super::Enforced::default());
            }
            (Jail(job), enforced)
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
        // SAFETY: `Jail` is a newtype over the handle; this test lives in the same module tree.
        unsafe { *(jail as *const super::Jail as *const HANDLE) }
    }
}

/// Linux and macOS jails arrive with the rest of PS-A2 (Landlock, seccomp and rlimits on Linux; a
/// generated Seatbelt profile on macOS — both measured on CI in run 35378619727). Until then this
/// host enforces nothing at the OS level and says so, which is the honest report: the channel still
/// means the guest holds no DeluluLang authority.
#[cfg(not(windows))]
mod other {
    pub struct Jail;

    pub fn confine(_child: &std::process::Child, _limits: super::Limits) -> (Jail, super::Enforced) {
        (Jail, super::Enforced::default())
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
            match super::confine_filesystem(&chan) {
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
