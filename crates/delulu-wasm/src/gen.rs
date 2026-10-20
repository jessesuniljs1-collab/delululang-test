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
    let src = format!("module gp\nfn g(x: Int) -> Int {{ {g_body} }}\nfn f(a: Int, b: Int, c: Int) -> Int {{ {f_body} }}\n");
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
        // Negation goes through the checked multiply on WASM (WASM-NEG-1); here it is exercised on ordinary values.
        6 => format!("(-{})", sub(r)),
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
    // A third of the conditions are short-circuits whose right side divides by the left side's operand: it faults
    // exactly when that operand is 0, which the left side rules out, so an engine that evaluates both sides faults
    // where the interpreter does not (WASM-SHORT-1).
    match r.below(6) {
        0 => {
            let d = gen_leaf(r, vars);
            format!("({d} != 0 && {} / {d} {op} {})", gen_leaf(r, vars), gen_leaf(r, vars))
        }
        1 => {
            let d = gen_leaf(r, vars);
            format!("({d} == 0 || {} % {d} {op} {})", gen_leaf(r, vars), gen_leaf(r, vars))
        }
        _ => format!("{} {op} {}", gen_leaf(r, vars), gen_leaf(r, vars)),
    }
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
    // The `Str` methods (on the WASM engine since 2026-10-20), from a stream of their own, before the comparison line so
    // that line stays the last.
    let mut s = Rng::new(seed ^ 0xE703_7ED1_A0B4_28DB);
    if s.chance(1, 2) {
        body.push_str(&format!("  out.println({})\n", gen_str_method(&mut s, 2)));
    }
    // `parse_int` (on the WASM engine since 2026-10-20), from a stream of its own: text that parses — padded, signed, the
    // ends of the `Int` range — and text that does not, or a number printed and read back.
    let mut p = Rng::new(seed ^ 0x6A09_E667_F3BC_C909);
    if p.chance(1, 3) {
        const NUMS: [&str; 10] =
            ["42", " -17 ", "+5", "9223372036854775807", "-9223372036854775808", "9223372036854775808", "", "1_000", "x1", "4 2"];
        let text = if p.chance(1, 3) {
            format!("str({})", gen_safe_int(&mut p, 2))
        } else {
            format!("\"{}\"", NUMS[p.below(NUMS.len() as u64) as usize])
        };
        body.push_str(&format!(
            "  out.println(match parse_int({text}) {{\n    Some(n) => str(n),\n    None => \"none\",\n  }})\n"
        ));
    }
    // `Float` (on the WASM engine since 2026-10-20), from a stream of its own: arithmetic that overflows to infinity, makes
    // a NaN or a negative zero, `%`, comparisons and the casts — printed through `str`, or cast back to an `Int`.
    let mut f = Rng::new(seed ^ 0xBB67_AE85_84CA_A73B);
    if f.chance(1, 3) {
        let x = gen_float(&mut f, 3);
        body.push_str(&if f.chance(1, 4) {
            format!("  out.println(str(int({x})))\n")
        } else {
            format!("  out.println(str({x}))\n")
        });
    }
    // `str`, `==` and `!=` of a list and an option (on the WASM engine since 2026-10-20, through the host's value
    // interface), from a stream of their own: `Float`s, so a NaN, an infinity and a negative zero are rendered and
    // compared inside them; a list against itself as often as against another. Written `]==[` without spaces, so the
    // count of `Str == Str` lines, which looks for ` == `, stays that line's.
    let mut v = Rng::new(seed ^ 0x3C6E_F372_FE94_F82B);
    if v.chance(1, 3) {
        let xs: Vec<String> = (0..1 + v.below(3)).map(|_| gen_float(&mut v, 2)).collect();
        let list = format!("[{}]", xs.join(", "));
        let other = if v.chance(1, 2) { list.clone() } else { format!("[{}]", gen_float(&mut v, 2)) };
        body.push_str(&format!(
            "  out.println(str({list}) + \" \" + str(Some({})) + \" \" + str({list}=={other}) + str({list}!={other}))\n",
            xs[0]
        ));
    }
    // `Str == Str` and `!=` (on the WASM engine since 2026-10-20), from a stream of their own so every other line is what
    // this seed always gave: a string against itself as often as against another.
    let mut q = Rng::new(seed ^ 0x9FB2_1C65_1E98_DF25);
    if q.chance(1, 2) {
        let (a, b) = (gen_str_expr(&mut q, 1), gen_str_expr(&mut q, 1));
        let same = if q.chance(1, 2) { a.clone() } else { b.clone() };
        body.push_str(&format!("  out.println(str({a} == {same}) + str({a} != {b}))\n"));
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

/// A `Str` from a chain of `Str` methods over text that is not all ASCII — whitespace beyond ASCII, a `ß` that
/// uppercases to two letters, a titlecase `ǅ` — with patterns that sometimes match and sometimes are empty, and slice
/// bounds that are sometimes negative, past the end or crossed; a `Bool` or a count is printed through `str`.
fn gen_str_method(r: &mut Rng, depth: u32) -> String {
    const TEXTS: [&str; 7] = ["  ab c ", "Straße", "héllo ✓", "x.y.x", "MiXeD", "", "\\u{3000}ǅé\\t"];
    const PATS: [&str; 8] = ["", "a", "x", "ß", "é", " ", "X", "l"];
    let pat = |r: &mut Rng| format!("\"{}\"", PATS[r.below(PATS.len() as u64) as usize]);
    let recv = if depth == 0 || r.chance(1, 3) {
        format!("\"{}\"", TEXTS[r.below(TEXTS.len() as u64) as usize])
    } else {
        gen_str_method(r, depth - 1)
    };
    match r.below(8) {
        0 => format!("{recv}.trim()"),
        1 => format!("{recv}.to_upper()"),
        2 => format!("{recv}.to_lower()"),
        3 => format!("str({recv}.chars().len())"),
        4 => format!("str({recv}.contains({}))", pat(r)),
        5 => format!("str({recv}.starts_with({}))", pat(r)),
        6 => format!("{recv}.replace({}, {})", pat(r), pat(r)),
        _ => format!("{recv}.slice({}, {})", r.below(9) as i64 - 2, r.below(9) as i64 - 2),
    }
}

/// A `Float` expression: leaves that are exact and inexact, huge enough to overflow when multiplied, zeros of both signs,
/// and `float` of an `Int`; every operator, `%` included; an `if` on a comparison; a negation.
fn gen_float(r: &mut Rng, depth: u32) -> String {
    const LEAVES: [&str; 8] = ["0.1", "2.5", "-3.75", "1.0e300", "0.0", "-0.0", "7.0", "1.0e200"];
    if depth == 0 || r.chance(1, 3) {
        return match r.below(5) {
            0 => format!("float({})", gen_safe_int(r, 1)),
            _ => format!("({})", LEAVES[r.below(LEAVES.len() as u64) as usize]),
        };
    }
    let sub = |r: &mut Rng| gen_float(r, depth - 1);
    match r.below(8) {
        0 => format!("({} + {})", sub(r), sub(r)),
        1 => format!("({} - {})", sub(r), sub(r)),
        2 => format!("({} * {})", sub(r), sub(r)),
        3 => format!("({} / {})", sub(r), sub(r)),
        4 => format!("({} % {})", sub(r), sub(r)),
        5 => format!("(-{})", sub(r)),
        _ => {
            const OPS: [&str; 6] = ["<", "<=", ">", ">=", "==", "!="];
            let op = OPS[r.below(OPS.len() as u64) as usize];
            format!("(if {} {op} {} {{ {} }} else {{ {} }})", sub(r), sub(r), sub(r), sub(r))
        }
    }
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

// ----- RW 3.4: random LOOP programs (`while`, `break`, `continue`, assignment) ---------------------
//
// Bounded `while` loops over an accumulator: the counter advances first in every body (so a
// `continue` cannot spin), a `continue` and a `break` sometimes guard the rest, an inner loop
// sometimes runs, and the arithmetic is the pure generator's — so signed overflow and divide-by-zero
// occur inside loops, and both engines must agree on the value or both fault. Half the programs bind a
// function value `h` before the loop (capturing `a`), call it in the loop and in the tail, and some
// shadow it in the loop's body or in an `if` there — the WASM backend inlines function values, and
// which `h` a call reaches is the scope question both engines must answer alike (WASM-HYGIENE-2).

/// Generate a random `f(a, b, c) -> Int` built on bounded `while` loops, plus three arguments — half
/// the time small, so the loops run to their end before an overflow. Deterministic in `seed`; it
/// type-checks and stays inside the WASM fragment.
pub fn random_loop_program(seed: u64) -> (String, Vec<i64>) {
    let mut r = Rng::new(seed);
    let outer = ["acc", "i", "a", "b", "c"];
    let mut body = String::from("    i = i + 1\n");
    if r.chance(1, 3) {
        body.push_str(&format!("    if {} {{ continue }}\n", gen_cmp(&mut r, &outer)));
    }
    if r.chance(1, 3) {
        body.push_str(&format!("    if {} {{ break }}\n", gen_cmp(&mut r, &outer)));
    }
    if r.chance(1, 3) {
        let inner = ["acc", "i", "j", "a"];
        let limit = r.below(6);
        let step = gen_expr(&mut r, 2, &inner, false);
        let stop = if r.chance(1, 2) { format!("      if {} {{ break }}\n", gen_cmp(&mut r, &inner)) } else { String::new() };
        body.push_str(&format!("    var j = 0\n    while j < {limit} {{\n      j = j + 1\n{stop}      acc = {step}\n    }}\n"));
    }
    // The function value draws from a stream of its own, so every other line is what this seed always gave.
    let mut q = Rng::new(seed ^ 0xA076_1D64_78BD_642F);
    let lambda = |q: &mut Rng| format!("fn(x: Int) -> Int {{ {} }}", gen_expr(q, 2, &["x", "a"], false));
    let (mut bind, tail) = if q.chance(1, 2) {
        if q.chance(1, 2) {
            body.push_str(&format!("    let h = {}\n", lambda(&mut q)));
        } else if q.chance(1, 2) {
            body.push_str(&format!(
                "    if {} {{\n      let h = {}\n      acc = h(acc)\n    }}\n",
                gen_cmp(&mut q, &outer),
                lambda(&mut q)
            ));
        }
        body.push_str("    acc = h(acc)\n");
        (format!("  let h = {}\n", lambda(&mut q)), "h(acc)")
    } else {
        (String::new(), "acc")
    };
    // A list, from a stream of its own as the function value's is (the WASM `List`): built before the loop, pushed to,
    // indexed (perhaps outside it — DL0903 on both engines), read with `get`, walked with `for` and mapped after it.
    let mut l = Rng::new(seed ^ 0xE703_7ED1_A0B4_28DB);
    let mut list_decl = String::new();
    let mut after = String::new();
    if l.chance(1, 2) {
        let items: Vec<String> = (0..1 + l.below(3)).map(|_| gen_leaf(&mut l, &["a", "b", "c"])).collect();
        list_decl = format!("  let xs = [{}]\n", items.join(", "));
        if l.chance(1, 2) {
            body.push_str(&format!("    xs.push({})\n", gen_expr(&mut l, 1, &outer, false)));
        }
        if l.chance(1, 2) {
            body.push_str(&format!("    acc = acc + xs[{}]\n", l.below(5)));
        }
        if l.chance(1, 2) {
            body.push_str(&format!(
                "    acc = match xs.get({}) {{\n      Some(v) => v,\n      None => acc,\n    }}\n",
                l.below(5)
            ));
        }
        after = if l.chance(1, 2) {
            "  for x in xs {\n    acc = acc + x\n  }\n".to_string()
        } else {
            format!(
                "  for y in xs.map(fn(x: Int) -> Int {{ {} }}) {{\n    acc = acc - y\n  }}\n",
                gen_expr(&mut l, 2, &["x", "a"], false)
            )
        };
    }
    // A function named as a value (RW 3.4), from a stream of its own as the list's is: a top-level `step`, which the outer
    // `h` is sometimes bound to (`let h = step`) and `map` is sometimes given (`xs.map(step)`) in place of a lambda.
    let mut n = Rng::new(seed ^ 0x2545_F491_4F6C_DD1D);
    let mut step = String::new();
    if n.chance(1, 3) {
        step = format!("fn step(x: Int) -> Int {{\n  {}\n}}\n", gen_expr(&mut n, 2, &["x"], false));
        if !bind.is_empty() {
            bind = "  let h = step\n".to_string();
        }
        if let (true, Some(at)) = (n.chance(1, 2), after.find("xs.map(fn")) {
            let end = after[at..].find(") {\n").map(|e| at + e).expect("the map's lambda ends its line");
            after.replace_range(at..end, "xs.map(step");
        }
    }
    body.push_str(&format!("    acc = {}\n", gen_expr(&mut r, 2, &outer, false)));
    let init = gen_leaf(&mut r, &["a", "b", "c"]);
    let limit = r.below(13);
    let src = format!(
        "module gl\n{step}fn f(a: Int, b: Int, c: Int) -> Int {{\n{bind}{list_decl}  var acc = {init}\n  var i = 0\n  while i < {limit} {{\n{body}  }}\n{after}  {tail}\n}}\n"
    );
    let small = r.chance(1, 2);
    let arg = |r: &mut Rng| if small { r.below(100) as i64 - 50 } else { gen_arg(r) };
    let args = vec![arg(&mut r), arg(&mut r), arg(&mut r)];
    (src, args)
}
