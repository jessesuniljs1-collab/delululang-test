//! **RW 2.36: ranges** (D-V2-224) — `lo..hi` (up to but not including `hi`) and `lo..=hi` (up to and including it), the
//! prelude's `range(lo, hi)` written as operators: a `List[Int]`, counted without a list by a `for`, on both engines.
//!
//! What these hold, through the real binary: the refusals say how a range is written (a `Float` bound, a third
//! argument, a chained `a..b..c`, `...`); the free builtins' argument counts and types are read at last (`range(1.5,
//! "x")` was an empty list, `len(m)` of a `Map` was 0, `str(1, 2)` printed `1`); `delulu fmt` prints a range close and
//! keeps exactly the parentheses its level needs; and a program runs the same on the interpreter and `--engine wasm`.

use std::path::PathBuf;
use std::process::{Command, Output};

/// Write `src` to a directory of its own and run `delulu ARGS… FILE`.
fn delulu(src: &str, args: &[&str]) -> Output {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir: PathBuf = std::env::temp_dir().join(format!("delulu-ranges-{}-{n}", std::process::id()));
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

/// A `main` with `body` in it, the console bound as `c`.
fn main_with(body: &str) -> String {
    format!("module p\n\nfn main(root: Root) ! {{Write}} {{\n    let c = root.console()\n{body}\n}}\n")
}

fn run(src: &str, wasm: bool) -> String {
    let args: &[&str] = if wasm { &["run", "--engine", "wasm", "--grant", "console"] } else { &["run", "--grant", "console"] };
    let o = delulu(src, args);
    assert!(o.status.success(), "wasm={wasm}: {}", text(&o));
    String::from_utf8_lossy(&o.stdout).replace("\r\n", "\n")
}

/// `delulu check` refuses the program with `code`, and its words hold every one of `needles`.
fn refused(src: &str, code: &str, needles: &[&str]) {
    let o = delulu(src, &["check"]);
    let t = text(&o);
    assert!(!o.status.success(), "should be refused with {code}:\n{src}\n{t}");
    assert!(t.contains(&format!("error[{code}]")), "expected {code}:\n{src}\n{t}");
    for n in needles {
        assert!(t.contains(n), "expected {n:?} in:\n{t}");
    }
}

/// The everyday forms, on both engines, printing the same — and the same as Python's `range`.
#[test]
fn ranges_run_the_same_on_both_engines() {
    let src = main_with(
        r#"    for i in 0..3 {
        c.print(str(i) + " ")
    }
    for i in 1..=3 {
        c.print(str(i * 10) + " ")
    }
    c.println("")
    let n = 5
    let squares = (1..=n).map(fn(i: Int) -> Int { i * i })
    c.println(str(squares) + " " + str(len(squares)))
    c.println(str((0..n).reverse()) + " " + str((0..=20).step_by(5)))
    c.println(str(range(2, 4)) + " " + str(4..2) + " " + str(0..n + 1 == range(0, 6)))"#,
    );
    let want = "0 1 2 10 20 30 \n[1, 4, 9, 16, 25] 5\n[4, 3, 2, 1, 0] [0, 5, 10, 15, 20]\n[2, 3] [] true\n";
    assert_eq!(run(&src, false), want);
    assert_eq!(run(&src, true), want);
}

/// A range too long to hold as a list is DL0908 on both engines, in the same words, where the interpreter aborted
/// (`memory allocation of 40000000000000 bytes failed`); iterated by `for`, it builds nothing and runs.
#[test]
fn a_range_too_long_for_a_list_is_dl0908_and_iterates_in_a_for() {
    let src = main_with("    let xs = 0..1000000000000\n    c.println(str(xs.len()))");
    for wasm in [false, true] {
        let args: &[&str] =
            if wasm { &["run", "--engine", "wasm", "--grant", "console"] } else { &["run", "--grant", "console"] };
        let o = delulu(&src, args);
        let t = text(&o);
        assert!(!o.status.success(), "wasm={wasm}: {t}");
        assert!(t.contains("DL0908") && t.contains("names 1000000000000 values, more than a list holds"), "wasm={wasm}: {t}");
        assert!(!t.contains("memory allocation") && !t.contains("panicked"), "wasm={wasm}: {t}");
    }
    let looped = main_with(
        "    var seen = 0\n    for i in 0..1000000000000 {\n        if i == 3 {\n            break\n        }\n        seen += 1\n    }\n    c.println(str(seen))",
    );
    assert_eq!(run(&looped, false), "3\n");
    assert_eq!(run(&looped, true), "3\n");
}

/// Each refusal names the mistake and how a range is written.
#[test]
fn a_misspelled_range_is_refused_with_its_spelling() {
    refused(&main_with("    let xs = 1.5..3"), "DL0401", &["a range counts in `Int`s", "`lo..hi`", "float(i) * 0.5"]);
    refused(&main_with("    let xs = 0..\"a\""), "DL0401", &["a bound of `lo..hi` is an `Int`"]);
    refused(&main_with("    let xs = 0..=2.0"), "DL0401", &["`lo..=hi`"]);
    refused(&main_with("    let xs = range(1.5, \"x\")"), "DL0401", &["`range(lo, hi)`"]);
    refused(&main_with("    let xs = range(1)"), "DL0403", &["`range` expects 2 argument(s), found 1", "`lo..hi`"]);
    refused(&main_with("    let xs = range(0, 10, 2)"), "DL0403", &["a range has no step", "`(lo..hi).step_by(k)`"]);
    refused(&main_with("    let xs = 0..3..5"), "DL0201", &["a range has one `..`"]);
    refused(&main_with("    let xs = 0...3"), "DL0101", &["`lo..hi`", "`lo..=hi`"]);
    refused(&main_with("    let xs = (0..3).step_by(1.5)"), "DL0401", &[]);
}

/// RW 2.36: the free builtins read their argument count and types. Each of these checked clean and ran — `str(1, 2)`
/// printed `1`, `Some()` was `Some(())`, `len(5)` was 0 — and each is refused now; the right count and type still check.
#[test]
fn every_free_builtin_reads_its_arguments() {
    for (expr, code, needle) in [
        ("str()", "DL0403", "`str` expects 1 argument(s), found 0"),
        ("str(1, 2)", "DL0403", "`str` expects 1 argument(s), found 2"),
        ("len()", "DL0403", "`len` expects 1 argument(s)"),
        ("len(\"ab\", \"c\")", "DL0403", "found 2"),
        ("len(5)", "DL0401", "`len` counts a `Str`, a `List` or a `Map`, and this is `Int`"),
        ("int()", "DL0403", "`int` expects 1 argument(s)"),
        ("int(1.5, 2.5)", "DL0403", "found 2"),
        ("float()", "DL0403", "`float` expects 1 argument(s)"),
        ("parse_int()", "DL0403", "`parse_int` expects 1 argument(s)"),
        ("parse_int(5)", "DL0401", "parse_int() reads a Str"),
        ("parse_float(1)", "DL0401", "parse_float() reads a Str"),
        ("Some()", "DL0403", "`Some` expects 1 argument(s), found 0"),
        ("Some(1, 2)", "DL0403", "found 2"),
        ("Ok(1, 2)", "DL0403", "`Ok` expects 1 argument(s)"),
        ("Err()", "DL0403", "`Err` expects 1 argument(s)"),
    ] {
        refused(&main_with(&format!("    let v = {expr}\n    c.println(str(v))")), code, &[needle]);
    }
    refused(&main_with("    push(5, 3)"), "DL0401", &["`push(list, value)` pushes onto a `List`, and this is `Int`"]);
    refused(&main_with("    let xs: ref List[Int] = [1]\n    push(xs)"), "DL0403", &["`push` expects 2 argument(s), found 1"]);
    // The control: the right counts and types check, and `len` of a `Map` counts its pairs on both engines.
    let ok = main_with(
        "    let m = Map()\n    m.insert(\"a\", 1)\n    m.insert(\"b\", 2)\n    let xs: ref List[Int] = [1]\n    push(xs, 2)\n    \
         c.println(str(len(m)) + str(len(\"héllo\")) + str(len(xs)) + str(Some(1)) + str(parse_int(\"7\")) + str(int(2.5)) + str(float(2)))",
    );
    assert_eq!(run(&ok, false), "252Some(1)Some(7)22.0\n");
    assert_eq!(run(&ok, true), "252Some(1)Some(7)22.0\n");
}

/// `delulu fmt` prints a range close, `0..n` and `1..=n + 1`, keeps the parentheses a method's receiver and a range
/// operand need and no others, and `--check` accepts what it printed.
#[test]
fn fmt_prints_a_range_as_written_and_is_stable() {
    let src = "module p\n\nfn f(n: Int) -> List[Int] {\n    for i in 0 .. n {\n        n + i\n    }\n    let a = 1 ..= n + 1\n    let b = (0..3).reverse()\n    let d = (0..n).concat(n..=(n + 2))\n    a.concat(b).concat(d).concat((0 .. 9).step_by(3))\n}\n";
    let bin = env!("CARGO_BIN_EXE_delulu");
    let dir = std::env::temp_dir().join(format!("delulu-ranges-fmt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("p.delulu");
    std::fs::write(&file, src).unwrap();
    let o = Command::new(bin).arg("fmt").arg(&file).env("DELULU_NO_FIRST_RUN", "1").output().unwrap();
    assert!(o.status.success(), "{}", text(&o));
    let out = std::fs::read_to_string(&file).unwrap();
    for line in [
        "    for i in 0..n {\n",
        "    let a = 1..=n + 1\n",
        "    let b = (0..3).reverse()\n",
        "    let d = (0..n).concat(n..=n + 2)\n",
        "    a.concat(b).concat(d).concat((0..9).step_by(3))\n",
    ] {
        assert!(out.contains(line), "expected {line:?} in:\n{out}");
    }
    let o = Command::new(bin).args(["fmt", "--check"]).arg(&file).env("DELULU_NO_FIRST_RUN", "1").output().unwrap();
    assert!(o.status.success(), "fmt --check refuses what fmt printed: {}", text(&o));
    let _ = std::fs::remove_dir_all(&dir);
}
