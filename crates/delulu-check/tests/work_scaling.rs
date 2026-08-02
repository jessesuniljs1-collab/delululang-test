//! The checker's cost must stay linear in the thing being varied — asserted, not hoped.
//!
//! This project has already shipped one accidentally-quadratic checker path. Campaign finding
//! **C48**: `field_type` cloned the whole type definition on every field access, so an N-field
//! record read N times cost N² deep copies — 632 ms at N=2000, against 173 ms for a file five times
//! its size (`measurements/scale/RECORD.md`). Nothing failed. It was found by someone deciding to
//! measure, and it could return tomorrow behind any innocuous `.clone()`.
//!
//! **Why this gate counts allocations rather than milliseconds.** A wall-clock gate on a shared
//! machine is a flake, and a flaky gate gets deleted or ignored — this repository has been bitten by
//! exactly that once already (the criterion-10 satellite's DEBUG-timing failure). Allocation counts
//! are a deterministic function of the input and the pinned toolchain, so the same tree gives the
//! same number on Windows, Linux and macOS alike.
//!
//! **And why it asserts a SHAPE rather than a constant.** A gate that pinned "checking this program
//! allocates N times" would fail on every harmless refactor and teach everyone to re-bless it
//! without reading. What cannot be allowed to change is the *curve*: doubling the input must roughly
//! double the work. That is immune to constant drift and is precisely what C48 violated — and the
//! ratio it would have reported for C48's shape is measured below and printed when the gate fires.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

/// Counts allocations on the current thread only.
///
/// Cargo runs tests in parallel threads, so a process-wide counter would report whatever the other
/// tests happened to be doing. The cell is `const`-initialised so that reading it cannot itself
/// allocate and re-enter this allocator.
struct Counting;

thread_local! {
    static ALLOCS: Cell<u64> = const { Cell::new(0) };
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        // `try_with` rather than `with`: during thread teardown the TLS slot is gone, and a panic
        // inside the allocator would abort the process.
        let _ = COUNTING.try_with(|on| {
            if on.get() {
                let _ = ALLOCS.try_with(|n| n.set(n.get() + 1));
            }
        });
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
        let _ = COUNTING.try_with(|on| {
            if on.get() {
                let _ = ALLOCS.try_with(|n| n.set(n.get() + 1));
            }
        });
        unsafe { System.realloc(p, l, new) }
    }
}

#[global_allocator]
static A: Counting = Counting;

/// Allocations performed while checking `src`, on this thread.
fn allocations_to_check(src: &str) -> u64 {
    ALLOCS.with(|n| n.set(0));
    COUNTING.with(|c| c.set(true));
    let checked = delulu_check::check_source(0, src);
    COUNTING.with(|c| c.set(false));
    let count = ALLOCS.with(|n| n.get());
    // Keep the result alive past the measurement so nothing is optimised away, and assert the
    // program was the one intended: a corpus that failed to parse would allocate almost nothing and
    // the ratio would look wonderful.
    assert!(
        checked.diagnostics.is_empty(),
        "the generated corpus must check clean, or this measures error reporting: {:?}",
        &checked.diagnostics[..checked.diagnostics.len().min(3)]
    );
    count
}

/// An N-field record with a function that reads every field — **C48's exact shape**.
///
/// Each read is its own `let _ =` statement rather than one N-term sum, and that is deliberate
/// twice over. `measurements/scale/RECORD.md`'s isolation table separated "N locals, N-term sum"
/// from "N fields, N accesses" precisely because only their *product* was quadratic; conflating
/// them again would measure the wrong thing. And an N-term sum builds an N-deep expression that
/// overflows a default 2 MiB test-thread stack at N=400 — the reason `main.rs` runs the CLI on a
/// 512 MiB stack, and campaign finding C21's still-open residual for library embeddings.
fn records(n: usize) -> String {
    let mut s = String::from("module w\n\ntype R {\n");
    for i in 0..n {
        s.push_str(&format!("  f{i}: Int,\n"));
    }
    s.push_str("}\n\nfn total(r: R) -> Int {\n");
    for i in 0..n {
        s.push_str(&format!("  let _ = r.f{i}\n"));
    }
    s.push_str("  0\n}\n");
    s
}

/// N sibling functions — the shape `measurements/scale/RECORD.md` calls `wide`.
fn wide(n: usize) -> String {
    let mut s = String::from("module w\n");
    for i in 0..n {
        s.push_str(&format!("\nfn f{i}(n: Int) -> Int {{ n + {i} }}\n"));
    }
    s
}

/// Doubling the input may not more-than-double the work.
///
/// The bound is 2.6×, not 2.0×: a linear algorithm with any per-item bookkeeping lands slightly
/// above 2, and the point is to catch a curve bending upward, not to police a constant.
///
/// **Measured headroom, 2026-08-02** — the numbers that justify the bound rather than a guess at
/// it. Healthy: `records` **1.88**, `wide` **1.97**. With C48 deliberately reintroduced (one
/// `.clone()` restored on `field_type`'s record path in `check.rs`): **3.89**, and this gate was
/// **observed failing** on it before being trusted. So 2.6 sits ~35% above the healthy ratio and
/// ~33% below the defect — wide enough not to churn, tight enough to catch the one bug this
/// project has actually had.
fn assert_roughly_linear(name: &str, mk: impl Fn(usize) -> String, n: usize) {
    let a = allocations_to_check(&mk(n));
    let b = allocations_to_check(&mk(n * 2));
    let ratio = b as f64 / a as f64;
    assert!(
        ratio <= 2.6,
        "checking `{name}` is no longer linear: {n} fields/items allocated {a}, {} allocated {b} \
         — a ratio of {ratio:.2} where doubling should cost about 2. Something on the per-item \
         path is doing work proportional to the whole program (campaign finding C48 was a \
         whole-definition `.clone()` on the field-access path). This is a measurement, not a \
         style rule: find what became superlinear.",
        n * 2
    );
}

#[test]
fn field_access_stays_linear_in_record_width() {
    assert_roughly_linear("records", records, 200);
}

#[test]
fn checking_stays_linear_in_the_number_of_functions() {
    assert_roughly_linear("wide", wide, 400);
}

/// The gate must be able to fail. A counter that never counts would let both tests above pass while
/// measuring nothing at all — the empty-experiment failure `measurements/METHODOLOGY.md` rule 2
/// forbids.
#[test]
fn the_counter_actually_counts() {
    let small = allocations_to_check(&wide(10));
    let large = allocations_to_check(&wide(200));
    assert!(small > 0, "the allocator hook recorded nothing, so the gate above guards nothing");
    assert!(
        large > small * 4,
        "a 20x larger program allocated {large} against {small} — the counter is not tracking the \
         work it claims to"
    );
}
