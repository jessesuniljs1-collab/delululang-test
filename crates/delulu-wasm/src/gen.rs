//! A tiny generator of random *pure* DeluluLang programs, used to differentially test the WASM
//! backend against the interpreter (Phase 3c). Every generated program is: terminating (no
//! recursion — `f` may call `g`, `g` calls nothing), overflow-free (only `+`/`-` over small
//! bounded operands, no `*`/`/`/`%`), and trap-free (no division), so BOTH engines must return the
//! same `Ok(i64)` for the same arguments. Any divergence is a backend correctness bug.

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
    let args = vec![
        r.below(11) as i64 - 5,
        r.below(11) as i64 - 5,
        r.below(11) as i64 - 5,
    ];
    (src, args)
}

fn gen_expr(r: &mut Rng, depth: u32, vars: &[&str], allow_call: bool) -> String {
    if depth == 0 || r.chance(1, 3) {
        return gen_leaf(r, vars);
    }
    let choices = if allow_call { 4 } else { 3 };
    match r.below(choices) {
        0 => format!("({} + {})", gen_expr(r, depth - 1, vars, allow_call), gen_expr(r, depth - 1, vars, allow_call)),
        1 => format!("({} - {})", gen_expr(r, depth - 1, vars, allow_call), gen_expr(r, depth - 1, vars, allow_call)),
        2 => format!(
            "if {} {{ {} }} else {{ {} }}",
            gen_cmp(r, vars),
            gen_expr(r, depth - 1, vars, allow_call),
            gen_expr(r, depth - 1, vars, allow_call)
        ),
        _ => format!("g({})", gen_leaf(r, vars)),
    }
}

fn gen_leaf(r: &mut Rng, vars: &[&str]) -> String {
    if r.chance(1, 2) {
        vars[r.below(vars.len() as u64) as usize].to_string()
    } else {
        r.below(7).to_string()
    }
}

fn gen_cmp(r: &mut Rng, vars: &[&str]) -> String {
    const OPS: [&str; 6] = ["<", "<=", ">", ">=", "==", "!="];
    let op = OPS[r.below(OPS.len() as u64) as usize];
    format!("{} {op} {}", gen_leaf(r, vars), gen_leaf(r, vars))
}
