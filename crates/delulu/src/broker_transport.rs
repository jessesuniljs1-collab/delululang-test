//! Phase 5f — the local IPC transport (spec §2): Windows named pipe / Unix domain socket, one
//! request per connection, blocking std I/O (head-chef ruling 1 — no async runtime).
//!
//! **Peer identity = OS-authenticated same-user (playbook trap 7).** The broker serves exactly one
//! OS user; peer creds only confirm "same user," never multi-tenant auth (out of scope, spec §10).
//! - Windows: the pipe is created with an **owner-only DACL** (only the creating user's SID may
//!   open it — the OS refuses any other user), and `accept` additionally verifies the connecting
//!   process's token user SID equals the server's via `GetNamedPipeClientProcessId` + `EqualSid`.
//! - Unix: the socket lives in a `0700` directory owned by the user (the OS refuses other users).
//!   `SO_PEERCRED`/`getpeereid` is a documented post-chunk-3 hardening (no `libc` dep is pulled in
//!   for v0.5; the directory mode already bounds access to the same user).
//!
//! The transport address is derived from the state directory, so a test with its own temp state dir
//! gets its own pipe/socket and never collides with a real daemon or a parallel test run.

use std::io::{self, Read, Write};
use std::path::Path;

/// A short, stable 64-bit hash of the canonical state-dir path — the per-instance transport suffix.
///
/// Windows-only: the Unix `imp` derives its socket path directly from the state dir, so this hash
/// has no caller off Windows. Gating it keeps the Linux/macOS build warning-clean (a dead private
/// fn is a `clippy` finding) rather than carrying a symbol no platform but one ever names.
#[cfg(windows)]
fn state_hash(state_dir: &Path) -> u64 {
    let canon = std::fs::canonicalize(state_dir).unwrap_or_else(|_| state_dir.to_path_buf());
    let s = canon.to_string_lossy().to_lowercase();
    // FNV-1a 64-bit — no crate needed, deterministic, plenty for a name suffix.
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

// ===================================================================================================
// Windows: named pipe with an owner-only DACL + peer-SID verification.
// ===================================================================================================
#[cfg(windows)]
mod imp {
    use super::*;
    use std::ptr;

    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, LocalFree, ERROR_FILE_NOT_FOUND, ERROR_PIPE_BUSY,
        ERROR_PIPE_CONNECTED, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::Security::Authorization::{
        ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
    };
    use windows_sys::Win32::Security::{
        CopySid, EqualSid, GetLengthSid, GetTokenInformation, TokenUser, PSID, SECURITY_ATTRIBUTES,
        TOKEN_QUERY, TOKEN_USER,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FlushFileBuffers, ReadFile, WriteFile, OPEN_EXISTING, PIPE_ACCESS_DUPLEX,
    };
    use windows_sys::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, GetNamedPipeClientProcessId,
        PeekNamedPipe, WaitNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES,
        PIPE_WAIT,
    };
    use windows_sys::Win32::System::Threading::{
        GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    const PIPE_BUF: u32 = 64 * 1024;

    pub fn address_for(state_dir: &Path) -> String {
        format!(r"\\.\pipe\delulu-broker-{:016x}", state_hash(state_dir))
    }

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn last_err(ctx: &str) -> io::Error {
        let code = unsafe { GetLastError() };
        io::Error::other(format!("{ctx}: win32 error {code}"))
    }

    /// The user SID (owned bytes) behind a process handle.
    unsafe fn process_user_sid(process: HANDLE) -> io::Result<Vec<u8>> {
        let mut token: HANDLE = ptr::null_mut();
        if OpenProcessToken(process, TOKEN_QUERY, &mut token) == 0 {
            return Err(last_err("OpenProcessToken"));
        }
        let mut len: u32 = 0;
        // First call sizes the buffer (expected to "fail" with insufficient-buffer, setting len).
        GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut len);
        if len == 0 {
            CloseHandle(token);
            return Err(last_err("GetTokenInformation(size)"));
        }
        let mut buf = vec![0u8; len as usize];
        let ok = GetTokenInformation(token, TokenUser, buf.as_mut_ptr() as *mut _, len, &mut len);
        CloseHandle(token);
        if ok == 0 {
            return Err(last_err("GetTokenInformation"));
        }
        let tu = &*(buf.as_ptr() as *const TOKEN_USER);
        let psid = tu.User.Sid;
        let sid_len = GetLengthSid(psid);
        let mut sid = vec![0u8; sid_len as usize];
        if CopySid(sid_len, sid.as_mut_ptr() as PSID, psid) == 0 {
            return Err(last_err("CopySid"));
        }
        Ok(sid)
    }

    unsafe fn sid_to_string(sid: PSID) -> io::Result<String> {
        let mut pstr: *mut u16 = ptr::null_mut();
        if ConvertSidToStringSidW(sid, &mut pstr) == 0 {
            return Err(last_err("ConvertSidToStringSidW"));
        }
        // Read the wide string up to its nul terminator.
        let mut len = 0usize;
        while *pstr.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(pstr, len);
        let s = String::from_utf16_lossy(slice);
        LocalFree(pstr as _);
        Ok(s)
    }

    /// A held security descriptor (owner-only DACL) plus its `SECURITY_ATTRIBUTES`. Freed on drop.
    struct OwnerOnlySd {
        sd: *mut core::ffi::c_void,
    }
    impl Drop for OwnerOnlySd {
        fn drop(&mut self) {
            if !self.sd.is_null() {
                unsafe { LocalFree(self.sd as _) };
            }
        }
    }

    /// Build an owner-only security descriptor for `sid_string`: a protected DACL with a single
    /// Allow-Generic-All ACE for that SID. Any other user is denied by the OS.
    fn owner_only_sd(sid_string: &str) -> io::Result<OwnerOnlySd> {
        let sddl = format!("D:P(A;;GA;;;{sid_string})");
        let wsddl = to_wide(&sddl);
        let mut sd: *mut core::ffi::c_void = ptr::null_mut();
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wsddl.as_ptr(),
                SDDL_REVISION_1,
                &mut sd,
                ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(last_err("ConvertStringSecurityDescriptorToSecurityDescriptorW"));
        }
        Ok(OwnerOnlySd { sd })
    }

    pub struct Listener {
        name_w: Vec<u16>,
        address: String,
        server_sid: Vec<u8>,
        sd: OwnerOnlySd,
    }

    impl Listener {
        pub fn bind(state_dir: &Path) -> io::Result<Listener> {
            let address = address_for(state_dir);
            let server_sid = unsafe { process_user_sid(GetCurrentProcess())? };
            let sid_str = unsafe { sid_to_string(server_sid.as_ptr() as PSID)? };
            let sd = owner_only_sd(&sid_str)?;
            Ok(Listener { name_w: to_wide(&address), address, server_sid, sd })
        }

        pub fn address(&self) -> &str {
            &self.address
        }

        /// Accept one client, verifying it runs as the same OS user (peer-SID check). Loops past a
        /// mismatched peer (belt-and-suspenders behind the owner-only DACL) until a same-user client
        /// connects or a hard error occurs.
        pub fn accept(&self) -> io::Result<Connection> {
            loop {
                let sa = SECURITY_ATTRIBUTES {
                    nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                    lpSecurityDescriptor: self.sd.sd,
                    bInheritHandle: 0,
                };
                let h = unsafe {
                    CreateNamedPipeW(
                        self.name_w.as_ptr(),
                        PIPE_ACCESS_DUPLEX,
                        PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
                        PIPE_UNLIMITED_INSTANCES,
                        PIPE_BUF,
                        PIPE_BUF,
                        0,
                        &sa,
                    )
                };
                if h == INVALID_HANDLE_VALUE {
                    return Err(last_err("CreateNamedPipeW"));
                }
                let connected = unsafe { ConnectNamedPipe(h, ptr::null_mut()) };
                if connected == 0 {
                    let e = unsafe { GetLastError() };
                    if e != ERROR_PIPE_CONNECTED {
                        unsafe {
                            DisconnectNamedPipe(h);
                            CloseHandle(h);
                        }
                        return Err(io::Error::other(format!("ConnectNamedPipe: win32 error {e}")));
                    }
                }
                // Peer identity (same-user) verification.
                match self.verify_peer(h) {
                    Ok(true) => return Ok(Connection { handle: h, server: true, read_timeout: None }),
                    Ok(false) => {
                        // A different user slipped past the DACL (should be impossible) — refuse and
                        // wait for the next client. Fail closed.
                        unsafe {
                            DisconnectNamedPipe(h);
                            CloseHandle(h);
                        }
                        continue;
                    }
                    Err(e) => {
                        unsafe {
                            DisconnectNamedPipe(h);
                            CloseHandle(h);
                        }
                        return Err(e);
                    }
                }
            }
        }

        /// The peer-SID check: does the connecting process run as the same user as the server?
        fn verify_peer(&self, pipe: HANDLE) -> io::Result<bool> {
            let mut pid: u32 = 0;
            if unsafe { GetNamedPipeClientProcessId(pipe, &mut pid) } == 0 {
                return Err(last_err("GetNamedPipeClientProcessId"));
            }
            let proc = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
            if proc.is_null() {
                return Err(last_err("OpenProcess"));
            }
            let client_sid = unsafe { process_user_sid(proc) };
            unsafe { CloseHandle(proc) };
            let client_sid = client_sid?;
            let equal = unsafe {
                EqualSid(self.server_sid.as_ptr() as PSID, client_sid.as_ptr() as PSID)
            };
            Ok(equal != 0)
        }
    }

    pub struct Connection {
        handle: HANDLE,
        server: bool,
        /// Bound on a single blocking read (IPC-1 / DEADMAN-1). `None` = block as before. When set,
        /// `read` polls `PeekNamedPipe` (non-blocking) until data is available or the deadline passes,
        /// so a peer that connects and never sends cannot hang us forever. Still blocking std I/O — no
        /// async runtime, no thread pool — so the head-chef "blocking, single-thread" design holds.
        read_timeout: Option<std::time::Duration>,
    }

    // The handle is used from a single thread (the blocking serve loop / a single client call).
    unsafe impl Send for Connection {}

    impl Connection {
        /// Bound each subsequent blocking read to `dur` (`None` clears it). Infallible on Windows —
        /// the deadline is enforced in `read` by polling, so there is no OS call that can fail here.
        pub fn set_read_timeout(&mut self, dur: Option<std::time::Duration>) -> io::Result<()> {
            self.read_timeout = dur;
            Ok(())
        }
    }

    impl Read for Connection {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            // When a timeout is set, wait for data with a non-blocking peek rather than blocking in
            // ReadFile. A legit peer that has already sent its frame sees data on the first peek, so
            // this adds no latency to the common path; only a stalled/absent peer hits the poll.
            if let Some(timeout) = self.read_timeout {
                let deadline = std::time::Instant::now() + timeout;
                loop {
                    let mut avail: u32 = 0;
                    let ok = unsafe {
                        PeekNamedPipe(self.handle, ptr::null_mut(), 0, ptr::null_mut(), &mut avail, ptr::null_mut())
                    };
                    if ok == 0 {
                        let e = unsafe { GetLastError() };
                        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, format!("PeekNamedPipe: win32 error {e}")));
                    }
                    if avail > 0 {
                        break; // data is available; the ReadFile below will not block
                    }
                    if std::time::Instant::now() >= deadline {
                        return Err(io::Error::new(io::ErrorKind::TimedOut, "broker pipe read timed out"));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }
            let mut read: u32 = 0;
            let want = buf.len().min(u32::MAX as usize) as u32;
            let ok = unsafe { ReadFile(self.handle, buf.as_mut_ptr(), want, &mut read, ptr::null_mut()) };
            if ok == 0 {
                let e = unsafe { GetLastError() };
                // A broken pipe (daemon gone / client closed) surfaces as EOF-like 0 or an error;
                // report it so `read_exact` fails fast (fail-closed) rather than hanging.
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, format!("ReadFile: win32 error {e}")));
            }
            Ok(read as usize)
        }
    }

    impl Write for Connection {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let mut wrote: u32 = 0;
            let want = buf.len().min(u32::MAX as usize) as u32;
            let ok = unsafe { WriteFile(self.handle, buf.as_ptr(), want, &mut wrote, ptr::null_mut()) };
            if ok == 0 {
                let e = unsafe { GetLastError() };
                return Err(io::Error::new(io::ErrorKind::BrokenPipe, format!("WriteFile: win32 error {e}")));
            }
            Ok(wrote as usize)
        }
        fn flush(&mut self) -> io::Result<()> {
            unsafe { FlushFileBuffers(self.handle) };
            Ok(())
        }
    }

    impl Drop for Connection {
        fn drop(&mut self) {
            unsafe {
                FlushFileBuffers(self.handle);
                if self.server {
                    DisconnectNamedPipe(self.handle);
                }
                CloseHandle(self.handle);
            }
        }
    }

    /// Connect to the daemon's pipe. A missing pipe (daemon down) fails FAST (never hangs) so the
    /// caller fails closed (DL1401). A busy pipe is waited on briefly, then retried.
    ///
    /// **The reconnect-race window.** The daemon serves one request per pipe instance and re-creates
    /// the instance between requests (the blocking single-thread design, spec §2 / head-chef ruling
    /// 1). In the microsecond gap between a client disconnecting and the server's next
    /// `CreateNamedPipeW`, a client reconnecting for its *next* request gets `ERROR_FILE_NOT_FOUND`
    /// even though the daemon is alive. We retry that error within a SHORT bounded window (with a
    /// tiny backoff) so the race resolves — a genuinely-down daemon still fails closed well inside
    /// the window, so invariant 27's "before the next use, fast, never a hang" holds. This is the
    /// canonical Windows named-pipe client pattern (retry FILE_NOT_FOUND/PIPE_BUSY, bounded).
    pub fn connect(state_dir: &Path) -> io::Result<Connection> {
        let name_w = to_wide(&address_for(state_dir));
        // Bounded so a truly-down daemon fails closed fast (≪ the 10 s the fail-closed tests allow);
        // the reconnect gap is sub-millisecond, so this window covers it with vast margin.
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1000);
        loop {
            let h = unsafe {
                CreateFileW(
                    name_w.as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    0,
                    ptr::null_mut(),
                    OPEN_EXISTING,
                    0,
                    ptr::null_mut(),
                )
            };
            if h != INVALID_HANDLE_VALUE {
                return Ok(Connection { handle: h, server: false, read_timeout: None });
            }
            let e = unsafe { GetLastError() };
            let before_deadline = std::time::Instant::now() < deadline;
            if e == ERROR_PIPE_BUSY && before_deadline {
                // All instances busy: wait for one to free up, then retry.
                unsafe { WaitNamedPipeW(name_w.as_ptr(), 200) };
                continue;
            }
            if e == ERROR_FILE_NOT_FOUND && before_deadline {
                // Transient: the daemon is mid-reconnect between requests. Brief backoff, retry.
                std::thread::sleep(std::time::Duration::from_millis(2));
                continue;
            }
            // Daemon down (window elapsed) or a hard error → fail closed, fast.
            return Err(io::Error::new(io::ErrorKind::NotConnected, format!("connect: win32 error {e}")));
        }
    }
}

// ===================================================================================================
// Unix: domain socket in a 0700 directory (same-user boundary via directory mode).
// ===================================================================================================
#[cfg(unix)]
mod imp {
    use super::*;
    use std::os::unix::net::{UnixListener, UnixStream};

    pub fn address_for(state_dir: &Path) -> String {
        // The socket lives beside the other broker state so a temp state dir isolates the transport.
        state_dir.join("broker.sock").to_string_lossy().to_string()
    }

    /// The kernel's limit on a Unix socket path — `sizeof(sun_path)`, including its NUL terminator.
    ///
    /// **macOS is 104 and Linux is 108**, and the difference is not academic here: macOS hands out
    /// temp directories like `/var/folders/j7/8k3l…0000gn/T/`, roughly fifty characters before a
    /// caller has named anything. A state directory under one of those, plus `broker.sock`, lands
    /// within about a dozen bytes of the macOS ceiling on paths that are comfortable on Linux.
    ///
    /// The project has never run on a Mac, so this is a hazard reasoned about rather than observed —
    /// which is exactly why it should fail *legibly* if it ever fires. `ENAMETOOLONG` from
    /// `UnixListener::bind` surfaces as "File name too long" with no number, no limit, and no
    /// indication that the platform is the variable.
    const SUN_PATH_MAX: usize = if cfg!(target_os = "macos") { 104 } else { 108 };

    /// Refuse a socket path the kernel cannot hold, naming the limit and the overrun.
    ///
    /// Checked *before* `bind` rather than translating its error afterwards, because the failure is
    /// worth describing in terms the caller can act on: shorten the state directory. Returns the
    /// address when it fits.
    pub(super) fn checked_address(state_dir: &Path) -> io::Result<String> {
        let address = address_for(state_dir);
        // The NUL terminator counts against the limit, so a path of exactly SUN_PATH_MAX is already
        // one byte too long.
        if address.len() >= SUN_PATH_MAX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "the broker socket path is {} bytes and this platform allows {} (including the \
                     terminator): {address}\n  \
                     use a shorter --state-dir; macOS allows 104 where Linux allows 108, so a path \
                     that fits on Linux can still be refused on a Mac",
                    address.len(),
                    SUN_PATH_MAX - 1,
                ),
            ));
        }
        Ok(address)
    }

    pub struct Listener {
        inner: UnixListener,
        address: String,
    }

    impl Listener {
        pub fn bind(state_dir: &Path) -> io::Result<Listener> {
            std::fs::create_dir_all(state_dir)?;
            // 0700: only the owner (same user) can traverse into the dir and reach the socket. Verify
            // it actually stuck and warn if not — on a non-POSIX filesystem (9p/DrvFs/NFS) chmod is a
            // silent no-op and this boundary would not exist (P21). This dir holds broker.key,
            // secrets.json and the audit log, so one warning here covers them all.
            crate::signing::set_owner_only_or_warn(state_dir, 0o700, "broker state directory");
            let address = checked_address(state_dir)?;
            let _ = std::fs::remove_file(&address); // clear a stale socket from a previous run
            let inner = UnixListener::bind(&address)?;
            Ok(Listener { inner, address })
        }

        pub fn address(&self) -> &str {
            &self.address
        }

        pub fn accept(&self) -> io::Result<Connection> {
            // Peer identity on Unix is the 0700 directory (OS same-user boundary). SO_PEERCRED /
            // getpeereid is a documented post-chunk-3 hardening (no libc dependency pulled in for
            // v0.5) — flagged in the report; the directory mode already bounds access to this user.
            let (stream, _addr) = self.inner.accept()?;
            Ok(Connection { inner: stream })
        }
    }

    pub struct Connection {
        inner: UnixStream,
    }

    impl Connection {
        /// Bound each subsequent blocking read to `dur` (`None` clears it) via the OS socket read
        /// timeout (`SO_RCVTIMEO`). This is what stops a hung broker from blocking the dead-man
        /// watchdog's authority probe forever on Unix (DEADMAN-1) and a stalled client from hanging
        /// the serve loop (IPC-1). A zero duration is rejected by the OS, surfaced as the caller's error.
        pub fn set_read_timeout(&mut self, dur: Option<std::time::Duration>) -> io::Result<()> {
            self.inner.set_read_timeout(dur)
        }
    }

    impl Read for Connection {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.inner.read(buf)
        }
    }
    impl Write for Connection {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.inner.write(buf)
        }
        fn flush(&mut self) -> io::Result<()> {
            self.inner.flush()
        }
    }

    pub fn connect(state_dir: &Path) -> io::Result<Connection> {
        // The same check on the client side: a caller that cannot even name the socket should be
        // told why, not handed the kernel's word for it.
        let stream = UnixStream::connect(checked_address(state_dir)?)?;
        Ok(Connection { inner: stream })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// THE WITNESS, and it runs on Linux today. A path the kernel cannot hold is refused with
        /// the two numbers a reader needs, rather than surfacing as `ENAMETOOLONG`.
        ///
        /// The hazard this guards is macOS-specific — 104 bytes there against 108 here — and the
        /// project has no Mac. So the check is written to be exercised on the platform we *do* run:
        /// the limit differs by four bytes, the arithmetic is identical, and a Linux path long
        /// enough to trip 108 proves the branch works. That is a tested mechanism plus a reasoned
        /// constant, and the distinction is stated rather than blurred.
        #[test]
        fn a_socket_path_the_kernel_cannot_hold_is_refused_by_name() {
            let deep = std::path::PathBuf::from("/tmp").join("x".repeat(200));
            let err = checked_address(&deep).expect_err("a 200-byte component cannot fit sun_path");
            let msg = err.to_string();
            assert!(msg.contains("broker socket path is"), "the error names the problem: {msg}");
            assert!(
                msg.contains(&SUN_PATH_MAX.to_string()) || msg.contains(&(SUN_PATH_MAX - 1).to_string()),
                "and states the platform limit: {msg}"
            );
            assert!(msg.contains("--state-dir"), "and what to do about it: {msg}");
        }

        /// THE SKIP-BRANCH CASE: an ordinary state directory must still bind. A check that refused
        /// every path would pass the test above and break the broker.
        #[test]
        fn an_ordinary_state_directory_is_accepted() {
            let ok = std::path::PathBuf::from("/tmp/delulu-state");
            assert!(checked_address(&ok).is_ok(), "a normal path must not be refused");
        }
    }
}

pub use imp::{connect, Connection, Listener};

/// Refuse, before anything is spawned, a state directory whose transport address this platform
/// cannot hold.
///
/// Unix names the broker socket from the state directory, so a deep one can exceed `sun_path` (104
/// bytes on macOS, 108 on Linux; see `SUN_PATH_MAX` in the Unix transport). The listener refuses such
/// a path by name, but it runs inside the detached daemon, whose output goes to `broker.log` — so
/// `broker start` calls this first, and the refusal reaches the person who asked for the broker.
#[cfg(unix)]
pub fn check_state_dir(state_dir: &Path) -> io::Result<()> {
    imp::checked_address(state_dir).map(|_| ())
}

/// Windows names its pipe from a fixed-length hash of the state directory, so any directory fits.
#[cfg(windows)]
pub fn check_state_dir(_state_dir: &Path) -> io::Result<()> {
    Ok(())
}
