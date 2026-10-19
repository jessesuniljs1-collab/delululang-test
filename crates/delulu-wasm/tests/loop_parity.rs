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
    let g = Grants { console: true, fs_read: roots.iter().map(|p| p.to_string_lossy().to_string()).collect(), ..Default::default() };
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
  out.println(str(t) + " " + str("héllo ✓".len()) + " " + str("".len()))
}
"#;
    let out = assert_parity(src);
    assert_eq!(out, "37 333667 13\n00|13|26|39|\n7 1 10\n5 7 0\n", "the values themselves");
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
/// vacuous pass: most must run to a value, some must fault, and the loops must have run. The 50,000-program gate
/// (`differential.rs`, CI's `fuzz` job) runs the same generator.
#[test]
fn random_loop_programs_agree_with_the_interpreter() {
    let (mut agreed, mut faulted, mut ran) = (0u32, 0u32, 0u32);
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
    }
    assert!(agreed > 150 && faulted > 5 && ran > 300, "agreed {agreed}, faulted {faulted}, looping programs {ran}");
}

/// What stays outside the fragment is refused, never miscompiled: a `for` (a `List`), and an assignment from inside an
/// inlined function value to a variable of the frame that defined it — the same lambda reading the variable compiles.
#[test]
fn what_the_loop_fragment_leaves_out_is_refused() {
    let outside = [
        "module m\nfn f() -> Int {\n  var t = 0\n  for x in [1, 2, 3] { t = t + x }\n  t\n}\n",
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
