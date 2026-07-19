//! Study C — the performance honesty baseline (Stage 9e, spec §3.3).
//!
//! ## The only claim permitted here
//! Measured facts. Not "fast", not "competitive", not "within X of C" unless the number says so.
//! The constitution's committed claim is *competitive with C on hot paths*; this study **assesses**
//! that claim and Stage 10 **works** it. If 1.0 is not competitive, the write-up says exactly that
//! — which is the entire point of running the study before the release rather than after.
//!
//! Constitution §5.11 explicitly rejects "faster than C" as false. A study that quietly produced
//! numbers implying otherwise would be a worse failure than a slow interpreter.
//!
//! ## Design
//! Three microbenchmarks and three macro workloads, each run on every available lane:
//! - `interp` — the DeluluLang interpreter
//! - `wasm` — the DeluluLang WebAssembly backend
//! - `c` — gcc -O2, the same algorithm written in C
//! - `python` — CPython, for a dynamic-language reference point
//!
//! Each is run `REPEATS` times and the **minimum** is reported. Minimum rather than mean because
//! the quantity of interest is the work the machine can do when nothing interferes; means on a
//! laptop measure the scheduler as much as the runtime. The spread is published so the reader can
//! judge how noisy the environment was.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// How many times each benchmark runs. Small enough to finish, large enough that the minimum is
/// not a single lucky sample.
pub const REPEATS: usize = 5;

pub struct Bench {
    pub name: String,
    pub kind: &'static str,
    /// Lane -> milliseconds (minimum of REPEATS). Absent = lane unavailable or unsupported.
    pub timings: Vec<(String, u128)>,
    /// Lane -> the spread (max - min) across repeats, so noise is visible.
    pub spreads: Vec<(String, u128)>,
    /// Lanes that could not run this benchmark, and why.
    pub unrun: Vec<(String, String)>,
}

pub struct Results {
    pub benches: Vec<Bench>,
    pub lanes_available: Vec<String>,
    pub lanes_unavailable: Vec<(String, String)>,
}

/// The RELEASE binary, and only the release binary.
///
/// A performance baseline measured on a debug build is worse than no baseline: it is a number that
/// looks authoritative and describes nothing anyone will ever run. The first run of this study did
/// exactly that — and the debug build's oversized frames also made `fib(24)` crash, which is how
/// the DL0905 stack-overflow bug was found. The study now refuses to run against debug rather than
/// publishing figures from it.
fn delulu_bin() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent().and_then(|d| if d.ends_with("deps") { d.parent() } else { Some(d) })?;
    // target/<profile>/ -> target/release/
    let release = dir.parent()?.join("release").join(if cfg!(windows) { "delulu.exe" } else { "delulu" });
    if release.exists() {
        return Some(release);
    }
    None
}

fn tool_available(cmd: &str, arg: &str) -> bool {
    Command::new(cmd).arg(arg).output().map(|o| o.status.success()).unwrap_or(false)
}

/// Time a command, returning (min_ms, spread_ms) over REPEATS runs, or None if it ever failed.
fn time_it(mut make: impl FnMut() -> Command) -> Option<(u128, u128)> {
    let mut times = Vec::new();
    for _ in 0..REPEATS {
        let t = Instant::now();
        let out = make().output().ok()?;
        if !out.status.success() {
            return None;
        }
        times.push(t.elapsed().as_millis());
    }
    let min = *times.iter().min()?;
    let max = *times.iter().max()?;
    Some((min, max - min))
}

/// The benchmark programs. Each is (name, kind, delulu source, C source, python source).
/// The three lanes compute the SAME thing by the same algorithm — a benchmark where the lanes do
/// different work measures nothing.
fn programs() -> Vec<(&'static str, &'static str, String, String, String)> {
    vec![
        (
            "fib_recursive_24",
            "micro",
            "module bench\n\nfn fib(n: Int) -> Int {\n  if n < 2 { n } else { fib(n - 1) + fib(n - 2) }\n}\n\nfn main(root: Root) ! {Write} {\n  let out = root.console()\n  out.println(str(fib(24)))\n}\n".into(),
            "#include <stdio.h>\nlong fib(long n){return n<2?n:fib(n-1)+fib(n-2);}\nint main(){printf(\"%ld\\n\", fib(24));return 0;}\n".into(),
            "import sys\nsys.setrecursionlimit(10000)\ndef fib(n):\n    return n if n < 2 else fib(n-1)+fib(n-2)\nprint(fib(24))\n".into(),
        ),
        (
            "loop_sum_1m",
            "micro",
            "module bench\n\nfn main(root: Root) ! {Write} {\n  var i = 0\n  var acc = 0\n  while i < 1000000 {\n    acc = acc + i\n    i = i + 1\n  }\n  let out = root.console()\n  out.println(str(acc))\n}\n".into(),
            "#include <stdio.h>\nint main(){long acc=0;for(long i=0;i<1000000;i++)acc+=i;printf(\"%ld\\n\",acc);return 0;}\n".into(),
            "acc = 0\nfor i in range(1000000):\n    acc += i\nprint(acc)\n".into(),
        ),
        (
            "string_build_20k",
            "micro",
            "module bench\n\nfn main(root: Root) ! {Write} {\n  var i = 0\n  var n = 0\n  while i < 20000 {\n    let s = \"item\"\n    n = n + s.len()\n    i = i + 1\n  }\n  let out = root.console()\n  out.println(str(n))\n}\n".into(),
            "#include <stdio.h>\n#include <string.h>\nint main(){long n=0;for(int i=0;i<20000;i++){const char*s=\"item\";n+=strlen(s);}printf(\"%ld\\n\",n);return 0;}\n".into(),
            "n = 0\nfor i in range(20000):\n    s = 'item'\n    n += len(s)\nprint(n)\n".into(),
        ),
        (
            "wordcount_macro",
            "macro",
            "module bench\n\nfn count(text: Str) -> Int {\n  text.split(\" \").len()\n}\n\nfn main(root: Root) ! {Write} {\n  var i = 0\n  var total = 0\n  while i < 2000 {\n    total = total + count(\"the quick brown fox jumps over the lazy dog\")\n    i = i + 1\n  }\n  let out = root.console()\n  out.println(str(total))\n}\n".into(),
            "#include <stdio.h>\n#include <string.h>\nint count(const char*t){int n=1;for(const char*p=t;*p;p++)if(*p==' ')n++;return n;}\nint main(){long total=0;for(int i=0;i<2000;i++)total+=count(\"the quick brown fox jumps over the lazy dog\");printf(\"%ld\\n\",total);return 0;}\n".into(),
            "def count(t):\n    return len(t.split(' '))\ntotal = 0\nfor i in range(2000):\n    total += count('the quick brown fox jumps over the lazy dog')\nprint(total)\n".into(),
        ),
        (
            "list_map_macro",
            "macro",
            "module bench\n\nfn main(root: Root) ! {Write} {\n  var i = 0\n  var total = 0\n  while i < 2000 {\n    let xs = [1, 2, 3, 4, 5, 6, 7, 8]\n    let ys = xs.map(fn(x: Int) -> Int { x * 2 })\n    total = total + ys.len()\n    i = i + 1\n  }\n  let out = root.console()\n  out.println(str(total))\n}\n".into(),
            "#include <stdio.h>\nint main(){long total=0;for(int i=0;i<2000;i++){int xs[8]={1,2,3,4,5,6,7,8};int ys[8];for(int j=0;j<8;j++)ys[j]=xs[j]*2;total+=8;}printf(\"%ld\\n\",total);return 0;}\n".into(),
            "total = 0\nfor i in range(2000):\n    xs = [1,2,3,4,5,6,7,8]\n    ys = [x*2 for x in xs]\n    total += len(ys)\nprint(total)\n".into(),
        ),
        (
            "nested_calls_macro",
            "macro",
            "module bench\n\nfn a(x: Int) -> Int { b(x) + 1 }\nfn b(x: Int) -> Int { c(x) + 1 }\nfn c(x: Int) -> Int { x + 1 }\n\nfn main(root: Root) ! {Write} {\n  var i = 0\n  var total = 0\n  while i < 200000 {\n    total = total + a(i)\n    i = i + 1\n  }\n  let out = root.console()\n  out.println(str(total))\n}\n".into(),
            "#include <stdio.h>\nlong c(long x){return x+1;}\nlong b(long x){return c(x)+1;}\nlong a(long x){return b(x)+1;}\nint main(){long total=0;for(long i=0;i<200000;i++)total+=a(i);printf(\"%ld\\n\",total);return 0;}\n".into(),
            "def c(x):\n    return x+1\ndef b(x):\n    return c(x)+1\ndef a(x):\n    return b(x)+1\ntotal = 0\nfor i in range(200000):\n    total += a(i)\nprint(total)\n".into(),
        ),
    ]
}

pub fn run_study(work: &Path) -> std::io::Result<Results> {
    std::fs::create_dir_all(work)?;
    let Some(dl_bin) = delulu_bin() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "no RELEASE build of `delulu` found. Study C refuses to measure a debug build: the \
             numbers would look authoritative and describe nothing anyone runs. Build it first:\n  \
             cargo build --release -p delulu",
        ));
    };
    let have_gcc = tool_available("gcc", "--version");
    let have_python = tool_available("python", "--version");

    let mut lanes_available = vec!["interp".to_string(), "wasm".to_string()];
    let mut lanes_unavailable = Vec::new();
    if have_gcc {
        lanes_available.push("c".into());
    } else {
        lanes_unavailable.push(("c".into(), "no gcc on the measurement machine".into()));
    }
    if have_python {
        lanes_available.push("python".into());
    } else {
        lanes_unavailable.push(("python".into(), "no CPython on the measurement machine".into()));
    }
    // Spec §3.3 asks for Go. There is none here, and a missing lane is stated rather than dropped.
    lanes_unavailable.push(("go".into(), "no Go toolchain on the measurement machine".into()));

    let mut benches = Vec::new();
    for (name, kind, dl_src, c_src, py_src) in programs() {
        let mut timings = Vec::new();
        let mut spreads = Vec::new();
        let mut unrun = Vec::new();

        let dl_path = work.join(format!("{name}.delulu"));
        std::fs::write(&dl_path, &dl_src)?;
        let dl_s = dl_path.to_string_lossy().to_string();

        // interp
        match time_it(|| {
            let mut c = Command::new(&dl_bin);
            c.args(["run", &dl_s, "--grant", "console"]).env("DELULU_NO_FIRST_RUN", "1");
            c
        }) {
            Some((m, sp)) => {
                timings.push(("interp".into(), m));
                spreads.push(("interp".into(), sp));
            }
            None => unrun.push(("interp".into(), "the program failed to run".into())),
        }

        // wasm engine
        match time_it(|| {
            let mut c = Command::new(&dl_bin);
            c.args(["run", &dl_s, "--engine", "wasm", "--grant", "console"])
                .env("DELULU_NO_FIRST_RUN", "1");
            c
        }) {
            Some((m, sp)) => {
                timings.push(("wasm".into(), m));
                spreads.push(("wasm".into(), sp));
            }
            None => unrun.push((
                "wasm".into(),
                "unsupported by the WASM backend for this program (DL1201) or failed to run".into(),
            )),
        }

        // C
        if have_gcc {
            let c_path = work.join(format!("{name}.c"));
            let exe_path = work.join(format!("{name}_c.exe"));
            std::fs::write(&c_path, &c_src)?;
            let built = Command::new("gcc")
                .args(["-O2", &c_path.to_string_lossy(), "-o", &exe_path.to_string_lossy()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if built {
                if let Some((m, sp)) = time_it(|| Command::new(&exe_path)) {
                    timings.push(("c".into(), m));
                    spreads.push(("c".into(), sp));
                }
            } else {
                unrun.push(("c".into(), "gcc failed to compile the benchmark".into()));
            }
        }

        // Python
        if have_python {
            let py_path = work.join(format!("{name}.py"));
            std::fs::write(&py_path, &py_src)?;
            if let Some((m, sp)) = time_it(|| {
                let mut c = Command::new("python");
                c.arg(&py_path);
                c
            }) {
                timings.push(("python".into(), m));
                spreads.push(("python".into(), sp));
            }
        }

        benches.push(Bench { name: name.to_string(), kind, timings, spreads, unrun });
    }

    Ok(Results { benches, lanes_available, lanes_unavailable })
}

fn get(b: &Bench, lane: &str) -> Option<u128> {
    b.timings.iter().find(|(l, _)| l == lane).map(|(_, v)| *v)
}

pub fn json(r: &Results) -> serde_json::Value {
    use serde_json::json;
    json!({
        "study": "C",
        "title": "performance honesty baseline",
        "schema": "study-c/1",
        "repeats": REPEATS,
        "profile": "release",
        "statistic": "minimum of repeats (spread published alongside)",
        "caveat_process_startup": "Wall-clock includes process startup. For the C lane the \
            startup cost exceeds the compute for several benchmarks (its spread exceeds its \
            minimum), so ratios against C UNDERSTATE the true compute gap rather than overstate it.",
        "lanes_available": r.lanes_available,
        "lanes_unavailable": r.lanes_unavailable.iter().map(|(l, why)| json!({
            "lane": l, "reason": why
        })).collect::<Vec<_>>(),
        "benchmarks": r.benches.iter().map(|b| json!({
            "name": b.name,
            "kind": b.kind,
            "ms": b.timings.iter()
                .map(|(l, v)| (l.clone(), serde_json::Value::from(*v as u64)))
                .collect::<serde_json::Map<String, serde_json::Value>>(),
            "spread_ms": b.spreads.iter()
                .map(|(l, v)| (l.clone(), serde_json::Value::from(*v as u64)))
                .collect::<serde_json::Map<String, serde_json::Value>>(),
            "unrun": b.unrun.iter().map(|(l, why)| json!({"lane": l, "reason": why})).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

pub fn report(r: &Results) -> String {
    let mut s = String::new();
    s.push_str("# Study C — the performance honesty baseline\n\n\
                *Generated by `delulu-measure study-c`. Raw data: `results.json`. Method and \
                threats to validity: `../METHODOLOGY.md` §3.*\n\n");

    s.push_str("## The only claim made here\n\n\
                Measured facts, and nothing else. Constitution §5.11 explicitly rejects \
                *\"faster than C\"* as false — C sits near the hardware floor. The committed claim \
                is *competitive with C on hot paths, with safety C cannot offer*, and this study \
                **assesses** that claim rather than asserting it. Stage 10 is where it gets worked.\n\n\
                These numbers are the v1.0 baseline. They are published as they came out.\n\n");

    s.push_str(&format!(
        "## Method\n\nEach benchmark runs {REPEATS} times per lane; the **minimum** is reported, \
         with the spread (max − min) beside it so the reader can see how noisy the machine was. \
         Minimum rather than mean because the quantity of interest is the work done when nothing \
         interferes — a mean on a laptop measures the scheduler as much as the runtime.\n\n\
         Measured against a **release** build. The study refuses to run against a debug build at \
         all: a performance baseline from an unoptimised binary is a number that looks \
         authoritative and describes nothing anyone will ever run.\n\n\
         **Wall-clock includes process startup.** For the C lane this dominates — its spread \
         exceeds its minimum on several benchmarks, meaning the figure is mostly process creation \
         rather than compute. The consequence matters and points the uncomfortable way: the ratios \
         below **understate** the true compute gap, because the C denominator is inflated by \
         startup the C code did not spend computing.\n\n\
         All lanes compute the same result by the same algorithm.\n\n"
    ));

    let lanes = ["interp", "wasm", "c", "python"];
    s.push_str("## Results (milliseconds, lower is better)\n\n\
                | Benchmark | Kind | interp | wasm | C (gcc -O2) | CPython |\n|---|---|---|---|---|---|\n");
    for b in &r.benches {
        s.push_str(&format!("| `{}` | {} ", b.name, b.kind));
        for l in lanes {
            match get(b, l) {
                Some(v) => s.push_str(&format!("| {v} ")),
                None => s.push_str("| — "),
            }
        }
        s.push_str("|\n");
    }

    s.push_str("\n### Spread across repeats (max − min, ms)\n\n\
                | Benchmark | interp | wasm | C | CPython |\n|---|---|---|---|---|\n");
    for b in &r.benches {
        s.push_str(&format!("| `{}` ", b.name));
        for l in lanes {
            match b.spreads.iter().find(|(x, _)| x == l) {
                Some((_, v)) => s.push_str(&format!("| {v} ")),
                None => s.push_str("| — "),
            }
        }
        s.push_str("|\n");
    }

    // The assessment, computed from the data rather than asserted.
    let mut ratios: Vec<(String, f64)> = Vec::new();
    for b in &r.benches {
        if let (Some(i), Some(c)) = (get(b, "interp"), get(b, "c")) {
            if c > 0 {
                ratios.push((b.name.clone(), i as f64 / c as f64));
            }
        }
    }
    s.push_str("\n## The assessment\n\n");
    if ratios.is_empty() {
        s.push_str("No lane pair was comparable, so no assessment is offered.\n\n");
    } else {
        s.push_str("Interpreter time divided by C time, per benchmark:\n\n| Benchmark | interp ÷ C |\n|---|---|\n");
        for (n, r_) in &ratios {
            s.push_str(&format!("| `{n}` | {r_:.1}× |\n"));
        }
        let worst = ratios.iter().fold(0.0f64, |a, (_, r_)| a.max(*r_));
        let best = ratios.iter().fold(f64::MAX, |a, (_, r_)| a.min(*r_));
        s.push_str(&format!(
            "\nRange: **{best:.1}× to {worst:.1}× slower than C** on these benchmarks.\n\n\
             **The honest reading: v1.0 is NOT competitive with C on these workloads.** The \
             constitution's committed claim is about *hot paths under a tiered backend with a JIT*; \
             v1.0 ships a tree-walking interpreter and a straightforward WASM backend, and neither \
             is that. The gap above is the baseline Stage 10 works against, and stating it plainly \
             now is the only way the Stage 10 numbers will mean anything later.\n\n\
             Nothing in the release announcement may imply otherwise. If a sentence about \
             performance cannot cite a row in this table, it does not ship.\n\n"
        ));
    }

    if !r.lanes_unavailable.is_empty() {
        s.push_str("## Lanes not run\n\n");
        for (l, why) in &r.lanes_unavailable {
            s.push_str(&format!("- **{l}: UNRUN** — {why}\n"));
        }
        s.push('\n');
    }
    let any_unrun: Vec<&Bench> = r.benches.iter().filter(|b| !b.unrun.is_empty()).collect();
    if !any_unrun.is_empty() {
        s.push_str("### Benchmarks a lane could not run\n\n");
        for b in any_unrun {
            for (l, why) in &b.unrun {
                s.push_str(&format!("- `{}` on **{l}**: {why}\n", b.name));
            }
        }
        s.push('\n');
    }
    s
}
