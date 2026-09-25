//! PS-B-03: identity separation — a sandbox guest that runs as a principal other than the operator.
//!
//! Every boundary PS-A built holds the guest to what it may DO: one process, a memory and processor
//! ceiling, no desktop, and on Linux and macOS a filesystem and network the guest never sees. What
//! none of them changed is WHO the guest is. On Windows it ran with the operator's own token, inside a
//! Job Object, so a guest that escaped the interpreter could read `broker.key`, the root policy and
//! every file the operator can (`V2_SECURITY_MODEL.md` §3; threat T14).
//!
//! **Windows: a per-run AppContainer.** The guest is started in a fresh AppContainer, created for the
//! run and deleted after it, holding NO capabilities — so no network at all, and none of the
//! operator's files, because the operator's files are not granted to that identity. Two things had to
//! change to make that possible, both measured in the PS-B-03 feasibility run before this was
//! written:
//!
//! - The guest's own binary must be loadable by an identity that can read nothing of the operator's.
//!   It runs from a RUNTIME COPY: the executable and the few non-system libraries this process has
//!   loaded (`python313.dll` on the default build) are copied once per build into a directory whose
//!   only extra grant is read-and-execute for `ALL APPLICATION PACKAGES`. That grant exposes code any
//!   copy of DeluluLang ships, and nothing else: it is on that directory alone, never the state
//!   directory, the program, or the operator's files.
//! - The channel moves to an inherited pipe (`pipe_channel.rs`), because an AppContainer's named
//!   pipes live in its own namespace: one duplex pipe whose client end the host opens and hands down.
//!
//! The same experiment is the evidence for what the identity buys: the contained child could not
//! read the operator's file, could not read a key in a state directory, could not list or write the
//! operator's directory and could not open a connection, while the SAME binary without the container
//! did all five. T14's witness below repeats it against the runtime copy this module prepares.
//!
//! **Linux: a subordinate uid in the guest's own user namespace (PS-B-03b).** The guest is born in a
//! new user namespace where it is uid 1 and gid 1, and those map — through the setuid helpers
//! `newuidmap`/`newgidmap` and the ranges `/etc/subuid` and `/etc/subgid` give this user — to ids
//! outside the operator's own. It drops every supplementary group and every capability before it
//! executes a line, so to the kernel it is a stranger: the operator's `0600` files, a `0700` home and
//! the state directory are closed to it, and only what EVERY account may read stays readable (which
//! Landlock then narrows further, where the kernel has it). Its channel is a socket pair it inherits,
//! and its binary is executed through an open descriptor, so neither needs a path the stranger could
//! not walk. Measured before it was built (`host-capability-probe`, `linux-subordinate-uid`): on
//! Ubuntu 24.04's default the distribution forbids it — AppArmor leaves an unprivileged user namespace
//! without the capabilities to set its ids — and on the same runner with that one restriction lifted,
//! a child mapped this way was refused the operator's file that the control read.
//!
//! **Where it is absent it says so.** A host that cannot create the profile, copy the runtime, map the
//! ids or start the process runs the guest as PS-A did and prints why; the run report's
//! `host_guarantees` names the identity only when it was applied. macOS gives an unprivileged
//! launcher no second identity (`V2_LOG.md`, PS-B-03); the documented recipe is a separate OS account
//! (`DEPLOYMENT.md` Tier 2), which remains the boundary for everything this cannot cover — the broker,
//! the CLI itself, and any run that is not sandboxed.

/// The words the run report and `doctor` use for the Windows boundary, so the two cannot drift.
pub const WINDOWS_GUARANTEE: &str =
    "a separate identity: a per-run AppContainer with no capabilities — no network, none of the operator's files";

/// The words for the Linux boundary. They claim less than Windows' on purpose: a subordinate uid is
/// refused what only the operator's account may touch, but it can still read what every account may,
/// write where every account may, and open a socket — those are Landlock's and seccomp's to refuse.
pub const LINUX_GUARANTEE: &str =
    "a separate identity: a subordinate uid in its own user namespace, with no supplementary groups or capabilities";

/// This platform's words. macOS never applies an identity, so its value is never matched.
#[cfg(windows)]
pub const GUARANTEE: &str = WINDOWS_GUARANTEE;
#[cfg(not(windows))]
pub const GUARANTEE: &str = LINUX_GUARANTEE;

#[cfg(windows)]
pub use win::{spawn_contained, ContainedGuest};

#[cfg(target_os = "linux")]
pub use linux::{spawn_contained, ContainedGuest};

#[cfg(target_os = "linux")]
mod linux {
    use std::ffi::OsStr;
    use std::io::{self, Read as _, Write as _};
    use std::os::fd::{AsRawFd as _, FromRawFd as _, OwnedFd, RawFd};
    use std::os::unix::net::UnixStream;
    use std::os::unix::process::CommandExt as _;
    use std::path::Path;
    use std::process::{Child, ChildStderr, Command, ExitStatus, Stdio};

    /// Who the guest is INSIDE its namespace. Not 0: a uid-0 process keeps its capabilities across
    /// `exec`, and a guest needs none. With no mapping for 0 at all, the namespace has no root.
    const NS_ID: u32 = 1;

    /// A range of ids this user may map, from `/etc/subuid` or `/etc/subgid`.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(super) struct Range {
        pub start: u32,
        pub count: u32,
    }

    /// The first range in `text` belonging to this user, named either way the files allow.
    pub(super) fn range_for(text: &str, name: Option<&str>, uid: u32) -> Option<Range> {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut f = line.split(':');
            let (Some(who), Some(start), Some(count)) = (f.next(), f.next(), f.next()) else { continue };
            if Some(who) != name && who.parse::<u32>().ok() != Some(uid) {
                continue;
            }
            let (Ok(start), Ok(count)) = (start.trim().parse::<u32>(), count.trim().parse::<u32>()) else { continue };
            // A range that would wrap, or that contains the operator's own uid, separates nothing.
            let Some(last) = start.checked_add(count.saturating_sub(1)) else { continue };
            if count == 0 || (start..=last).contains(&uid) {
                continue;
            }
            return Some(Range { start, count });
        }
        None
    }

    fn user_name(uid: u32) -> Option<String> {
        let mut buf = vec![0u8; 16 * 1024];
        // SAFETY: `pwd` and `buf` outlive the call, which writes only into them; `result` is checked
        // before `pwd` is read.
        unsafe {
            let mut pwd: libc::passwd = std::mem::zeroed();
            let mut result: *mut libc::passwd = std::ptr::null_mut();
            let rc = libc::getpwuid_r(uid, &mut pwd, buf.as_mut_ptr() as *mut libc::c_char, buf.len(), &mut result);
            if rc != 0 || result.is_null() || pwd.pw_name.is_null() {
                return None;
            }
            Some(std::ffi::CStr::from_ptr(pwd.pw_name).to_string_lossy().into_owned())
        }
    }

    /// The setuid helper, by absolute path. Never looked up on `PATH`: the environment must not
    /// choose the program that decides who the guest becomes.
    fn helper(name: &str) -> Option<&'static str> {
        let found = match name {
            "newuidmap" => ["/usr/bin/newuidmap", "/bin/newuidmap"],
            _ => ["/usr/bin/newgidmap", "/bin/newgidmap"],
        };
        found.into_iter().find(|p| Path::new(p).is_file())
    }

    /// What one launch will map: the ids chosen, and the helpers that will write them.
    struct Plan {
        uid: u32,
        gid: u32,
        newuidmap: &'static str,
        newgidmap: &'static str,
    }

    impl Plan {
        fn for_this_user() -> Result<Plan, String> {
            // SAFETY: getuid cannot fail.
            let uid = unsafe { libc::getuid() };
            let name = user_name(uid);
            let (Some(newuidmap), Some(newgidmap)) = (helper("newuidmap"), helper("newgidmap")) else {
                return Err("`newuidmap`/`newgidmap` are not installed (the `uidmap` package)".into());
            };
            let read = |p: &str| std::fs::read_to_string(p).unwrap_or_default();
            let (Some(us), Some(gs)) =
                (range_for(&read("/etc/subuid"), name.as_deref(), uid), range_for(&read("/etc/subgid"), name.as_deref(), uid))
            else {
                return Err("this user has no subordinate ids in /etc/subuid and /etc/subgid".into());
            };
            // A different id per run where the range allows it, so two guests are strangers to each
            // other too. The randomness is the standard library's per-process hash key: this chooses
            // among ids that are all equally the operator's to hand out, it guards no secret.
            use std::hash::{BuildHasher as _, Hasher as _};
            let mut h = std::collections::hash_map::RandomState::new().build_hasher();
            h.write_u32(std::process::id());
            let salt = h.finish();
            Ok(Plan {
                uid: us.start + (salt % us.count as u64) as u32,
                gid: gs.start + ((salt >> 32) % gs.count as u64) as u32,
                newuidmap,
                newgidmap,
            })
        }
    }

    /// A guest running as a subordinate uid, holding its channel as an inherited socket.
    pub struct ContainedGuest {
        child: Child,
        /// The host's end of the guest's channel.
        pub channel: Option<UnixStream>,
        pub stderr: Option<ChildStderr>,
        /// The uid the guest runs as, outside its namespace.
        #[allow(dead_code)]
        pub uid: u32,
    }

    impl ContainedGuest {
        pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
            self.child.try_wait()
        }

        pub fn wait(&mut self) -> io::Result<ExitStatus> {
            self.child.wait()
        }

        pub fn kill(&mut self) -> io::Result<()> {
            self.child.kill()
        }
    }

    // What the child reports to the helper thread: a tag byte, then four bytes (a pid, or an errno).
    const AT_PID: u8 = 0;
    const AT_UNSHARE: u8 = 1;
    const AT_SETGROUPS: u8 = 2;
    const AT_SETGID: u8 = 3;
    const AT_SETUID: u8 = 4;
    const AT_CAPS: u8 = 5;

    fn step(tag: u8) -> &'static str {
        match tag {
            AT_UNSHARE => "a user namespace could not be created",
            AT_SETGROUPS => "the guest could not drop its supplementary groups",
            AT_SETGID => "the guest could not take its subordinate gid",
            AT_SETUID => "the guest could not take its subordinate uid",
            AT_CAPS => "the guest could not drop its capabilities",
            _ => "the guest failed before it ran",
        }
    }

    /// Why a step failed, in terms an operator can act on. The errno alone ("Operation not
    /// permitted") does not say that the distribution forbids it or how that is changed.
    fn explain(tag: u8, errno: i32) -> String {
        let err = io::Error::from_raw_os_error(errno);
        let restricted = std::fs::read_to_string("/proc/sys/kernel/apparmor_restrict_unprivileged_userns")
            .map(|v| v.trim() == "1")
            .unwrap_or(false);
        if restricted && errno == libc::EPERM {
            format!(
                "{}: {err} — this host restricts unprivileged user namespaces \
                 (kernel.apparmor_restrict_unprivileged_userns = 1, Ubuntu's default since 23.10)",
                step(tag)
            )
        } else {
            format!("{}: {err}", step(tag))
        }
    }

    /// The helper thread's half: hear the child's pid, write its maps, tell it to go on, then hear
    /// whether it got as far as `exec`. Returns why not, if it did not.
    fn map_ids(mut from_child: std::fs::File, mut to_child: std::fs::File, plan: &Plan) -> Result<(), String> {
        let mut msg = [0u8; 5];
        if from_child.read_exact(&mut msg).is_err() {
            return Err("the guest process could not be started".into());
        }
        let value = u32::from_le_bytes([msg[1], msg[2], msg[3], msg[4]]);
        if msg[0] != AT_PID {
            return Err(explain(msg[0], value as i32));
        }
        let pid = value.to_string();
        let write_map = |helper: &str, id: u32| -> Result<(), String> {
            let out = Command::new(helper)
                .args([pid.as_str(), &NS_ID.to_string(), &id.to_string(), "1"])
                .env_clear()
                .stdin(Stdio::null())
                .output()
                .map_err(|e| format!("`{helper}` could not run: {e}"))?;
            if out.status.success() {
                Ok(())
            } else {
                let said = String::from_utf8_lossy(&out.stderr);
                Err(format!("`{helper}` refused the mapping: {}", said.trim()))
            }
        };
        let mapped = write_map(plan.newuidmap, plan.uid).and_then(|()| write_map(plan.newgidmap, plan.gid));
        let _ = to_child.write_all(&[u8::from(mapped.is_ok())]);
        drop(to_child);
        mapped?;
        // End of file here means the child reached `exec` (its copy closed on exec, the host's after
        // `spawn` returned). Anything else is the step that failed.
        match from_child.read_exact(&mut msg) {
            Ok(()) => Err(explain(msg[0], u32::from_le_bytes([msg[1], msg[2], msg[3], msg[4]]) as i32)),
            Err(_) => Ok(()),
        }
    }

    /// The child's half, between `fork` and `exec`. Only raw system calls: this runs in a copy of a
    /// multithreaded process, where anything that takes a lock may wait for ever.
    fn become_stranger(to_helper: RawFd, from_helper: RawFd) -> io::Result<()> {
        // SAFETY: every call is a raw system call on this process's own state or on the two pipe
        // descriptors `spawn_contained` made and keeps open until `exec`; the buffers outlive them.
        unsafe { become_stranger_raw(to_helper, from_helper) }
    }

    unsafe fn become_stranger_raw(to_helper: RawFd, from_helper: RawFd) -> io::Result<()> {
        let tell = |tag: u8, value: u32| {
            let v = value.to_le_bytes();
            let msg = [tag, v[0], v[1], v[2], v[3]];
            libc::write(to_helper, msg.as_ptr() as *const libc::c_void, msg.len());
        };
        let fail = |tag: u8| -> io::Result<()> {
            let errno = *libc::__errno_location();
            tell(tag, errno as u32);
            Err(io::Error::from_raw_os_error(errno))
        };
        if libc::unshare(libc::CLONE_NEWUSER) != 0 {
            return fail(AT_UNSHARE);
        }
        tell(AT_PID, libc::getpid() as u32);
        let mut go = [0u8; 1];
        if libc::read(from_helper, go.as_mut_ptr() as *mut libc::c_void, 1) != 1 || go[0] != 1 {
            // The helper could not write the maps and already knows why.
            return Err(io::Error::from_raw_os_error(libc::EPERM));
        }
        // Groups first: once the gid changes, the right to change them may be gone. An empty list,
        // because a supplementary group is exactly how the operator's files would stay reachable.
        if libc::syscall(libc::SYS_setgroups, 0usize, std::ptr::null::<libc::gid_t>()) != 0 {
            return fail(AT_SETGROUPS);
        }
        let id = NS_ID as libc::c_long;
        if libc::syscall(libc::SYS_setresgid, id, id, id) != 0 {
            return fail(AT_SETGID);
        }
        if libc::syscall(libc::SYS_setresuid, id, id, id) != 0 {
            return fail(AT_SETUID);
        }
        // The creator of a user namespace holds every capability inside it. `exec` as a non-root id
        // would clear them anyway; clearing them here as well means no step after this one — nor the
        // jail's own `pre_exec`, which runs next — acts with them.
        #[repr(C)]
        struct Header {
            version: u32,
            pid: i32,
        }
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct Data {
            effective: u32,
            permitted: u32,
            inheritable: u32,
        }
        let header = Header { version: 0x2008_0522, pid: 0 };
        let none = [Data { effective: 0, permitted: 0, inheritable: 0 }; 2];
        if libc::syscall(libc::SYS_capset, &header as *const Header, none.as_ptr()) != 0 {
            return fail(AT_CAPS);
        }
        Ok(())
    }

    fn pipe() -> Result<(std::fs::File, std::fs::File), String> {
        let mut fds = [0 as RawFd; 2];
        // SAFETY: `fds` is written by the call; both descriptors are owned immediately after.
        if unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) } != 0 {
            return Err(format!("no pipe: {}", io::Error::last_os_error()));
        }
        Ok(unsafe { (std::fs::File::from_raw_fd(fds[0]), std::fs::File::from_raw_fd(fds[1])) })
    }

    /// Start `exe args` as a stranger: a subordinate uid in a new user namespace, no groups, no
    /// capabilities, only `env` in its environment, its standard input the channel and its standard
    /// output nowhere. Standard error is captured when `capture_stderr` is set, and otherwise shared.
    /// `harden` adds the jail's own pre-`exec` steps; they run AFTER the identity change, because
    /// changing ids clears the parent-death signal the jail sets.
    pub fn spawn_contained(
        exe: &Path,
        args: &[&OsStr],
        env: &[(String, String)],
        capture_stderr: bool,
        harden: impl FnOnce(&mut Command),
    ) -> Result<ContainedGuest, String> {
        let plan = Plan::for_this_user()?;
        // Executed through this descriptor rather than by name: the stranger may not be able to walk
        // the operator's directories down to the binary, but a descriptor needs no walk — only the
        // file's own execute bit, which a built binary gives every account.
        let binary = std::fs::File::open(exe).map_err(|e| format!("the guest's binary cannot be opened: {e}"))?;
        let (host_end, guest_end) = UnixStream::pair().map_err(|e| format!("no channel: {e}"))?;
        let (from_child, to_helper) = pipe()?;
        let (from_helper, to_child) = pipe()?;
        let (to_helper_fd, from_helper_fd) = (to_helper.as_raw_fd(), from_helper.as_raw_fd());

        let mut cmd = Command::new(format!("/proc/self/fd/{}", binary.as_raw_fd()));
        cmd.arg0(exe).args(args).env_clear();
        for (k, v) in env {
            cmd.env(k, v);
        }
        // The operator's working directory may be closed to the stranger; the guest needs none.
        cmd.current_dir("/");
        cmd.stdin(Stdio::from(OwnedFd::from(guest_end)));
        cmd.stdout(Stdio::null());
        if capture_stderr {
            cmd.stderr(Stdio::piped());
        }
        // SAFETY: the hook runs in the forked child and `become_stranger` makes only raw system calls
        // on the two descriptors named, which stay open there until `exec` (they are close-on-exec,
        // so the guest never holds them).
        unsafe {
            cmd.pre_exec(move || become_stranger(to_helper_fd, from_helper_fd));
        }
        harden(&mut cmd);

        let mapping = std::thread::scope(|s| {
            let helper = s.spawn(|| map_ids(from_child, to_child, &plan));
            let spawned = cmd.spawn();
            // The host's copies go now, so the helper's last read ends when the child's do.
            drop(to_helper);
            drop(from_helper);
            let mapped = helper.join().unwrap_or_else(|_| Err("the id-mapping helper failed".into()));
            (spawned, mapped)
        });
        drop(binary);
        match mapping {
            (Ok(mut child), Ok(())) => {
                let stderr = child.stderr.take();
                Ok(ContainedGuest { child, channel: Some(host_end), stderr, uid: plan.uid })
            }
            (Ok(mut child), Err(why)) => {
                let _ = child.kill();
                let _ = child.wait();
                Err(why)
            }
            (Err(_), Err(why)) => Err(why),
            (Err(e), Ok(())) => Err(format!("the guest could not be started as a subordinate uid: {e}")),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn a_subordinate_range_is_found_by_name_or_uid_and_never_contains_the_operator() {
            let text = "# comment\nother:100000:65536\nme:165536:65536\n";
            assert_eq!(range_for(text, Some("me"), 1000), Some(Range { start: 165536, count: 65536 }));
            assert_eq!(range_for("1000:200000:10\n", Some("me"), 1000), Some(Range { start: 200000, count: 10 }));
            assert_eq!(range_for(text, Some("nobody-here"), 1000), None);
            // A range holding the operator's own uid would map the guest back onto the operator.
            assert_eq!(range_for("me:999:10\n", Some("me"), 1000), None);
            // Malformed, empty or wrapping ranges are skipped rather than trusted.
            assert_eq!(range_for("me:abc:10\nme:5:0\nme:4294967295:2\n", Some("me"), 1000), None);
        }

        const MODE: &str = "DELULU_IDENTITY_TEST_MODE";

        /// The child half of T14 on Linux. Without the variable it tests nothing and returns at once.
        /// It reports on its standard input — the channel socket, when it is started contained — so
        /// the host reads the answers the way it reads a guest's.
        #[test]
        fn t14_child() {
            let Ok(targets) = std::env::var(MODE) else { return };
            let t: Vec<&str> = targets.split('|').collect();
            let (secret, key, operator_dir, public) = (t[0], t[1], t[2], t[3]);
            let mut report = String::new();
            report.push_str(&format!("SECRET={}\n", std::fs::read(secret).is_ok()));
            report.push_str(&format!("STATE={}\n", std::fs::read(key).is_ok()));
            report.push_str(&format!("LIST={}\n", std::fs::read_dir(operator_dir).is_ok()));
            report.push_str(&format!("WRITE={}\n", std::fs::write(Path::new(operator_dir).join("escaped.txt"), b"x").is_ok()));
            report.push_str(&format!("PUBLIC={}\n", std::fs::read(public).is_ok()));
            // SAFETY: plain queries of this process's own credentials.
            let (uid, groups) = unsafe { (libc::getuid(), libc::getgroups(0, std::ptr::null_mut())) };
            report.push_str(&format!("UID={uid}\nGROUPS={groups}\n"));
            if std::env::var("DELULU_IDENTITY_TEST_CHANNEL").is_ok() {
                // SAFETY: descriptor 0 is the channel socket this child was started with.
                let mut chan = unsafe { UnixStream::from_raw_fd(0) };
                let _ = chan.write_all(report.as_bytes());
            } else {
                print!("{report}");
            }
        }

        /// **T14 on Linux**: a guest started as a subordinate uid cannot read the operator's files or
        /// the state directory. Measured with a control — the same binary, the same attempts, as the
        /// operator, which must succeed at every one — and a boundary check in the other direction:
        /// a file every account may read stays readable, because that is all the guarantee claims.
        ///
        /// A host that forbids the namespace (Ubuntu's default) cannot run the measurement; the test
        /// says so and passes, UNLESS `DELULU_REQUIRE_SUBORDINATE_UID` is set — as it is on the CI job
        /// whose runner is configured to allow it — so on that job this is a gate that can fail.
        #[test]
        fn t14_a_subordinate_uid_guest_cannot_read_the_operators_files_or_the_state_directory() {
            use std::os::unix::fs::PermissionsExt as _;
            let base = std::env::temp_dir().join(format!("delulu-t14-{}", std::process::id()));
            let (operator, state, public_dir) = (base.join("operator"), base.join("state"), base.join("public"));
            for (d, mode) in [(&base, 0o755), (&operator, 0o700), (&state, 0o700), (&public_dir, 0o755)] {
                std::fs::create_dir_all(d).unwrap();
                std::fs::set_permissions(d, std::fs::Permissions::from_mode(mode)).unwrap();
            }
            for (f, mode) in [
                (operator.join("secret.txt"), 0o600),
                (state.join("broker.key"), 0o600),
                (public_dir.join("readme.txt"), 0o644),
            ] {
                std::fs::write(&f, b"x").unwrap();
                std::fs::set_permissions(&f, std::fs::Permissions::from_mode(mode)).unwrap();
            }
            let targets = format!(
                "{}|{}|{}|{}",
                operator.join("secret.txt").display(),
                state.join("broker.key").display(),
                operator.display(),
                public_dir.join("readme.txt").display()
            );
            let args = ["--exact", "identity::linux::tests::t14_child", "--nocapture", "--test-threads=1"];
            let exe = std::env::current_exe().unwrap();

            let free = Command::new(&exe).args(args).env(MODE, &targets).output().unwrap();
            let free = String::from_utf8_lossy(&free.stdout).to_string();
            for want in ["SECRET=true", "STATE=true", "LIST=true", "WRITE=true", "PUBLIC=true"] {
                assert!(free.contains(want), "the operator's control could not do `{want}`, so nothing below would be measured:\n{free}");
            }
            let _ = std::fs::remove_file(operator.join("escaped.txt"));

            let mut env: Vec<(String, String)> =
                crate::guest::LOADER_ENV.iter().filter_map(|k| std::env::var(k).ok().map(|v| (k.to_string(), v))).collect();
            env.push((MODE.to_string(), targets));
            env.push(("DELULU_IDENTITY_TEST_CHANNEL".to_string(), "1".to_string()));
            let os: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
            let mut g = match spawn_contained(&exe, &os, &env, true, |_| {}) {
                Ok(g) => g,
                Err(why) => {
                    assert!(
                        std::env::var_os("DELULU_REQUIRE_SUBORDINATE_UID").is_none(),
                        "this host is configured to give a guest a subordinate uid, and it could not: {why}"
                    );
                    eprintln!("NOT MEASURED on this host: {why}");
                    let _ = std::fs::remove_dir_all(&base);
                    return;
                }
            };
            let mut out = String::new();
            let _ = g.channel.take().unwrap().read_to_string(&mut out);
            let mut err = String::new();
            let _ = g.stderr.take().unwrap().read_to_string(&mut err);
            let status = g.wait().unwrap();
            assert!(out.contains("SECRET="), "the contained child ran the probe ({status}):\n{out}\n{err}");
            for want in ["SECRET=false", "STATE=false", "LIST=false", "WRITE=false"] {
                assert!(out.contains(want), "T14: the subordinate-uid guest was not refused `{want}`:\n{out}");
            }
            assert!(out.contains("PUBLIC=true"), "a file every account may read stays readable — the claim is no wider:\n{out}");
            // SAFETY: getuid cannot fail.
            let me = unsafe { libc::getuid() };
            assert!(out.contains(&format!("UID={NS_ID}\n")), "inside its namespace the guest is uid {NS_ID}:\n{out}");
            assert!(out.contains("GROUPS=0\n"), "and holds no supplementary group:\n{out}");
            assert_ne!(g.uid, me, "and outside it, it is not the operator");
            assert!(!operator.join("escaped.txt").exists(), "and nothing was written");
            let _ = std::fs::remove_dir_all(&base);
        }
    }
}

#[cfg(windows)]
mod win {
    use std::ffi::OsStr;
    use std::io;
    use std::os::windows::ffi::OsStrExt as _;
    use std::os::windows::io::{AsRawHandle, FromRawHandle as _, OwnedHandle, RawHandle};
    use std::os::windows::process::ExitStatusExt as _;
    use std::path::{Path, PathBuf};
    use std::process::ExitStatus;

    use windows_sys::Win32::Foundation::{
        DuplicateHandle, GetLastError, LocalFree, SetHandleInformation, DUPLICATE_SAME_ACCESS, GENERIC_EXECUTE,
        GENERIC_READ, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE, WAIT_OBJECT_0,
    };
    use windows_sys::Win32::Security::Authorization::{
        ConvertStringSidToSidW, GetNamedSecurityInfoW, SetEntriesInAclW, SetNamedSecurityInfoW, EXPLICIT_ACCESS_W,
        GRANT_ACCESS, NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT, TRUSTEE_IS_SID, TRUSTEE_IS_WELL_KNOWN_GROUP, TRUSTEE_W,
    };
    use windows_sys::Win32::Security::Isolation::{
        CreateAppContainerProfile, DeleteAppContainerProfile, DeriveAppContainerSidFromAppContainerName,
    };
    use windows_sys::Win32::Security::{
        FreeSid, ACL, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR, PSID, SECURITY_ATTRIBUTES,
        SECURITY_CAPABILITIES, SUB_CONTAINERS_AND_OBJECTS_INHERIT,
    };
    use windows_sys::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE};
    use windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW;
    use windows_sys::Win32::System::Pipes::CreatePipe;
    use windows_sys::Win32::System::ProcessStatus::K32EnumProcessModules;
    use windows_sys::Win32::System::Threading::{
        CreateProcessW, DeleteProcThreadAttributeList, GetCurrentProcess, GetExitCodeProcess,
        InitializeProcThreadAttributeList, OpenProcess, ResumeThread, TerminateProcess, UpdateProcThreadAttribute,
        WaitForSingleObject, CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, INFINITE,
        LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION,
        PROC_THREAD_ATTRIBUTE_HANDLE_LIST, PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES, STARTF_USESTDHANDLES,
        STARTUPINFOEXW,
    };

    fn wide(s: &OsStr) -> Vec<u16> {
        s.encode_wide().chain(Some(0)).collect()
    }

    /// `HRESULT_FROM_WIN32(ERROR_ALREADY_EXISTS)`.
    const ALREADY_EXISTS: i32 = 0x800700B7_u32 as i32;
    /// The well-known SID every AppContainer's token carries.
    const ALL_APPLICATION_PACKAGES: &str = "S-1-15-2-1";
    const PROFILE_PREFIX: &str = "delulu.guest.";

    /// One run's AppContainer. Dropping it deletes the profile.
    struct Container {
        name: Vec<u16>,
        sid: PSID,
    }

    impl Container {
        fn create() -> Result<Container, String> {
            static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            prune_stale_profiles();
            let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let name = format!("{PROFILE_PREFIX}{}.{n}", std::process::id());
            let w = wide(OsStr::new(&name));
            let mut sid: PSID = std::ptr::null_mut();
            // SAFETY: `w` is NUL-terminated and outlives the call; `sid` is written by the call and
            // freed with `FreeSid` in `Drop`, as the documentation requires.
            let hr = unsafe { CreateAppContainerProfile(w.as_ptr(), w.as_ptr(), w.as_ptr(), std::ptr::null(), 0, &mut sid) };
            if hr == ALREADY_EXISTS {
                // A profile of this name survived a crashed run of a process with our pid. Reusing its
                // SID is safe (it holds no capabilities either); it is deleted with this one.
                let hr = unsafe { DeriveAppContainerSidFromAppContainerName(w.as_ptr(), &mut sid) };
                if hr != 0 {
                    return Err(format!("the AppContainer profile exists and its SID cannot be derived (0x{:08x})", hr as u32));
                }
            } else if hr != 0 {
                return Err(format!("CreateAppContainerProfile failed (0x{:08x})", hr as u32));
            }
            Ok(Container { name: w, sid })
        }
    }

    impl Drop for Container {
        fn drop(&mut self) {
            // SAFETY: both were produced by `create` and are released exactly once.
            unsafe {
                DeleteAppContainerProfile(self.name.as_ptr());
                FreeSid(self.sid);
            }
        }
    }

    /// Profiles left by runs that died before deleting theirs (a killed host). Best effort, and only
    /// ours: the name carries the creating process's id, and a profile is deleted only when no process
    /// with that id is running.
    fn prune_stale_profiles() {
        let Some(base) = std::env::var_os("LOCALAPPDATA").map(|p| PathBuf::from(p).join("Packages")) else { return };
        let Ok(entries) = std::fs::read_dir(&base) else { return };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let Some(rest) = name.strip_prefix(PROFILE_PREFIX) else { continue };
            let Some(pid) = rest.split('.').next().and_then(|p| p.parse::<u32>().ok()) else { continue };
            if pid == std::process::id() || process_alive(pid) {
                continue;
            }
            let w = wide(OsStr::new(&name));
            // SAFETY: a NUL-terminated name; the call only reads it.
            unsafe { DeleteAppContainerProfile(w.as_ptr()) };
        }
    }

    fn process_alive(pid: u32) -> bool {
        // SAFETY: the handle, when there is one, is owned and closed at once.
        let h = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if h.is_null() {
            return false;
        }
        let owned = unsafe { OwnedHandle::from_raw_handle(h as RawHandle) };
        let mut code = 0u32;
        let ok = unsafe { GetExitCodeProcess(owned.as_raw_handle() as HANDLE, &mut code) };
        // STILL_ACTIVE (259) while it runs.
        ok != 0 && code == 259
    }

    /// Every module this process has loaded from outside the Windows directory, the executable first.
    fn own_modules() -> Result<Vec<PathBuf>, String> {
        let exe = std::env::current_exe().map_err(|e| format!("cannot name this executable: {e}"))?;
        let mut out = vec![exe];
        let mut handles = vec![std::ptr::null_mut(); 512];
        let mut needed = 0u32;
        // SAFETY: the buffer is sized in bytes as the call requires; handles it returns are borrowed
        // module handles that need no release.
        let ok = unsafe {
            K32EnumProcessModules(
                GetCurrentProcess(),
                handles.as_mut_ptr(),
                (handles.len() * std::mem::size_of::<HANDLE>()) as u32,
                &mut needed,
            )
        };
        if ok == 0 {
            return Err("cannot list this process's modules".into());
        }
        let count = (needed as usize / std::mem::size_of::<HANDLE>()).min(handles.len());
        let system = std::env::var_os("SystemRoot").map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
        for &h in &handles[..count] {
            let mut buf = vec![0u16; 1024];
            let n = unsafe { GetModuleFileNameW(h as _, buf.as_mut_ptr(), buf.len() as u32) } as usize;
            if n == 0 || n >= buf.len() {
                continue;
            }
            let p = PathBuf::from(String::from_utf16_lossy(&buf[..n]));
            let lower = p.to_string_lossy().to_lowercase();
            if !system.is_empty() && lower.starts_with(&system) {
                continue;
            }
            if !out.iter().any(|q| q.to_string_lossy().eq_ignore_ascii_case(&p.to_string_lossy())) {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// Grant read and execute on `dir` (and what is created in it) to every AppContainer.
    fn grant_all_application_packages(dir: &Path) -> Result<(), String> {
        let path = wide(dir.as_os_str());
        let sid_text = wide(OsStr::new(ALL_APPLICATION_PACKAGES));
        // SAFETY: every out-pointer is written by the call that owns it and released with `LocalFree`
        // below; the explicit-access entry points at a SID that lives until the ACL is built.
        unsafe {
            let mut sid: PSID = std::ptr::null_mut();
            if ConvertStringSidToSidW(sid_text.as_ptr(), &mut sid) == 0 {
                return Err(format!("the ALL APPLICATION PACKAGES SID cannot be built ({})", GetLastError()));
            }
            let mut old: *mut ACL = std::ptr::null_mut();
            let mut sd: PSECURITY_DESCRIPTOR = std::ptr::null_mut();
            let e = GetNamedSecurityInfoW(
                path.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut old,
                std::ptr::null_mut(),
                &mut sd,
            );
            if e != 0 {
                LocalFree(sid as _);
                return Err(format!("cannot read the runtime directory's security ({e})"));
            }
            let entry = EXPLICIT_ACCESS_W {
                grfAccessPermissions: GENERIC_READ | GENERIC_EXECUTE,
                grfAccessMode: GRANT_ACCESS,
                grfInheritance: SUB_CONTAINERS_AND_OBJECTS_INHERIT,
                Trustee: TRUSTEE_W {
                    pMultipleTrustee: std::ptr::null_mut(),
                    MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
                    TrusteeForm: TRUSTEE_IS_SID,
                    TrusteeType: TRUSTEE_IS_WELL_KNOWN_GROUP,
                    ptstrName: sid as *mut u16,
                },
            };
            let mut new: *mut ACL = std::ptr::null_mut();
            let e = SetEntriesInAclW(1, &entry, old, &mut new);
            let r = if e != 0 {
                Err(format!("cannot build the runtime directory's grant ({e})"))
            } else {
                let e = SetNamedSecurityInfoW(
                    path.as_ptr(),
                    SE_FILE_OBJECT,
                    DACL_SECURITY_INFORMATION,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    new,
                    std::ptr::null(),
                );
                if e != 0 { Err(format!("cannot grant the runtime directory ({e})")) } else { Ok(()) }
            };
            LocalFree(new as _);
            LocalFree(sd as _);
            LocalFree(sid as _);
            r
        }
    }

    /// The runtime copy: this executable and its non-system libraries, in a directory an AppContainer
    /// may read. Built once per build (keyed by each file's path, size and modification time), in a
    /// scratch directory renamed into place only when complete, so a concurrent run never sees half
    /// a copy. Returns the copied executable.
    fn runtime_copy() -> Result<PathBuf, String> {
        let modules = own_modules()?;
        let mut key = blake3::Hasher::new();
        for m in &modules {
            let md = std::fs::metadata(m).map_err(|e| format!("cannot read `{}`: {e}", m.display()))?;
            key.update(m.as_os_str().as_encoded_bytes());
            key.update(&md.len().to_le_bytes());
            let t = md.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).unwrap_or_default();
            key.update(&t.as_nanos().to_le_bytes());
        }
        let key = key.finalize().to_hex()[..16].to_string();
        let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        let base = local.join("DeluluLang").join("guest-runtime");
        let dir = base.join(&key);
        let exe_name = modules[0].file_name().ok_or("the executable has no file name")?.to_owned();
        prune_unused_copies(&base, &key);
        if let Some(exe) = complete(&dir, &exe_name) {
            return Ok(exe);
        }
        std::fs::create_dir_all(&base).map_err(|e| format!("cannot create `{}`: {e}", base.display()))?;
        let tmp = base.join(format!("{key}.tmp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).map_err(|e| format!("cannot create `{}`: {e}", tmp.display()))?;
        // The grant FIRST, so every file copied in inherits it rather than being fixed up afterwards.
        grant_all_application_packages(&tmp)?;
        for m in &modules {
            let name = m.file_name().ok_or("a module has no file name")?;
            std::fs::copy(m, tmp.join(name)).map_err(|e| format!("cannot copy `{}`: {e}", m.display()))?;
        }
        std::fs::write(tmp.join(COMPLETE), b"").map_err(|e| format!("cannot finish the runtime copy: {e}"))?;
        if std::fs::rename(&tmp, &dir).is_ok() {
            return Ok(dir.join(exe_name));
        }
        // Another run finished the same copy first, and theirs is as good as ours.
        if let Some(exe) = complete(&dir, &exe_name) {
            let _ = std::fs::remove_dir_all(&tmp);
            return Ok(exe);
        }
        // What is there is a broken copy (one whose unlocked files an interrupted cleanup removed).
        // Ours is complete, so it is used where it stands; the broken one is pruned once unused.
        Ok(tmp.join(exe_name))
    }

    const COMPLETE: &str = ".complete";

    /// A finished copy's executable, touching its marker: the marker's time is when the copy was last
    /// USED, which is what pruning goes by.
    fn complete(dir: &Path, exe_name: &OsStr) -> Option<PathBuf> {
        let marker = dir.join(COMPLETE);
        let exe = dir.join(exe_name);
        if !marker.is_file() || !exe.is_file() {
            return None;
        }
        if let Ok(f) = std::fs::OpenOptions::new().write(true).open(&marker) {
            let _ = f.set_modified(std::time::SystemTime::now());
        }
        Some(exe)
    }

    /// Copies no run has used for a day: other builds, abandoned scratch copies, broken ones. By
    /// last use rather than by "not this build", because two builds are often in use at once — the
    /// command and a test binary — and each deleting the other's copy would pull a running guest's
    /// files out from under it. (Windows would refuse to delete the running image, and delete the
    /// rest, leaving a copy that is neither whole nor gone.)
    fn prune_unused_copies(base: &Path, current: &str) {
        let Ok(entries) = std::fs::read_dir(base) else { return };
        let day = std::time::Duration::from_secs(24 * 60 * 60);
        for e in entries.flatten() {
            if e.file_name().to_string_lossy() == current {
                continue;
            }
            let path = e.path();
            let last = std::fs::metadata(path.join(COMPLETE))
                .or_else(|_| std::fs::metadata(&path))
                .and_then(|m| m.modified())
                .ok();
            let idle = last.and_then(|t| t.elapsed().ok()).is_some_and(|d| d > day);
            if idle {
                let _ = std::fs::remove_dir_all(&path);
            }
        }
    }

    /// Quote one argument the way `CommandLineToArgvW` reads it back.
    fn quote(arg: &OsStr, out: &mut Vec<u16>) {
        let s: Vec<u16> = arg.encode_wide().collect();
        let needs = s.is_empty() || s.iter().any(|&c| c == b' ' as u16 || c == b'\t' as u16 || c == b'"' as u16);
        if !needs {
            out.extend_from_slice(&s);
            return;
        }
        out.push(b'"' as u16);
        let mut backslashes = 0usize;
        for &c in &s {
            if c == b'\\' as u16 {
                backslashes += 1;
                continue;
            }
            if c == b'"' as u16 {
                out.extend(std::iter::repeat_n(b'\\' as u16, backslashes * 2 + 1));
            } else {
                out.extend(std::iter::repeat_n(b'\\' as u16, backslashes));
            }
            backslashes = 0;
            out.push(c);
        }
        out.extend(std::iter::repeat_n(b'\\' as u16, backslashes * 2));
        out.push(b'"' as u16);
    }

    /// A guest started in its own AppContainer, suspended, with its channel on an inherited pipe.
    pub struct ContainedGuest {
        process: OwnedHandle,
        thread: OwnedHandle,
        exited: Option<u32>,
        /// The host's end of the channel: one duplex pipe, the guest's standard input and output.
        pub channel: Option<crate::pipe_channel::HostPipe>,
        /// The guest's standard error, when the caller asked to capture it.
        pub stderr: Option<std::fs::File>,
        // Declared last so it is dropped after the process has been waited for in `Drop`.
        _container: Container,
    }

    impl ContainedGuest {
        /// Start the suspended guest. `false` means it could not be, and must not be waited on.
        pub fn resume(&self) -> bool {
            // SAFETY: the thread handle is owned by this value.
            unsafe { ResumeThread(self.thread.as_raw_handle() as HANDLE) != u32::MAX }
        }

        pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
            if let Some(c) = self.exited {
                return Ok(Some(ExitStatus::from_raw(c)));
            }
            // SAFETY: the process handle is owned by this value.
            if unsafe { WaitForSingleObject(self.process.as_raw_handle() as HANDLE, 0) } != WAIT_OBJECT_0 {
                return Ok(None);
            }
            self.collect().map(Some)
        }

        pub fn wait(&mut self) -> io::Result<ExitStatus> {
            if let Some(c) = self.exited {
                return Ok(ExitStatus::from_raw(c));
            }
            // SAFETY: as above.
            unsafe { WaitForSingleObject(self.process.as_raw_handle() as HANDLE, INFINITE) };
            self.collect()
        }

        fn collect(&mut self) -> io::Result<ExitStatus> {
            let mut code = 0u32;
            // SAFETY: as above.
            if unsafe { GetExitCodeProcess(self.process.as_raw_handle() as HANDLE, &mut code) } == 0 {
                return Err(io::Error::last_os_error());
            }
            self.exited = Some(code);
            Ok(ExitStatus::from_raw(code))
        }

        pub fn kill(&mut self) -> io::Result<()> {
            if self.exited.is_some() {
                return Ok(());
            }
            // SAFETY: as above. Terminating a process that has just exited fails harmlessly.
            unsafe { TerminateProcess(self.process.as_raw_handle() as HANDLE, 1) };
            Ok(())
        }
    }

    impl AsRawHandle for ContainedGuest {
        fn as_raw_handle(&self) -> RawHandle {
            self.process.as_raw_handle()
        }
    }

    impl Drop for ContainedGuest {
        fn drop(&mut self) {
            // The profile must not be deleted under a running process; a guest is never left behind.
            if self.exited.is_none() {
                let _ = self.kill();
                unsafe { WaitForSingleObject(self.process.as_raw_handle() as HANDLE, 5_000) };
            }
        }
    }

    /// An inheritable pipe: returns (the end the child inherits, the end the host keeps). Used for a
    /// captured standard error; the channel itself is `pipe_channel::duplex_pair`.
    fn pipe(child_reads: bool) -> Result<(OwnedHandle, OwnedHandle), String> {
        let sa = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: std::ptr::null_mut(),
            bInheritHandle: 1,
        };
        let (mut r, mut w): (HANDLE, HANDLE) = (std::ptr::null_mut(), std::ptr::null_mut());
        // SAFETY: both handles are written by the call and immediately owned.
        if unsafe { CreatePipe(&mut r, &mut w, &sa, 0) } == 0 {
            return Err(format!("CreatePipe failed ({})", unsafe { GetLastError() }));
        }
        let (r, w) = unsafe { (OwnedHandle::from_raw_handle(r as RawHandle), OwnedHandle::from_raw_handle(w as RawHandle)) };
        let (child, host) = if child_reads { (r, w) } else { (w, r) };
        // The host's end must not leak into the child, or the child would hold both ends and the
        // pipe would never report its end.
        unsafe { SetHandleInformation(host.as_raw_handle() as HANDLE, HANDLE_FLAG_INHERIT, 0) };
        Ok((child, host))
    }

    /// Start `args` of this program's runtime copy in a new AppContainer, SUSPENDED, with only
    /// `env` in its environment and its standard input and output as the channel. Standard error is
    /// captured when `capture_stderr` is set, and otherwise shared with this process's.
    pub fn spawn_contained(args: &[&OsStr], env: &[(String, String)], capture_stderr: bool) -> Result<ContainedGuest, String> {
        let exe = runtime_copy()?;
        let container = Container::create()?;
        // One duplex pipe: the guest's standard input and output are two handles on its end, the
        // host reads its own end directly with a deadline (`pipe_channel.rs`).
        let (host_end, in_child, out_child) = crate::pipe_channel::duplex_pair()?;
        let (err_child, err_host): (Option<OwnedHandle>, Option<OwnedHandle>) = if capture_stderr {
            let (c, h) = pipe(false)?;
            (Some(c), Some(h))
        } else {
            // This process's standard error, duplicated as an inheritable handle so it can be named in
            // the child's inheritance list. None when there is no standard error to share.
            let mine = unsafe { GetStdHandle(STD_ERROR_HANDLE) };
            let mut dup: HANDLE = std::ptr::null_mut();
            let ok = !mine.is_null()
                && mine != INVALID_HANDLE_VALUE
                && unsafe { DuplicateHandle(GetCurrentProcess(), mine, GetCurrentProcess(), &mut dup, 0, 1, DUPLICATE_SAME_ACCESS) } != 0;
            (if ok { Some(unsafe { OwnedHandle::from_raw_handle(dup as RawHandle) }) } else { None }, None)
        };

        let mut cmd: Vec<u16> = Vec::new();
        quote(exe.as_os_str(), &mut cmd);
        for a in args {
            cmd.push(b' ' as u16);
            quote(a, &mut cmd);
        }
        cmd.push(0);
        // Windows REQUIRES `LOCALAPPDATA` to start an AppContainer from an explicit environment: it
        // rewrites the variable to the container's own folder, and without it `CreateProcessW` fails
        // with 203, "the system could not find the environment option" (measured: the loader's four
        // variables alone fail; adding this one alone succeeds). The guest sees its container's
        // folder, not the operator's, which it could not read anyway.
        let mut env: Vec<(String, String)> = env.to_vec();
        if !env.iter().any(|(k, _)| k.eq_ignore_ascii_case("LOCALAPPDATA")) {
            if let Ok(v) = std::env::var("LOCALAPPDATA") {
                env.push(("LOCALAPPDATA".to_string(), v));
            }
        }
        let mut block: Vec<u16> = Vec::new();
        for (k, v) in &env {
            block.extend(OsStr::new(&format!("{k}={v}")).encode_wide());
            block.push(0);
        }
        block.push(0);
        if env.is_empty() {
            block.push(0);
        }
        let app = wide(exe.as_os_str());
        let cwd = wide(exe.parent().unwrap_or(Path::new(".")).as_os_str());

        let mut inherit: Vec<HANDLE> = vec![in_child.as_raw_handle() as HANDLE, out_child.as_raw_handle() as HANDLE];
        if let Some(e) = &err_child {
            inherit.push(e.as_raw_handle() as HANDLE);
        }
        // SAFETY: the attribute list lives in `buf` until `DeleteProcThreadAttributeList`; the values
        // it points at (`caps`, `inherit`) outlive `CreateProcessW`; every handle named is owned here.
        unsafe {
            let mut size = 0usize;
            InitializeProcThreadAttributeList(std::ptr::null_mut(), 2, 0, &mut size);
            let mut buf = vec![0u8; size];
            let list = buf.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;
            if InitializeProcThreadAttributeList(list, 2, 0, &mut size) == 0 {
                return Err(format!("InitializeProcThreadAttributeList failed ({})", GetLastError()));
            }
            // No capabilities: no network of any kind, no library, no device.
            let caps = SECURITY_CAPABILITIES {
                AppContainerSid: container.sid,
                Capabilities: std::ptr::null_mut(),
                CapabilityCount: 0,
                Reserved: 0,
            };
            let set = |attr: u32, value: *const core::ffi::c_void, size: usize| {
                UpdateProcThreadAttribute(list, 0, attr as usize, value, size, std::ptr::null_mut(), std::ptr::null()) != 0
            };
            if !set(PROC_THREAD_ATTRIBUTE_SECURITY_CAPABILITIES, &caps as *const _ as _, std::mem::size_of::<SECURITY_CAPABILITIES>())
                || !set(PROC_THREAD_ATTRIBUTE_HANDLE_LIST, inherit.as_ptr() as _, inherit.len() * std::mem::size_of::<HANDLE>())
            {
                let e = GetLastError();
                DeleteProcThreadAttributeList(list);
                return Err(format!("cannot describe the contained process ({e})"));
            }
            let mut si: STARTUPINFOEXW = std::mem::zeroed();
            si.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
            si.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
            si.StartupInfo.hStdInput = in_child.as_raw_handle() as HANDLE;
            si.StartupInfo.hStdOutput = out_child.as_raw_handle() as HANDLE;
            si.StartupInfo.hStdError = err_child.as_ref().map_or(std::ptr::null_mut(), |e| e.as_raw_handle() as HANDLE);
            si.lpAttributeList = list;
            let mut pi: PROCESS_INFORMATION = std::mem::zeroed();
            let ok = CreateProcessW(
                app.as_ptr(),
                cmd.as_mut_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1,
                EXTENDED_STARTUPINFO_PRESENT | CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT,
                block.as_ptr() as _,
                cwd.as_ptr(),
                &si.StartupInfo,
                &mut pi,
            );
            let err = GetLastError();
            DeleteProcThreadAttributeList(list);
            if ok == 0 {
                return Err(format!("the contained guest could not be created (error {err})"));
            }
            // The child's ends are closed here, in the host: from now on the pipes end when the guest
            // does.
            drop((in_child, out_child, err_child));
            Ok(ContainedGuest {
                process: OwnedHandle::from_raw_handle(pi.hProcess as RawHandle),
                thread: OwnedHandle::from_raw_handle(pi.hThread as RawHandle),
                exited: None,
                channel: Some(host_end),
                stderr: err_host.map(std::fs::File::from),
                _container: container,
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn arguments_are_quoted_the_way_windows_reads_them_back() {
            let mut out = Vec::new();
            for a in ["plain", "with space", "", "a\"b", "tail\\", "sp ace\\", "c:\\dir\\x"] {
                out.clear();
                quote(OsStr::new(a), &mut out);
                let s = String::from_utf16(&out).unwrap();
                let expected = match a {
                    "plain" => "plain",
                    "with space" => "\"with space\"",
                    "" => "\"\"",
                    "a\"b" => "\"a\\\"b\"",
                    "tail\\" => "tail\\",
                    "sp ace\\" => "\"sp ace\\\\\"",
                    "c:\\dir\\x" => "c:\\dir\\x",
                    _ => unreachable!(),
                };
                assert_eq!(s, expected, "`{a}`");
            }
        }

        const MODE: &str = "DELULU_IDENTITY_TEST_MODE";

        /// The child half of T14. Without the variable it tests nothing and returns at once; the test
        /// below re-enters this binary to run it, once contained and once not.
        #[test]
        fn t14_child() {
            let Ok(targets) = std::env::var(MODE) else { return };
            let t: Vec<&str> = targets.split('|').collect();
            let (secret, key, operator_dir, port) = (t[0], t[1], t[2], t[3]);
            println!("SECRET={}", std::fs::read(secret).is_ok());
            println!("STATE={}", std::fs::read(key).is_ok());
            println!("LIST={}", std::fs::read_dir(operator_dir).is_ok());
            println!("WRITE={}", std::fs::write(Path::new(operator_dir).join("escaped.txt"), b"x").is_ok());
            let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port.parse::<u16>().unwrap()));
            println!("CONNECT={}", std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(1500)).is_ok());
            println!("LOCALAPPDATA={}", std::env::var("LOCALAPPDATA").unwrap_or_default());
        }

        /// **T14** (`SANDBOX_THREAT_MODEL.md`): a guest launched as a different principal cannot read
        /// the broker's key or the state directory. Measured, not asserted — the same attempts are made
        /// twice, by the same binary: once as the operator (the control, which must succeed at every
        /// one of them, or the refusals below would prove nothing) and once in a per-run AppContainer
        /// started exactly as a sandboxed run starts its guest.
        #[test]
        fn t14_a_contained_guest_cannot_read_the_state_directory_or_reach_the_network() {
            let base = std::env::temp_dir().join(format!("delulu-t14-{}", std::process::id()));
            let (operator, state) = (base.join("operator"), base.join("state"));
            std::fs::create_dir_all(&operator).unwrap();
            std::fs::create_dir_all(&state).unwrap();
            std::fs::write(operator.join("secret.txt"), b"the operator's").unwrap();
            std::fs::write(state.join("broker.key"), b"a key").unwrap();
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            std::thread::spawn(move || {
                for c in listener.incoming() {
                    drop(c);
                }
            });
            let targets = format!(
                "{}|{}|{}|{port}",
                operator.join("secret.txt").display(),
                state.join("broker.key").display(),
                operator.display()
            );
            let args = ["--exact", "identity::win::tests::t14_child", "--nocapture", "--test-threads=1"];

            let free = std::process::Command::new(std::env::current_exe().unwrap())
                .args(args)
                .env(MODE, &targets)
                .output()
                .unwrap();
            let free = String::from_utf8_lossy(&free.stdout).to_string();
            for want in ["SECRET=true", "STATE=true", "LIST=true", "WRITE=true", "CONNECT=true"] {
                assert!(free.contains(want), "the uncontained control could not do `{want}`, so nothing below would be measured:\n{free}");
            }
            let _ = std::fs::remove_file(operator.join("escaped.txt"));

            let mut env: Vec<(String, String)> =
                crate::guest::LOADER_ENV.iter().filter_map(|k| std::env::var(k).ok().map(|v| (k.to_string(), v))).collect();
            env.push((MODE.to_string(), targets));
            let os: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
            let mut g = spawn_contained(&os, &env, true).expect("a contained guest starts on this host");
            assert!(g.resume(), "the contained guest resumes");
            let mut out = String::new();
            use std::io::Read as _;
            let _ = g.channel.take().unwrap().read_to_string(&mut out);
            let mut err = String::new();
            let _ = g.stderr.take().unwrap().read_to_string(&mut err);
            let status = g.wait().unwrap();
            assert!(out.contains("SECRET="), "the contained child ran the probe ({status}):\n{out}\n{err}");
            for want in ["SECRET=false", "STATE=false", "LIST=false", "WRITE=false", "CONNECT=false"] {
                assert!(out.contains(want), "T14: the contained guest was not refused `{want}`:\n{out}");
            }
            assert!(!operator.join("escaped.txt").exists(), "and nothing was written");
            // The one variable the OS makes us pass is not the operator's once the guest sees it.
            let theirs = out.lines().find_map(|l| l.strip_prefix("LOCALAPPDATA=")).unwrap_or("").trim().to_string();
            let ours = std::env::var("LOCALAPPDATA").unwrap_or_default();
            assert!(
                !theirs.is_empty() && !theirs.eq_ignore_ascii_case(&ours),
                "the guest's LOCALAPPDATA should be its container's folder, not the operator's `{ours}`: `{theirs}`"
            );
            let _ = std::fs::remove_dir_all(&base);
        }
    }
}
