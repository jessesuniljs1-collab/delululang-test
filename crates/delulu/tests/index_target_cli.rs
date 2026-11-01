//! **INDEX-TARGET-1: what an assignment's index holds is checked, asked for and judged as any other expression** —
//! found 2026-11-02 while writing DL0414 (a binding never read), whose first draft named `i` in `xs[i] = v` unread.
//!
//! `xs[e] = v` evaluates `e` when it runs (the interpreter's `assign`), but every walk of an assignment read only `v`.
//! The checker never checked `e`: an effect there joined no row, so a function `delulu check` passed and `delulu
//! authority` listed as pure printed; a name there was never resolved; a `Str` index passed; and a base that is no list
//! got a fresh type. The reference-capability pass never walked `e`, so a `consume` there left its binding alive and a
//! use after it — the unique reference already sent to an actor — was accepted. `authority --grants` and `--within`
//! never saw a mint, a file use or a request there (`--within` answered *within* for a write it answers *exceeds* for
//! one line up); `authority` and the atlas never saw an embedded-Python import there; and a plugin `load` there was
//! never listed. The capability check held throughout: a mint the grants did not cover still refused when it ran.
//!
//! `LValue::index_exprs` names the indices in the order they run, and each walk of `Stmt::Assign` walks them. These
//! drive the real binary; each program writes its effect in the index alone, and where a verdict is compared, the same
//! line written one statement up is the control.

use std::path::PathBuf;
use std::process::{Command, Output};

/// Write `src` to a directory of its own and run `delulu ARGS… FILE`.
fn delulu(src: &str, args: &[&str]) -> Output {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir: PathBuf = std::env::temp_dir().join(format!("delulu-index-target-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("p.delulu");
    std::fs::write(&file, src).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .arg(&file)
        .current_dir(&dir)
        .env("DELULU_STATE_DIR", dir.join("state"))
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("run the delulu binary");
    let _ = std::fs::remove_dir_all(&dir);
    out
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

/// `quiet` declares no row and prints through the index it writes. Before, `check` passed it, `authority` listed it
/// under *pure fns*, and the run printed.
const EFFECT: &str = r#"module ix

fn noisy(c: Cap[Console]) -> Int ! {Write} {
    c.println("an effect inside the index")
    0
}

fn quiet(c: Cap[Console]) -> Int {
    var xs = [1, 2, 3]
    xs[noisy(c)] = 9
    xs[0]
}

fn main(root: Root) ! {Write} {
    root.console().println(str(quiet(root.console())))
}
"#;

#[test]
fn an_effect_written_in_an_index_target_joins_the_row() {
    let o = delulu(EFFECT, &["check"]);
    let t = text(&o);
    assert!(!o.status.success(), "a pure function that prints through an index is refused: {t}");
    assert!(t.contains("DL0501") && t.contains("function `quiet` performs effect `Write`"), "{t}");

    // The control: declared, it checks clean and runs — the index is evaluated, and its line printed first.
    let declared = EFFECT.replace("fn quiet(c: Cap[Console]) -> Int {", "fn quiet(c: Cap[Console]) -> Int ! {Write} {");
    let o = delulu(&declared, &["check"]);
    assert!(o.status.success(), "{}", text(&o));
    let o = delulu(&declared, &["run", "--grant", "console"]);
    assert!(o.status.success(), "{}", text(&o));
    assert_eq!(String::from_utf8_lossy(&o.stdout).replace("\r\n", "\n"), "an effect inside the index\n9\n");
}

/// An index is checked as `xs[i]` is read: its names resolve, it is an `Int`, and the base is a list.
#[test]
fn an_index_target_is_checked_as_a_read_index_is() {
    let case = |stmt: &str| -> String {
        let src = format!(
            "module ix\n\nfn main(root: Root) ! {{Write}} {{\n    var xs = [1, 2, 3]\n    var n = 5\n    {stmt}\n    \
             root.console().println(str(xs[0]) + str(n))\n}}\n"
        );
        let o = delulu(&src, &["check"]);
        assert!(!o.status.success(), "`{stmt}` is refused: {}", text(&o));
        text(&o)
    };
    let unknown = case("xs[no_such_name] = 9");
    assert!(unknown.contains("DL0301") && unknown.contains("unknown name `no_such_name`"), "{unknown}");
    let not_int = case("xs[\"two\"] = 9");
    assert!(not_int.contains("index must be Int"), "{not_int}");
    let not_list = case("n[0] = 9");
    assert!(not_list.contains("DL0405") && not_list.contains("cannot index a value of type `Int`"), "{not_list}");
}

/// The unique list is sent to an actor inside the index; reading it after is a use of a dead binding (DL1602), as it
/// is when the same call is bound one line up. Before, the index was never walked and the read was accepted.
const CONSUME: &str = r#"module ix

actor Sink {
  var n: Int
  new() { self.n = 0 }
  be take(vs: iso List[Int]) ! {} { self.n = vs.len() }
}

fn give(s: Sink, vs: iso List[Int]) -> Int ! {Async} {
  s.take(consume vs)
  0
}

fn main(root: Root) ! {Async} {
  let s = spawn Sink()
  let xs: iso List[Int] = recover { [1] }
  var ys = [0, 0]
  ys[give(s, consume xs)] = 1
  let n = xs.len()
}
"#;

#[test]
fn a_consume_written_in_an_index_target_ends_its_binding() {
    let o = delulu(CONSUME, &["check"]);
    let t = text(&o);
    assert!(!o.status.success() && t.contains("DL1602") && t.contains("use of `xs` after `consume`"), "{t}");
    // The control: with no use after it, the consume is legal where it stands.
    let o = delulu(&CONSUME.replace("  let n = xs.len()\n", ""), &["check"]);
    assert!(!text(&o).contains("DL1602"), "{}", text(&o));
    // In a loop, the second turn consumes a binding the first already sent: the loop's pre-scan sees the index too.
    let looped = CONSUME.replace(
        "  ys[give(s, consume xs)] = 1\n  let n = xs.len()\n",
        "  var i = 0\n  while i < 2 {\n    ys[give(s, consume xs)] = 1\n    i = i + 1\n  }\n",
    );
    let o = delulu(&looped, &["check"]);
    let t = text(&o);
    assert!(!o.status.success() && t.contains("DL1602"), "a consume repeated by a loop is refused: {t}");
}

/// A capability minted in the index is asked for: `--grants` names it, and a boundary without it is exceeded.
#[test]
fn a_capability_minted_in_an_index_target_is_required() {
    let src = r#"module ix

fn wipe(w: Cap[FsWrite]) -> Int ! {Write} {
    match w.write_text("planted.txt", "outside the boundary") {
        Ok(u) => 0
        Err(e) => 1
    }
}

fn main(root: Root) ! {Write} {
    var xs = [1, 2, 3]
    xs[wipe(root.fs_write("./ix-out"))] = 9
    root.console().println(str(xs[0]))
}
"#;
    let o = delulu(src, &["authority", "--grants"]);
    assert!(text(&o).contains("--grant fs.write=./ix-out"), "{}", text(&o));
    let o = delulu(src, &["authority", "--within", "console"]);
    let t = text(&o);
    assert!(t.starts_with("exceeds:") && t.contains("fs.write=./ix-out"), "{t}");
}

/// A file use in the index is judged as the same use one line up: the path escapes the capability's scope, so the
/// verdict is *exceeds* with the same reason. Before, the index's verdict was *within*.
#[test]
fn a_file_use_in_an_index_target_is_judged_as_one_line_up() {
    let in_index = r#"module ix

fn main(root: Root) ! {Write} {
    let w = root.fs_write("./out")
    var xs = [1, 2, 3]
    xs[match w.write_text("../escape.txt", "x") {
        Ok(u) => 0
        Err(e) => 1
    }] = 9
    root.console().println(str(xs[0]))
}
"#;
    let one_line_up = r#"module ix

fn main(root: Root) ! {Write} {
    let w = root.fs_write("./out")
    var xs = [1, 2, 3]
    let i = match w.write_text("../escape.txt", "x") {
        Ok(u) => 0
        Err(e) => 1
    }
    xs[i] = 9
    root.console().println(str(xs[0]))
}
"#;
    let reason = "the write of `../escape.txt` through the capability for `./out` is refused at its call";
    for src in [one_line_up, in_index] {
        let o = delulu(src, &["authority", "--within", "console", "--within", "fs.write=./out"]);
        let t = text(&o);
        assert!(t.starts_with("exceeds:") && t.contains(reason) && t.contains("DL0904"), "{t}");
    }
}

/// Embedded Python imported in the index is code outside the guarantee: the import is seen, its grant named, the atlas
/// draws its edge, and the row that declares `ForeignCall` is not called needless. Before, all four said otherwise.
#[test]
fn foreign_code_written_in_an_index_target_is_outside_the_proof() {
    let src = r#"module ix

fn reach(py: Cap[Python]) -> Int ! {ForeignCall} {
    var xs = [1, 2, 3]
    xs[match py.import("socket") {
        Ok(m) => 0
        Err(e) => 1
    }] = 9
    xs[0]
}

fn main(root: Root) ! {ForeignCall, Write} {
    match root.python(root.foreign_load()) {
        Ok(py) => root.console().println(str(reach(py)))
        Err(_) => root.console().println("no python")
    }
}
"#;
    let o = delulu(src, &["check"]);
    assert!(o.status.success() && !text(&o).contains("DL0502"), "{}", text(&o));
    let o = delulu(src, &["authority"]);
    assert!(text(&o).contains("imports seen: [socket]"), "{}", text(&o));
    let o = delulu(src, &["authority", "--grants"]);
    assert!(text(&o).contains("--grant foreign.python=socket"), "{}", text(&o));
    let o = delulu(src, &["atlas", "--format", "tree"]);
    assert!(text(&o).contains("foreign:py:socket"), "{}", text(&o));
}

/// A plugin loaded in the index is listed with the plugins the program loads.
#[test]
fn a_plugin_loaded_in_an_index_target_is_listed() {
    let src = r#"module ix

fn loads(h: Cap[PluginHost]) -> Int ! {Load, Read} {
    var xs = [1, 2, 3]
    xs[match load(h, "hidden.dpx", Grant {
        effects: [], fs_read: [], fs_write: [], net: [], secrets: [], declassify: [],
        limits: Limits { fuel: 0, mem_mb: 0, wall_ms: 0 },
        require_signed: false,
    }) {
        Ok(p) => 0
        Err(e) => 1
    }] = 9
    xs[0]
}

fn main(root: Root) ! {Write, Load, Read} {
    root.console().println(str(loads(root.plugin_host())))
}
"#;
    let o = delulu(src, &["authority"]);
    let t = text(&o);
    assert!(t.contains("plugins:") && t.contains("hidden.dpx"), "{t}");
}
