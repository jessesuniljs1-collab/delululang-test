//! What one step of an interpreted loop allocates (A14, the dogfood finding: a merge sort of 100,000 `Int`s took 28 s on
//! the interpreter and well under one on WASM).
//!
//! Until 2026-10-29:
//! - a variable lived in a `HashMap<String, Value>` per scope, so each read hashed its name in every scope it passed,
//!   and each assignment allocated the name again;
//! - every block made a scope, `let` or not, and so did every arm of a `match`;
//! - a call collected its arguments in a vector, then made a scope for the parameters and another for the body;
//! - every `Some`, `None`, `Ok` and `Err` allocated its own name;
//! - every `==`, of two `Int`s too, allocated the vector of its walk.
//!
//! On Windows one allocation and its free cost about 55 ns, a quarter of each step of the merge. Each test counts the
//! allocations its thread makes while `main` runs, for [`STEPS`] steps and for twice as many, and takes the difference:
//! what the steps cost, without the run's fixed cost, and exact on any machine — no clock is read.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};

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

/// Run `src`'s `main` (with the console) on this thread; the allocations the run made, and what it printed.
fn allocations_of(src: &str) -> (u64, String) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{:?}", checked.diagnostics);
    let interp = Interp::new(&checked.module);
    let root = Value::Root(Rc::new(Grants { console: true, ..Default::default() }.build_root()));
    set_capture(true);
    let before = ALLOCATIONS.with(Cell::get);
    let r = interp.run_main(root);
    let made = ALLOCATIONS.with(Cell::get) - before;
    let out = take_capture().unwrap_or_default();
    r.unwrap_or_else(|e| panic!("the program faulted: {e:?}"));
    (made, out)
}

/// The allocations [`STEPS`] steps of `program(n)`'s loop make: the runs of `STEPS` and twice as many, differenced. Both
/// runs must print what `expect(n)` says, so a loop that stopped early cannot read as a cheap one.
fn allocations_of_steps(program: impl Fn(u64) -> String, expect: impl Fn(u64) -> String) -> u64 {
    let n = STEPS;
    let (one, out1) = allocations_of(&program(n));
    assert_eq!(out1, expect(n), "the run of {n} steps printed something else");
    let (two, out2) = allocations_of(&program(2 * n));
    assert_eq!(out2, expect(2 * n), "the run of {} steps printed something else", 2 * n);
    assert!(two >= one, "the longer run allocated less ({two} < {one})");
    two - one
}

/// Fail unless [`STEPS`] steps made at most `per_step` allocations each, saying what they made.
fn assert_at_most(made: u64, per_step: u64, what: &str) {
    assert!(
        made <= per_step * STEPS,
        "{what}: {made} allocations in {STEPS} steps, {:.2} a step, where {per_step} is what a step needs",
        made as f64 / STEPS as f64
    );
}

/// A loop of arithmetic, assignments, `==`, a `let`-less `if` and a `match` whose arms bind nothing: nothing to
/// allocate, and nothing is.
#[test]
fn a_counting_loop_allocates_nothing_per_step() {
    let program = |n: u64| {
        format!(
            "module m
fn main(root: Root) ! {{Write}} {{
    let out = root.console()
    var i = 0
    var s = 0
    while i < {n} {{
        if i % 2 == 0 {{
            s = s + 1
        }} else {{
            s = s + 2
        }}
        match i % 3 {{
            0 => {{
                s = s + 10
            }}
            _ => {{
                s = s + 0
            }}
        }}
        i = i + 1
    }}
    out.println(str(s))
}}
"
        )
    };
    let made = allocations_of_steps(program, |n| format!("{}\n", n / 2 * 3 + n.div_ceil(3) * 10));
    assert_at_most(made, 0, "a counting loop");
}

/// A call to a two-parameter function: its scope and the room for its two names, nothing more — no vector of
/// arguments, no second scope for the body, no copy of either name.
#[test]
fn a_call_allocates_its_scope_and_nothing_else() {
    let program = |n: u64| {
        format!(
            "module m
fn add(a: Int, b: Int) -> Int {{
    a + b
}}
fn main(root: Root) ! {{Write}} {{
    let out = root.console()
    var i = 0
    var s = 0
    while i < {n} {{
        s = add(s, i)
        i = i + 1
    }}
    out.println(str(s))
}}
"
        )
    };
    let made = allocations_of_steps(program, |n| format!("{}\n", n * (n - 1) / 2));
    assert_at_most(made, 2, "a call");
}

/// A call to a five-parameter function: its scope and the room for its five names, made once (RW 3.24). The room was
/// made for four and grown when the fifth name arrived, a third allocation on every call.
#[test]
fn a_call_of_five_parameters_makes_its_room_once() {
    let program = |n: u64| {
        format!(
            "module m
fn add5(a: Int, b: Int, c: Int, d: Int, e: Int) -> Int {{
    a + b + c + d + e
}}
fn main(root: Root) ! {{Write}} {{
    let out = root.console()
    var i = 0
    var s = 0
    while i < {n} {{
        s = s + add5(i, 1, 2, 3, 4)
        i = i + 1
    }}
    out.println(str(s))
}}
"
        )
    };
    let made = allocations_of_steps(program, |n| format!("{}\n", n * (n - 1) / 2 + 10 * n));
    assert_at_most(made, 2, "a call of five parameters");
}

/// A block of five `let`s: its scope and the room for its five names, made once (RW 3.24) — as a call's are, and for
/// the same reason.
#[test]
fn a_block_of_five_lets_makes_its_room_once() {
    let program = |n: u64| {
        format!(
            "module m
fn main(root: Root) ! {{Write}} {{
    let out = root.console()
    var i = 0
    var s = 0
    while i < {n} {{
        if i >= 0 {{
            let a = i
            let b = a + 1
            let c = b + 1
            let d = c + 1
            let e = d + 1
            s = s + e
        }}
        i = i + 1
    }}
    out.println(str(s))
}}
"
        )
    };
    let made = allocations_of_steps(program, |n| format!("{}\n", n * (n - 1) / 2 + 4 * n));
    assert_at_most(made, 2, "a block of five lets");
}

/// The merge's inner read, `at0(xs, i)`: the call (2), the method's argument (1), the `Some` it answers (2: the shared
/// payload and its vector — the name `Some` is shared), and the arm that binds `v` (2: its scope and its name's room).
#[test]
fn reading_a_list_through_an_option_allocates_seven_times() {
    let program = |n: u64| {
        format!(
            "module m
fn at0(xs: List[Int], i: Int) -> Int {{
    match xs.get(i) {{
        Some(v) => v
        None => 0
    }}
}}
fn main(root: Root) ! {{Write}} {{
    let out = root.console()
    let xs = [1, 2, 3]
    var i = 0
    var s = 0
    while i < {n} {{
        s = s + at0(xs, i % 3)
        i = i + 1
    }}
    out.println(str(s))
}}
"
        )
    };
    // The steps read 1, 2, 3, 1, ...: 1,000 of them end on a 1, 2,000 on a 2.
    let made = allocations_of_steps(program, |n| format!("{}\n", n / 3 * 6 + if n % 3 == 1 { 1 } else { 3 }));
    assert_at_most(made, 7, "a read through `Some`");
}

/// What the scopes that are no longer made used to hold, held where it is now: a `let` in a nested block shadows only
/// there; an assignment inside a `let`-less block, or an arm that binds nothing, reaches the variable outside; a closure
/// made inside a `let`-less block of a loop reads the iteration it was made in; an arm's binding is its own; a scope
/// with more names than a scan covers finds each one, and rebinds it in place.
#[test]
fn bindings_are_where_they_were_without_the_empty_scopes() {
    let lets: String = (0..40).map(|k| format!("    let v{k} = {k}\n")).collect();
    let src = format!(
        "module m
fn pick(o: Option[Int], d: Int) -> Int {{
    var r = d
    match o {{
        Some(x) => {{
            r = x
        }}
        None => {{
            r = r + 100
        }}
    }}
    r
}}
fn main(root: Root) ! {{Write}} {{
    let out = root.console()
    let x = 1
    var y = 0
    if x == 1 {{
        let x = 2
        y = x
    }}
    out.println(str(x) + \" \" + str(y))
    let fs = []
    var i = 0
    while i < 3 {{
        let k = i * 10
        if true {{
            fs.push(fn() -> Int {{ k + i }})
        }}
        i = i + 1
    }}
    let got = fs.map(fn(f: fn() -> Int) -> Int {{ f() }})
    out.println(str(got))
    out.println(str(pick(Some(7), 0)) + \" \" + str(pick(None, 5)))
{lets}    let v20 = v20 + v39
    out.println(str(v0 + v16 + v17 + v20 + v39))
}}
"
    );
    let (_, out) = allocations_of(&src);
    // The closures read `k` from the iteration they were made in and `i` as it is when they run (3: the loop is over).
    assert_eq!(out, "1 2\n[3, 13, 23]\n7 105\n131\n");
}
