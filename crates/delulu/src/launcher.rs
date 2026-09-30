//! PS-E-04 (`V2_OPENSHELL_STUDY.md` §4.4): an external launcher, resolved once, hashed, pinnable — and,
//! on Linux, started as the very file that was hashed.
//!
//! `--sandbox-backend external:CMD` handed the command's first word to the operating system at spawn,
//! which looks a bare name up on `PATH` — an empty or relative entry included, so with `.` ahead of the
//! operator's directory a `lnch` planted in the working directory ran instead (LAUNCHER-SPELL-1,
//! witnessed on `cfc5b01`) — and the report named the word, never the bytes. ADAPTER-SPELL-1 was the
//! same shape for hardware drivers (D-V2-50), and the same resolver answers both now:
//! [`crate::cli::resolve_driver`]. The resolved file is opened once and its bytes hashed with BLAKE3
//! (the repository's file hash); the report and the `sandbox-launch` record carry the path and the
//! digest; `--launcher-digest HEX` refuses any other file before anything starts.
//!
//! What starts is, on Linux, the descriptor the digest was read from (`fexecve`), so a path swapped
//! between the hash and the start — by anyone who can write the launcher's directory, or a link on its
//! way — is not what runs. On Windows, which has no `fexecve`, the file is held open from the hash until
//! the run ends, sharing reads only: it cannot be written, or renamed or deleted away — a rename over it
//! is a deletion — so the path started names the bytes hashed (witnessed on Windows first: a launcher
//! renamed over while the host hashed it was what ran). On macOS the resolved path is started, and that
//! window stays open to such a writer; the deployment rule for a driver holds for a launcher too
//! (`DEPLOYMENT.md` §5: only the operator can write where it lives). Linux does not close a change to the
//! file's own BYTES, in place, by someone who may write the file itself; Windows' share mode does.

use std::io::{self, Read as _};
use std::path::PathBuf;

/// The launcher this run will start: where its word resolved, the digest of its bytes, and the open file
/// those bytes were read from.
pub(crate) struct Launcher {
    /// The absolute path the operator's word resolved to. Links are not resolved (`resolve_driver`).
    pub(crate) path: PathBuf,
    /// BLAKE3 of the bytes read from `file`, in lowercase hex.
    pub(crate) blake3: String,
    /// The file the digest was read from: on Linux, the one started; on Windows, held — sharing reads
    /// only — for as long as this value lives, which is the run.
    file: std::fs::File,
    /// A `#!` script: its interpreter reads it back through `/dev/fd/N`, so on Linux the descriptor must
    /// survive the start rather than close with it.
    script: bool,
}

/// `--launcher-digest HEX`: 64 hexadecimal characters, in either case, kept in lowercase.
pub(crate) fn pinned_digest(given: &str) -> Result<String, String> {
    let g = given.trim();
    if g.len() == 64 && g.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(g.to_ascii_lowercase())
    } else {
        Err(format!(
            "`--launcher-digest {given}` is not a BLAKE3 digest (64 hex characters — the `launcher_blake3` a run's \
             report names, or `b3sum LAUNCHER`)"
        ))
    }
}

impl Launcher {
    /// Resolve the launcher's first word, open that file, and hash what was opened.
    pub(crate) fn resolve(program: &str) -> io::Result<Launcher> {
        let path = crate::cli::resolve_driver(program).ok_or_else(|| {
            io::Error::other(format!(
                "the external launcher `{program}` could not be started: there is no such program (a bare name is \
                 looked up in PATH's absolute directories; a name with a separator is a path from the working directory)"
            ))
        })?;
        let cannot = |e: io::Error| io::Error::other(format!("the external launcher `{}` cannot be read to be hashed: {e}", path.display()));
        let mut open = std::fs::OpenOptions::new();
        open.read(true);
        // Never blocks: a name swapped for a FIFO after `resolve_driver` saw a file would hang the open.
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::custom_flags(&mut open, libc::O_NONBLOCK);
        // Windows: sharing reads only — the standard library's default shares writing and deletion too, and
        // a launcher renamed over while it was hashed was the one started. Nobody can then write, rename or
        // delete it until this `Launcher` is dropped; starting it (a read) is still shared.
        #[cfg(windows)]
        std::os::windows::fs::OpenOptionsExt::share_mode(&mut open, windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ);
        let mut file = open.open(&path).map_err(cannot)?;
        // What was OPENED is what is judged: `resolve_driver` looked at the name, a moment before.
        if !file.metadata().map_err(cannot)?.is_file() {
            return Err(io::Error::other(format!("the external launcher `{}` is not a file", path.display())));
        }
        let mut hasher = blake3::Hasher::new();
        let mut buf = vec![0u8; 1 << 16];
        let mut script = None;
        loop {
            let n = file.read(&mut buf).map_err(cannot)?;
            if n == 0 {
                break;
            }
            // The first read of a regular file returns its first bytes, however few it returns.
            script.get_or_insert(buf[..n].starts_with(b"#!"));
            hasher.update(&buf[..n]);
        }
        Ok(Launcher { path, blake3: hasher.finalize().to_hex().to_string(), file, script: script.unwrap_or(false) })
    }

    /// The operator's pin, checked against the bytes that were read.
    pub(crate) fn check_pin(&self, pinned: Option<&str>) -> Result<(), String> {
        match pinned {
            Some(pin) if pin != self.blake3 => Err(format!(
                "`{}` is not the pinned launcher: its BLAKE3 is {}, and the run pinned {pin}. Nothing was started. If \
                 the launcher was changed on purpose, pin its new digest; if not, find out who changed it.",
                self.path.display(),
                self.blake3
            )),
            _ => Ok(()),
        }
    }

    /// Arrange for `c` — a command built on [`Launcher::path`] — to start the file that was hashed.
    ///
    /// Linux: the child replaces itself with the open descriptor (`fexecve`), as the last thing before
    /// the exec the standard library would do by name — after every earlier `pre_exec` step — with the
    /// command's arguments and the environment it was given: this process's own, with the command's
    /// settings applied. That environment is built here, from `c` as it stands, so call this after the
    /// last `env`. (It is not read from `environ` in the child: witnessed, the standard library has not
    /// installed the command's environment there when a `pre_exec` step runs — the launcher was started
    /// with none of the words the host set.) A script's descriptor is kept open across the start, because its interpreter reads the script back
    /// through `/dev/fd/N`; the launcher then holds a read-only descriptor of its own script, nothing more.
    /// Everything the child needs is built here, before the fork: nothing may allocate after it.
    ///
    /// Elsewhere this does nothing: the command starts [`Launcher::path`] by name — on Windows while the
    /// file is held against any writer (the module's note).
    pub(crate) fn start_as_hashed(&self, c: &mut std::process::Command) -> io::Result<()> {
        #[cfg(target_os = "linux")]
        {
            use std::ffi::CString;
            use std::os::fd::AsRawFd as _;
            use std::os::unix::ffi::OsStrExt as _;
            use std::os::unix::process::CommandExt as _;

            struct Argv {
                _owned: Vec<CString>,
                ptrs: Vec<*const libc::c_char>,
            }
            // SAFETY: the pointers point into `_owned`, which moves with them and is never changed.
            unsafe impl Send for Argv {}
            unsafe impl Sync for Argv {}

            let nul = |b: &[u8]| {
                CString::new(b).map_err(|_| io::Error::other("the external launcher's words or environment hold a NUL byte"))
            };
            let mut owned = vec![nul(self.path.as_os_str().as_bytes())?];
            for a in c.get_args() {
                owned.push(nul(a.as_bytes())?);
            }
            let mut ptrs: Vec<*const libc::c_char> = owned.iter().map(|s| s.as_ptr()).collect();
            ptrs.push(std::ptr::null());
            let argv = Argv { _owned: owned, ptrs };
            let mut vars: std::collections::BTreeMap<std::ffi::OsString, std::ffi::OsString> = std::env::vars_os().collect();
            for (k, v) in c.get_envs() {
                match v {
                    Some(v) => vars.insert(k.to_os_string(), v.to_os_string()),
                    None => vars.remove(k),
                };
            }
            let mut owned = Vec::with_capacity(vars.len());
            for (k, v) in &vars {
                owned.push(nul(&[k.as_bytes(), b"=", v.as_bytes()].concat())?);
            }
            let mut ptrs: Vec<*const libc::c_char> = owned.iter().map(|s| s.as_ptr()).collect();
            ptrs.push(std::ptr::null());
            let envp = Argv { _owned: owned, ptrs };
            let fd = self.file.as_raw_fd();
            let script = self.script;
            // SAFETY: between fork and exec only async-signal-safe calls (`fcntl`, `fexecve`) on memory
            // built before the fork.
            unsafe {
                c.pre_exec(move || {
                    // The whole of each, so the closure owns the wrappers and not just their pointers.
                    let (argv, envp) = (&argv, &envp);
                    if script && libc::fcntl(fd, libc::F_SETFD, 0) != 0 {
                        return Err(io::Error::last_os_error());
                    }
                    libc::fexecve(fd, argv.ptrs.as_ptr(), envp.ptrs.as_ptr());
                    // `fexecve` returns only when it failed.
                    Err(io::Error::last_os_error())
                });
            }
        }
        #[cfg(not(target_os = "linux"))]
        let _ = (c, &self.file, self.script);
        Ok(())
    }
}
