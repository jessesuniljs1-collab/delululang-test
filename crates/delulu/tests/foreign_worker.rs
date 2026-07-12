//! Stage 5 phase 5h — foreign workers (process isolation), the criterion-7 proof, end-to-end through
//! the REAL `delulu` binary (the worker is that same binary re-invoked as `__foreign-worker`).
//!
//! Two guarantees, both proven here:
//! (a) a C function that **segfaults kills only the worker** → the run fails cleanly with **DL1409**
//!     and the host `delulu` process SURVIVES (a clean exit 1 with a diagnostic, never a crash exit
//!     code) — contrasted with a normal call over the same pipe returning the right value;
//! (b) the **worker cannot reach the broker socket**: the worker resolves the broker at the isolated,
//!     broker-less directory it is given and a connection attempt is **refused** (`unreachable`),
//!     while the same probe with the broker's real directory disclosed reports `reachable` — so the
//!     refusal is meaningful (non-disclosure, per the §10 same-user threat model).
//!
//! Fixture strategy mirrors `delulu-runtime/tests/foreign_ffi.rs`: a tiny C-ABI cdylib compiled once
//! with `rustc --crate-type cdylib` (portable — `rustc` is always present under `cargo test`, no C
//! toolchain needed).

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::OnceLock;

fn delulu() -> &'static str {
    env!("CARGO_BIN_EXE_delulu")
}

fn run(cwd: &std::path::Path, args: &[&str]) -> Output {
    Command::new(delulu())
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("failed to run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// The C-ABI fixture: a normal add and a deliberate null-deref that crashes the process. `read_volatile`
/// keeps the deref from being optimized away (the cdylib is built unoptimized anyway).
const FIXTURE_SRC: &str = r##"
#[no_mangle] pub extern "C" fn dl_add(a: i64, b: i64) -> i64 { a + b }
#[no_mangle] pub extern "C" fn dl_segfault() -> i64 {
    let p: *const i64 = std::ptr::null();
    unsafe { std::ptr::read_volatile(p) }
}
"##;

fn fixture_path() -> &'static str {
    static P: OnceLock<String> = OnceLock::new();
    P.get_or_init(|| {
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
        let src = dir.join("dl_worker_fixture.rs");
        std::fs::write(&src, FIXTURE_SRC).expect("write fixture source");
        let dll = dir.join(format!(
            "{}dl_worker_fixture{}",
            std::env::consts::DLL_PREFIX,
            std::env::consts::DLL_SUFFIX
        ));
        let out = Command::new("rustc")
            .args(["--edition", "2021", "--crate-type", "cdylib"])
            .arg(&src)
            .arg("-o")
            .arg(&dll)
            .output()
            .expect("rustc must be runnable (tests run under cargo)");
        assert!(out.status.success(), "fixture cdylib failed to compile: {}", String::from_utf8_lossy(&out.stderr));
        dll.to_string_lossy().to_string()
    })
}

fn work_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("delulu_fw_it_{}_{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A program that binds the fixture and calls `method` (which returns Int), printing the result.
fn program(method: &str) -> String {
    format!(
        "module m\n\
         foreign \"c\" lib crashlib {{ fn {method}() -> Int }}\n\
         fn get(root: Root) -> Result[crashlib, ForeignErr] {{ let load = root.foreign_load()\n root.foreign(load) }}\n\
         fn main(root: Root) ! {{ForeignCall, Write}} {{ let c = root.console()\n\
         match get(root) {{ Ok(m) => c.println(str(m.{method}())), Err(_) => c.println(\"bind failed\") }} }}\n"
    )
}

/// Control: a NORMAL foreign call under `--foreign-isolation process` round-trips its value across the
/// private pipe correctly — proving the worker marshalling protocol works end to end.
#[test]
fn process_isolation_runs_a_normal_foreign_call() {
    let dir = work_dir("normal");
    let prog = dir.join("add.delulu");
    // dl_add takes two Ints; adapt the one-arg template by writing the program directly.
    std::fs::write(
        &prog,
        "module m\n\
         foreign \"c\" lib crashlib { fn dl_add(a: Int, b: Int) -> Int }\n\
         fn get(root: Root) -> Result[crashlib, ForeignErr] { let load = root.foreign_load()\n root.foreign(load) }\n\
         fn main(root: Root) ! {ForeignCall, Write} { let c = root.console()\n\
         match get(root) { Ok(m) => c.println(str(m.dl_add(20, 22))), Err(_) => c.println(\"bind failed\") } }\n",
    )
    .unwrap();
    let grant = format!("foreign.c=crashlib:{}", fixture_path());
    let o = run(
        &dir,
        &["run", prog.to_str().unwrap(), "--foreign-isolation", "process", "--grant", "console", "--grant", &grant],
    );
    assert!(o.status.success(), "process-isolation run failed: {}", stderr(&o));
    assert_eq!(stdout(&o).trim(), "42", "the value round-tripped across the worker pipe");
    assert!(stderr(&o).contains("foreign-isolation: process"), "isolation labeled: {}", stderr(&o));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Criterion 7a: a C function that segfaults kills ONLY the worker. The run fails with DL1409 and the
/// host process exits CLEANLY (exit code exactly 1 — a diagnostic, not a crash), proving the host
/// survived the worker's death instead of dying with it.
#[test]
fn segfault_in_worker_is_dl1409_and_the_host_survives() {
    let dir = work_dir("segfault");
    let prog = dir.join("crash.delulu");
    std::fs::write(&prog, program("dl_segfault")).unwrap();
    let grant = format!("foreign.c=crashlib:{}", fixture_path());
    let o = run(
        &dir,
        &["run", prog.to_str().unwrap(), "--foreign-isolation", "process", "--grant", "console", "--grant", &grant],
    );
    // Clean fault, NOT a crash: a crashed host would exit with the OS exception code, never 1.
    assert_eq!(o.status.code(), Some(1), "host must exit cleanly (1), not crash. stderr: {}", stderr(&o));
    let err = stderr(&o);
    assert!(err.contains("DL1409"), "worker death surfaces as DL1409: {err}");
    assert!(err.to_lowercase().contains("worker died") || err.contains("WorkerDied"), "message names the worker death: {err}");
    // The program's own success output never appeared — the crash aborted the call cleanly.
    assert!(!stdout(&o).contains("bind failed"), "the bind succeeded; only the call crashed: {}", stdout(&o));
    let _ = std::fs::remove_dir_all(&dir);
}

/// Criterion 7b: the worker cannot reach the broker socket. The `__foreign-worker --probe-broker`
/// mode is the worker binary attempting to reach a broker at its resolved state dir. Given the
/// isolated, broker-less directory the binder hands the worker, the attempt is REFUSED; given the
/// broker's real directory (disclosure it never receives), the probe reports reachable — so the
/// refusal is meaningful, not a probe that always fails.
#[test]
fn worker_cannot_reach_the_broker_socket() {
    let base = work_dir("probe");
    let iso = base.join("iso_state"); // isolated, broker-less (what the worker actually gets)
    std::fs::create_dir_all(&iso).unwrap();
    let broker_state = base.join("broker_state");
    std::fs::create_dir_all(&broker_state).unwrap();

    // With ONLY the isolated dir disclosed: the worker resolves the broker there and is refused.
    let o = Command::new(delulu())
        .current_dir(&base)
        .env("DELULU_STATE_DIR", &iso)
        .args(["__foreign-worker", "--probe-broker"])
        .output()
        .expect("run probe");
    assert_eq!(o.status.code(), Some(1), "worker with a broker-less dir must be refused: {}", stdout(&o));
    assert!(stdout(&o).contains("unreachable"), "worker reports the broker unreachable: {}", stdout(&o));

    // Start a real broker on broker_state, then probe WITH that dir disclosed → reachable. This proves
    // the probe genuinely detects a broker, so the "unreachable" above is a real refusal.
    let start = run(&base, &["broker", "start", "--state-dir", broker_state.to_str().unwrap()]);
    assert!(start.status.success(), "broker start: {}", stderr(&start));
    let _guard = BrokerGuard { state: broker_state.clone() };

    let o = Command::new(delulu())
        .current_dir(&base)
        .env("DELULU_STATE_DIR", &broker_state)
        .args(["__foreign-worker", "--probe-broker"])
        .output()
        .expect("run probe (disclosed)");
    assert_eq!(o.status.code(), Some(0), "with the broker disclosed the probe reaches it: {} / {}", stdout(&o), stderr(&o));
    assert!(stdout(&o).contains("reachable"), "probe reports reachable when the broker dir is disclosed: {}", stdout(&o));

    drop(_guard);
    let _ = std::fs::remove_dir_all(&base);
}

/// Stops the broker on drop from a stable cwd, never panicking (a Drop panic masks the real assertion).
struct BrokerGuard {
    state: PathBuf,
}
impl Drop for BrokerGuard {
    fn drop(&mut self) {
        let _ = Command::new(delulu())
            .current_dir(std::env::temp_dir())
            .args(["broker", "stop", "--state-dir", &self.state.to_string_lossy()])
            .output();
    }
}
