//! RW 4.67: a secret's bytes leave no copy behind in this process once a send is done.
//!
//! Under the daemon custody a secret's bytes enter the program process only to be sent: the broker hands them to the
//! egress client for one origin (`Custody::send_secret`, D-V2-134). On their way to the wire they are copied by code that
//! is not ours — the custody's answer as it is read and parsed, `reveal`'s copy, each header the egress client builds,
//! reqwest's header value, hyper's serialized request — and every one of those buffers is freed without being cleared.
//! Routine run 25's red team found one copy of the serialized request in the run's heap after the send, until the
//! process ended (F4); on-demand routine run 12 read it again by scanning the run's memory: `…on: hunter2-…`, in the
//! arena of reqwest's runtime thread.
//!
//! No call site can clear a buffer it never sees. So the rule is the allocator's: while a [`Window`] is open, every
//! block the `delulu` binary frees is zeroed first, and a block that moves when it grows is copied, zeroed and freed —
//! on every thread, because the request is made on reqwest's. Two windows cover a send: one while the custody is asked
//! for the bytes (`interp::broker_secret_send` — its answer is read and parsed into buffers freed as it goes), and one
//! around the request that carries them (`prim`, for every secret-bearing request, under any custody), which closes
//! only after the request's client is dropped and its runtime thread joined. Between the two the bytes live in one
//! `SecretVal`, which zeroes its own buffer as it is dropped (P17-F).
//!
//! **The limit, said plainly.** The window is read by the `delulu` binary's global allocator
//! (`crates/delulu/src/ceiling.rs`); a program embedding this crate under another allocator gets no scrubbing. It
//! clears what is freed, not what is still held: a buffer that keeps the bytes past the window would keep them (none
//! does today — the run's memory is scanned after the send, `broker_cli`). And the embedded custody holds every
//! secret's bytes in this process for the whole run by design; there the window clears only the send's own copies.

use std::sync::atomic::{AtomicUsize, Ordering};

/// How many windows are open in this process.
static OPEN: AtomicUsize = AtomicUsize::new(0);

/// While one of these lives, every block the `delulu` binary's allocator frees is zeroed first ([`is_open`]).
#[must_use = "a window closes when it is dropped"]
pub struct Window(());

impl Window {
    /// Opens a window: from here until it is dropped, each free in this process is scrubbed.
    pub fn open() -> Window {
        OPEN.fetch_add(1, Ordering::SeqCst);
        Window(())
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        OPEN.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Is a window open? Asked by the allocator on every free, so it is one atomic load and allocates nothing.
#[inline]
pub fn is_open() -> bool {
    OPEN.load(Ordering::Acquire) != 0
}

/// Zeroes `len` bytes at `ptr` in writes the compiler may not remove as dead stores before the block is freed.
///
/// # Safety
/// `ptr` must be valid for writes of `len` bytes.
pub unsafe fn wipe(ptr: *mut u8, len: usize) {
    let mut i = 0;
    // Bytes up to a word boundary, then words, then the tail: volatile throughout, so no store is elided.
    while i < len && !(ptr as usize + i).is_multiple_of(std::mem::align_of::<usize>()) {
        // SAFETY: `i < len`, within the block the caller vouched for.
        unsafe { std::ptr::write_volatile(ptr.add(i), 0) };
        i += 1;
    }
    while i + std::mem::size_of::<usize>() <= len {
        // SAFETY: in bounds, and aligned by the loop above.
        unsafe { std::ptr::write_volatile(ptr.add(i).cast::<usize>(), 0) };
        i += std::mem::size_of::<usize>();
    }
    while i < len {
        // SAFETY: `i < len`.
        unsafe { std::ptr::write_volatile(ptr.add(i), 0) };
        i += 1;
    }
    std::sync::atomic::compiler_fence(Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_is_open_until_its_last_holder_drops_it() {
        // Other tests in this binary may hold a window; this one counts relative to them.
        let before = OPEN.load(Ordering::SeqCst);
        let a = Window::open();
        let b = Window::open();
        assert!(is_open());
        drop(a);
        assert!(is_open(), "one window is still open");
        drop(b);
        assert_eq!(OPEN.load(Ordering::SeqCst), before);
    }

    #[test]
    fn a_wipe_zeroes_every_byte_whatever_the_alignment() {
        for start in 0..9 {
            for len in [0usize, 1, 7, 8, 9, 31, 64, 65] {
                let mut buf = vec![0xA5u8; 96];
                // SAFETY: `start + len <= 96`.
                unsafe { wipe(buf.as_mut_ptr().add(start), len) };
                assert!(buf[start..start + len].iter().all(|&b| b == 0), "start {start}, len {len}");
                assert!(buf[..start].iter().chain(&buf[start + len..]).all(|&b| b == 0xA5), "outside {start}+{len}");
            }
        }
    }
}
