//! The guest's end of a memory ceiling (campaign finding SANDBOX-STOP-1).
//!
//! A sandboxed guest runs under an OS memory ceiling — a Job Object's per-process limit on Windows,
//! an rlimit on Linux. When the program reaches it, the OS refuses the next allocation, and the Rust
//! runtime's answer to a refused allocation is to print `memory allocation of N bytes failed` and
//! abort. That is what the 2026-09-27 agent pass saw: the ceiling held, but the operator read an
//! implementation detail on their terminal, the host could not tell a ceiling from a crash, and the
//! run report said only `exit: 1` (the same program at L0 is stopped by name, with `stopped_by`).
//!
//! In guest mode this allocator turns a refused allocation into a quiet, immediate end with one exit
//! status, [`GUEST_MEMORY_EXIT`], that the host reads as "the guest reached its memory ceiling". The
//! PROGRAM cannot choose the guest's exit status — its own exit is a message on the channel, and it
//! has no way to end the process otherwise — so the status is the runtime's statement, not the
//! program's. Outside guest mode nothing changes: a refused allocation reaches Rust's own handler.
//!
//! The end is `_exit` (Unix) or `TerminateProcess` on itself (Windows): no allocation, no destructor,
//! no unwinding, nothing that could itself need the memory that was just refused.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, Ordering};

/// The exit status of a guest whose allocation was refused at its memory ceiling.
pub const GUEST_MEMORY_EXIT: i32 = 197;

static GUEST: AtomicBool = AtomicBool::new(false);
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
static VM_GUEST: AtomicBool = AtomicBool::new(false);

/// The line a microVM guest writes on its serial console as it ends at the ceiling. A microVM guest is
/// PID 1 of its own kernel, so its exit status never reaches the host; its console does, relayed by
/// the host (`microvm.rs::relay_console`), which recognises this line. The program cannot write to
/// that console — every effect it has is performed by the host — so the line is the runtime's.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub const VM_MEMORY_MARKER: &[u8] = b"\ndelulu-guest: memory ceiling reached\n";

/// This process is a sandbox guest from here on.
pub fn enter_guest_mode() {
    GUEST.store(true, Ordering::SeqCst);
}

/// This process is a microVM's guest (see [`VM_MEMORY_MARKER`]).
#[cfg(target_os = "linux")]
pub fn enter_vm_guest_mode() {
    VM_GUEST.store(true, Ordering::SeqCst);
    GUEST.store(true, Ordering::SeqCst);
}

/// The system allocator, with the guest's ceiling ending described above.
pub struct CeilingAware;

// SAFETY: every method forwards to `System` with the caller's arguments unchanged; the only addition
// is what happens after a null result, which never returns in guest mode.
unsafe impl GlobalAlloc for CeilingAware {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwarded as received.
        refused_ends_a_guest(unsafe { System.alloc(layout) })
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: forwarded as received.
        refused_ends_a_guest(unsafe { System.alloc_zeroed(layout) })
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: forwarded as received.
        refused_ends_a_guest(unsafe { System.realloc(ptr, layout, new_size) })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: forwarded as received.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[inline]
fn refused_ends_a_guest(p: *mut u8) -> *mut u8 {
    if p.is_null() && GUEST.load(Ordering::Relaxed) {
        end_at_the_ceiling();
    }
    p
}

#[cold]
fn end_at_the_ceiling() -> ! {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    // SAFETY: `write` of a static buffer and `_exit` allocate nothing and run nothing.
    unsafe {
        if VM_GUEST.load(Ordering::Relaxed) {
            libc::write(2, VM_MEMORY_MARKER.as_ptr().cast(), VM_MEMORY_MARKER.len());
        }
        libc::_exit(GUEST_MEMORY_EXIT)
    }
    #[cfg(windows)]
    // SAFETY: terminating this process through its own pseudo-handle, which needs no closing.
    unsafe {
        use windows_sys::Win32::System::Threading::{GetCurrentProcess, TerminateProcess};
        TerminateProcess(GetCurrentProcess(), GUEST_MEMORY_EXIT as u32);
        std::process::abort()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
    std::process::abort()
}
