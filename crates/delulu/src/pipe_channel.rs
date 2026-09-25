//! PS-B-03: the sandbox channel of a Windows guest started under a separate identity.
//!
//! A guest in an AppContainer cannot be reached the way PS-A reached it. There the guest listened on a
//! named pipe in a channel directory and the host connected, but an AppContainer lives in its own
//! object namespace, so a pipe it creates is not one the host can open by name. Inheritance needs no
//! name: the host makes the pipe, the guest is born holding its end, and nothing is looked up on
//! either side.
//!
//! **One duplex pipe, read directly on both sides.** The first version (PS-B-03) used two anonymous
//! pipes, and since an anonymous pipe has no read deadline, a thread drained each reading end into a
//! queue that a read then waited on. That was correct and it was slow: every frame crossed a thread on
//! the way in, so one round trip woke four threads where Linux wakes two. PS-B-04's measurement put
//! the channel at 48.6 µs per effect on the workstation and 67 µs on a Windows CI runner — over the
//! 50 µs line the record fixed before any number was taken — and a local experiment removing the two
//! hops measured 30.1 µs (`measurements/sandbox-channel/RECORD.md`). So:
//!
//! - **The host** holds the server end of one duplex named pipe, opened OVERLAPPED, and reads it on
//!   the calling thread with a real deadline: the read is issued, waited on for at most the deadline,
//!   and cancelled if it did not finish. Writes are bounded the same way, which the anonymous pipe
//!   never was — a guest that stops reading can no longer hold the host's write for ever. The pipe
//!   carries the broker's owner-only descriptor, admits one instance and no remote client, and its
//!   name is random; the host opens the client end itself, at once, and hands it down by inheritance.
//! - **The guest** reads its end directly, and a WATCHDOG enforces its deadline: when a read has been
//!   waiting longer than the deadline, the guest ends. That is the only honest action open to it — a
//!   silent host is not something a guest can repair — and ending is what the queued read's timeout
//!   led to anyway. The watchdog costs nothing per read (two atomic stores); it wakes on its own tick.

use std::io::{self, Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock, Weak};
use std::time::{Duration, Instant};

#[cfg(windows)]
pub use win::{duplex_pair, HostPipe};

/// Milliseconds since this process first asked, plus one, so that 0 can mean "no read waiting".
fn now_ms() -> u64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64 + 1
}

/// What the watchdog shares with the channel: when the current read began (0 = none) and the deadline.
struct Watch {
    since: AtomicU64,
    deadline_ms: AtomicU64,
}

/// The guest's side of the channel: direct reads and writes, and a deadline kept by a watchdog.
pub struct GuestChannel<R: Read, W: Write> {
    reader: R,
    writer: W,
    watch: Arc<Watch>,
}

impl<R: Read, W: Write> GuestChannel<R, W> {
    /// `on_silence` runs, once, on the watchdog's thread when a read outlives the deadline. A guest
    /// passes a function that ends the process; the tests pass one that records it.
    pub fn new(reader: R, writer: W, on_silence: impl FnOnce() + Send + 'static) -> io::Result<Self> {
        let watch = Arc::new(Watch { since: AtomicU64::new(0), deadline_ms: AtomicU64::new(0) });
        let weak: Weak<Watch> = Arc::downgrade(&watch);
        std::thread::Builder::new().name("delulu-channel-watchdog".into()).spawn(move || {
            loop {
                // The channel is gone: nothing left to watch.
                let Some(w) = weak.upgrade() else { return };
                let deadline = w.deadline_ms.load(Ordering::Relaxed);
                let since = w.since.load(Ordering::Relaxed);
                if deadline != 0 && since != 0 && now_ms().saturating_sub(since) > deadline {
                    on_silence();
                    return;
                }
                drop(w);
                // A quarter of the deadline, at most a tenth of a second: a deadline is kept to within
                // that, and a channel with no deadline set is checked rarely.
                let tick = if deadline == 0 { 100 } else { (deadline / 4).clamp(1, 100) };
                std::thread::sleep(Duration::from_millis(tick));
            }
        })?;
        Ok(GuestChannel { reader, writer, watch })
    }

    /// The same contract as a socket's: a zero deadline is refused, `None` waits for ever.
    pub fn set_read_timeout(&mut self, deadline: Option<Duration>) -> io::Result<()> {
        if deadline == Some(Duration::ZERO) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "a zero read deadline"));
        }
        let ms = deadline.map_or(0, |d| (d.as_millis() as u64).max(1));
        self.watch.deadline_ms.store(ms, Ordering::Relaxed);
        Ok(())
    }
}

impl<R: Read, W: Write> Read for GuestChannel<R, W> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.watch.since.store(now_ms(), Ordering::Relaxed);
        let got = self.reader.read(out);
        self.watch.since.store(0, Ordering::Relaxed);
        got
    }
}

impl<R: Read, W: Write> Write for GuestChannel<R, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(windows)]
mod win {
    use std::io::{self, Read, Write};
    use std::os::windows::io::{AsRawHandle as _, FromRawHandle as _, OwnedHandle, RawHandle};
    use std::time::Duration;

    use windows_sys::Win32::Foundation::{
        DuplicateHandle, GetLastError, DUPLICATE_SAME_ACCESS, ERROR_BROKEN_PIPE, ERROR_IO_PENDING,
        ERROR_PIPE_CONNECTED, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0, WAIT_TIMEOUT,
    };
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, ReadFile, WriteFile, FILE_FLAG_FIRST_PIPE_INSTANCE, FILE_FLAG_OVERLAPPED, OPEN_EXISTING,
        PIPE_ACCESS_DUPLEX,
    };
    use windows_sys::Win32::System::Pipes::{
        ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE, PIPE_WAIT,
    };
    use windows_sys::Win32::System::Threading::{CreateEventW, GetCurrentProcess, WaitForSingleObject, INFINITE};
    use windows_sys::Win32::System::IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED};

    const BUF: u32 = 64 * 1024;

    /// The host's end: the server side of the duplex pipe, overlapped, with one event for its I/O.
    pub struct HostPipe {
        handle: OwnedHandle,
        event: OwnedHandle,
        deadline: Option<Duration>,
    }

    impl HostPipe {
        /// A zero deadline is refused, as a socket refuses it; `None` waits for ever.
        pub fn set_read_timeout(&mut self, deadline: Option<Duration>) -> io::Result<()> {
            if deadline == Some(Duration::ZERO) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "a zero read deadline"));
            }
            self.deadline = deadline;
            Ok(())
        }

        /// Issue one overlapped operation and wait for it within the deadline. `start` makes the
        /// call; the count comes from `GetOverlappedResult` whichever way it completed.
        fn io(&mut self, start: impl FnOnce(HANDLE, *mut OVERLAPPED) -> i32) -> io::Result<usize> {
            let h = self.handle.as_raw_handle() as HANDLE;
            // SAFETY: `ov` lives on this frame until the operation has completed or been cancelled
            // AND waited for (both paths below end in a waiting `GetOverlappedResult`), so the kernel
            // never writes into a dead frame; the event is this pipe's own.
            unsafe {
                let mut ov: OVERLAPPED = std::mem::zeroed();
                ov.hEvent = self.event.as_raw_handle() as HANDLE;
                if start(h, &mut ov) == 0 {
                    let e = GetLastError();
                    if e == ERROR_BROKEN_PIPE {
                        return Ok(0);
                    }
                    if e != ERROR_IO_PENDING {
                        return Err(io::Error::from_raw_os_error(e as i32));
                    }
                    let wait = self.deadline.map_or(INFINITE, |d| d.as_millis().min(u32::MAX as u128 - 1) as u32);
                    let w = WaitForSingleObject(ov.hEvent, wait);
                    if w == WAIT_TIMEOUT {
                        CancelIoEx(h, &ov);
                        let mut n = 0u32;
                        // Wait for the cancellation itself; the operation may have completed first.
                        if GetOverlappedResult(h, &ov, &mut n, 1) != 0 && n > 0 {
                            return Ok(n as usize);
                        }
                        return Err(io::Error::new(
                            io::ErrorKind::TimedOut,
                            "no frame from the other side before the channel deadline",
                        ));
                    }
                    if w != WAIT_OBJECT_0 {
                        CancelIoEx(h, &ov);
                        let mut n = 0u32;
                        GetOverlappedResult(h, &ov, &mut n, 1);
                        return Err(io::Error::last_os_error());
                    }
                }
                let mut n = 0u32;
                if GetOverlappedResult(h, &ov, &mut n, 1) == 0 {
                    let e = GetLastError();
                    return if e == ERROR_BROKEN_PIPE { Ok(0) } else { Err(io::Error::from_raw_os_error(e as i32)) };
                }
                Ok(n as usize)
            }
        }
    }

    impl Read for HostPipe {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if out.is_empty() {
                return Ok(0);
            }
            let len = out.len().min(u32::MAX as usize) as u32;
            let ptr = out.as_mut_ptr();
            // SAFETY: `out` outlives the operation (`io` waits for completion or cancellation).
            self.io(|h, ov| unsafe { ReadFile(h, ptr, len, std::ptr::null_mut(), ov) })
        }
    }

    impl Write for HostPipe {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if buf.is_empty() {
                return Ok(0);
            }
            let len = buf.len().min(u32::MAX as usize) as u32;
            let ptr = buf.as_ptr();
            // SAFETY: as for `read`.
            match self.io(|h, ov| unsafe { WriteFile(h, ptr, len, std::ptr::null_mut(), ov) })? {
                0 => Err(io::Error::new(io::ErrorKind::BrokenPipe, "the guest's end of the channel is closed")),
                n => Ok(n),
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    /// Make the channel: the host's overlapped end, and the guest's end twice — one inheritable
    /// handle for its standard input and a duplicate for its standard output, so the guest can own
    /// each as a separate file. Both of the guest's handles are synchronous.
    pub fn duplex_pair() -> Result<(HostPipe, OwnedHandle, OwnedHandle), String> {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(0);
        // Random so a name cannot be predicted and squatted; `FIRST_PIPE_INSTANCE` refuses a name
        // someone already holds, and one instance means one client — which the host is, at once.
        let nonce: u64 = {
            use std::hash::{BuildHasher as _, Hasher as _};
            let mut h = std::collections::hash_map::RandomState::new().build_hasher();
            h.write_u32(NEXT.fetch_add(1, Ordering::Relaxed));
            h.finish()
        };
        let name = wide(&format!(r"\\.\pipe\delulu-guest-{}-{nonce:016x}", std::process::id()));
        let sd = crate::broker_transport::owner_only_for_this_user().map_err(|e| format!("no owner-only descriptor: {e}"))?;
        let private = SECURITY_ATTRIBUTES {
            nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: sd.as_ptr(),
            bInheritHandle: 0,
        };
        // SAFETY: every handle below is owned as soon as it is created; `name` and `private` outlive
        // the calls that read them.
        unsafe {
            let server = CreateNamedPipeW(
                name.as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                1,
                BUF,
                BUF,
                0,
                &private,
            );
            if server == INVALID_HANDLE_VALUE {
                return Err(format!("the channel pipe could not be created ({})", GetLastError()));
            }
            let server = OwnedHandle::from_raw_handle(server as RawHandle);
            let inherit = SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: std::ptr::null_mut(),
                bInheritHandle: 1,
            };
            let client = CreateFileW(name.as_ptr(), GENERIC_READ | GENERIC_WRITE, 0, &inherit, OPEN_EXISTING, 0, std::ptr::null_mut());
            if client == INVALID_HANDLE_VALUE {
                return Err(format!("the guest's end of the channel could not be opened ({})", GetLastError()));
            }
            let client = OwnedHandle::from_raw_handle(client as RawHandle);
            // The client above is ours: with one instance, nothing else can be connected. Confirm it
            // rather than assume it.
            let event = CreateEventW(std::ptr::null(), 1, 0, std::ptr::null());
            if event.is_null() {
                return Err(format!("no event for the channel ({})", GetLastError()));
            }
            let event = OwnedHandle::from_raw_handle(event as RawHandle);
            let mut ov: OVERLAPPED = std::mem::zeroed();
            ov.hEvent = event.as_raw_handle() as HANDLE;
            if ConnectNamedPipe(server.as_raw_handle() as HANDLE, &mut ov) != 0 || GetLastError() != ERROR_PIPE_CONNECTED {
                let h = server.as_raw_handle() as HANDLE;
                CancelIoEx(h, &ov);
                let mut n = 0u32;
                GetOverlappedResult(h, &ov, &mut n, 1);
                return Err("the channel pipe was not connected to the guest's end".into());
            }
            let mut dup: HANDLE = std::ptr::null_mut();
            if DuplicateHandle(GetCurrentProcess(), client.as_raw_handle() as HANDLE, GetCurrentProcess(), &mut dup, 0, 1, DUPLICATE_SAME_ACCESS) == 0 {
                return Err(format!("the guest's end of the channel could not be duplicated ({})", GetLastError()));
            }
            let dup = OwnedHandle::from_raw_handle(dup as RawHandle);
            Ok((HostPipe { handle: server, event, deadline: None }, client, dup))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// Both directions work, the end of the guest's handles is the end of the channel, and a
        /// silent guest meets the host's deadline instead of hanging it.
        #[test]
        fn the_host_end_reads_writes_ends_and_keeps_its_deadline() {
            let (mut host, a, b) = duplex_pair().expect("a channel pipe");
            let (mut guest_in, mut guest_out) = (std::fs::File::from(a), std::fs::File::from(b));
            host.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
            host.write_all(b"hello").unwrap();
            let mut got = [0u8; 5];
            guest_in.read_exact(&mut got).unwrap();
            assert_eq!(&got, b"hello");
            guest_out.write_all(b"back").unwrap();
            let mut got = [0u8; 4];
            host.read_exact(&mut got).unwrap();
            assert_eq!(&got, b"back");

            let started = std::time::Instant::now();
            let e = host.read(&mut got).unwrap_err();
            assert_eq!(e.kind(), io::ErrorKind::TimedOut, "{e}");
            assert!(started.elapsed() >= Duration::from_millis(250), "the deadline was kept, not skipped");
            assert!(started.elapsed() < Duration::from_secs(5), "and it bounded the wait");
            // After a timed-out read the channel still works.
            guest_out.write_all(b"z").unwrap();
            let mut one = [0u8; 1];
            host.read_exact(&mut one).unwrap();
            assert_eq!(&one, b"z");

            drop((guest_in, guest_out));
            assert_eq!(host.read(&mut one).unwrap(), 0, "the guest's end closed: the channel ended");
            assert!(host.write(b"x").is_err(), "and a write to it fails rather than hangs");
            assert!(host.set_read_timeout(Some(Duration::ZERO)).is_err());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A reader that yields its bytes and then blocks until the test lets it go.
    struct Slow(Vec<u8>, std::sync::mpsc::Receiver<()>);
    impl Read for Slow {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if self.0.is_empty() {
                let _ = self.1.recv();
                return Ok(0);
            }
            let n = out.len().min(self.0.len());
            out[..n].copy_from_slice(&self.0[..n]);
            self.0.drain(..n);
            Ok(n)
        }
    }

    /// A read that outlives the deadline fires the silence action once; reads that return in time,
    /// and time spent NOT reading, never do.
    #[test]
    fn a_silent_host_trips_the_watchdog_and_a_prompt_one_never_does() {
        let (release, gate) = std::sync::mpsc::channel();
        let (fired_tx, fired) = std::sync::mpsc::channel();
        let mut c = GuestChannel::new(Slow(b"frame".to_vec(), gate), Vec::new(), move || {
            let _ = fired_tx.send(());
        })
        .unwrap();
        c.set_read_timeout(Some(Duration::from_millis(150))).unwrap();
        let mut got = [0u8; 5];
        c.read_exact(&mut got).unwrap();
        assert_eq!(&got, b"frame");
        // Idle, not reading: longer than the deadline, and nothing fires.
        std::thread::sleep(Duration::from_millis(400));
        assert!(fired.try_recv().is_err(), "time spent not reading is not silence");
        // Now a read that blocks: the watchdog must fire while it waits.
        let waiter = std::thread::spawn(move || {
            let r = c.read(&mut [0u8; 1]);
            (c, r)
        });
        fired.recv_timeout(Duration::from_secs(5)).expect("the watchdog fired on a read past the deadline");
        let _ = release.send(());
        let (mut c, _) = waiter.join().unwrap();
        c.write_all(b"xy").unwrap();
        assert_eq!(c.writer, b"xy", "writes go straight through");
        assert!(c.set_read_timeout(Some(Duration::ZERO)).is_err(), "a zero deadline is refused, as a socket refuses it");
    }
}
