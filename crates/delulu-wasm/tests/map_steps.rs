//! What one `Map` lookup and update costs the WASM engine's host (A5, the agent test campaign: a word count over 2 MB took
//! 6.3 s on the interpreter and 16.8 s on `--engine wasm`).
//!
//! A `Map` lives in the guest's memory as pairs in key order, and the host finds a key by binary search. Until 2026-10-29
//! each probe read its pair's key out into a new `MapKey` — for a `Str` key a byte vector and a `String` — and each call
//! copied the map's header into a vector too, so a lookup in 38 words made about a dozen allocations, and the word count's
//! loop ran slower than the interpreter's. The keys are now compared where they lie.
//!
//! The test counts the allocations its thread makes while the module runs, for [`STEPS`] steps and for twice as many, and
//! takes the difference — what the steps cost, exact on any machine, no clock read.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

use delulu_check::check_source;
use delulu_wasm::{compile_checked, run_main, HostConfig};

/// The system allocator, counting the allocations each thread makes.
struct Counting;

thread_local! {
    static ALLOCATIONS: Cell<u64> = const { Cell::new(0) };
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let _ = ALLOCATIONS.try_with(|n| n.set(n.get() + 1));
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

/// How many steps the shorter run takes; the longer takes twice as many.
const STEPS: u64 = 1000;

/// A loop of `n` steps, each reading a word's count from a `Map[Str, Int]` of eight words and writing it back one higher.
fn word_count(n: u64) -> String {
    format!(
        "module m
fn main(root: Root) ! {{Write}} {{
    let out = root.console()
    let words = [\"alpha\", \"beta\", \"gamma\", \"delta\", \"epsilon\", \"zeta\", \"eta\", \"theta\"]
    let m: ref Map[Str, Int] = Map()
    var i = 0
    while i < {n} {{
        match words.get(i % 8) {{
            Some(w) => {{
                match m.get(w) {{
                    Some(c) => m.insert(w, c + 1)
                    None => m.insert(w, 1)
                }}
            }}
            None => {{
                i = i + 0
            }}
        }}
        i = i + 1
    }}
    match m.get(\"alpha\") {{
        Some(c) => out.println(str(m.len()) + \" \" + str(c))
        None => out.println(\"none\")
    }}
}}
"
    )
}

/// The allocations the run of `n` steps makes on this thread while the module runs, and what it printed.
fn allocations_of(n: u64) -> (u64, String) {
    let src = word_count(n);
    let checked = check_source(0, &src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm = compile_checked(&checked.module, &checked.result).unwrap_or_else(|e| panic!("should compile: {e:?}\n{src}"));
    let cfg = HostConfig { console: true, ..HostConfig::default() };
    let before = ALLOCATIONS.with(Cell::get);
    let out = run_main(&wasm, &cfg).unwrap_or_else(|e| panic!("the run faulted: {}", e.message()));
    (ALLOCATIONS.with(Cell::get) - before, out)
}

#[test]
fn a_map_lookup_and_update_allocate_nothing_in_the_host() {
    let (one, out1) = allocations_of(STEPS);
    assert_eq!(out1, format!("8 {}\n", STEPS.div_ceil(8)), "the run of {STEPS} steps");
    let (two, out2) = allocations_of(2 * STEPS);
    assert_eq!(out2, format!("8 {}\n", (2 * STEPS).div_ceil(8)), "the run of {} steps", 2 * STEPS);
    let made = two.saturating_sub(one);
    assert_eq!(
        made,
        0,
        "{made} allocations in {STEPS} steps of a `get` and an `insert`, {:.2} a step, where the host needs none",
        made as f64 / STEPS as f64
    );
}
