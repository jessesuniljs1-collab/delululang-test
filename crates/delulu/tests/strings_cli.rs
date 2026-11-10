//! **RW 2.37: strings** (D-V2-225) — f-strings, `s[i]`, the rest of an everyday string's table and `Float.to_fixed`,
//! through the real binary, with RW 2.48's words for `1e300`.
//!
//! What these hold: an f-string's hole is code the checker and `authority` see like any other — an effect in a hole is
//! in its function's row (DL0501 when undeclared), a capability used in a hole is counted, and a function whose holes
//! are pure stays pure; a hole is `str(hole)`, so a secret has no text there (DL0604); the refusals name the spelling
//! (Python's `{x:.2f}`, an empty hole, a lone `}`, `s[i] = …`, `1e300`); `delulu fmt` prints an f-string as written; and
//! a program prints the same on the interpreter and on `--engine wasm`.

use std::path::PathBuf;
use std::process::{Command, Output};

/// Write `src` to a directory of its own and run `delulu ARGS… FILE`.
fn delulu(src: &str, args: &[&str]) -> Output {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir: PathBuf = std::env::temp_dir().join(format!("delulu-strings-{}-{n}", std::process::id()));
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
    let dir: PathBuf = std::env::temp_dir().join(format!("delulu-strings-fmt-{}", std::process::id()));
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
    let out = std::fs::read_to_string(&file).unwrap();
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

/// The everyday forms, on both engines, printing the same — and what Python prints for the same program (its
/// exponents aside: DeluluLang prints a Float as its literal is spelled).
#[test]
fn the_everyday_string_forms_print_the_same_on_both_engines() {
    let src = main_with(
        r#"    let name = "Ada"
    let total = 1234.5678
    let s = "héllo, wörld"
    c.println(f"hi {name}, {len(name)} letters; total {total.to_fixed(2)} {{braces}}")
    c.println(f"{s[0]}{s[1]} {s.count("l")} {s.ends_with("rld")}")
    c.println(f"[{"7".pad_left(3, "0")}][{"ab".repeat(2)}][{"  x ".trim_start()}]")
    c.println(str(100.0.exp()))"#,
    );
    let want = "hi Ada, 3 letters; total 1234.57 {braces}\nhé 3 true\n[007][abab][x ]\n2.6881171418161356e43\n";
    assert_eq!(run(&src, false), want);
    assert_eq!(run(&src, true), want);
}

/// A hole is code: an effect there is the function's, the checker refuses it undeclared, and `authority` counts it.
#[test]
fn an_effect_in_a_hole_is_in_its_functions_row() {
    let src = "module p\n\nfn note(out: Cap[Console], n: Int) -> Int ! {Write} {\n    out.println(\"side\")\n    n\n}\n\n\
               fn label(out: Cap[Console]) -> Str {\n    f\"value {note(out, 3)}\"\n}\n\n\
               fn main(root: Root) ! {Write} {\n    let c = root.console()\n    c.println(label(c))\n}\n";
    refused(src, "DL0501", &["function `label` performs effect `Write` not declared in its row"]);
}

#[test]
fn authority_counts_what_a_hole_uses_and_a_pure_hole_stays_pure() {
    let src = "module p\n\nfn badge(n: Int) -> Str {\n    f\"#{n}\"\n}\n\n\
               fn main(root: Root) ! {Read, Write} {\n    let c = root.console()\n    let fs = root.fs_read(\"data\")\n\
               \x20   c.println(f\"{badge(1)} {fs.read_text(\"data/a.txt\")}\")\n}\n";
    let o = delulu(src, &["authority", "--json"]);
    assert!(o.status.success(), "{}", text(&o));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("JSON");
    let a = &v["authority"];
    assert_eq!(a["effects"], serde_json::json!(["Read", "Write"]), "{a}");
    assert!(a["required_grants"].as_array().unwrap().iter().any(|g| g == "fs.read=data"), "the read in a hole: {a}");
    assert_eq!(a["pure_functions"], serde_json::json!(["badge"]), "{a}");
}

/// A hole is `str(hole)`: a secret, a capability or `Root` has no text there, as in `str`.
#[test]
fn a_secret_in_a_hole_has_no_text() {
    refused(
        "module p\n\nfn f(s: Secret[Str]) -> Str {\n    f\"key {s}\"\n}\n",
        "DL0604",
        &["value of type `Secret[Str]` cannot be stringified", "an f-string hole"],
    );
    refused(main_with("    c.println(f\"cap {c}\")").as_str(), "DL0604", &["`Cap[Console]` cannot be stringified"]);
}

#[test]
fn the_refusals_say_how_it_is_written() {
    refused(
        main_with("    let x = 3.14159\n    c.println(f\"{x:.2f}\")").as_str(),
        "DL0201",
        &["not a format spec", "{x.to_fixed(2)}"],
    );
    refused(main_with("    c.println(f\"a{}b\")").as_str(), "DL0202", &["an empty `{}` in an f-string", "`{{}}`"]);
    refused(main_with("    c.println(f\"a}b\")").as_str(), "DL0101", &["write `}}` for a brace"]);
    refused(
        main_with("    var s = \"abc\"\n    s[0] = \"x\"\n    c.println(s)").as_str(),
        "DL0405",
        &["a `Str` cannot be changed in place", "s.slice(0, i)"],
    );
    // RW 2.48: once, with the spelling — it was a missing `)` and an unknown name `e300`.
    refused(main_with("    c.println(str(1e300))").as_str(), "DL0104", &["write `1.0e300`"]);
    // The width first, then the fill: both arguments swapped are two mismatches, each named.
    let o = delulu(&main_with("    c.println(\"ab\".pad_left(\" \", 8))"), &["check"]);
    let t = text(&o);
    assert!(!o.status.success() && t.contains("expected `Int`, found `Str`") && t.contains("expected `Str`, found `Int`"), "{t}");
}

/// `delulu fmt` prints an f-string as written: braces doubled, a literal hole as text, every hole on its line.
#[test]
fn fmt_keeps_an_fstring_as_written() {
    let src = main_with("    let n = 2\n    c.println(f\"a {{b}} {n+1} {\"lit\"} {[n,n]}\")");
    let out = formatted(&src);
    assert!(out.contains(r#"c.println(f"a {{b}} {n + 1} lit {[n, n]}")"#), "{out}");
    assert_eq!(formatted(&out), out, "idempotent");
}
