//! D-V2-177 — a capability or a secret held where a value goes — in a list, an option, a record's field, a map's value — on
//! the WASM engine, held to the interpreter. A slot holds the handle's index into the host's table, as a variable does; the
//! guest never reads through it. Before, each was refused with DL1201 and the program ran on the interpreter.

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

/// The console capability in a list walked by `for` and indexed, in an option taken apart by `match`, in a record's field,
/// and as a map's value fetched by `get`; each prints through the handle it holds, and a function takes a list of them.
#[test]
fn a_capability_in_a_list_an_option_a_record_and_a_map_acts_as_on_the_interpreter() {
    let src = r#"module m

type Outs {
  out: Cap[Console],
  n: Int,
}

fn each(outs: List[Cap[Console]], line: Str) ! {Write} {
  for o in outs {
    o.println(line)
  }
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  let outs = [out, out]
  each(outs, "from a list")
  outs[1].println("by index " + str(outs.len()))
  let maybe = Some(out)
  match maybe {
    Some(o) => o.println("from an option"),
    None => out.println("none"),
  }
  let r = Outs { out: out, n: 3 }
  r.out.println("from a record " + str(r.n))
  let byname: ref Map[Str, Cap[Console]] = Map()
  byname.insert("main", out)
  match byname.get("main") {
    Some(o) => o.println("from a map " + str(byname.len())),
    None => out.println("none"),
  }
  let inferred = Map()
  inferred.insert("spare", out)
  var later = None
  later = inferred.get("spare")
  match later {
    Some(o) => o.println("from a map and an option the checker typed"),
    None => out.println("none"),
  }
}
"#;
    assert_eq!(
        assert_parity(src),
        "from a list\nfrom a list\nby index 2\nfrom an option\nfrom a record 3\nfrom a map 1\n\
         from a map and an option the checker typed\n"
    );
}

/// A secret held in a list or an option compiles as its handle; what would read its bytes stays on the interpreter
/// (DL1205), however it was reached.
#[test]
fn a_secret_in_a_list_or_an_option_is_a_handle_and_exposing_it_stays_dl1205() {
    let compile = |body: &str| {
        let src = format!(
            "module m\nfn f(root: Root) -> Bool ! {{Declassify}} {{ let key = root.secret(\"API_KEY\")\n {body}\n true }}\n"
        );
        let checked = check_source(0, &src);
        assert!(!checked.has_errors(), "{body}: {:?}", checked.diagnostics);
        compile_checked(&checked.module, &checked.result).map(|_| ()).map_err(|e| e.code())
    };
    assert_eq!(compile("let keys = [key, key]\n let n = keys.len()"), Ok(()), "a list of secrets is a list of handles");
    assert_eq!(compile("let k = Some(key)"), Ok(()), "an option of a secret is a handle's cell");
    assert_eq!(compile("var held = None\n held = Some(key)"), Ok(()), "and one whose type only the checker names");
    assert_eq!(
        compile("let keys = [key]\n let d = root.declassify()\n let s = keys[0].expose(d)"),
        Err("DL1205"),
        "expose reveals the bytes, from a list as from a variable"
    );
}
