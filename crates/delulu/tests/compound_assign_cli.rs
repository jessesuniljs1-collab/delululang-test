//! **RW 2.34: compound assignment** — `x += e`, `-=`, `*=`, `/=` and `%=`, on a local, a record's field and a list's
//! index.
//!
//! The parser rewrites `t op= e` into `t = t op e` (`parser.rs` `compound_assignment`), so the checker and every walk of
//! an assignment see an ordinary one. What the rewrite alone got wrong, and these tests hold: the target is written
//! twice, as the read and as the place, and the right side runs between them. A right side that moves a name the target
//! reads — `bump(xs)` setting `xs[0]` in `ys[xs[0]] += bump(xs)`, a closure assigning `i` in `ys[i] += f()`, a function
//! giving a record a new field value in `h.leaf.n += swap(h)` — made the read and the write two different places. Both
//! engines now find the place before the right side runs (`Stmt::Assign::compound`); a plain `=` still runs its value
//! first. These drive the real binary on the interpreter and, where the WASM fragment reaches, on `--engine wasm`.

use std::path::PathBuf;
use std::process::{Command, Output};

/// Write `src` to a directory of its own and run `delulu ARGS… FILE`.
fn delulu(src: &str, args: &[&str]) -> Output {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir: PathBuf = std::env::temp_dir().join(format!("delulu-compound-{}-{n}", std::process::id()));
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

/// Run `src` on the interpreter, or on WASM with `wasm`, and return what it printed; the run must succeed.
fn run(src: &str, wasm: bool) -> String {
    let args: &[&str] = if wasm { &["run", "--engine", "wasm", "--grant", "console"] } else { &["run", "--grant", "console"] };
    let o = delulu(src, args);
    assert!(o.status.success(), "wasm={wasm}: {}", text(&o));
    String::from_utf8_lossy(&o.stdout).replace("\r\n", "\n")
}

/// Every operator on a local `Int`, a `Float`, a `Str` and a record's field: the whole right side is the right operand
/// (`x *= 2 + 3` multiplies by 5), division and remainder truncate as `/` and `%` do, and both engines print the same.
const OPERATORS: &str = r#"module ops

type Acc {
  n: Int,
  label: Str,
}

fn main(root: Root) ! {Write} {
  let out = root.console()
  var x = 2
  x *= 2 + 3
  out.println(str(x))
  x -= 10 - 4
  out.println(str(x))
  x += 100
  x /= 7
  out.println(str(x))
  x %= 4
  out.println(str(x))
  var neg = -7
  neg /= 2
  var rem = -7
  rem %= 3
  out.println(str(neg) + " " + str(rem))
  var f = 1.5
  f *= 4.0
  f -= 0.5
  out.println(str(f))
  var s = "ab"
  s += "cd"
  s += str(x)
  out.println(s)
  let a = Acc { n: 1, label: "a" }
  a.n += 41
  a.label += "b"
  a.n *= 2
  out.println(str(a.n) + " " + a.label)
}
"#;

#[test]
fn every_operator_updates_its_target_on_both_engines() {
    let want = "10\n4\n14\n2\n-3 -1\n5.5\nabcd2\n84 ab\n";
    assert_eq!(run(OPERATORS, false), want);
    assert_eq!(run(OPERATORS, true), want, "the WASM engine agrees");
}

/// `ys[xs[0]] += bump(xs)`: `bump` sets `xs[0]` to 5 and returns 1. The slot read is `ys[0]`, and so is the slot written
/// — `11 60`, as Python answers. With the place found after the right side, `ys[0]` was read and `ys[5]` written: `10 11`.
const MOVED_INDEX: &str = r#"module moved

fn bump(xs: ref List[Int]) -> Int {
  xs[0] = 5
  1
}

fn main(root: Root) ! {Write} {
  var xs = [0, 0, 0, 0, 0, 0]
  var ys = [10, 20, 30, 40, 50, 60]
  ys[xs[0]] += bump(xs)
  root.console().println(str(ys[0]) + " " + str(ys[5]))
}
"#;

/// `ys[i] += f()`: the closure assigns `i = 3` and returns 1. The slot is the one `i` named when the statement began.
const MOVED_NAME: &str = r#"module moved

fn main(root: Root) ! {Write} {
  var i = 0
  let f = fn() -> Int {
    i = 3
    1
  }
  var ys = [10, 20, 30, 40]
  ys[i] += f()
  root.console().println(str(ys[0]) + " " + str(ys[3]) + " " + str(i))
}
"#;

/// `h.leaf.n += swap(h)`: `swap` gives `h` a new leaf. The leaf written is the one read, `first` — on both engines.
const MOVED_RECORD: &str = r#"module moved

type Leaf {
  n: Int,
}

type Holder {
  leaf: ref Leaf,
}

fn swap(h: ref Holder) -> Int {
  h.leaf = Leaf { n: 100 }
  1
}

fn main(root: Root) ! {Write} {
  let first = Leaf { n: 5 }
  var h = Holder { leaf: first }
  h.leaf.n += swap(h)
  root.console().println(str(first.n) + " " + str(h.leaf.n))
}
"#;

#[test]
fn the_place_is_found_before_the_right_side_runs() {
    assert_eq!(run(MOVED_INDEX, false), "11 60\n", "the slot read is the slot written");
    assert_eq!(run(MOVED_NAME, false), "11 40 3\n", "the slot `i` named when the statement began");
    assert_eq!(run(MOVED_RECORD, false), "6 100\n", "the leaf read is the leaf written");
    assert_eq!(run(MOVED_RECORD, true), "6 100\n", "the WASM engine agrees");
    // The control: a plain `=` runs its value first and then finds its place, as it always has — written out long, the
    // same statement reads `ys[0]` and writes `ys[5]`, and the record's new leaf.
    let long = MOVED_INDEX.replace("ys[xs[0]] += bump(xs)", "ys[xs[0]] = ys[xs[0]] + bump(xs)");
    assert_eq!(run(&long, false), "10 11\n");
    let long = MOVED_RECORD.replace("h.leaf.n += swap(h)", "h.leaf.n = h.leaf.n + swap(h)");
    assert_eq!(run(&long, false), "5 6\n");
    assert_eq!(run(&long, true), "5 6\n");
}

/// An index holding a call would run the call twice; it is refused before anything runs, in words, with the fix. The
/// control, the index bound first, runs.
#[test]
fn a_target_holding_a_call_is_refused_in_words() {
    let src = r#"module calls

fn main(root: Root) ! {Write} {
  var xs = [1, 2, 3]
  xs[xs.len() - 1] += 10
  root.console().println(str(xs))
}
"#;
    let o = delulu(src, &["run", "--grant", "console"]);
    let t = text(&o);
    assert!(!o.status.success(), "{t}");
    assert!(t.contains("DL0207") && t.contains("bind the index first"), "{t}");
    assert!(o.stdout.is_empty(), "nothing ran: {t}");
    let bound = src.replace("xs[xs.len() - 1] += 10", "let last = xs.len() - 1\n  xs[last] += 10");
    assert_eq!(run(&bound, false), "[1, 2, 13]\n");
}

/// A compound assignment means its long form: where the long form is refused, so is the short one, with the same code;
/// where the long form faults, so does the short one, with the same code, on both engines.
#[test]
fn a_compound_assignment_is_refused_and_faults_as_its_long_form_does() {
    let wrap = |body: &str| {
        format!("module same\n\nfn main(root: Root) ! {{Write}} {{\n  {body}\n  root.console().println(\"ran\")\n}}\n")
    };
    let codes = |o: &Output| {
        let t = text(o);
        let mut v: Vec<String> = t
            .match_indices("DL")
            .filter_map(|(i, _)| t.get(i..i + 6))
            .filter(|c| c[2..].chars().all(|ch| ch.is_ascii_digit()))
            .map(str::to_string)
            .collect();
        v.sort();
        v.dedup();
        v
    };
    for (short, long, code) in [
        ("var s = \"a\"\n  s += 1", "var s = \"a\"\n  s = s + 1", "DL0401"),
        ("var b = true\n  b -= 1", "var b = true\n  b = b - 1", "DL0401"),
        ("var x = 9223372036854775807\n  x += 1", "var x = 9223372036854775807\n  x = x + 1", "DL0901"),
        ("var x = 7\n  x /= 0", "var x = 7\n  x = x / 0", "DL0902"),
        ("var x = 7\n  x %= 0", "var x = 7\n  x = x % 0", "DL0902"),
    ] {
        for engine in [&["run", "--grant", "console"][..], &["run", "--engine", "wasm", "--grant", "console"][..]] {
            let (s, l) = (delulu(&wrap(short), engine), delulu(&wrap(long), engine));
            assert!(!s.status.success() && !l.status.success(), "{short:?} on {engine:?}: {}{}", text(&s), text(&l));
            assert!(codes(&s).contains(&code.to_string()), "{short:?} on {engine:?}: {}", text(&s));
            assert_eq!(codes(&s), codes(&l), "{short:?} on {engine:?}: {} / {}", text(&s), text(&l));
            assert!(!text(&s).contains("ran"), "{short:?} stopped before the next line");
        }
    }
}

/// `delulu fmt` prints a compound assignment as written, and `--check` accepts what it printed.
#[test]
fn fmt_prints_a_compound_assignment_as_written() {
    let src = "module f\n\nfn main(root: Root) ! {Write} {\n    var total = 0\n    total+=2\n    total  -=1\n    root.console().println(str(total))\n}\n";
    let dir = std::env::temp_dir().join(format!("delulu-compound-fmt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("f.delulu");
    std::fs::write(&file, src).unwrap();
    let bin = env!("CARGO_BIN_EXE_delulu");
    let o = Command::new(bin).arg("fmt").arg(&file).env("DELULU_NO_FIRST_RUN", "1").output().unwrap();
    assert!(o.status.success(), "{}", text(&o));
    let printed = std::fs::read_to_string(&file).unwrap().replace("\r\n", "\n");
    assert!(printed.contains("    total += 2\n    total -= 1\n"), "{printed}\n{}", text(&o));
    let o = Command::new(bin).args(["fmt", "--check"]).arg(&file).env("DELULU_NO_FIRST_RUN", "1").output().unwrap();
    assert!(o.status.success(), "{}", text(&o));
    let _ = std::fs::remove_dir_all(&dir);
}
