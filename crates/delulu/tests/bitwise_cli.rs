//! **RW 2.35: bitwise operators** (D-V2-226) — `&`, `|`, `^`, `~`, `<<` and `>>` on an `Int`'s 64 bits, their compound
//! forms and the `0b` and `0o` literals, through the real binary.
//!
//! What these hold: a program prints the same on the interpreter and on `--engine wasm`, and what Python prints; a
//! shift count outside 0 to 63 is DL0908 on both engines, after the lines printed before it; each refusal names the
//! operator the writer meant, once, with no second error after it (two `Bool`s under `&`, `|` or `^`; a `Float`; `!`
//! on an `Int`; `~` on a `Bool`; a line beginning with `|`; a digit the base lacks; a literal past `Int`'s largest; a
//! call inside a compound target); a function that only computes bits is pure; and `delulu fmt` keeps each literal's
//! base and shows the grouping wherever a bitwise operator stands beside a different one.

use std::path::PathBuf;
use std::process::{Command, Output};

/// Write `src` to a directory of its own and run `delulu ARGS… FILE`.
fn delulu(src: &str, args: &[&str]) -> Output {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir: PathBuf = std::env::temp_dir().join(format!("delulu-bitwise-{}-{n}", std::process::id()));
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

/// `delulu fmt` on `src` in a directory of its own: the file's text afterwards.
fn formatted(src: &str) -> String {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir: PathBuf = std::env::temp_dir().join(format!("delulu-bitwise-fmt-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("p.delulu");
    std::fs::write(&file, src).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .arg("fmt")
        .arg(&file)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("run the delulu binary");
    assert!(o.status.success(), "{}", text(&o));
    let out = std::fs::read_to_string(&file).unwrap().replace("\r\n", "\n");
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

/// A function of an `Int`, a `Bool` and a `Float` whose body is `body`.
fn fn_with(body: &str) -> String {
    format!("module p\n\nfn f(a: Int, b: Bool, x: Float) -> Int {{\n{body}\n}}\n")
}

fn run(src: &str, wasm: bool) -> String {
    let args: &[&str] = if wasm { &["run", "--engine", "wasm", "--grant", "console"] } else { &["run", "--grant", "console"] };
    let o = delulu(src, args);
    assert!(o.status.success(), "wasm={wasm}: {}", text(&o));
    String::from_utf8_lossy(&o.stdout).replace("\r\n", "\n")
}

/// `delulu check` refuses the program with `code`, once, and its words hold every one of `needles`.
fn refused(src: &str, code: &str, needles: &[&str]) {
    let o = delulu(src, &["check"]);
    let t = text(&o);
    assert!(!o.status.success(), "should be refused with {code}:\n{src}\n{t}");
    assert_eq!(t.matches("error[").count(), 1, "one refusal, no cascade:\n{src}\n{t}");
    assert!(t.contains(&format!("error[{code}]")), "expected {code}:\n{src}\n{t}");
    for n in needles {
        assert!(t.contains(n), "expected {n:?} in:\n{t}");
    }
}

/// Every operator, the compound forms and the three bases, on both engines, printing what Python prints for the same
/// program (`~` is Python's `~`; `1 << 63` is taken to 64 bits, Python's `-2**63`).
#[test]
fn every_operator_prints_the_same_on_both_engines_and_as_python() {
    let src = main_with(
        r#"    var flags = 0
    flags |= 1 << 3
    flags |= 0b101
    flags &= ~0b1
    flags ^= 0o20
    c.println(f"{flags} {6 & 3} {6 | 3} {6 ^ 3} {~5} {1 << 4} {256 >> 4} {-8 >> 1}")
    c.println(f"{1 << 63} {0xFF00 >> 8} {0xdead_BEEF & 0xFFFF} {(0x12 << 8) | 0x34}")
    c.println(str((flags & 0b100) != 0))"#,
    );
    // Python: flags = ((0 | 8 | 5) & ~1) ^ 16 == 28; 0xdead_beef & 0xffff == 48879; (0x12 << 8) | 0x34 == 4660.
    let want = "28 2 7 5 -6 16 16 -4\n-9223372036854775808 255 48879 4660\ntrue\n";
    assert_eq!(run(&src, false), want);
    assert_eq!(run(&src, true), want);
}

/// A count outside 0 to 63 stops the program with DL0908 on both engines, exit 1, the line printed before it kept.
#[test]
fn a_shift_count_outside_0_to_63_is_dl0908_on_both_engines_after_what_was_printed() {
    let src = main_with("    c.println(\"before\")\n    let n = 64\n    c.println(str(1 << n))\n    c.println(\"after\")");
    for args in [&["run", "--grant", "console"][..], &["run", "--engine", "wasm", "--grant", "console"][..]] {
        let o = delulu(&src, args);
        let t = text(&o);
        assert_eq!(o.status.code(), Some(1), "{args:?}: {t}");
        assert!(String::from_utf8_lossy(&o.stdout).starts_with("before"), "{args:?}: {t}");
        assert!(!t.contains("after"), "{args:?}: {t}");
        assert!(t.contains("DL0908"), "{args:?}: {t}");
        assert!(t.contains("`<<` moves an `Int`'s 64 bits by a count from 0 to 63; it was given a count of 64"), "{args:?}: {t}");
    }
}

/// Each refusal names the operator or the spelling the writer meant, once.
#[test]
fn each_refusal_names_what_the_writer_meant_once() {
    // Two `Bool`s: other languages allow `&`, `|` and `^` there. One refusal, not a second for the `if` condition.
    refused(&fn_with("    if b & b { 1 } else { 0 }"), "DL0401", &["both sides are `Bool`", "write `&&`"]);
    refused(&fn_with("    if b | true { 1 } else { 0 }"), "DL0401", &["write `||`"]);
    refused(&fn_with("    if b ^ b { 1 } else { 0 }"), "DL0401", &["write `!=`"]);
    // A `Float` has no bits to work on.
    refused(&fn_with("    a & x"), "DL0401", &["this operand is a `Float`", "`int(x)`"]);
    refused(&fn_with("    a << 2.0"), "DL0401", &["`<<` works on the bits of an `Int`", "a `Float`"]);
    // `!` and `~` swapped.
    refused(&fn_with("    if !a { 1 } else { 0 }"), "DL0401", &["`!` negates a `Bool`", "write `~`"]);
    // Whichever type its place wants: here an `Int` function's result, where a `Bool` would draw a second refusal.
    refused(&fn_with("    !a"), "DL0401", &["write `~`"]);
    refused(&fn_with("    if true { ~b } else { 0 }"), "DL0401", &["write `!`"]);
    refused(&fn_with("    if ~b { 1 } else { 0 }"), "DL0401", &["`~` flips the bits of an `Int`", "write `!`"]);
    // A line that begins with the operator: the expression ended at the line break.
    refused(&fn_with("    a\n        | 4"), "DL0202", &["end the line above with `|`"]);
    // Literals: a digit the base lacks, and a bit pattern past `Int`'s largest.
    refused(&fn_with("    0b102"), "DL0104", &["`2` is not a binary digit, in the literal `0b102`"]);
    refused(&fn_with("    0o8"), "DL0104", &["`8` is not an octal digit"]);
    refused(&fn_with("    0xFFFF_FFFF_FFFF_FFFF"), "DL0104", &["write `1 << 63`", "`~0` is every bit"]);
    // A compound target holding a call would run the call twice (RW 2.34's rule, here for `|=`).
    refused(
        "module p\n\nfn k() -> Int {\n    0\n}\n\nfn f() -> Int {\n    var xs = [1, 2]\n    xs[k()] |= 4\n    xs[0]\n}\n",
        "DL0207",
        &["a compound assignment (`|=`)", "bind the index first"],
    );
}

/// Bits are data: a function that only computes them declares no effect and is counted pure.
#[test]
fn a_function_that_only_computes_bits_is_pure() {
    let src = "module p\n\nfn low_byte(n: Int) -> Int {\n    n & 0xFF\n}\n\n\
               fn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(str(low_byte(0x1234) << 1))\n}\n";
    let o = delulu(src, &["authority", "--json"]);
    assert!(o.status.success(), "{}", text(&o));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("JSON");
    assert_eq!(v["authority"]["pure_functions"], serde_json::json!(["low_byte"]), "{v}");
}

/// `delulu fmt` keeps each literal as written — its base, its digits, its grouping, the base's letter lowered — and
/// shows the grouping wherever a bitwise operator stands beside a different binary operator.
#[test]
fn fmt_keeps_each_base_and_shows_the_grouping_of_mixed_operators() {
    let src = fn_with(
        "    let m = 0XFF00|0B1010_1010|0O17|0xdead_BEEF\n    let n = a&b|a^a<<2+1\n    let z = a&1==0 && 0!=a>>2\n    \
         let w = m+n&0xFFFF\n    let v = ~m>>1|1<<n\n    w+v",
    );
    let out = formatted(&src);
    for line in [
        "    let m = 0xFF00 | 0b1010_1010 | 0o17 | 0xdead_BEEF\n",
        "    let n = (a & b) | (a ^ (a << (2 + 1)))\n",
        "    let z = (a & 1) == 0 && 0 != (a >> 2)\n",
        "    let w = (m + n) & 0xFFFF\n",
        "    let v = (~m >> 1) | (1 << n)\n",
        "    w + v\n",
    ] {
        assert!(out.contains(line), "expected {line:?} in:\n{out}");
    }
    assert_eq!(formatted(&out), out, "idempotent");
}
