//! **LATE-LET-1: a closure never reads a name a `let` binds after it was made** — found 2026-11-01 in the weekly
//! verification sweep, on both engines.
//!
//! The checker resolves each name a closure reads where the closure is written, and a `let` is not recursive: it binds a
//! NEW name for the statements after it. Both engines read a closure's names, when it runs, in the scopes it was made
//! over; a `let` made in one of those scopes afterwards bound there, and the closure read it. So a closure typed pure
//! called the effectful function a later `let` gave its callee's name — `delulu check` clean, `delulu authority`
//! *"(none — provably pure)"*, and the run printed — and through `Secret.map` the callback printed the secret's
//! plaintext with no `Declassify` anywhere: C88's shape, R-2 reopened with R-4. A later `let` of another type made an
//! `Int` addition yield the string `two40`. The capability check held throughout (no `--grant console`, no output).
//!
//! These drive the real binary on the interpreter; the WASM engine's half is `delulu-wasm`'s `loop_parity`
//! (`a_function_value_never_reads_a_name_bound_after_it`), which holds both engines to the same values.

use std::path::PathBuf;
use std::process::{Command, Output};

/// Write `src` to a directory of its own and run `delulu ARGS… FILE`.
fn delulu(src: &str, args: &[&str]) -> Output {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir: PathBuf = std::env::temp_dir().join(format!("delulu-late-let-{}-{n}", std::process::id()));
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

/// Three ways a later `let` reached a closure: in the scope the closure was made in, in a scope above the one it was made
/// in (the closure kept in a `var` from an inner block, and the later value made elsewhere, so that only the closure's
/// own making holds that scope), and past a parameter of the same name. Each function is pure to the checker, so the
/// program is; before, each printed its line.
const EFFECTS: &str = r#"module late

fn swapped_callee(out: Cap[Console]) -> Int {
    let g = fn() -> Int { 0 }
    let h = fn() -> Int { g() }
    let g = fn() -> Int ! {Write} {
        out.println("LEAK: a swapped callee")
        1
    }
    h()
}

fn quiet() -> Int { 0 }

fn loud(out: Cap[Console]) -> List[fn() -> Int ! {Write}] {
    [fn() -> Int ! {Write} {
        out.println("LEAK: through a var")
        1
    }]
}

fn through_a_var(out: Cap[Console]) -> Int {
    var k = quiet
    if true {
        let z = 0
        k = fn() -> Int { quiet() + z }
    }
    let quiet = loud(out)[0]
    k()
}

fn past_a_parameter(out: Cap[Console], g: fn() -> Int) -> Int {
    let h = fn() -> Int { g() }
    let g = fn() -> Int ! {Write} {
        out.println("LEAK: past a parameter")
        1
    }
    h()
}

fn main(root: Root) {
    let out = root.console()
    let n = swapped_callee(out) + through_a_var(out) + past_a_parameter(out, quiet)
}
"#;

#[test]
fn a_program_the_checker_proved_pure_performs_no_effect_a_later_let_named() {
    let checked = delulu(EFFECTS, &["check"]);
    assert!(checked.status.success(), "{}", text(&checked));
    let authority = delulu(EFFECTS, &["authority"]);
    assert!(text(&authority).contains("(none — provably pure)"), "{}", text(&authority));
    let run = delulu(EFFECTS, &["run", "--grant", "console"]);
    assert!(run.status.success(), "{}", text(&run));
    assert!(!text(&run).contains("LEAK"), "a function `authority` called pure performed an effect:\n{}", text(&run));
    assert_eq!(text(&run), "", "nothing at all is printed");
}

/// A `Secret.map` callback typed pure, whose callee's name a later `let` gave a function that prints its argument.
const SECRET: &str = r#"module late_secret

fn peek(out: Cap[Console], s: Secret[Str]) -> Int {
    let g = fn(x: Str) -> Str { x }
    let h = fn(x: Str) -> Str { g(x) }
    let g = fn(x: Str) -> Str ! {Write} {
        out.println("plaintext: " + x)
        x
    }
    let m = s.map(h)
    0
}

fn main(root: Root) {
    let n = peek(root.console(), root.secret("K"))
}
"#;

#[test]
fn a_secret_map_callback_never_reaches_a_function_a_later_let_named() {
    let authority = delulu(SECRET, &["authority"]);
    assert!(text(&authority).contains("(none — provably pure)"), "{}", text(&authority));
    let run = delulu(SECRET, &["run", "--grant", "console", "--grant", "secret:K=hunter2"]);
    assert!(run.status.success(), "{}", text(&run));
    assert!(!text(&run).contains("hunter2"), "a secret's plaintext left a program with no Declassify:\n{}", text(&run));
}

/// The values: a closure kept in a list reads the `x` it was written under, not the `Str` a later `let` bound; and a
/// closure made in a `let`'s own value reads the name the `let` hides, not the list the `let` binds (a `let` is not
/// recursive). Before: `two40 seven`, then a list added to an `Int`.
const VALUES: &str = r#"module late_values

fn keep(g: fn() -> Int) -> List[fn() -> Int] { [g] }

fn main(root: Root) ! {Write} {
    let out = root.console()
    let x = 7
    let gs = keep(fn() -> Int { x })
    let x = "seven"
    let g = gs[0]
    out.println(str(g() + 1) + " " + x)
    let y = 7
    let y = [fn() -> Int { y }]
    let f = y[0]
    out.println(str(f() + 1))
}
"#;

#[test]
fn a_closure_reads_each_name_as_it_was_where_the_closure_was_written() {
    let run = delulu(VALUES, &["run", "--grant", "console"]);
    assert!(run.status.success(), "{}", text(&run));
    assert_eq!(String::from_utf8_lossy(&run.stdout), "8 seven\n8\n", "{}", text(&run));
}
