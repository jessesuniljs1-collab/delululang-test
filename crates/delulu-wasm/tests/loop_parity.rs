//! RW 3.4 — `while`, `break`, `continue` and assignment to a local on the WASM engine, held to the interpreter.
//!
//! The backend refused every loop (DL1201), so a program that counted, accumulated or built a string ran only on the
//! interpreter — four of the six Study-C kernels among them. It compiles them now, and every allocation it makes, and
//! the host makes in its memory, grows that memory when it must (the heap had been a fixed 1 MiB, which a loop or a
//! recursion that builds a string outgrew). The interpreter is the reference: the same output, or both a fault.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_and_run_int, compile_module, gen::random_loop_program, run_main, HostConfig};

/// Run `main` on both engines with the console (and, if given, a read root) and return `(wasm, interp)`.
fn run_both(src: &str, fs_root: Option<&std::path::Path>) -> (Result<String, String>, Result<String, String>) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm = compile_module(&checked.module).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
    let roots: Vec<std::path::PathBuf> = fs_root.into_iter().map(|p| p.to_path_buf()).collect();
    let cfg = HostConfig { console: true, fs_read_roots: roots.clone(), ..HostConfig::default() };
    let w = run_main(&wasm, &cfg).map_err(|e| format!("{e:?}"));
    set_capture(true);
    let g =
        Grants { console: true, fs_read: roots.iter().map(|p| p.to_string_lossy().to_string()).collect(), ..Default::default() };
    let i = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root()))).map_err(|e| format!("{e:?}"));
    let out = take_capture().unwrap_or_default();
    (w, i.map(|_| out))
}

fn assert_parity(src: &str) -> String {
    let (w, i) = run_both(src, None);
    assert_eq!(w, i, "two-engine parity broken for:\n{src}");
    w.unwrap_or_else(|e| panic!("both engines failed: {e}\n{src}"))
}

#[test]
fn loops_break_continue_and_assignment_match_the_interpreter() {
    let src = r#"module m

fn sum_to(n: Int) -> Int {
  var i = 0
  var s = 0
  while i < n {
    i = i + 1
    if i % 3 == 0 { continue }
    if i > 1000 { break }
    s = s + i
  }
  s
}

fn first_square_over(limit: Int) -> Int {
  var k = 0
  while true {
    k = k + 1
    if k * k > limit { return k }
  }
  0
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println(str(sum_to(10)) + " " + str(sum_to(5000)) + " " + str(first_square_over(150)))
  var row = 0
  var grid = ""
  while row < 4 {
    var col = 0
    while col < 5 {
      col = col + 1
      if col == 2 { continue }
      if col == 4 { break }
      grid = grid + str(row * col)
    }
    grid = grid + "|"
    row = row + 1
  }
  out.println(grid)
  var never = 7
  while false { never = 0 }
  let x = 1
  var y = 0
  while y < 2 {
    let x = 10
    y = y + x
  }
  out.println(str(never) + " " + str(x) + " " + str(y))
  var ok = true
  var t = 0
  while ok {
    t = t + 1
    ok = t < 5
  }
  out.println(str(t) + " " + str("héllo ✓".len()) + " " + str("".len()) + " " + str(ok) + " " + str(t > 4))
}
"#;
    let out = assert_parity(src);
    assert_eq!(out, "37 333667 13\n00|13|26|39|\n7 1 10\n5 7 0 false true\n", "the values themselves");
}

/// The heap had been a fixed 1 MiB with no `memory.grow`, and the bump allocator never frees: any program whose strings
/// added up past it trapped on WASM while the interpreter ran it. Witnessed with a recursion and no `.len()`, which both
/// compiled before RW 3.4 — red on the old backend (`out of bounds memory access`), so the growth is measured, not assumed.
#[test]
fn a_string_built_past_the_first_heap_grows_the_memory() {
    let src = r#"module m

fn grow(s: Str, n: Int) -> Str {
  if n == 0 { s } else { grow(s + "abcdefghijabcdefghijabcdefghijabcdefghijabcdefghij", n - 1) }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  out.println(grow("", 400))
}
"#;
    // 400 concatenations of a growing string allocate about 4 MB — four times the first heap. The interpreter recurses
    // 400 deep: on a thread with room for it.
    let out = std::thread::Builder::new().stack_size(256 << 20).spawn(move || assert_parity(src)).unwrap().join().unwrap();
    assert_eq!(out.len(), 400 * 50 + 1);
}

/// **WASM-HYGIENE-1.** The backend inlines a generic function and the lambdas passed to it, and an inlined body looked
/// names up through every frame below it: a lambda's captured `n` found the generic function's own `n`. The interpreter
/// printed 4, the WASM engine 103 — red on the old backend. Each inlined body sees what was visible where it was written.
#[test]
fn an_inlined_lambda_sees_the_names_where_it_was_written() {
    let src = r#"module m

fn apply[T](g: fn(T) -> T, x: T) -> T {
  let n = 100
  g(x)
}

fn twice[T](g: fn(T) -> T, x: T) -> T {
  let x = apply(g, x)
  apply(g, x)
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let n = 1
  let add = fn(v: Int) -> Int { v + n }
  out.println(str(apply(fn(v: Int) -> Int { v + n }, 3)) + " " + str(twice(add, 10)) + " " + str(apply(add, n)))
}
"#;
    assert_eq!(assert_parity(src), "4 12 2\n");
}

/// **WASM-HYGIENE-2.** A function value bound inside a block went into the function's own frame of function values, so
/// an inner `let f = fn…` replaced the outer `f` for the rest of the function: after an `if` whose body shadowed it the
/// interpreter printed 1 and the WASM engine 2 — red on the old backend, where that `if` compiled before RW 3.4 too — and
/// after a `while` whose body did, 1 against 0. A block's function values are the block's, in every block.
#[test]
fn a_function_value_bound_in_a_block_is_that_blocks() {
    let src = r#"module m

fn after_if(k: Int) -> Int {
  let f = fn(x: Int) -> Int { x + 1 }
  var seen = 0
  if k > 0 {
    let f = fn(x: Int) -> Int { x + 2 }
    seen = f(0)
  } else {
    let f = fn(x: Int) -> Int { x + 3 }
    seen = f(0)
  }
  seen * 10 + f(0)
}

fn after_while(n: Int) -> Int {
  let f = fn(x: Int) -> Int { x + 1 }
  var i = 0
  var acc = 0
  while i < n {
    let f = fn(x: Int) -> Int { x * 100 }
    acc = acc + f(1)
    i = i + 1
  }
  acc + f(0)
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let f = fn(x: Int) -> Int { x + 7 }
  if true {
    let f = fn(x: Int) -> Int { x + 2 }
    out.println(str(f(0)))
  }
  out.println(str(f(0)) + " " + str(after_if(1)) + " " + str(after_if(0)) + " " + str(after_while(3)))
}
"#;
    assert_eq!(assert_parity(src), "2\n7 21 31 301\n", "the values themselves");
}

/// **LATE-LET-1.** A function value never reads a name a `let` binds after it was made: the checker resolved its names
/// where it was written. Both engines read them where it ran — the interpreter in the scope it closed over, the WASM
/// engine in the frames it was defined over, by index — and a later `let` bound in that same scope or frame. So `h`
/// called the second `g` (33 where 23), `f` read the `Str` a later `x` bound (`two40`; the WASM engine refused the `+`
/// as DL1201), and in `inner` both `f` and `g` read the last `a` (900001 where 321001). The engines agreed with each
/// other throughout, which is why no parity gate saw it; the values are the witness. `inner` also holds the frames a
/// `let` opens to its block: after the `if`, `a` is the function's 1 again, not the 20 or 300 bound inside it.
#[test]
fn a_function_value_never_reads_a_name_bound_after_it() {
    let src = r#"module m

fn after(k: Int) -> Int {
  let g = fn(x: Int) -> Int { x + 1 }
  let h = fn(x: Int) -> Int { g(x) * 10 }
  let g = fn(x: Int) -> Int { x + 2 }
  h(k) + g(k)
}

fn inner(k: Int) -> Int {
  let a = 1
  var seen = 0
  if k > 0 {
    let f = fn() -> Int { a }
    let a = 20
    let g = fn() -> Int { a + f() }
    let a = 300
    seen = g() + a
  }
  seen * 1000 + a
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let x = 1
  let f = fn() -> Int { x }
  let x = "two"
  out.println(str(f() + 40) + " " + x + " " + str(after(1)) + " " + str(inner(1)))
}
"#;
    assert_eq!(assert_parity(src), "41 two 23 321001\n", "the values themselves");
}

/// **CTOR-SHADOW-1.** A function may share its name with a constructor of the module's own sum type, and the checker
/// resolves the name to the constructor, as a call and as a value. The interpreter asked the module's functions first
/// for a call, so `Circle(3)` ran the function (300, an `Int`, where the checker typed a `Shape`, and the `match` on it
/// faulted). And it turned a capitalized name into a constructor before asking the functions for a value, so `Double`
/// passed to `apply` was a constructor with no fields, *"not callable"* (DL0907). A constructor whose name is lowercase
/// (`cell`, `empty`), which the checker admits, was no capitalized name either, so the interpreter faulted *"unbound
/// name"* on it as a value and as a call. The WASM engine resolved all of these as the checker does.
#[test]
fn a_constructor_is_built_where_a_function_shares_its_name_and_a_function_is_passed_where_none_does() {
    let src = r#"module m

type Shape = Circle(Int) | Dot

fn Circle(x: Int) -> Int { x * 100 }

fn Dot() -> Int { 7 }

fn Double(x: Int) -> Int { x * 2 }

fn apply[T](g: fn(T) -> T, x: T) -> T { g(x) }

type Slot = cell(Int) | empty

fn size(s: Slot) -> Int { match s { cell(n) => n, _ => 0 } }

fn main(root: Root) ! {Write} {
  let out = root.console()
  let s: Shape = Circle(3)
  let d = Dot
  let a = match s { Circle(n) => n, Dot => 0 }
  let b = match d { Circle(n) => n, Dot => 0 - 1 }
  let e = empty
  let c = Circle(4)
  let f = match c { Circle(n) => n, Dot => 0 }
  out.println(str(a) + " " + str(b) + " " + str(apply(Double, 5)) + " " + str(size(cell(8))) + str(size(e)) + " " + str(f))
}
"#;
    assert_eq!(assert_parity(src), "3 -1 10 80 4\n", "the values themselves");
}

/// **WASM-BUILTIN-SHADOW-1** (found 2026-11-01 beside CTOR-SHADOW-1): a local function value named like a prelude
/// builtin is called, as the checker resolves the name to its innermost binding. The WASM engine asked the builtins
/// first, so `str(5)` and `int(1.5)` ran the builtins and printed `5 builtin`; the interpreter called the locals.
#[test]
fn a_local_named_like_a_builtin_is_called_on_both_engines() {
    let src = r#"module m

fn named(n: Int) -> Str { if n == 77 { "seventy-seven" } else { "builtin" } }

fn main(root: Root) ! {Write} {
  let out = root.console()
  let str = fn(x: Int) -> Str { "local" }
  let int = fn(x: Float) -> Int { 77 }
  out.println(str(5) + " " + named(int(1.5)))
}
"#;
    assert_eq!(assert_parity(src), "local seventy-seven\n", "the values themselves");
}

/// The host writes what it reads into the guest's heap too (`fs.read_text`), and grows the memory by the same rule — a
/// file larger than the first heap was a refusal on WASM alone.
#[test]
fn the_host_grows_the_memory_for_what_it_writes() {
    let dir = std::env::temp_dir().join(format!("delulu_wasm_grow_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("big.txt"), "z".repeat(3 << 20)).unwrap();
    let root = dir.to_string_lossy().replace('\\', "/");
    let src = format!(
        "module m\nfn main(root: Root) ! {{Write, Read}} {{\n  let out = root.console()\n  let fs = root.fs_read(\"{root}\")\n  \
         match fs.read_text(\"big.txt\") {{ Ok(c) => out.println(str(c.len())), Err(e) => out.println(\"refused\") }}\n}}\n"
    );
    let (w, i) = run_both(&src, Some(&dir));
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(w, i, "two-engine parity");
    assert_eq!(w.unwrap(), format!("{}\n", 3 << 20));
}

/// The loop generator's programs on both engines — the value, or both a fault (overflow, a zero divisor) — and not a
/// vacuous pass: most must run to a value, some must fault, the loops must have run, and many must shadow a function
/// value inside the loop (WASM-HYGIENE-2). The 50,000-program gate (`differential.rs`, CI's `fuzz` job) runs the same
/// generator.
#[test]
fn random_loop_programs_agree_with_the_interpreter() {
    let (mut agreed, mut faulted, mut ran, mut shadowed, mut listed) = (0u32, 0u32, 0u32, 0u32, 0u32);
    // A top-level function named as a value: `h` bound to it, and `map` given it (RW 3.4).
    let (mut bound, mut mapped) = (0u32, 0u32);
    for k in 0..400u64 {
        let (src, args) = random_loop_program(k.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let checked = check_source(0, &src);
        assert!(!checked.has_errors(), "a generated program must check clean:\n{src}\n{:?}", checked.diagnostics);
        let iv = Interp::new(&checked.module).call_int_fn("f", &args);
        let wv = compile_and_run_int(&checked.module, "f", &args);
        match (&iv, &wv) {
            (Ok(a), Ok(b)) if a == b => agreed += 1,
            (Err(_), Err(_)) => faulted += 1,
            _ => panic!("engines disagree: interp={iv:?} wasm={wv:?} args={args:?}\n{src}"),
        }
        if src.contains("while i < ") && !src.contains("while i < 0 ") {
            ran += 1;
        }
        if src.matches("let h = ").count() > 1 {
            shadowed += 1;
        }
        // The WASM `List`: a list built, and pushed to or indexed or read with `get` in the loop.
        if src.contains("let xs = [") && (src.contains("xs.push(") || src.contains("xs[") || src.contains("xs.get(")) {
            listed += 1;
        }
        bound += u32::from(src.contains("let h = step"));
        mapped += u32::from(src.contains("xs.map(step)"));
    }
    eprintln!(
        "loop programs: agreed {agreed}, faulted {faulted}, looping {ran}, shadowing a function value {shadowed}, with a \
         list {listed}, binding a named function {bound}, mapping one {mapped}"
    );
    assert!(
        agreed > 150 && faulted > 5 && ran > 300 && shadowed > 80 && listed > 120 && bound > 30 && mapped > 10,
        "agreed {agreed}, faulted {faulted}, looping programs {ran}, shadowing programs {shadowed}, list programs {listed}, \
         named-function programs {bound} bound and {mapped} mapped"
    );
}

/// What stays outside the fragment is refused, never miscompiled: a `for` over a list of functions (a `for` over a list
/// compiles since the WASM `List`, `list_parity.rs`, and over a list of lists since `nested_parity.rs`), and an
/// assignment from inside an inlined function value to a variable of the frame that defined it — the same lambda reading
/// the variable compiles.
#[test]
fn what_the_loop_fragment_leaves_out_is_refused() {
    let outside = [
        "module m\nfn f() -> Int {\n  var t = 0\n  for g in [fn(x: Int) -> Int { x }] { t = t + g(1) }\n  t\n}\n",
        "module m\nfn apply[T](g: fn(T) -> T, x: T) -> T { g(x) }\nfn f() -> Int {\n  var n = 0\n  let h = fn(v: Int) -> Int {\n    n = v\n    v + 1\n  }\n  apply(h, 3)\n}\n",
    ];
    for src in outside {
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{src}\n{:?}", checked.diagnostics);
        assert!(compile_module(&checked.module).is_err(), "stays on the interpreter:\n{src}");
    }
    let reads = "module m\nfn apply[T](g: fn(T) -> T, x: T) -> T { g(x) }\nfn f() -> Int {\n  var n = 0\n  let h = fn(v: Int) -> Int { v + n }\n  apply(h, 3)\n}\n";
    let checked = check_source(0, reads);
    assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
    assert!(compile_module(&checked.module).is_ok(), "a lambda that only reads its frame's variable compiles");
}
