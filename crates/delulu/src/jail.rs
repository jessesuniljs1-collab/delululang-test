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

/// The macOS jail: the guest runs under a Seatbelt profile that denies file writes and the network,
/// and allows exactly one path — the channel socket it talks to the host on.
///
/// The shape is the measured one. A deny-by-default profile aborted even a plain C program before it
/// reached its socket (experiment run 35391102215, exit 134), while allow-default with targeted
/// denies connected and still refused writes. So this is what macOS enforces today, and the report
/// says exactly that rather than implying a deny-default jail. Tightening it to deny-default, with
/// the loader and Mach allowances a Mach-O binary needs, stays open work.
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
    // itself is right: `(allow network-bind (literal …))` passes when the path matches (run
    // 35395762998). This is the campaign's recurring shape — a security decision made on an
    // unnormalized representation — so both spellings are named.
    let raw = dir.join("broker.sock");
    let resolved = std::fs::canonicalize(dir).map(|d| d.join("broker.sock")).unwrap_or_else(|_| raw.clone());
    let (raw, resolved) = (raw.display().to_string(), resolved.display().to_string());
    // Writes and the network are denied everywhere but the channel socket: a guest performs no
    // effects, so it writes nothing and reaches nothing of its own. The host performs both.
    let profile = format!(
        "(version 1)\n\
         (allow default)\n\
         (deny file-write*)\n\
         (deny network*)\n\
         (allow file-read* file-write* (literal \"{raw}\") (literal \"{resolved}\"))\n\
         (allow network-bind network-outbound (literal \"{raw}\") (literal \"{resolved}\"))\n"
    );
    let path = dir.join("guest.sb");
    std::fs::write(&path, profile).ok()?;
    let mut cmd = std::process::Command::new("/usr/bin/sandbox-exec");
    cmd.arg("-f").arg(&path).arg(exe).args(args);
    Some((cmd, vec!["no file writes", "no network but the channel"]))
}

/// Other platforms harden after the spawn, or not yet at all.
#[cfg(not(target_os = "linux"))]
pub fn harden(cmd: &mut std::process::Command, _limits: Limits) -> Vec<&'static str> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        // Created SUSPENDED so the jail is applied before the guest's first instruction rather than
        // a moment after it. `resume` below starts it once the Job Object is in place.
        const CREATE_SUSPENDED: u32 = 0x0000_0004;
        cmd.creation_flags(CREATE_SUSPENDED);
    }
    let _ = cmd;
    Vec::new()
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
    let rules: BTreeMap<i64, Vec<seccompiler::SeccompRule>> =
        denied.iter().map(|s| (*s as i64, Vec::new())).collect();
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
