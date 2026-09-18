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
#[derive(Clone, Copy, Debug)]
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

    impl Jail {
        pub fn is_enforced(&self) -> bool {
            !self.0.is_null()
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
        assert!(jail.is_enforced(), "the job was not applied at all");

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

    impl Jail {
        pub fn is_enforced(&self) -> bool {
            false
        }
    }

    pub fn confine(_child: &std::process::Child, _limits: super::Limits) -> (Jail, super::Enforced) {
        (Jail, super::Enforced::default())
    }
}
