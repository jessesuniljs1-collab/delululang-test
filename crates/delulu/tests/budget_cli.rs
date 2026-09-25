//! PS-B-01 at the command line: the resource family (`SANDBOX_TEST_PLAN.md` §5.5) for the MAIN program.
//!
//! **C-06, flipped.** The characterization pinned that an ordinary `run` had no bound on either
//! engine (NE-22). It has budgets now — D-V2-25's 1 GiB and 5 minutes by default — and a run that
//! spends one is stopped, attributed and reported.
//!
//! Every child is run against its own deadline, so a missing watchdog makes these tests FAIL rather
//! than hang the suite: an endless loop with no budget behind it would otherwise be a test that never
//! ends, which is a test nobody can read the result of.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn tmp(name: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let d = std::env::temp_dir().join(format!("delulu-budget-{name}-{}-{t}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("a temp directory");
    d
}

struct Ran {
    code: Option<i32>,
    stdout: String,
    stderr: String,
    took: Duration,
}

/// Run `delulu` in `dir` and give it `limit` to finish. `None` means it was still running at the
/// deadline — it has been killed, and that is the failure the caller reports.
fn run(dir: &Path, args: &[&str], limit: Duration) -> Option<Ran> {
    let started = Instant::now();
    let mut child = Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(dir)
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .env("DELULU_STATE_DIR", dir.join("state"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the delulu binary runs");
    // Drain both pipes on their own threads, so a chatty child cannot block on a full pipe.
    let mut out = child.stdout.take().unwrap();
    let mut err = child.stderr.take().unwrap();
    let o = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        s
    });
    let e = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        s
    });
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return Some(Ran {
                code: status.code(),
                stdout: o.join().unwrap(),
                stderr: e.join().unwrap(),
                took: started.elapsed(),
            });
        }
        if started.elapsed() > limit {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn program(dir: &Path, name: &str, body: &str) {
    std::fs::write(
        dir.join(name),
        format!("module m\n\nfn main(root: Root) ! {{Write}} {{\n    let out = root.console()\n    out.println(\"started\")\n{body}}}\n"),
    )
    .unwrap();
}

fn report(dir: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(dir.join("rep.json")).expect("the run report exists");
    serde_json::from_str(text.trim()).expect("one JSON value")
}

const SPIN: &str = "    var i = 0\n    while true {\n        i = i + 1\n    }\n";
const GROW: &str = "    var s = \"x\"\n    while true {\n        s = s + s\n    }\n";

/// A loop that never ends is stopped by its processor-time budget — attributed to `cpu` by the
/// watchdog's own measurement, with the report written and the exit a failure.
#[test]
fn an_endless_loop_is_stopped_by_its_cpu_budget() {
    let dir = tmp("cpu");
    program(&dir, "spin.delulu", SPIN);
    let r = run(&dir, &["run", "spin.delulu", "--grant", "console", "--limits", "cpu=1", "--report-out", "rep.json"], Duration::from_secs(60))
        .expect("an endless loop under `--limits cpu=1` must be STOPPED, not left running");
    assert_eq!(r.code, Some(1), "{}{}", r.stdout, r.stderr);
    assert!(r.stdout.contains("started"), "the program did run: {}", r.stdout);
    assert!(r.stderr.contains("processor time") && r.stderr.contains("budget"), "{}", r.stderr);
    let v = report(&dir);
    assert_eq!(v["outcome"]["stopped_by"]["dimension"], "cpu", "{v}");
    assert_eq!(v["outcome"]["ran"], true, "{v}");
    assert_eq!(v["summary"]["errors"], 1, "a stopped run never reports zero errors: {v}");
    assert_eq!(v["sandbox"]["limits"]["cpu_seconds"], 1, "{v}");
    assert!(r.took < Duration::from_secs(30), "stopped promptly after its budget: {:?}", r.took);
}

/// Memory that grows without bound is stopped by its memory budget, long before the machine's.
#[test]
fn a_runaway_allocation_is_stopped_by_its_memory_budget() {
    let dir = tmp("mem");
    program(&dir, "grow.delulu", GROW);
    let budget = 256u64 * 1024 * 1024;
    let limit = format!("mem={budget}");
    let r = run(&dir, &["run", "grow.delulu", "--grant", "console", "--limits", &limit, "--report-out", "rep.json"], Duration::from_secs(60))
        .expect("a doubling string under a 256 MiB budget must be STOPPED");
    assert_eq!(r.code, Some(1), "{}{}", r.stdout, r.stderr);
    assert!(r.stderr.contains("memory") && r.stderr.contains("budget"), "{}", r.stderr);
    let v = report(&dir);
    assert_eq!(v["outcome"]["stopped_by"]["dimension"], "memory", "{v}");
    let observed = v["outcome"]["stopped_by"]["observed_bytes"].as_u64().unwrap();
    assert!(observed > budget, "the measurement that stopped it exceeded the budget: {v}");
}

/// NE-22's own shape, the one the red team measured past a gigabyte: an actor that works slowly and
/// a `main` that floods it, so the mailbox grows without bound under `--grant console` alone. The
/// messages are the allocation — copied across the actor boundary, queued, never drained in time —
/// and the budget has to see memory that no single `Value` in the program holds.
#[test]
fn an_actor_mailbox_flood_is_stopped_by_the_memory_budget() {
    let dir = tmp("flood");
    let payload = "x".repeat(1024);
    std::fs::write(
        dir.join("flood.delulu"),
        format!(
            "module flood\n\nactor Sink {{\n    var got: Int\n\n    new() {{\n        self.got = 0\n    }}\n\n    \
             be take(s: Str) {{\n        var i = 0\n        while i < 100000 {{\n            i = i + 1\n        }}\n        \
             self.got = self.got + 1\n    }}\n}}\n\n\
             fn main(root: Root) ! {{Async, Write}} {{\n    let out = root.console()\n    out.println(\"started\")\n    \
             let sink = spawn Sink()\n    let payload = \"{payload}\"\n    while true {{\n        sink.take(payload)\n    }}\n}}\n"
        ),
    )
    .unwrap();
    let r = run(
        &dir,
        &["run", "flood.delulu", "--grant", "console", "--limits", "mem=268435456,cpu=60", "--report-out", "rep.json"],
        Duration::from_secs(90),
    )
    .expect("a flooded mailbox under a 256 MiB budget must be STOPPED");
    assert_eq!(r.code, Some(1), "{}{}", r.stdout, r.stderr);
    let v = report(&dir);
    assert_eq!(v["outcome"]["stopped_by"]["dimension"], "memory", "the MAILBOX is what grew: {v}\n{}", r.stderr);
}

/// The same budget holds on the WASM engine — the watchdog measures the process, not an engine.
///
/// The WASM fragment has no `while` (DL1201), so the endless work here is naive recursion: `fib(60)`
/// makes on the order of 10^12 calls from a stack only sixty frames deep, which no depth bound
/// touches and which compiled code would need hours to finish. Only a processor-time budget stops it.
#[test]
fn the_wasm_engine_is_held_to_the_same_budget() {
    let dir = tmp("wasm");
    std::fs::write(
        dir.join("fib.delulu"),
        "module m\n\nfn fib(n: Int) -> Int {\n    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }\n}\n\n\
         fn main(root: Root) ! {Write} {\n    let out = root.console()\n    out.println(\"started\")\n    \
         out.println(str(fib(60)))\n}\n",
    )
    .unwrap();
    let r = run(
        &dir,
        &["run", "fib.delulu", "--engine", "wasm", "--grant", "console", "--limits", "cpu=1", "--report-out", "rep.json"],
        Duration::from_secs(60),
    )
    .expect("hours of work on the WASM engine must be STOPPED by a one-second budget");
    assert_eq!(r.code, Some(1), "{}{}", r.stdout, r.stderr);
    let v = report(&dir);
    assert_eq!(v["outcome"]["ran"], true, "it must actually have run on the WASM engine: {v}\n{}", r.stderr);
    assert_eq!(v["outcome"]["stopped_by"]["dimension"], "cpu", "{v}");
}

/// An ordinary run carries D-V2-25's budgets and says so, and a run that finishes is not stopped.
#[test]
fn an_ordinary_run_reports_the_default_budget_and_finishes() {
    let dir = tmp("default");
    program(&dir, "ok.delulu", "");
    let r = run(&dir, &["run", "ok.delulu", "--grant", "console", "--report-out", "rep.json"], Duration::from_secs(60)).unwrap();
    assert_eq!(r.code, Some(0), "{}{}", r.stdout, r.stderr);
    let v = report(&dir);
    assert_eq!(v["sandbox"]["limits"]["memory_bytes"], 1u64 << 30, "{v}");
    assert_eq!(v["sandbox"]["limits"]["cpu_seconds"], 300, "{v}");
    assert!(v["sandbox"]["limits"]["wall_seconds"].is_null(), "{v}");
    assert!(v["outcome"].get("stopped_by").is_none(), "{v}");
}

/// A wall-clock budget, which has no default, applies when the operator names one.
#[test]
fn a_wall_budget_applies_when_named() {
    let dir = tmp("wall");
    program(&dir, "spin.delulu", SPIN);
    let r = run(&dir, &["run", "spin.delulu", "--grant", "console", "--limits", "wall=1,cpu=120", "--report-out", "rep.json"], Duration::from_secs(60))
        .expect("stopped by the wall budget");
    assert_eq!(r.code, Some(1));
    assert_eq!(report(&dir)["outcome"]["stopped_by"]["dimension"], "wall");
}

/// Never unlimited, never a dimension nobody enforces: refused before anything runs.
#[test]
fn a_budget_that_cannot_be_meant_is_refused_before_the_run() {
    let dir = tmp("refuse");
    program(&dir, "ok.delulu", "");
    for bad in ["mem=0", "cpu=0", "wall=0", "disk=100", "mem=lots"] {
        let r = run(&dir, &["run", "ok.delulu", "--grant", "console", "--limits", bad], Duration::from_secs(60)).unwrap();
        assert_eq!(r.code, Some(2), "`{bad}`: {}{}", r.stdout, r.stderr);
        assert!(!r.stdout.contains("started"), "`{bad}` must refuse BEFORE the program runs: {}", r.stdout);
    }
}
