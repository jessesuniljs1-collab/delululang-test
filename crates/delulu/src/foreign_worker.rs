//! Phase 5h — foreign workers (process isolation), redeeming Stage 4's honesty note.
//!
//! Under `--foreign-isolation process` each granted C library runs in an isolated **worker
//! subprocess** — the same `delulu` executable re-invoked with the hidden `__foreign-worker`
//! subcommand — speaking the Stage-4 marshalling ([`delulu_runtime::FVal`]) over a **private pipe**.
//! The headline guarantee (spec §5, criterion 7): a worker that dies mid-call (a C segfault, a hard
//! crash) becomes [`ForeignErr::WorkerDied`] and **the host process survives** — it does not abort
//! alongside the worker.
//!
//! ## Roles and channel
//!
//! The **worker is the server** and the **host is the client** on the private channel: the worker
//! binds a [`broker_transport::Listener`] on a per-worker temp directory and `accept`s exactly one
//! persistent connection; the host connects with the bounded [`broker_transport::connect`]. Making
//! the worker the server bounds the *host's* wait (connect has a deadline) — an unbounded `accept`
//! on the host would hang if the worker never came up. The channel reuses the hardened same-user
//! transport (owner-only DACL / `0700` dir, peer-SID same-user check) chunk 3 already ships; the
//! `delulu-broker-` pipe-name prefix is cosmetic — each worker's channel is a distinct, per-worker,
//! broker-unrelated pipe.
//!
//! ## Non-disclosure of the broker (criterion 7b)
//!
//! The worker is never handed the broker's socket: it is spawned with **no broker address in argv**
//! and with `DELULU_STATE_DIR` explicitly pointed at an isolated, broker-less directory, and no
//! broker handle is inherited (the host holds no persistent broker connection at spawn time, and
//! std-handle inheritance is cleared). So when the worker resolves "where the broker is" it lands on
//! an empty directory and a connection attempt is refused. Per the §10 threat model this is
//! **non-disclosure**, not kernel enforcement — a same-user process is out of scope for hard blocks.
//!
//! ## Kill-on-host-death
//!
//! - Windows: a **Job Object** with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. The host holds the job
//!   handle; if the host dies the handle closes and the worker is killed.
//! - Unix: `prctl(PR_SET_PDEATHSIG, SIGKILL)` in the child (a `pre_exec` hook). A full seccomp
//!   profile is a documented post-chunk stub — `PR_SET_PDEATHSIG` is the must-have (head-chef
//!   ruling 4). This path is compiled but only CI (Linux) exercises it.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use delulu_runtime::foreign::{self, LoadedLib};
use delulu_runtime::{BoundForeign, FKind, FVal, ForeignBinder, ForeignErr, ForeignSig};

use crate::broker_ipc::{read_frame, write_frame};
use crate::broker_transport::{self, Connection};

/// The hidden subcommand name the worker binary is re-invoked with.
pub const WORKER_SUBCOMMAND: &str = "__foreign-worker";

// ===================================================================================================
// Wire types — the Stage-4 marshallable scalar set, CBOR-framed (head-chef ruling 1: the worker
// carries EXACTLY Int/Float/Bool/Str/Unit/ForeignPtr; no new types cross). Mirrors of the runtime's
// `FVal`/`FKind`/`ForeignSig`/`ForeignErr` so `delulu-runtime` needs no serde dependency.
// ===================================================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum WireVal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Unit,
    Ptr(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum WireForeignErr {
    NotGranted,
    SymbolMissing(String),
    BadReturn(String),
    Unavailable(String),
    WorkerDied(String),
}

/// One foreign function's marshalling signature, as `u8` kind codes (see [`fkind_to_u8`]).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct WireSig {
    name: String,
    params: Vec<u8>,
    ret: u8,
}

/// Host → worker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum WorkerReq {
    /// Load + resolve the library at `path` with `sigs` (fail-fast; spec §4.2). One per worker.
    Bind { path: String, sigs: Vec<WireSig>, max_ret: u64 },
    /// Marshal + call `method` with `args`. A segfault here kills the worker and NO response is sent.
    Call { method: String, args: Vec<WireVal> },
    /// Ask the worker to exit cleanly.
    Shutdown,
}

/// Worker → host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum WorkerResp {
    Bound { ok: bool, err: Option<WireForeignErr> },
    Value(WireVal),
    Err(WireForeignErr),
}

// ----- scalar conversions -------------------------------------------------------------------------

fn fkind_to_u8(k: FKind) -> u8 {
    match k {
        FKind::Int => 0,
        FKind::Float => 1,
        FKind::Bool => 2,
        FKind::Str => 3,
        FKind::Unit => 4,
        FKind::Ptr => 5,
    }
}

fn u8_to_fkind(b: u8) -> FKind {
    match b {
        0 => FKind::Int,
        1 => FKind::Float,
        2 => FKind::Bool,
        3 => FKind::Str,
        4 => FKind::Unit,
        _ => FKind::Ptr,
    }
}

fn fval_to_wire(v: &FVal) -> WireVal {
    match v {
        FVal::Int(i) => WireVal::Int(*i),
        FVal::Float(f) => WireVal::Float(*f),
        FVal::Bool(b) => WireVal::Bool(*b),
        FVal::Str(s) => WireVal::Str(s.to_string()),
        FVal::Unit => WireVal::Unit,
        FVal::Ptr(p) => WireVal::Ptr(*p as u64),
    }
}

fn wire_to_fval(v: WireVal) -> FVal {
    match v {
        WireVal::Int(i) => FVal::Int(i),
        WireVal::Float(f) => FVal::Float(f),
        WireVal::Bool(b) => FVal::Bool(b),
        WireVal::Str(s) => FVal::Str(Rc::from(s.as_str())),
        WireVal::Unit => FVal::Unit,
        WireVal::Ptr(p) => FVal::Ptr(p as usize),
    }
}

fn err_to_wire(e: &ForeignErr) -> WireForeignErr {
    match e {
        ForeignErr::NotGranted => WireForeignErr::NotGranted,
        ForeignErr::SymbolMissing(s) => WireForeignErr::SymbolMissing(s.clone()),
        ForeignErr::BadReturn(s) => WireForeignErr::BadReturn(s.clone()),
        ForeignErr::Unavailable(s) => WireForeignErr::Unavailable(s.clone()),
        ForeignErr::WorkerDied(s) => WireForeignErr::WorkerDied(s.clone()),
    }
}

fn wire_to_err(e: WireForeignErr) -> ForeignErr {
    match e {
        WireForeignErr::NotGranted => ForeignErr::NotGranted,
        WireForeignErr::SymbolMissing(s) => ForeignErr::SymbolMissing(s),
        WireForeignErr::BadReturn(s) => ForeignErr::BadReturn(s),
        WireForeignErr::Unavailable(s) => ForeignErr::Unavailable(s),
        WireForeignErr::WorkerDied(s) => ForeignErr::WorkerDied(s),
    }
}

fn sig_to_wire(s: &ForeignSig) -> WireSig {
    WireSig {
        name: s.name.clone(),
        params: s.params.iter().map(|k| fkind_to_u8(*k)).collect(),
        ret: fkind_to_u8(s.ret),
    }
}

fn wire_to_sig(s: WireSig) -> ForeignSig {
    ForeignSig {
        name: s.name,
        params: s.params.into_iter().map(u8_to_fkind).collect(),
        ret: u8_to_fkind(s.ret),
    }
}

// ===================================================================================================
// Host side — the binder that spawns a worker per bind, and the connection that forwards calls.
// ===================================================================================================

static WORKER_SEQ: AtomicU64 = AtomicU64::new(0);

/// Injected into the interpreter for `--foreign-isolation process` (spec §5). Each `bind` spawns one
/// isolated worker subprocess for one granted library.
pub struct WorkerBinder {
    /// The `delulu` executable to re-invoke as the worker (this process, in production).
    exe: PathBuf,
}

impl WorkerBinder {
    /// Build a binder that re-invokes the current executable as the worker.
    pub fn new() -> std::io::Result<WorkerBinder> {
        Ok(WorkerBinder { exe: std::env::current_exe()? })
    }
}

impl ForeignBinder for WorkerBinder {
    fn bind(
        &self,
        lib_name: &str,
        path: &str,
        sigs: Vec<ForeignSig>,
        max_ret: usize,
    ) -> Result<Box<dyn BoundForeign>, ForeignErr> {
        match WorkerConn::spawn(&self.exe, lib_name, path, sigs, max_ret) {
            Ok(conn) => Ok(Box::new(conn) as Box<dyn BoundForeign>),
            Err(e) => Err(e),
        }
    }
}

/// A per-worker private channel dir + an isolated broker-less state dir, plus the resolved worker exe.
struct WorkerPaths {
    channel_dir: PathBuf,
    iso_state: PathBuf,
}

fn worker_paths() -> WorkerPaths {
    let n = WORKER_SEQ.fetch_add(1, Ordering::Relaxed);
    let base = std::env::temp_dir().join(format!("delulu_fw_{}_{}", std::process::id(), n));
    WorkerPaths {
        channel_dir: base.join("chan"),
        // Deliberately NOT created: the worker resolves the broker here and finds nothing (criterion
        // 7b non-disclosure). It must NOT be the channel dir (whose socket the worker itself serves).
        iso_state: base.join("iso_state"),
    }
}

/// Build (but do not spawn) the worker command. Factored out so a unit test can assert the isolation
/// contract structurally (no broker address in argv; `DELULU_STATE_DIR` overridden to a broker-less
/// dir) without spawning a process.
pub(crate) fn build_worker_command(exe: &Path, channel_dir: &Path, iso_state: &Path) -> Command {
    let mut cmd = Command::new(exe);
    cmd.arg(WORKER_SUBCOMMAND).arg("--channel").arg(channel_dir);
    // Non-disclosure (criterion 7b): the worker's view of "the broker" is this empty, broker-less
    // directory — never the host's real state dir. Overrides any inherited DELULU_STATE_DIR.
    cmd.env("DELULU_STATE_DIR", iso_state);
    // A stable, owned cwd (never the caller's — a child locks its cwd on Windows).
    cmd.current_dir(channel_dir);
    // The worker inherits NONE of our stdio (a worker holding a copy of a captured parent pipe would
    // hang the caller — the exact chunk-3 bug). Its own stdio is null.
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    cmd
}

impl WorkerConn {
    fn spawn(
        exe: &Path,
        lib_name: &str,
        path: &str,
        sigs: Vec<ForeignSig>,
        max_ret: usize,
    ) -> Result<WorkerConn, ForeignErr> {
        let paths = worker_paths();
        std::fs::create_dir_all(&paths.channel_dir)
            .map_err(|e| ForeignErr::Unavailable(format!("worker channel dir: {e}")))?;

        // Absolute lib path so the worker (whose cwd is its channel dir) resolves the same binary the
        // caller intended, regardless of a relative grant path.
        let abs_path = std::fs::canonicalize(path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string());

        let mut cmd = build_worker_command(exe, &paths.channel_dir, &paths.iso_state);
        // Clear inheritance on OUR std handles just before spawning (Rust spawns bInheritHandles=TRUE).
        #[cfg(windows)]
        unsafe {
            crate::brokerd::clear_std_handle_inheritance();
        }
        #[cfg(target_os = "linux")]
        set_pdeathsig(&mut cmd);

        let child = cmd
            .spawn()
            .map_err(|e| ForeignErr::Unavailable(format!("cannot spawn foreign worker `{lib_name}`: {e}")))?;

        // Kill-on-host-death: assign the worker to a kill-on-close Job Object (Windows). Best-effort:
        // if assignment fails, the guard still kills the child explicitly on drop.
        #[cfg(windows)]
        let job = unsafe { win::assign_kill_on_close(&child) };
        #[cfg(not(windows))]
        let job = ();

        let mut guard = WorkerGuard {
            child: Some(child),
            channel_dir: paths.channel_dir.clone(),
            #[cfg(windows)]
            _job: job,
        };
        #[cfg(not(windows))]
        let _ = job;

        // Host is the client: connect (bounded) to the worker's server. A worker that never comes up
        // times out → Unavailable (the guard kills it).
        let conn = match connect_with_deadline(&paths.channel_dir, Duration::from_secs(10)) {
            Ok(c) => c,
            Err(e) => return Err(ForeignErr::Unavailable(format!("worker `{lib_name}` did not accept a connection: {e}"))),
        };

        let sig_map: HashMap<String, ForeignSig> = sigs.iter().map(|s| (s.name.clone(), s.clone())).collect();
        let mut conn = conn;

        // Bind handshake: send the path + sigs; the worker loads and replies.
        let req = WorkerReq::Bind {
            path: abs_path,
            sigs: sigs.iter().map(sig_to_wire).collect(),
            max_ret: max_ret as u64,
        };
        if write_frame(&mut conn, &req).is_err() {
            return Err(ForeignErr::WorkerDied("worker channel closed during bind handshake".into()));
        }
        match read_frame::<_, WorkerResp>(&mut conn) {
            Ok(WorkerResp::Bound { ok: true, .. }) => {}
            Ok(WorkerResp::Bound { ok: false, err }) => {
                return Err(err.map(wire_to_err).unwrap_or_else(|| ForeignErr::Unavailable("worker bind failed".into())));
            }
            Ok(_) => return Err(ForeignErr::WorkerDied("worker sent an unexpected bind response".into())),
            Err(_) => return Err(ForeignErr::WorkerDied("worker died during the bind handshake".into())),
        }

        // Success: the guard now lives inside the connection and cleans up on drop.
        let guard = std::mem::replace(&mut guard, WorkerGuard::empty());
        Ok(WorkerConn { conn: std::cell::RefCell::new(conn), sigs: sig_map, _guard: guard })
    }
}

/// The host-side handle to one isolated worker (implements [`BoundForeign`]). Holds the private
/// channel and the sig map (so `sig` needs no round-trip); the guard kills the worker on drop.
pub struct WorkerConn {
    conn: std::cell::RefCell<Connection>,
    sigs: HashMap<String, ForeignSig>,
    _guard: WorkerGuard,
}

impl BoundForeign for WorkerConn {
    fn sig(&self, method: &str) -> Option<ForeignSig> {
        self.sigs.get(method).cloned()
    }

    fn call(&self, method: &str, args: &[FVal], _max_ret: usize) -> Result<FVal, ForeignErr> {
        let req = WorkerReq::Call {
            method: method.to_string(),
            args: args.iter().map(fval_to_wire).collect(),
        };
        let mut conn = self.conn.borrow_mut();
        // ANY transport failure — a broken pipe on write, EOF on read — means the worker died. That is
        // the headline: the host turns the worker's death into a value, and keeps running.
        if write_frame(&mut *conn, &req).is_err() {
            return Err(ForeignErr::WorkerDied("channel write failed (worker process gone)".into()));
        }
        match read_frame::<_, WorkerResp>(&mut *conn) {
            Ok(WorkerResp::Value(v)) => Ok(wire_to_fval(v)),
            Ok(WorkerResp::Err(e)) => Err(wire_to_err(e)),
            Ok(WorkerResp::Bound { .. }) => Err(ForeignErr::WorkerDied("worker sent a bind response to a call".into())),
            Err(_) => Err(ForeignErr::WorkerDied("worker channel closed mid-call (segfault / hard crash)".into())),
        }
    }
}

/// Kills the worker + removes its channel dir on drop, never panicking (a Drop panic would mask a
/// real assertion — the exact chunk-3 lesson).
struct WorkerGuard {
    child: Option<Child>,
    channel_dir: PathBuf,
    #[cfg(windows)]
    _job: win::Job,
}

impl WorkerGuard {
    fn empty() -> WorkerGuard {
        WorkerGuard {
            child: None,
            channel_dir: PathBuf::new(),
            #[cfg(windows)]
            _job: win::Job::null(),
        }
    }
}

impl Drop for WorkerGuard {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Explicit kill (belt-and-suspenders behind the Job Object / PDEATHSIG); wait so the OS
            // releases the process's lock on its cwd before we remove the channel dir.
            let _ = child.kill();
            let _ = child.wait();
        }
        // Dropping `_job` (Windows) closes the job handle → kill-on-close fires for good measure.
        if !self.channel_dir.as_os_str().is_empty() {
            let _ = std::fs::remove_dir_all(&self.channel_dir);
        }
    }
}

/// Repeatedly attempt the bounded transport connect until `deadline` — covers a slow worker startup
/// (each inner `connect` already fails fast when the pipe is absent).
fn connect_with_deadline(channel_dir: &Path, timeout: Duration) -> std::io::Result<Connection> {
    let deadline = Instant::now() + timeout;
    loop {
        match broker_transport::connect(channel_dir) {
            Ok(c) => return Ok(c),
            Err(e) => {
                if Instant::now() >= deadline {
                    return Err(e);
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
    }
}

// ===================================================================================================
// Worker side — the `__foreign-worker` subcommand: load ONE library, serve calls over the channel.
// ===================================================================================================

/// The `delulu __foreign-worker` entry point (spec §5). Flags:
/// - `--channel <dir>`: the private channel directory (the worker binds a listener here).
/// - `--probe-broker`: diagnostic mode (criterion 7b) — attempt to reach a broker at the worker's
///   resolved state dir and print `reachable`/`unreachable`, then exit (0/1). Never used in
///   production; the real `WorkerBinder` never passes it.
pub fn run_worker(args: &[String]) -> i32 {
    // Silence Windows crash dialogs so a segfault in native code is prompt and CI-safe (no WerFault).
    #[cfg(windows)]
    unsafe {
        SetErrorMode(SEM_FAILCRITICALERRORS | SEM_NOGPFAULTERRORBOX);
    }

    let mut channel: Option<String> = None;
    let mut probe_broker = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--channel" if i + 1 < args.len() => {
                channel = Some(args[i + 1].clone());
                i += 1;
            }
            "--probe-broker" => probe_broker = true,
            _ => {}
        }
        i += 1;
    }

    if probe_broker {
        // The worker's ONLY view of "the broker" is its resolved state dir (DELULU_STATE_DIR, which
        // the binder points at a broker-less directory). A connection attempt there is refused.
        let reachable = crate::brokerd::resolve_state_dir(None)
            .map(|dir| broker_transport::connect(&dir).is_ok())
            .unwrap_or(false);
        if reachable {
            println!("reachable");
            return 0;
        } else {
            println!("unreachable");
            return 1;
        }
    }

    let Some(channel) = channel else {
        // No channel → nothing to serve (a misinvocation); fail closed.
        return 2;
    };
    let channel_dir = PathBuf::from(channel);

    let listener = match broker_transport::Listener::bind(&channel_dir) {
        Ok(l) => l,
        Err(_) => return 2,
    };
    let mut conn = match listener.accept() {
        Ok(c) => c,
        Err(_) => return 2,
    };

    let mut lib: Option<LoadedLib> = None;
    let mut max_ret = foreign::DEFAULT_MAX_RET;

    // Serve until the host closes the channel (EOF) or sends `Shutdown` — a bounded read that fails
    // fast if the host dies (never a hang).
    while let Ok(req) = read_frame::<_, WorkerReq>(&mut conn) {
        match req {
            WorkerReq::Bind { path, sigs, max_ret: mr } => {
                max_ret = mr as usize;
                let sigs: Vec<ForeignSig> = sigs.into_iter().map(wire_to_sig).collect();
                let resp = match foreign::load_and_resolve(&path, sigs) {
                    Ok(l) => {
                        lib = Some(l);
                        WorkerResp::Bound { ok: true, err: None }
                    }
                    Err(e) => WorkerResp::Bound { ok: false, err: Some(err_to_wire(&e)) },
                };
                if write_frame(&mut conn, &resp).is_err() {
                    break;
                }
            }
            WorkerReq::Call { method, args } => {
                let resp = match &lib {
                    None => WorkerResp::Err(WireForeignErr::Unavailable("no library bound in this worker".into())),
                    Some(l) => {
                        let fargs: Vec<FVal> = args.into_iter().map(wire_to_fval).collect();
                        // If the native code segfaults, the process dies HERE — no response is sent,
                        // and the host's next read returns EOF → ForeignErr::WorkerDied. That is the
                        // whole point of the isolation.
                        match foreign::call(l, &method, &fargs, max_ret) {
                            Ok(fv) => WorkerResp::Value(fval_to_wire(&fv)),
                            Err(e) => WorkerResp::Err(err_to_wire(&e)),
                        }
                    }
                };
                if write_frame(&mut conn, &resp).is_err() {
                    break;
                }
            }
            WorkerReq::Shutdown => break,
        }
    }
    0
}

// ----- Linux: PR_SET_PDEATHSIG (kill the worker if the host dies) ---------------------------------
// `prctl`/`PR_SET_PDEATHSIG` are Linux-only (libc does not define them for Apple/BSD — a bare
// `cfg(unix)` here breaks the macOS build). On macOS there is no PDEATHSIG equivalent; the worker
// is reaped by `WorkerGuard`'s explicit kill on drop (portable std), and a kqueue
// `EVFILT_PROC`-based watch is a documented possible hardening if a macOS lane ever goes live.

#[cfg(target_os = "linux")]
fn set_pdeathsig(cmd: &mut Command) {
    use std::os::unix::process::CommandExt as _;
    // SAFETY: `pre_exec` runs in the forked child before `exec`, calling only the async-signal-safe
    // `prctl`. Best-effort — a failure just means the worker is not auto-reaped on host death (the
    // host's Drop guard still kills it explicitly). A full seccomp profile is a documented post-chunk
    // stub (head-chef ruling 4): PR_SET_PDEATHSIG is the must-have.
    unsafe {
        cmd.pre_exec(|| {
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL as libc::c_ulong, 0, 0, 0);
            Ok(())
        });
    }
}

// ----- Windows: SetErrorMode (suppress crash dialogs) + Job Object (kill-on-close) ----------------

#[cfg(windows)]
const SEM_FAILCRITICALERRORS: u32 = 0x0001;
#[cfg(windows)]
const SEM_NOGPFAULTERRORBOX: u32 = 0x0002;

#[cfg(windows)]
extern "system" {
    fn SetErrorMode(uMode: u32) -> u32;
}

#[cfg(windows)]
mod win {
    use std::os::windows::io::AsRawHandle as _;

    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    /// A held Job Object handle. Closing it (drop) fires `KILL_ON_JOB_CLOSE` for any assigned process.
    pub struct Job(HANDLE);

    impl Job {
        pub fn null() -> Job {
            Job(std::ptr::null_mut())
        }
    }

    impl Drop for Job {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CloseHandle(self.0) };
            }
        }
    }

    /// Create a kill-on-close Job Object and assign `child` to it. Best-effort: returns a null [`Job`]
    /// if the OS refuses (e.g. an outer job that forbids nesting) — the caller's Drop guard still
    /// kills the child explicitly.
    ///
    /// # Safety
    /// Calls Win32 job APIs with a valid child process handle.
    pub unsafe fn assign_kill_on_close(child: &std::process::Child) -> Job {
        let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if job.is_null() {
            return Job::null();
        }
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let ok = SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );
        if ok == 0 {
            CloseHandle(job);
            return Job::null();
        }
        let handle = child.as_raw_handle() as HANDLE;
        if AssignProcessToJobObject(job, handle) == 0 {
            // Could not assign (nested-job refusal, etc.) — drop the job; explicit kill still applies.
            CloseHandle(job);
            return Job::null();
        }
        Job(job)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_scalar_round_trips_the_marshallable_set() {
        for v in [
            FVal::Int(-42),
            FVal::Float(3.5),
            FVal::Bool(true),
            FVal::Str(Rc::from("héllo")),
            FVal::Unit,
            FVal::Ptr(0xdead_beef),
        ] {
            let round = wire_to_fval(fval_to_wire(&v));
            match (&v, &round) {
                (FVal::Str(a), FVal::Str(b)) => assert_eq!(a, b),
                _ => assert_eq!(format!("{v:?}"), format!("{round:?}")),
            }
        }
    }

    #[test]
    fn wire_sig_round_trips_every_kind() {
        let sig = ForeignSig {
            name: "f".into(),
            params: vec![FKind::Int, FKind::Float, FKind::Bool, FKind::Str, FKind::Unit, FKind::Ptr],
            ret: FKind::Ptr,
        };
        let round = wire_to_sig(sig_to_wire(&sig));
        assert_eq!(round.name, "f");
        assert_eq!(round.params.len(), 6);
        assert!(matches!(round.ret, FKind::Ptr));
    }

    /// Criterion 7b (structural): the worker command discloses NO broker address — its
    /// `DELULU_STATE_DIR` is an isolated, broker-less directory (never the host's real state dir),
    /// and no broker path appears in argv.
    #[test]
    fn worker_command_does_not_disclose_the_broker() {
        let exe = PathBuf::from("delulu");
        let chan = PathBuf::from("/tmp/delulu_fw_test/chan");
        let iso = PathBuf::from("/tmp/delulu_fw_test/iso_state");
        let cmd = build_worker_command(&exe, &chan, &iso);

        let state = cmd
            .get_envs()
            .find(|(k, _)| *k == std::ffi::OsStr::new("DELULU_STATE_DIR"))
            .and_then(|(_, v)| v)
            .expect("worker sets DELULU_STATE_DIR");
        assert_eq!(state, std::ffi::OsStr::new(&iso), "worker's broker view is the isolated dir");
        assert_ne!(state, std::ffi::OsStr::new(&chan), "the isolated state dir must NOT be the channel dir");

        // No argument is a broker socket/pipe address.
        for a in cmd.get_args() {
            let s = a.to_string_lossy();
            assert!(!s.contains("broker.sock"), "no broker socket in argv: {s}");
            assert!(!s.contains(r"\pipe\delulu-broker"), "no broker pipe in argv: {s}");
        }
    }
}
