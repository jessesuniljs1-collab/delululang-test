//! DATA-DEPTH-1 — a value nested past the native stack, on the WASM engine. `str` and `==` of a list, an option, a record
//! or a sum are answered host-side: the host rebuilds the value from the guest's memory (`reify.rs`) and asks the
//! interpreter's `Value::display` and `Value::eq`. All three recursed once per level, so a linked list of 100,000
//! records built in a loop — well inside the memory budget — overflowed the host's stack and killed the process, and a
//! hand-written module could do the same with a descriptor nested a million deep. Each now walks a worklist, and a
//! descriptor deeper than any the compiler writes is refused in words. As with the interpreter's teardown locks, the
//! falsification signal is the test binary dying.

use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{set_capture, take_capture, Grants, Interp, Value};
use delulu_wasm::{compile_checked, run_main, HostConfig};

/// Run `main` on both engines with the console; both outputs, or each fault's text.
fn run_both(src: &str) -> (Result<String, String>, Result<String, String>) {
    let checked = check_source(0, src);
    assert!(!checked.has_errors(), "program should check clean:\n{src}\n{:?}", checked.diagnostics);
    let wasm =
        compile_checked(&checked.module, &checked.result).unwrap_or_else(|e| panic!("should compile to WASM: {e:?}\n{src}"));
    let w = run_main(&wasm, &HostConfig { console: true, ..HostConfig::default() }).map_err(|e| e.message());
    set_capture(true);
    let g = Grants { console: true, ..Default::default() };
    let i = Interp::new(&checked.module).run_main(Value::Root(Rc::new(g.build_root()))).map_err(|e| format!("{e:?}"));
    let out = take_capture().unwrap_or_default();
    (w, i.map(|_| out))
}

/// Two linked lists of 100,000 records, built apart in a loop: `str` of one, and `==` of the two, equal and then not.
#[test]
fn a_value_nested_past_the_stack_shows_and_compares_on_both_engines() {
    let src = r#"module m

type Node {
  v: Int,
  next: Option[ref Node],
}

fn chain(n: Int, last: Int) -> ref Node {
  var head = Node { v: last, next: None }
  var i = 1
  while i < n {
    head = Node { v: i, next: Some(head) }
    i = i + 1
  }
  head
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let a = chain(100000, 0)
  out.println(str(str(a).len()))
  out.println(str(a == chain(100000, 0)) + " " + str(a == chain(100000, 1)))
}
"#;
    let (w, i) = run_both(src);
    assert_eq!(w, i, "two-engine parity broken");
    assert_eq!(w.expect("both engines answer"), "3088888\ntrue false\n");
}

/// One cell held twice side by side — an option, a list, a record, each in two fields of one record — shows whole each
/// time: the host refuses only a variant cell met again *inside* itself, so its rebuilding of a cell ends when the cell's
/// own does.
#[test]
fn a_cell_held_twice_side_by_side_shows_whole_each_time() {
    let src = r#"module m

type Leaf {
  n: Int,
}

type Pair {
  a: Option[Int],
  b: Option[Int],
  c: ref List[Int],
  d: ref List[Int],
  e: ref Leaf,
  f: ref Leaf,
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let o = Some(1)
  let l: ref List[Int] = [2]
  let leaf = Leaf { n: 3 }
  let p = Pair { a: o, b: o, c: l, d: l, e: leaf, f: leaf }
  out.println(str(p))
  out.println(str(p == p))
}
"#;
    let (w, i) = run_both(src);
    assert_eq!(w, i, "two-engine parity broken");
    assert_eq!(
        w.expect("both engines answer"),
        "Pair { a: Some(1), b: Some(1), c: [2], d: [2], e: Leaf { n: 3 }, f: Leaf { n: 3 } }\ntrue\n"
    );
}

/// A hand-written module asks the host to show a value by a descriptor a million `List`s deep: refused in the words any
/// descriptor the compiler did not write is, never a crash.
#[test]
fn a_descriptor_nested_past_any_written_type_is_refused_in_words() {
    let depth = 1_000_000usize;
    let len = (depth + 1) as u32;
    let le: String = len.to_le_bytes().iter().map(|b| format!("\\{b:02x}")).collect();
    let wat = format!(
        r#"(module
  (import "delulu:value" "display" (func $display (param i32 i32) (result i32)))
  (memory (export "memory") 32)
  (global (export "__heap") (mut i32) (i32.const 1048576))
  (data (i32.const 1024) "{le}{desc}i")
  (func (export "main") (param i32) (drop (call $display (i32.const 0) (i32.const 1024)))))"#,
        desc = "l".repeat(depth)
    );
    let got = run_main(wat.as_bytes(), &HostConfig { console: true, ..HostConfig::default() });
    let err = got.expect_err("a descriptor this deep is refused").message();
    assert!(err.contains("DL0903") && err.contains("type descriptor is not one the WASM engine wrote"), "{err}");
}
