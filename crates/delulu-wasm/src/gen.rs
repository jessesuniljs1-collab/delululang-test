//! A tiny generator of random *pure* DeluluLang programs, used to differentially test the WASM
//! backend against the interpreter (Phase 3c/3d). Every generated program is terminating (no
//! recursion — `f` may call `g`, `g` calls nothing). It DELIBERATELY exercises the arithmetic edge
//! cases: `+`/`-`/`*` over large operands (so signed overflow occurs) and `/`/`%` by small
//! divisors including 0 (so divide-by-zero occurs). With checked-arithmetic codegen (Phase 3d),
//! the two engines must agree on BOTH the value (when both succeed) AND the fault (when overflow
//! or div-by-zero occurs both must error). Any Ok-vs-differing-Ok or Ok-vs-Err is a backend bug.

/// A deterministic xorshift PRNG (reproducible from a seed).
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
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
        self.next() % n.max(1)
    }
    fn chance(&mut self, num: u64, den: u64) -> bool {
        self.below(den) < num
    }
}

/// Generate a random pure program with an entry function `f(a,b,c) -> Int` (and a helper
/// `g(x) -> Int`), plus three small arguments to call `f` with. Deterministic in `seed`.
pub fn random_pure_program(seed: u64) -> (String, Vec<i64>) {
    let mut r = Rng::new(seed);
    let g_body = gen_expr(&mut r, 2, &["x"], false);
    let f_body = gen_expr(&mut r, 3, &["a", "b", "c"], true);
    let src = format!(
        "module gp\nfn g(x: Int) -> Int {{ {g_body} }}\nfn f(a: Int, b: Int, c: Int) -> Int {{ {f_body} }}\n"
    );
    // Arguments span a large range so multiplication overflows and edge values appear.
    let args = vec![gen_arg(&mut r), gen_arg(&mut r), gen_arg(&mut r)];
    (src, args)
}

fn gen_arg(r: &mut Rng) -> i64 {
    r.below(4_000_000_000) as i64 - 2_000_000_000
}

fn gen_expr(r: &mut Rng, depth: u32, vars: &[&str], allow_call: bool) -> String {
    if depth == 0 || r.chance(1, 3) {
        return gen_leaf(r, vars);
    }
    let choices = if allow_call { 8 } else { 7 };
    let sub = |r: &mut Rng| gen_expr(r, depth - 1, vars, allow_call);
    match r.below(choices) {
        0 => format!("({} + {})", sub(r), sub(r)),
        1 => format!("({} - {})", sub(r), sub(r)),
        2 => format!("({} * {})", sub(r), sub(r)),
        // Divisor is a small leaf (0..3) so divide-by-zero occurs ~1/4 of the time.
        3 => format!("({} / {})", sub(r), r.below(4)),
        4 => format!("({} % {})", sub(r), r.below(4)),
        5 => format!("if {} {{ {} }} else {{ {} }}", gen_cmp(r, vars), sub(r), sub(r)),
        6 => gen_leaf(r, vars),
        _ => format!("g({})", gen_leaf(r, vars)),
    }
}

fn gen_leaf(r: &mut Rng, vars: &[&str]) -> String {
    match r.below(4) {
        0 | 1 => vars[r.below(vars.len() as u64) as usize].to_string(),
        2 => r.below(10).to_string(),
        _ => r.below(4_000_000_000).to_string(), // large, so `*` can overflow
    }
}

fn gen_cmp(r: &mut Rng, vars: &[&str]) -> String {
    const OPS: [&str; 6] = ["<", "<=", ">", ">=", "==", "!="];
    let op = OPS[r.below(OPS.len() as u64) as usize];
    format!("{} {op} {}", gen_leaf(r, vars), gen_leaf(r, vars))
}

// ----- Phase 3m: random CONSOLE programs (str/concat/print parity) --------------------------
//
// Unlike the pure-int generator above, this one exercises the whole console+str+concat surface and
// is deliberately FAULT-FREE (small operands, nonzero divisors) so every program succeeds and the
// two engines' printed output can be compared directly, line for line.

/// Generate a random `main(root)` that prints several lines built from `str(Int)` of safe arithmetic
/// and string concatenations. Deterministic in `seed`; type-checks and stays inside the WASM
/// fragment, so it runs on both engines and their output must be byte-identical.
pub fn random_console_program(seed: u64) -> String {
    let mut r = Rng::new(seed);
    let nlines = 1 + r.below(4); // 1..=4 println lines
    let mut body = String::from("  let out = root.console()\n");
    for _ in 0..nlines {
        body.push_str(&format!("  out.println({})\n", gen_str_expr(&mut r, 2)));
    }
    format!("module gc\nfn main(root: Root) ! {{Write}} {{\n{body}}}\n")
}

fn gen_str_expr(r: &mut Rng, depth: u32) -> String {
    if depth == 0 || r.chance(1, 2) {
        return match r.below(3) {
            0 => format!("\"{}\"", gen_word(r)),
            _ => format!("str({})", gen_safe_int(r, 2)),
        };
    }
    format!("({} + {})", gen_str_expr(r, depth - 1), gen_str_expr(r, depth - 1))
}

fn gen_word(r: &mut Rng) -> &'static str {
    const WORDS: [&str; 6] = ["x", "n=", "val:", "->", "sum ", "!"];
    WORDS[r.below(WORDS.len() as u64) as usize]
}

/// A small, overflow-free, fault-free Int expression (operands in `-20..=79`, divisors `1..=9`).
fn gen_safe_int(r: &mut Rng, depth: u32) -> String {
    if depth == 0 || r.chance(1, 3) {
        return (r.below(100) as i64 - 20).to_string();
    }
    let sub = |r: &mut Rng| gen_safe_int(r, depth - 1);
    match r.below(6) {
        0 => format!("({} + {})", sub(r), sub(r)),
        1 => format!("({} - {})", sub(r), sub(r)),
        2 => format!("({} * {})", sub(r), sub(r)),
        3 => format!("({} / {})", sub(r), 1 + r.below(9)),
        4 => format!("({} % {})", sub(r), 1 + r.below(9)),
        _ => format!("if {} {{ {} }} else {{ {} }}", gen_safe_cmp(r), sub(r), sub(r)),
    }
}

fn gen_safe_cmp(r: &mut Rng) -> String {
    const OPS: [&str; 6] = ["<", "<=", ">", ">=", "==", "!="];
    let a = r.below(40) as i64 - 20;
    let b = r.below(40) as i64 - 20;
    format!("{a} {} {b}", OPS[r.below(OPS.len() as u64) as usize])
}
