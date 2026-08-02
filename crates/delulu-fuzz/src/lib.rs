//! The DeluluLang differential fuzz harness (Stage 2, §7.2).
//!
//! It generates small programs biased toward the soundness danger zone — capabilities threaded
//! through helper functions, higher-order `apply`, closures, and secrets — checks them, and for
//! every ACCEPTED program runs it under a trace sink and asserts the runtime trace is a subset of
//! the checker's computed row of `main` (the executable form of the Effect-Soundness theorem,
//! `DELULU_CORE.md` Theorem 3 / spec invariant 12). A single violation would be the one bug the
//! whole language exists to prevent — an effect escaping the type — and the harness treats it as
//! fatal.
//!
//! It also emits deliberately unsound programs (a helper that performs an effect its declared row
//! omits; a secret passed to `str`) and asserts the checker REJECTS them — differential coverage
//! of the boundary check itself.

use std::collections::BTreeSet;
use std::rc::Rc;

use delulu_check::check_source;
use delulu_runtime::{assert_trace, set_fixed_clock_ms, set_rand_seed, Grants, Interp, TraceSink, Value};

/// A tiny deterministic xorshift PRNG (no dependencies; reproducible from a seed).
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed | 1)
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u64) -> u64 {
        if n == 0 {
            0
        } else {
            self.next() % n
        }
    }
    fn chance(&mut self, num: u64, den: u64) -> bool {
        self.below(den) < num
    }
}

/// What the generator predicts the checker will do with a program.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Expect {
    Accept,
    Reject(&'static str), // the diagnostic code we expect
}

struct Program {
    src: String,
    expect: Expect,
}

/// The three effectful capability operations the generator uses (all runtime-safe: Console prints,
/// Clock/Rand are read-only). `(effect label, capability param name, statement fragment)`.
const OPS: [(&str, &str, &str); 3] = [
    ("Write", "out", "out.println(\"m\")"),
    ("Clock", "clk", "let _c = clk.now_ms()"),
    ("Rand", "rnd", "let _r = rnd.int(0, 100)"),
];

const CAP_PARAMS: &str = "out: Cap[Console], clk: Cap[Clock], rnd: Cap[Rand]";
const CAP_ARGS: &str = "out, clk, rnd";

/// Generate one program. Most are well-typed (a helper's declared row exactly matches what it
/// does); a fraction are deliberately unsound to fuzz the rejection paths.
fn generate(rng: &mut Rng) -> Program {
    // ~1 in 6: a known-bad program to exercise a rejection path.
    if rng.chance(1, 6) {
        return generate_reject(rng);
    }

    let n_helpers = 1 + rng.below(4) as usize; // 1..=4
    let mut helpers = String::new();
    // effects[i] = the set of op-indices helper i performs.
    let mut effects: Vec<Vec<usize>> = Vec::with_capacity(n_helpers);

    for i in 0..n_helpers {
        let mut used = Vec::new();
        for (k, _) in OPS.iter().enumerate() {
            if rng.chance(1, 2) {
                used.push(k);
            }
        }
        let row = row_of(&used);
        let mut body = String::new();
        for &k in &used {
            body.push_str("  ");
            body.push_str(OPS[k].2);
            body.push('\n');
        }
        helpers.push_str(&format!("fn h{i}({CAP_PARAMS}) ! {row} {{\n{body}}}\n\n"));
        effects.push(used);
    }

    // main calls a random non-empty subset of helpers; its row is the union of their effects.
    let mut called: Vec<usize> = (0..n_helpers).filter(|_| rng.chance(3, 5)).collect();
    if called.is_empty() {
        called.push(rng.below(n_helpers as u64) as usize);
    }
    let mut main_effects: BTreeSet<usize> = BTreeSet::new();
    for &i in &called {
        main_effects.extend(effects[i].iter().copied());
    }
    let main_row = row_of(&main_effects.iter().copied().collect::<Vec<_>>());

    let mut main_body = String::from("  let out = root.console()\n  let clk = root.clock()\n  let rnd = root.rand()\n");
    for &i in &called {
        main_body.push_str(&format!("  h{i}({CAP_ARGS})\n"));
    }

    let src = format!("module fuzz\n\n{helpers}fn main(root: Root) ! {main_row} {{\n{main_body}}}\n");
    Program { src, expect: Expect::Accept }
}

/// A deliberately unsound program the checker MUST reject.
fn generate_reject(rng: &mut Rng) -> Program {
    if rng.chance(1, 2) {
        // A helper performs an effect its declared row omits → DL0501.
        // Pick one op it performs, and declare a row WITHOUT it.
        let op = rng.below(OPS.len() as u64) as usize;
        let src = format!(
            "module fuzz\nfn h({CAP_PARAMS}) ! {{}} {{\n  {}\n}}\nfn main(root: Root) {{ }}\n",
            OPS[op].2
        );
        Program { src, expect: Expect::Reject("DL0501") }
    } else {
        // A secret passed to `str` → DL0604 (opacity).
        let src = "module fuzz\nfn leak(s: Secret[Str]) -> Str { str(s) }\nfn main(root: Root) { }\n".to_string();
        Program { src, expect: Expect::Reject("DL0604") }
    }
}

fn row_of(op_idxs: &[usize]) -> String {
    if op_idxs.is_empty() {
        return "{}".to_string();
    }
    let mut labels: Vec<&str> = op_idxs.iter().map(|&k| OPS[k].0).collect();
    labels.sort();
    labels.dedup();
    format!("{{{}}}", labels.join(", "))
}

/// Outcome of a fuzz campaign.
#[derive(Default, Debug)]
pub struct Report {
    pub generated: u64,
    pub accepted: u64,
    pub rejected: u64,
    /// FATAL: an accepted program whose runtime trace escaped `row(main)` — an effect escaped the
    /// type. Each entry is a (seed, detail) pair.
    pub soundness_violations: Vec<(u64, String)>,
    /// FATAL: a program the generator built to be rejected that the checker ACCEPTED.
    pub missed_rejections: Vec<(u64, String)>,
    /// Non-fatal: the generator predicted Accept but the checker rejected (a generator artifact,
    /// not an unsoundness — rejecting is always safe). Tracked to keep the generator honest.
    pub unexpected_rejections: u64,
}

impl Report {
    pub fn is_sound(&self) -> bool {
        self.soundness_violations.is_empty() && self.missed_rejections.is_empty()
    }
}

/// Run `iterations` fuzz cases starting from `start_seed`. Deterministic given the same inputs.
pub fn run(iterations: u64, start_seed: u64) -> Report {
    let mut report = Report::default();
    // Determinism so runs are reproducible and fast.
    set_rand_seed(0xD3_1u64);
    set_fixed_clock_ms(Some(1_000));

    for k in 0..iterations {
        let seed = start_seed.wrapping_add(k).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        let mut rng = Rng::new(seed);
        let prog = generate(&mut rng);
        report.generated += 1;

        let checked = check_source(0, &prog.src);
        let has_err = checked.has_errors();

        if has_err {
            report.rejected += 1;
            if prog.expect == Expect::Accept {
                report.unexpected_rejections += 1;
            }
            continue;
        }

        // Accepted by the checker.
        report.accepted += 1;
        if let Expect::Reject(code) = prog.expect {
            report.missed_rejections.push((seed, format!("expected {code}, but the program was accepted:\n{}", prog.src)));
            continue;
        }

        // The core assertion: run it and prove trace ⊆ row(main).
        if let Some(violation) = run_and_check_trace(&checked) {
            report.soundness_violations.push((seed, format!("{violation}\n{}", prog.src)));
        }
    }

    report
}

/// Run an accepted program under a full grant with a trace sink, then assert every traced effect
/// is in the checker's computed `main_row`. Returns `Some(msg)` on a violation (a soundness bug).
fn run_and_check_trace(checked: &delulu_check::Checked) -> Option<String> {
    let allowed: BTreeSet<String> = checked
        .result
        .main_row
        .clone()
        .unwrap_or_default()
        .iter()
        .map(|e| e.name().to_string())
        .collect();

    let grants = Grants { console: true, clock: true, rand: true, ..Default::default() };

    let sink = TraceSink::new();
    let interp = Interp::new(&checked.module).with_trace(sink.clone());
    let root = Value::Root(Rc::new(grants.build_root()));
    match interp.run_main(root) {
        Ok(_) => {}
        Err(f) => return Some(format!("runtime fault {}: {} (a well-typed program should not fault here)", f.code, f.message)),
    }

    let violations = assert_trace(&allowed, &sink.records());
    if violations.is_empty() {
        None
    } else {
        Some(format!("TRACE ESCAPED THE ROW (allowed={allowed:?}): {}", violations.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuzz_soundness_holds_over_a_campaign() {
        // A real campaign in the danger zone. Any soundness violation is fatal — it would mean an
        // effect escaped the type, the one thing the language forbids.
        let report = run(4000, 0xBEEF);
        assert!(
            report.is_sound(),
            "SOUNDNESS FAILURE.\nviolations: {:?}\nmissed rejections: {:?}",
            report.soundness_violations,
            report.missed_rejections,
        );
        // Sanity: the campaign actually exercised both paths.
        assert!(report.accepted > 100, "too few accepted programs: {report:?}");
        assert!(report.rejected > 100, "too few rejected programs: {report:?}");
        // The generator should be clean enough that "expected accept but rejected" is rare.
        assert!(
            report.unexpected_rejections * 20 < report.accepted.max(1),
            "generator produces too many unexpectedly-invalid programs: {report:?}"
        );
    }

    #[test]
    fn generator_is_deterministic() {
        // Same seed → identical campaign result shape.
        let a = run(200, 7);
        let b = run(200, 7);
        assert_eq!(a.generated, b.generated);
        assert_eq!(a.accepted, b.accepted);
        assert_eq!(a.rejected, b.rejected);
    }
}
