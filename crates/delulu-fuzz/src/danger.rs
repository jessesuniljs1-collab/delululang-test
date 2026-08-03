//! The danger-zone generator (campaign P17-D).
//!
//! # Why this module exists
//!
//! The original generator in [`crate`] emits exactly **four** program templates: a helper taking
//! three fixed capability parameters whose body is a concatenation of three fixed statements, a
//! `main` that calls a subset of those helpers, and two hard-coded rejection strings. It contains
//! no generics, no closures, no higher-order builtins, no `Secret` operations beyond `str(s)`, no
//! row variables, and no control flow.
//!
//! **It therefore could not have found either of the two soundness holes this project has had:**
//!
//! - **C88** needed `fn go[T](xs: List[Int], f: T) -> Int { let ys = xs.map(f) … }` — a bare type
//!   parameter reaching a higher-order builtin. The generator cannot write a type parameter.
//! - **IF-1** needed `k.map(fn(x) { … })` composed with `.verify(…)` and an `if`. The generator
//!   cannot write a closure, a `Secret.map`, or a branch.
//!
//! Its `trace ⊆ row` assertion is real and worth keeping — but it only ever ran on programs built
//! by concatenating three statements known to be safe. A generator that cannot express the
//! dangerous shape is not testing for it. **The grammar IS the coverage.**
//!
//! # What this adds
//!
//! Parameterised families over the shapes that have actually broken the language, each with a
//! predicted verdict the harness checks. Parameterised, not templated: the effect sets, the
//! declared rows, the nesting and the arity vary, so these are generated programs rather than more
//! examples. Every shape family below was first probed against the real CLI, and the accept/reject
//! split recorded here is what the checker actually did.

use crate::{Expect, Program, Rng};

/// Runtime-safe effectful operations, as `(effect label, capability binding, statement)`.
/// These three are the ones the fuzz harness can actually GRANT and RUN, so programs built from
/// them are executed and their traces checked. Secret-family shapes are check-only (see below).
const OPS: [(&str, &str); 3] =
    [("Write", "out.println(\"m\")"), ("Clock", "let _c = clk.now_ms()"), ("Rand", "let _r = rnd.int(0, 100)")];

const PRELUDE: &str = "  let out = root.console()\n  let clk = root.clock()\n  let rnd = root.rand()\n";

/// Pick a non-empty subset of the op indices.
fn pick_ops(rng: &mut Rng) -> Vec<usize> {
    let mut v: Vec<usize> = (0..OPS.len()).filter(|_| rng.chance(1, 2)).collect();
    if v.is_empty() {
        v.push(rng.below(OPS.len() as u64) as usize);
    }
    v
}

fn row_of(idxs: &[usize]) -> String {
    if idxs.is_empty() {
        return "{}".to_string();
    }
    let mut l: Vec<&str> = idxs.iter().map(|&k| OPS[k].0).collect();
    l.sort_unstable();
    l.dedup();
    format!("{{{}}}", l.join(", "))
}

/// Statements for a set of ops, joined with `;` (DeluluLang needs a newline or `;` between
/// statements — a bare space is DL0209).
fn stmts(idxs: &[usize]) -> String {
    idxs.iter().map(|&k| OPS[k].1).collect::<Vec<_>>().join("; ")
}

/// Drop one effect from `full` so the declared row is missing something the body performs.
/// Returns `None` when `full` has a single element and dropping would leave the honest empty row
/// (still a valid under-declaration, so the caller handles it).
fn drop_one(rng: &mut Rng, full: &[usize]) -> Vec<usize> {
    if full.is_empty() {
        return Vec::new();
    }
    let victim = rng.below(full.len() as u64) as usize;
    full.iter().enumerate().filter(|(i, _)| *i != victim).map(|(_, &k)| k).collect()
}

/// A closure argument to a higher-order builtin. The row must surface into the caller (R-4).
/// Under-declaring `main` is DL0501.
fn higher_order_closure(rng: &mut Rng) -> Program {
    let ops = pick_ops(rng);
    let honest = rng.chance(2, 3);
    let declared = if honest { ops.clone() } else { drop_one(rng, &ops) };
    let src = format!(
        "module fuzz\n\nfn main(root: Root) ! {} {{\n{PRELUDE}  let xs = [1, 2, 3]\n  let ys = xs.map(fn(x: Int) -> Int ! {} {{ {}; x }})\n}}\n",
        row_of(&declared),
        row_of(&ops),
        stmts(&ops),
    );
    Program { src, expect: if honest { Expect::Accept } else { Expect::Reject("DL0501") } }
}

/// The **C88 family**: a bare type parameter reaching a higher-order builtin, so the callback's
/// row cannot be determined. Must be refused (DL0401) rather than assumed pure — an undetermined
/// row is not an empty row.
fn bare_type_param(rng: &mut Rng) -> Program {
    let ops = pick_ops(rng);
    // C88 had a twin. `List.map` dropped the callback's row (an effect escaped); `Secret.map` did
    // the same and handed the closure the PLAINTEXT, so it leaked a live secret with no
    // `Declassify` anywhere. Both go through the same "argument is not syntactically `Type::Fn`"
    // branch, so both belong in the corpus.
    if rng.chance(1, 2) {
        let src = format!(
            "module fuzz\n\nfn go[T](xs: List[Int], f: T) -> Int {{\n  let ys = xs.map(f)\n  1\n}}\n\nfn main(root: Root) ! {} {{\n{PRELUDE}  let n = go([1, 2], fn(x: Int) -> Int ! {} {{ {}; x }})\n}}\n",
            row_of(&ops),
            row_of(&ops),
            stmts(&ops),
        );
        Program { src, expect: Expect::Reject("DL0401") }
    } else {
        // The `Secret.map` twin. Check-only: it needs a secret grant the harness does not hold.
        let src = format!(
            "module fuzz\n\nfn go[T](k: Secret[Str], f: T) -> Int {{\n  let m = k.map(f)\n  1\n}}\n\nfn main(root: Root) ! {} {{\n{PRELUDE}  let k = root.secret(\"K\")\n  let n = go(k, fn(x: Str) -> Str ! {} {{ {}; x }})\n}}\n",
            row_of(&ops),
            row_of(&ops),
            stmts(&ops),
        );
        Program { src, expect: Expect::Reject("DL0401") }
    }
}

/// A row-polymorphic helper that INVOKES its function argument. The caller's row must contain the
/// callback's, whatever `e` binds to.
fn row_polymorphic_apply(rng: &mut Rng) -> Program {
    let ops = pick_ops(rng);
    let honest = rng.chance(2, 3);
    let declared = if honest { ops.clone() } else { drop_one(rng, &ops) };
    let src = format!(
        "module fuzz\n\nfn apply[e](f: fn() -> Unit ! e) -> Unit ! e {{ f() }}\n\nfn main(root: Root) ! {} {{\n{PRELUDE}  apply(fn() -> Unit ! {} {{ {} }})\n}}\n",
        row_of(&declared),
        row_of(&ops),
        stmts(&ops),
    );
    Program { src, expect: if honest { Expect::Accept } else { Expect::Reject("DL0501") } }
}

/// A closure that captures a capability, is bound to a name, then called. The capture must not
/// launder the row.
fn captured_closure(rng: &mut Rng) -> Program {
    let ops = pick_ops(rng);
    let honest = rng.chance(2, 3);
    let declared = if honest { ops.clone() } else { drop_one(rng, &ops) };
    let src = format!(
        "module fuzz\n\nfn main(root: Root) ! {} {{\n{PRELUDE}  let f = fn() -> Unit ! {} {{ {} }}\n  f()\n}}\n",
        row_of(&declared),
        row_of(&ops),
        stmts(&ops),
    );
    Program { src, expect: if honest { Expect::Accept } else { Expect::Reject("DL0501") } }
}

/// A function taking a function, called twice — the row must not be halved or dropped by nesting.
fn nested_higher_order(rng: &mut Rng) -> Program {
    let ops = pick_ops(rng);
    let honest = rng.chance(2, 3);
    let declared = if honest { ops.clone() } else { drop_one(rng, &ops) };
    let r = row_of(&ops);
    let src = format!(
        "module fuzz\n\nfn twice(f: fn() -> Unit ! {r}) -> Unit ! {r} {{ f(); f() }}\n\nfn main(root: Root) ! {} {{\n{PRELUDE}  twice(fn() -> Unit ! {r} {{ {} }})\n}}\n",
        row_of(&declared),
        stmts(&ops),
    );
    Program { src, expect: if honest { Expect::Accept } else { Expect::Reject("DL0501") } }
}

/// The **IF-1 family**: `Secret.map` hands its closure the plaintext and `Secret.verify` returns a
/// `Bool` derived from it, so `verify` carries `Declassify` (closing rule R-2b). A probe that does
/// not declare it is DL0501. CHECK-ONLY: the harness grants console/clock/rand, not secrets, so
/// these are never executed — the assertion is the accept/reject verdict, not a trace.
fn secret_oracle(rng: &mut Rng) -> Program {
    let declare = rng.chance(1, 2);
    let (probe_row, main_row) =
        if declare { (" ! {Declassify}", "{Write, Declassify}") } else { ("", "{Write}") };
    let src = format!(
        "module fuzz\n\nfn probe(k: Secret[Str], c: Str) -> Bool{probe_row} {{\n  let a = k.map(fn(x: Str) -> Str {{ if x == c {{ \"Y\" }} else {{ \"N\" }} }})\n  let b = k.map(fn(x: Str) -> Str {{ \"Y\" }})\n  a.verify(b)\n}}\n\nfn main(root: Root) ! {main_row} {{\n  let out = root.console()\n  let k = root.secret(\"K\")\n  if probe(k, \"a\") {{ out.println(\"y\") }} else {{ out.println(\"n\") }}\n}}\n",
    );
    Program { src, expect: if declare { Expect::Accept } else { Expect::Reject("DL0501") } }
}

/// `Secret.expose` without the `Declassify` row (R-2). Check-only, as above.
fn expose_undeclared(_rng: &mut Rng) -> Program {
    Program {
        src: "module fuzz\n\nfn r(d: Cap[Declassify], s: Secret[Str]) -> Str { s.expose(d) }\n\nfn main(root: Root) { }\n"
            .to_string(),
        expect: Expect::Reject("DL0501"),
    }
}

/// Programs in these families are never executed by the harness (they need grants it does not
/// hold, or have no runnable `main` effect); only the accept/reject verdict is asserted.
pub fn is_check_only(src: &str) -> bool {
    src.contains("Secret[") || src.contains("root.secret(")
}

/// Generate one danger-zone program.
pub fn generate(rng: &mut Rng) -> Program {
    match rng.below(7) {
        0 => higher_order_closure(rng),
        1 => bare_type_param(rng),
        2 => row_polymorphic_apply(rng),
        3 => captured_closure(rng),
        4 => nested_higher_order(rng),
        5 => secret_oracle(rng),
        _ => expose_undeclared(rng),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A generator that has silently stopped generating looks exactly like a generator that finds
    /// nothing. This pins that every family is actually emitted, and that the two shapes matching
    /// the project's real soundness holes are among them.
    #[test]
    fn every_danger_family_is_actually_generated() {
        let mut rng = Rng::new(0xD00D);
        let mut saw_generic = false; // the C88 shape
        let mut saw_secret_oracle = false; // the IF-1 shape
        let mut saw_row_var = false;
        let mut saw_closure_capture = false;
        let mut saw_higher_order = false;
        let mut saw_nested = false;

        for _ in 0..4000 {
            let p = generate(&mut rng);
            if p.src.contains("fn go[T]") {
                saw_generic = true;
                assert_eq!(
                    p.expect,
                    Expect::Reject("DL0401"),
                    "a bare type parameter reaching a higher-order builtin must be REFUSED — an \
                     undetermined row is not an empty row (campaign finding C88)"
                );
            }
            if p.src.contains("a.verify(b)") {
                saw_secret_oracle = true;
            }
            if p.src.contains("fn apply[e]") {
                saw_row_var = true;
            }
            if p.src.contains("let f = fn()") {
                saw_closure_capture = true;
            }
            if p.src.contains("xs.map(fn(") {
                saw_higher_order = true;
            }
            if p.src.contains("fn twice(") {
                saw_nested = true;
            }
        }

        assert!(saw_generic, "the C88 family (generic reaching a higher-order builtin) was never generated");
        assert!(saw_secret_oracle, "the IF-1 family (Secret.map + verify) was never generated");
        assert!(saw_row_var, "the row-variable family was never generated");
        assert!(saw_closure_capture, "the captured-closure family was never generated");
        assert!(saw_higher_order, "the higher-order-closure family was never generated");
        assert!(saw_nested, "the nested-higher-order family was never generated");
    }

    /// The original generator could not express any of these. This documents the gap that made it
    /// blind to both of the project's soundness holes: the grammar IS the coverage.
    #[test]
    fn the_danger_grammar_covers_what_the_original_could_not() {
        let mut rng = Rng::new(0xBEEF);
        let mut corpus = String::new();
        for _ in 0..2000 {
            corpus.push_str(&generate(&mut rng).src);
        }
        for construct in ["[T]", "[e]", "fn(x: Int)", "fn(x: Str)", ".map(", ".verify(", ".expose(", "if "] {
            assert!(
                corpus.contains(construct),
                "the danger corpus never emitted `{construct}` — the original generator's blind \
                 spot has been reproduced rather than closed"
            );
        }
    }
}
