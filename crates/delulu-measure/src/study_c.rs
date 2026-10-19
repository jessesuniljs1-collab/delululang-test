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
    /// Lane -> microseconds (minimum of REPEATS). Absent = lane unavailable or unsupported. Microseconds, not
    /// milliseconds: on Linux a C process takes under one, and whole milliseconds read it as 0 or 1 — a ratio's
    /// denominator made of rounding, and a 0 that silently dropped a benchmark from every ratio (2026-10-19).
    pub timings: Vec<(String, u128)>,
    /// Lane -> the spread (max - min) across repeats, in microseconds, so noise is visible.
    pub spreads: Vec<(String, u128)>,
    /// Lanes that could not run this benchmark, and why.
    pub unrun: Vec<(String, String)>,
}

pub struct Results {
    pub benches: Vec<Bench>,
    pub lanes_available: Vec<String>,
    pub lanes_unavailable: Vec<(String, String)>,
    /// When the study ran (UTC): a timing is a fact about a day and a machine, never about the language alone.
    pub measured_at: String,
    /// What it ran on, as the machine reports it: the operating system, the processor, each lane's toolchain.
    pub machine: Vec<(String, String)>,
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

/// Time a command, returning (min_us, spread_us, what its first run printed) over REPEATS runs, or None if it ever
/// failed.
fn time_it(mut make: impl FnMut() -> Command) -> Option<(u128, u128, String)> {
    let mut times = Vec::new();
    let mut printed = None;
    for _ in 0..REPEATS {
        let t = Instant::now();
        let out = make().output().ok()?;
        if !out.status.success() {
            return None;
        }
        times.push(t.elapsed().as_micros());
        printed.get_or_insert_with(|| String::from_utf8_lossy(&out.stdout).into_owned());
    }
    let min = *times.iter().min()?;
    let max = *times.iter().max()?;
    Some((min, max - min, printed?))
}

/// Two lanes gave the same answer: the same text, whatever their line endings and trailing space.
pub fn same_answer(reference: &str, printed: &str) -> bool {
    reference.replace("\r\n", "\n").trim_end() == printed.replace("\r\n", "\n").trim_end()
}

/// A lane's timing, or why it is not one. A lane is timed only if it printed what the interpreter — the language's
/// reference engine — printed: a benchmark where the lanes do different work measures nothing, and until 2026-10-19
/// nothing checked it, so a lane that exited 0 with another answer would have been timed as though it had done the work
/// (the WASM lane had just gone from one benchmark to four).
pub fn checked(got: Option<(u128, u128, String)>, reference: Option<&str>, failed: &str) -> Result<(u128, u128), String> {
    let (ms, spread, printed) = got.ok_or_else(|| failed.to_string())?;
    match reference {
        None => Err("not timed: the interpreter, whose answer every lane is checked against, did not run".into()),
        Some(r) if !same_answer(r, &printed) => {
            Err(format!("not timed: printed {:?} where the interpreter printed {:?}", printed.trim_end(), r.trim_end()))
        }
        Some(_) => Ok((ms, spread)),
    }
}

/// `secs` after the Unix epoch as `YYYY-MM-DD HH:MM UTC` — the civil calendar from a day count (H. Hinnant's
/// `civil_from_days`), so the harness needs no date crate.
pub fn utc_minute(secs: u64) -> String {
    let z = (secs / 86_400) as i64 + 719_468;
    let rem = secs % 86_400;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02} UTC", rem / 3_600, rem % 3_600 / 60)
}

/// What the study ran on, as the machine itself reports it — each entry only where the machine can say.
fn machine(dl_bin: &Path) -> Vec<(String, String)> {
    let first_line = |cmd: &str, args: &[&str]| -> Option<String> {
        let o = Command::new(cmd).args(args).output().ok()?;
        let text = if o.stdout.is_empty() { o.stderr } else { o.stdout };
        String::from_utf8_lossy(&text).lines().next().map(|l| l.trim().to_string()).filter(|l| !l.is_empty())
    };
    let mut m = vec![("os".to_string(), format!("{} {}", std::env::consts::OS, std::env::consts::ARCH))];
    if let Ok(n) = std::thread::available_parallelism() {
        m.push(("logical CPUs".into(), n.to_string()));
    }
    if let Some(k) = first_line("uname", &["-sr"]) {
        m.push(("kernel".into(), k));
    }
    let cpu = std::fs::read_to_string("/proc/cpuinfo").ok().and_then(|info| {
        info.lines().find(|l| l.starts_with("model name")).and_then(|l| l.split_once(':')).map(|(_, v)| v.trim().to_string())
    });
    if let Some(cpu) = cpu {
        m.push(("processor".into(), cpu));
    }
    for (lane, cmd) in [("delulu", dl_bin.to_string_lossy().to_string()), ("gcc", "gcc".into()), ("python", "python".into())] {
        if let Some(v) = first_line(&cmd, &["--version"]) {
            m.push((lane.into(), v));
        }
    }
    m
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

        // interp — the reference: every other lane must print what it printed
        let interp = time_it(|| {
            let mut c = Command::new(&dl_bin);
            c.args(["run", &dl_s, "--grant", "console"]).env("DELULU_NO_FIRST_RUN", "1");
            c
        });
        let reference = interp.as_ref().map(|(_, _, printed)| printed.clone());
        let mut put = |lane: &str, r: Result<(u128, u128), String>| match r {
            Ok((ms, spread)) => {
                timings.push((lane.into(), ms));
                spreads.push((lane.into(), spread));
            }
            Err(why) => unrun.push((lane.into(), why)),
        };
        put("interp", checked(interp, reference.as_deref(), "the program failed to run"));

        // wasm engine
        let wasm = time_it(|| {
            let mut c = Command::new(&dl_bin);
            c.args(["run", &dl_s, "--engine", "wasm", "--grant", "console"]).env("DELULU_NO_FIRST_RUN", "1");
            c
        });
        put(
            "wasm",
            checked(wasm, reference.as_deref(), "unsupported by the WASM backend for this program (DL1201) or failed to run"),
        );

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
                put("c", checked(time_it(|| Command::new(&exe_path)), reference.as_deref(), "the program failed to run"));
            } else {
                put("c", Err("gcc failed to compile the benchmark".into()));
            }
        }

        // Python
        if have_python {
            let py_path = work.join(format!("{name}.py"));
            std::fs::write(&py_path, &py_src)?;
            let py = time_it(|| {
                let mut c = Command::new("python");
                c.arg(&py_path);
                c
            });
            put("python", checked(py, reference.as_deref(), "the program failed to run"));
        }

        benches.push(Bench { name: name.to_string(), kind, timings, spreads, unrun });
    }

    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    Ok(Results { benches, lanes_available, lanes_unavailable, measured_at: utc_minute(secs), machine: machine(&dl_bin) })
}

fn get(b: &Bench, lane: &str) -> Option<u128> {
    b.timings.iter().find(|(l, _)| l == lane).map(|(_, v)| *v)
}

/// Microseconds as milliseconds, for the record and the tables.
pub fn ms(us: u128) -> f64 {
    us as f64 / 1_000.0
}

pub fn json(r: &Results) -> serde_json::Value {
    use serde_json::json;
    json!({
        "study": "C",
        "title": "performance honesty baseline",
        "schema": "study-c/2",
        "resolution": "timed in microseconds; published in milliseconds with three decimals (study-c/1 truncated to whole milliseconds)",
        "repeats": REPEATS,
        "profile": "release",
        "statistic": "minimum of repeats (spread published alongside)",
        "measured_at": r.measured_at,
        "machine": r.machine.iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::from(v.clone())))
            .collect::<serde_json::Map<String, serde_json::Value>>(),
        "answers_checked": "a lane is timed only if it printed what the interpreter printed",
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
                .map(|(l, v)| (l.clone(), serde_json::Value::from(ms(*v))))
                .collect::<serde_json::Map<String, serde_json::Value>>(),
            "spread_ms": b.spreads.iter()
                .map(|(l, v)| (l.clone(), serde_json::Value::from(ms(*v))))
                .collect::<serde_json::Map<String, serde_json::Value>>(),
            "unrun": b.unrun.iter().map(|(l, why)| json!({"lane": l, "reason": why})).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

pub fn report(r: &Results) -> String {
    let mut s = String::new();
    s.push_str(
        "# Study C — the performance honesty baseline\n\n\
                *Generated by `delulu-measure study-c`. Raw data: `results.json`. Method and \
                threats to validity: `../METHODOLOGY.md` §3.*\n\n",
    );

    s.push_str(
        "## The only claim made here\n\n\
                Measured facts, and nothing else. Constitution §5.11 explicitly rejects \
                *\"faster than C\"* as false — C sits near the hardware floor. The committed claim \
                is *competitive with C on hot paths, with safety C cannot offer*, and this study \
                **assesses** that claim rather than asserting it. Stage 10 is where it gets worked.\n\n",
    );
    s.push_str(&format!(
        "These numbers were measured **{}** on the machine below, and are published as they came out. The first \
         run (2026-07-20) was the v1.0 baseline; each earlier run is kept whole under `runs/`.\n\n",
        r.measured_at
    ));

    s.push_str(&format!(
        "## Method\n\nEach benchmark runs {REPEATS} times per lane; the **minimum** is reported, \
         with the spread (max − min) beside it so the reader can see how noisy the machine was. \
         Minimum rather than mean because the quantity of interest is the work done when nothing \
         interferes — a mean on a laptop measures the scheduler as much as the runtime. Each run is \
         timed in microseconds and published in milliseconds (the first runs truncated to whole \
         milliseconds, which read a C process on Linux as 0 or 1).\n\n\
         Measured against a **release** build. The study refuses to run against a debug build at \
         all: a performance baseline from an unoptimised binary is a number that looks \
         authoritative and describes nothing anyone will ever run.\n\n\
         **Wall-clock includes process startup.** For the C lane this dominates — its spread \
         exceeds its minimum on several benchmarks, meaning the figure is mostly process creation \
         rather than compute. The consequence matters and points the uncomfortable way: the ratios \
         below **understate** the true compute gap, because the C denominator is inflated by \
         startup the C code did not spend computing.\n\n\
         All lanes compute the same result by the same algorithm, and the study checks it: a lane is timed \
         only if it printed what the interpreter printed. A lane that printed anything else is listed under \
         *Benchmarks a lane could not run*, with both answers, and never timed.\n\n"
    ));

    s.push_str("## The machine\n\n| | |\n|---|---|\n");
    for (k, v) in &r.machine {
        s.push_str(&format!("| {k} | {} |\n", v.replace('|', "\\|")));
    }
    s.push('\n');

    let lanes = ["interp", "wasm", "c", "python"];
    s.push_str(
        "## Results (milliseconds, lower is better)\n\n\
                | Benchmark | Kind | interp | wasm | C (gcc -O2) | CPython |\n|---|---|---|---|---|---|\n",
    );
    for b in &r.benches {
        s.push_str(&format!("| `{}` | {} ", b.name, b.kind));
        for l in lanes {
            match get(b, l) {
                Some(v) => s.push_str(&format!("| {:.2} ", ms(v))),
                None => s.push_str("| — "),
            }
        }
        s.push_str("|\n");
    }

    s.push_str(
        "\n### Spread across repeats (max − min, ms)\n\n\
                | Benchmark | interp | wasm | C | CPython |\n|---|---|---|---|---|\n",
    );
    for b in &r.benches {
        s.push_str(&format!("| `{}` ", b.name));
        for l in lanes {
            match b.spreads.iter().find(|(x, _)| x == l) {
                Some((_, v)) => s.push_str(&format!("| {:.2} ", ms(*v))),
                None => s.push_str("| — "),
            }
        }
        s.push_str("|\n");
    }

    // The assessment, computed from the data rather than asserted.
    let mut ratios: Vec<(String, f64)> = Vec::new();
    let mut zero_c: Vec<&str> = Vec::new();
    for b in &r.benches {
        if let (Some(i), Some(c)) = (get(b, "interp"), get(b, "c")) {
            if c > 0 {
                ratios.push((b.name.clone(), i as f64 / c as f64));
            } else {
                zero_c.push(b.name.as_str());
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
        s.push_str(&format!("\nRange: **{best:.1}× to {worst:.1}× slower than C** on these benchmarks.\n\n"));
        if !zero_c.is_empty() {
            s.push_str(&format!(
                "Left out of every ratio, because its C time read 0 µs: {}.\n\n",
                zero_c.iter().map(|n| format!("`{n}`")).collect::<Vec<_>>().join(", ")
            ));
        }
        let wasm: Vec<(&str, f64)> = r
            .benches
            .iter()
            .filter_map(|b| match (get(b, "wasm"), get(b, "c")) {
                (Some(w), Some(c)) if c > 0 => Some((b.name.as_str(), w as f64 / c as f64)),
                _ => None,
            })
            .collect();
        s.push_str(&format!(
            "The WASM engine (`--engine wasm`) ran **{} of {}** benchmarks with the interpreter's answer.",
            r.benches.iter().filter(|b| get(b, "wasm").is_some()).count(),
            r.benches.len()
        ));
        if wasm.is_empty() {
            s.push_str("\n\n");
        } else {
            s.push_str(" WASM time divided by C time where both ran:\n\n| Benchmark | wasm ÷ C |\n|---|---|\n");
            for (n, r_) in &wasm {
                s.push_str(&format!("| `{n}` | {r_:.1}× |\n"));
            }
            s.push('\n');
        }
        let who = r.machine.iter().find(|(k, _)| k == "delulu").map(|(_, v)| format!("`{v}`")).unwrap_or("this build".into());
        if best > 1.0 {
            s.push_str(&format!(
                "**The honest reading: {who} is NOT competitive with C on these workloads.** The \
                 constitution's committed claim is about *hot paths under a tiered backend with a JIT*; \
                 DeluluLang ships a tree-walking interpreter and a WASM backend without the DIR-level \
                 optimizer, and neither is that. The gap above is the baseline Stage 10 works against, and \
                 stating it plainly is the only way later numbers will mean anything.\n\n"
            ));
        }
        s.push_str(
            "Nothing in a release announcement may imply otherwise. If a sentence about \
             performance cannot cite a row in this table, it does not ship.\n\n",
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_minute_is_the_civil_calendar() {
        // Read against Python's `datetime.fromtimestamp(s, timezone.utc)`, a leap day and a century among them.
        for (secs, want) in [
            (0, "1970-01-01 00:00 UTC"),
            (951_782_400, "2000-02-29 00:00 UTC"),
            (951_868_799, "2000-02-29 23:59 UTC"),
            (1_792_416_058, "2026-10-19 13:20 UTC"),
            (4_102_444_800, "2100-01-01 00:00 UTC"),
        ] {
            assert_eq!(utc_minute(secs), want, "{secs}");
        }
    }

    #[test]
    fn a_lane_is_timed_only_with_the_interpreters_answer() {
        assert!(same_answer("199990000\n", "199990000\r\n"), "line endings are not the answer");
        assert!(!same_answer("199990000\n", "199990001\n"));
        assert_eq!(checked(Some((7, 1, "46368\n".into())), Some("46368\n"), "failed"), Ok((7, 1)));
        let wrong = checked(Some((7, 1, "2\n".into())), Some("46368\n"), "failed").unwrap_err();
        assert!(wrong.contains("not timed") && wrong.contains("\"2\"") && wrong.contains("\"46368\""), "{wrong}");
        assert!(checked(Some((7, 1, "46368\n".into())), None, "failed").unwrap_err().contains("did not run"));
        assert_eq!(checked(None, Some("46368\n"), "it failed").unwrap_err(), "it failed");
    }

    /// The report names when and where it was measured, and a lane with another answer appears among the lanes that
    /// could not run — never in the table of timings, never in a ratio.
    #[test]
    fn the_report_names_its_machine_and_never_times_a_wrong_answer() {
        let mut timings = vec![("interp".to_string(), 80_000u128), ("c".to_string(), 2_000)];
        let mut spreads = vec![("interp".to_string(), 3_000u128), ("c".to_string(), 1_000)];
        let mut unrun = Vec::new();
        let wasm = checked(Some((5_000, 1_000, "2\n".into())), Some("46368\n"), "failed");
        match wasm {
            Ok((ms, sp)) => {
                timings.push(("wasm".into(), ms));
                spreads.push(("wasm".into(), sp));
            }
            Err(why) => unrun.push(("wasm".to_string(), why)),
        }
        let r = Results {
            benches: vec![Bench { name: "fib_recursive_24".into(), kind: "micro", timings, spreads, unrun }],
            lanes_available: vec!["interp".into(), "wasm".into(), "c".into()],
            lanes_unavailable: vec![],
            measured_at: utc_minute(1_792_416_058),
            machine: vec![("os".into(), "linux x86_64".into()), ("delulu".into(), "delulu 1.0.0".into())],
        };
        let text = report(&r);
        assert!(text.contains("measured **2026-10-19 13:20 UTC**") && text.contains("| os | linux x86_64 |"), "{text}");
        assert!(text.contains("`fib_recursive_24` on **wasm**: not timed: printed \"2\""), "{text}");
        assert!(text.contains("| `fib_recursive_24` | micro | 80.00 | — | 2.00 |"), "the wrong answer has no timing:\n{text}");
        assert!(text.contains("ran **0 of 1** benchmarks") && !text.contains("wasm ÷ C |\n|---|---|\n| `"), "{text}");
        assert!(text.contains("`delulu 1.0.0` is NOT competitive"), "{text}");
        let j = json(&r);
        assert_eq!(j["measured_at"], "2026-10-19 13:20 UTC");
        assert_eq!(j["machine"]["os"], "linux x86_64");
        assert!(j["benchmarks"][0]["ms"].get("wasm").is_none(), "{j}");
        assert_eq!(j["benchmarks"][0]["ms"]["c"], 2.0);
    }

    /// A C process on Linux takes well under a millisecond. Whole milliseconds read it as 0 or 1: the first 2026-10-19
    /// run printed C as `1` and `0`, and the benchmark whose C read 0 vanished from every ratio without a word.
    #[test]
    fn a_sub_millisecond_time_is_kept_and_a_zero_is_named() {
        let bench = |name: &str, c: u128| Bench {
            name: name.into(),
            kind: "micro",
            timings: vec![("interp".into(), 349_000), ("c".into(), c)],
            spreads: vec![],
            unrun: vec![],
        };
        let r = Results {
            benches: vec![bench("loop_sum_1m", 640), bench("nested_calls_macro", 0)],
            lanes_available: vec!["interp".into(), "c".into()],
            lanes_unavailable: vec![],
            measured_at: utc_minute(0),
            machine: vec![],
        };
        let text = report(&r);
        assert!(text.contains("| `loop_sum_1m` | micro | 349.00 | — | 0.64 |"), "{text}");
        assert!(text.contains("| `loop_sum_1m` | 545.3× |"), "{text}");
        assert!(text.contains("Left out of every ratio, because its C time read 0 µs: `nested_calls_macro`."), "{text}");
        assert_eq!(json(&r)["benchmarks"][0]["ms"]["c"], 0.64);
    }

    /// `time_it` times a real process in microseconds — no process starts in under 100 µs, so a reading below that is a
    /// truncation, not a measurement — and keeps what it printed.
    #[test]
    fn time_it_reads_a_process_in_microseconds() {
        let me = std::env::current_exe().unwrap();
        let (min, spread, printed) = time_it(|| {
            let mut c = Command::new(&me);
            c.arg("--list");
            c
        })
        .expect("the test binary lists its tests");
        assert!((100..60_000_000).contains(&min) && spread < 60_000_000, "{min} µs, spread {spread} µs");
        assert!(printed.contains("time_it_reads_a_process_in_microseconds"), "{printed}");
    }
}
