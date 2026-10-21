//! RW 3.4 — a list, a map, an option or a result held where a value goes — in a list, a map, a variant's field, a record's
//! field — on the WASM engine, held to the interpreter. A slot holds a pointer to its cell, as it holds a record's, so a
//! list of lists, a map of lists, an option of an option, a record holding a map compile; each is shared as the
//! interpreter shares it (a list pushed to is seen through every list holding it), compared and shown by the host's value
//! interface, and taken apart by `match` at every depth. Before, each was refused with DL1201 and ran on the interpreter.

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

fn assert_parity(src: &str) -> String {
    let (w, i) = run_both(src);
    assert_eq!(w, i, "two-engine parity broken for:\n{src}");
    w.unwrap_or_else(|e| panic!("both engines failed: {e}\n{src}"))
}

/// Lists of lists built, pushed to, indexed two deep and walked by `for`, `map` and `filter`; a list of options and its
/// `contains`; one map held twice in a list, written after; a nested option taken apart by `match` and compared; a map of
/// lists; and a record whose fields hold a list of lists, a map and an option of a list.
#[test]
fn lists_maps_and_options_nest_as_on_the_interpreter() {
    let src = r#"module m

type Bag {
  items: ref List[ref List[Int]],
  index: ref Map[Str, Int],
  maybe: Option[ref List[Str]],
}

fn total(xss: ref List[ref List[Int]]) -> Int {
  var t = 0
  for xs in xss {
    for x in xs {
      t = t + x
    }
  }
  t
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let xss: ref List[ref List[Int]] = [[1, 2], [3], []]
  xss.push([4, 5, 6])
  out.println(str(xss) + " " + str(xss.len()) + " " + str(xss[3][2]) + " " + str(total(xss)))
  let opts = [Some(1), None, Some(3)]
  out.println(str(opts) + " " + str(opts.contains(None)) + " " + str(opts.contains(Some(2))))
  let ms: ref List[ref Map[Str, Int]] = []
  let m: ref Map[Str, Int] = Map()
  m.insert("a", 1)
  ms.push(m)
  ms.push(m)
  m.insert("b", 2)
  out.println(str(ms))
  let nested = Some(Some(5))
  let label = match nested {
    Some(Some(n)) => "two " + str(n),
    Some(None) => "one",
    None => "zero",
  }
  out.println(label + " " + str(nested == Some(Some(5))))
  let byname: ref Map[Str, ref List[Int]] = Map()
  byname.insert("odd", [1, 3])
  byname.insert("even", [2])
  out.println(str(byname) + " " + str(byname.get("odd")))
  let b = Bag { items: xss, index: m, maybe: Some(["x"]) }
  out.println(str(b.items.len()) + " " + str(b.index) + " " + str(b.maybe))
  let firsts = xss.map(fn(xs: ref List[Int]) -> Int { xs.len() })
  out.println(str(firsts) + " " + str(xss.filter(fn(xs: ref List[Int]) -> Bool { xs.is_empty() })))
}
"#;
    assert_eq!(
        assert_parity(src),
        "[[1, 2], [3], [], [4, 5, 6]] 4 6 21\n\
         [Some(1), None, Some(3)] true false\n\
         [{a: 1, b: 2}, {a: 1, b: 2}]\n\
         two 5 true\n\
         {even: [2], odd: [1, 3]} Some([1, 3])\n\
         4 {a: 1, b: 2} Some([x])\n\
         [2, 1, 0, 3] [[]]\n"
    );
}

/// One list held twice in a list and once in a record, pushed to after: every holder sees the push, and `==` and
/// `contains` compare by contents. A record's field holding a map, written after; a result of lists taken apart; a sum
/// whose variant holds a list; an empty list of lists, its type written, pushed to; `find` and `fold` over lists of
/// lists; three levels of lists.
#[test]
fn a_nested_cell_is_shared_compared_and_faulted_as_on_the_interpreter() {
    let src = r#"module m

type N {
  v: Int,
  next: Option[ref List[Int]],
}

type N2 {
  v: Int,
  m: ref Map[Str, Int],
}

type Tree = Leaf | Node(ref List[Int])

fn size(t: Tree) -> Int {
  match t {
    Node(xs) => xs.len(),
    Leaf => 0,
  }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let inner: ref List[Int] = [1]
  let outer: ref List[ref List[Int]] = [inner, inner]
  inner.push(2)
  out.println(str(outer) + " " + str(outer[0] == outer[1]) + " " + str(outer.contains([1, 2])) + " " + str(outer.contains([2])))
  let n = N { v: 1, next: Some(inner) }
  inner.push(3)
  out.println(str(n) + " " + str(match n.next { Some(xs) => xs.len(), None => 0 }))
  let mm: ref Map[Str, Int] = Map()
  let n2 = N2 { v: 2, m: mm }
  mm.insert("k", 9)
  out.println(str(n2) + " " + str(n2.m.get("k")))
  let r: Result[ref List[Int], ref List[Str]] = Ok([4, 5])
  let e: Result[ref List[Int], ref List[Str]] = Err(["bad"])
  out.println(str(r) + " " + str(e) + " " + str(match r { Ok(xs) => xs.len(), Err(es) => 0 - es.len() }))
  out.println(str(size(Node([7, 8, 9]))) + " " + str(size(Leaf)) + " " + str(Node([1]) == Node([1])))
  let empty: ref List[ref List[Int]] = []
  empty.push([])
  out.println(str(empty) + " " + str(empty[0].len()))
  let found = outer.find(fn(xs: ref List[Int]) -> Bool { xs.len() > 2 })
  let sum = outer.fold(0, fn(acc: Int, xs: ref List[Int]) -> Int { acc + xs.len() })
  out.println(str(found) + " " + str(sum))
  let deep = [[[1]], [[2, 3], []]]
  out.println(str(deep) + " " + str(deep[1][0][1]))
}
"#;
    assert_eq!(
        assert_parity(src),
        "[[1, 2], [1, 2]] true true false\n\
         N { v: 1, next: Some([1, 2, 3]) } 3\n\
         N2 { v: 2, m: {k: 9} } Some(9)\n\
         Ok([4, 5]) Err([bad]) 2\n\
         3 0 true\n\
         [[]] 0\n\
         Some([1, 2, 3]) 6\n\
         [[[1]], [[2, 3], []]] 3\n"
    );
}

/// An index past an inner list's end faults on both engines, in the interpreter's words.
#[test]
fn an_index_past_an_inner_lists_end_faults_as_on_the_interpreter() {
    let src = "module m\nfn main(root: Root) ! {Write} {\n  let xss = [[1, 2, 3]]\n  root.console().println(str(xss[0][9]))\n}\n";
    let (w, i) = run_both(src);
    let w = w.expect_err("the index faults on WASM");
    let i = i.expect_err("and on the interpreter");
    assert!(w.contains("DL0903: index 9 out of bounds (len 3)"), "{w}");
    assert!(i.contains("DL0903") && i.contains("index 9 out of bounds (len 3)"), "{i}");
}
