//! Phase 5f — the broker daemon (`delulu broker start|status|stop|rotate-key`) and its serve loop.
//!
//! The daemon owns exactly ONE [`delulu_broker::Broker`], one [`delulu_broker::AuditLog`], the broker
//! key, and (phase 5g) a secret store — all for one OS user. It speaks the [`crate::broker_ipc`]
//! protocol over the [`crate::broker_transport`] pipe/socket, one request per connection, on a single
//! blocking thread (head-chef ruling 1: no async; single-connection serve loop is acceptable for v0.5).
//!
//! **Fail-stop on inability to record (head-chef ruling, invariant 26).** For a synchronous-class op
//! the broker MUST append one audit record; if the append fails, the op is REFUSED (DL1401 broker-
//! failure class). This is not "enforcement reading the log" — it is fail-stop on inability to
//! record. A [`TrackingSink`] flips a flag the serve loop reads after each synchronous `check`.
//!
//! **Paths are resolved HERE only (chunk-2 ruling 2).** The library takes injected paths; the state
//! dir is `~/.delulu` (override `DELULU_STATE_DIR` / `--state-dir`, internal, for tests).

use std::cell::Cell;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use delulu_broker::{
    load_or_create_key, AuditEntry, AuditError, AuditLog, AuditRecord, AuditSink, Authority, Broker,
    Decision, GrantId, Holder, Op, Scopes, SecretStore,
};
use delulu_check::Effect;

use crate::broker_ipc::{
    read_frame, write_frame, AuthoritySpec, NodeInfo, ReqBody, Request, Response, WIRE_VERSION,
};
use crate::broker_transport::{self, Listener};

// ----- state-dir + file paths (the only place ~/.delulu is resolved) -----------------------------

/// Resolve the broker state directory: `--state-dir`/`DELULU_STATE_DIR` if set, else `~/.delulu`.
pub fn resolve_state_dir(explicit: Option<&str>) -> Option<PathBuf> {
    if let Some(d) = explicit {
        return Some(PathBuf::from(d));
    }
    if let Some(d) = std::env::var_os("DELULU_STATE_DIR") {
        return Some(PathBuf::from(d));
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".delulu"))
}

fn pid_path(state: &Path) -> PathBuf {
    state.join("broker.pid")
}
fn key_path(state: &Path) -> PathBuf {
    state.join("broker.key")
}
fn audit_dir(state: &Path) -> PathBuf {
    state.join("audit")
}
fn secrets_path(state: &Path) -> PathBuf {
    state.join("secrets.json")
}

// ----- the fail-stop-on-record audit sink --------------------------------------------------------

/// Wraps the file-backed [`AuditLog`] and flips a shared flag when an append fails, so the serve
/// loop can REFUSE a synchronous-class op the broker could not record (invariant 26, head-chef
/// ruling). The log stays observability-not-enforcement: the flag never *reads* the log to decide,
/// it only signals "could not write."
struct TrackingSink {
    inner: AuditLog,
    failed: Rc<Cell<bool>>,
}

impl AuditSink for TrackingSink {
    fn append(&mut self, entry: AuditEntry) -> Result<AuditRecord, AuditError> {
        match self.inner.append(entry) {
            Ok(r) => Ok(r),
            Err(e) => {
                self.failed.set(true);
                Err(e)
            }
        }
    }
}

// ----- wire <-> broker conversions ---------------------------------------------------------------

pub(crate) fn spec_to_authority(spec: &AuthoritySpec) -> Authority {
    let effects: Vec<Effect> = spec.effects.iter().filter_map(|e| Effect::core_from_name(e)).collect();
    Authority::new(
        effects,
        Scopes {
            fs_read: spec.fs_read.iter().cloned().collect(),
            fs_write: spec.fs_write.iter().cloned().collect(),
            net: spec.net.iter().cloned().collect(),
            secrets: spec.secrets.iter().cloned().collect(),
            declassify: spec.declassify.iter().cloned().collect(),
            foreign_c: spec.foreign_c.iter().cloned().collect(),
            foreign_python: spec.foreign_python.iter().cloned().collect(),
        },
    )
}

fn spec_holder(spec: &AuthoritySpec) -> Holder {
    let kind = if spec.holder_kind.is_empty() { "process" } else { spec.holder_kind.as_str() };
    Holder::new(kind, spec.holder_desc.clone(), "pending")
}

fn deny_response(d: &delulu_broker::Denial) -> Response {
    let diag = d.to_diagnostic();
    Response::Error { code: diag.code.to_string(), message: diag.message, requires_human: d.requires_human() }
}

/// One grant-tree node → its wire form (phase 5j `grants list|inspect`). `eff` is the node's
/// effective state (revocation + TTL folded against the broker clock). Holder fields are copied as
/// DATA — displayed by the CLI, never switched on (criterion 9).
fn node_info(n: &delulu_broker::Node, eff: delulu_broker::EffState) -> NodeInfo {
    let (state, by_seq) = match eff {
        delulu_broker::EffState::Live => ("live", None),
        delulu_broker::EffState::Revoked { by_seq } => ("revoked", Some(by_seq)),
        delulu_broker::EffState::Expired { .. } => ("expired", None),
    };
    let names = |s: &std::collections::BTreeSet<String>| s.iter().cloned().collect::<Vec<_>>();
    NodeInfo {
        id: n.id.as_str().to_string(),
        parent: n.parent.as_ref().map(|p| p.as_str().to_string()),
        holder_kind: n.holder.kind.clone(), // KIND_IS_DATA: displayed, never switched on
        holder_desc: n.holder.desc.clone(),
        holder_peer: n.holder.peer.clone(),
        state: state.to_string(),
        by_seq,
        ttl_millis: n.ttl_millis,
        created_millis: n.created_millis,
        audit_seq: n.audit_seq,
        effects: n.authority.effects.iter().map(|e| e.name().to_string()).collect(),
        fs_read: names(&n.authority.scopes.fs_read),
        fs_write: names(&n.authority.scopes.fs_write),
        net: names(&n.authority.scopes.net),
        secrets: names(&n.authority.scopes.secrets),
        declassify: names(&n.authority.scopes.declassify),
        foreign_c: names(&n.authority.scopes.foreign_c),
        foreign_python: names(&n.authority.scopes.foreign_python),
    }
}

// ----- the request handler -----------------------------------------------------------------------

/// Handle one request against the broker. Returns the response and whether the daemon should stop.
fn handle(
    broker: &mut Broker,
    secrets: &SecretStore,
    failed: &Rc<Cell<bool>>,
    pid: u32,
    req: Request,
) -> (Response, bool) {
    // Version gate (spec §8, DL1406). Any mismatch is answered, never acted on.
    if req.version != WIRE_VERSION {
        return (
            Response::Error {
                code: "DL1406".to_string(),
                message: format!(
                    "broker protocol version mismatch: server speaks `{WIRE_VERSION}`, client sent `{}` — upgrade",
                    req.version
                ),
                requires_human: true,
            },
            false,
        );
    }

    match req.body {
        ReqBody::Status => (Response::Status { pid, nodes: broker.len(), epoch: broker.epoch() }, false),
        ReqBody::Shutdown => (Response::Ok, true),
        ReqBody::RotateKey => {
            broker.rotate_key();
            (Response::Ok, false)
        }
        ReqBody::Issue(spec) => {
            let id = broker.issue(spec_holder(&spec), spec_to_authority(&spec), spec.ttl_millis);
            (Response::Issued { node: id.to_string() }, false)
        }
        ReqBody::Attenuate { parent, authority } => {
            let parent = GrantId::from_trusted(parent);
            match broker.attenuate(&parent, spec_to_authority(&authority), spec_holder(&authority), authority.ttl_millis) {
                Ok(id) => (Response::Issued { node: id.to_string() }, false),
                Err(d) => (deny_response(&d), false),
            }
        }
        ReqBody::Delegate { parent, authority, multi } => {
            let parent = GrantId::from_trusted(parent);
            match broker.delegate(&parent, spec_to_authority(&authority), spec_holder(&authority), authority.ttl_millis, multi) {
                Ok((id, token)) => (Response::Delegated { node: id.to_string(), token: token.into_string() }, false),
                Err(d) => (deny_response(&d), false),
            }
        }
        ReqBody::Redeem { token, peer } => {
            let token = delulu_broker::Token::from_wire(token);
            match broker.redeem(&token, peer) {
                Ok(id) => (Response::Redeemed { node: id.to_string() }, false),
                Err(d) => (deny_response(&d), false),
            }
        }
        ReqBody::Revoke { caller, target } => {
            let caller = GrantId::from_trusted(caller);
            let target = GrantId::from_trusted(target);
            match broker.revoke(&caller, &target) {
                Ok(out) => (
                    Response::Revoked {
                        by_seq: out.by_seq,
                        epoch: out.epoch,
                        newly_revoked: out.newly_revoked.iter().map(|g| g.to_string()).collect(),
                    },
                    false,
                ),
                Err(d) => (deny_response(&d), false),
            }
        }
        ReqBody::Check { node, op, arg } => {
            let Some(op) = Op::from_wire_name(&op) else {
                return (
                    Response::Error {
                        code: "DL0904".to_string(),
                        message: format!("unknown op `{op}` (fail closed)"),
                        requires_human: false,
                    },
                    false,
                );
            };
            let node = GrantId::from_trusted(node);
            failed.set(false);
            let decision = broker.check(&node, op, arg.as_deref());
            // Fail-stop on inability to record a synchronous-class use (invariant 26).
            if failed.get() {
                return (
                    Response::Error {
                        code: "DL1401".to_string(),
                        message: "broker could not append the audit record for a synchronous-class op — refused (fail closed, invariant 26)".to_string(),
                        requires_human: true,
                    },
                    false,
                );
            }
            let resp = match decision {
                Decision::Allow { audit_seq } => Response::Decision { allow: true, code: None, message: None, audit_seq },
                Decision::Deny(d) => {
                    let diag = d.to_diagnostic();
                    Response::Decision { allow: false, code: Some(diag.code.to_string()), message: Some(diag.message), audit_seq: None }
                }
            };
            (resp, false)
        }
        ReqBody::NodeState { node } => {
            let node = GrantId::from_trusted(node);
            match broker.effective_state(&node) {
                Some(eff) => {
                    let (state, by_seq, ttl_millis, now_millis) = match eff {
                        delulu_broker::EffState::Live => ("live", None, None, None),
                        delulu_broker::EffState::Revoked { by_seq } => ("revoked", Some(by_seq), None, None),
                        delulu_broker::EffState::Expired { ttl_millis, now_millis } => ("expired", None, Some(ttl_millis), Some(now_millis)),
                    };
                    (Response::NodeState { epoch: broker.epoch(), state: state.to_string(), by_seq, ttl_millis, now_millis }, false)
                }
                None => (
                    Response::Error { code: "DL0904".to_string(), message: format!("no such lease `{node}` (fail closed)"), requires_human: false },
                    false,
                ),
            }
        }
        ReqBody::Expose { node, name, span } => {
            let node = GrantId::from_trusted(node);
            failed.set(false);
            match broker.expose(&node, &name, secrets, span) {
                Ok(bytes) => {
                    if failed.get() {
                        return (
                            Response::Error {
                                code: "DL1401".to_string(),
                                message: "broker could not append the expose audit record — refused (fail closed, invariant 26)".to_string(),
                                requires_human: true,
                            },
                            false,
                        );
                    }
                    (Response::Exposed { bytes }, false)
                }
                Err(d) => (deny_response(&d), false),
            }
        }
        ReqBody::SecretMap { node, name, op, arg } => {
            let node = GrantId::from_trusted(node);
            match broker.secret_map(&node, &name, &op, arg.as_deref(), secrets) {
                Ok(new_name) => (Response::Mapped { name: new_name }, false),
                Err(d) => (deny_response(&d), false),
            }
        }
        ReqBody::Tree => (Response::Tree { text: broker.tree() }, false),
        ReqBody::List => {
            // A CLI/human read surface (phase 5j). Effective states are folded against the clock
            // so `grants list` shows what `check` would actually see right now.
            let nodes: Vec<NodeInfo> = broker
                .nodes()
                .iter()
                .map(|n| {
                    let eff = broker.effective_state(&n.id).unwrap_or(delulu_broker::EffState::Live);
                    node_info(n, eff)
                })
                .collect();
            (Response::Listed { nodes }, false)
        }
        ReqBody::Inspect { node } => {
            let node = GrantId::from_trusted(node);
            match (broker.inspect(&node), broker.effective_state(&node)) {
                (Some(n), Some(eff)) => (Response::Inspected { node: Box::new(node_info(n, eff)) }, false),
                _ => (
                    Response::Error { code: "DL0904".to_string(), message: format!("no such lease `{node}` (fail closed)"), requires_human: false },
                    false,
                ),
            }
        }
    }
}

// ----- the serve loop ----------------------------------------------------------------------------

/// Run the broker daemon serve loop until a `Shutdown` request (or a fatal transport error). Owns
/// the single `Broker` for this OS user. Blocking, single-connection, one request per connection.
pub fn serve(state_dir: &Path) -> io::Result<()> {
    std::fs::create_dir_all(state_dir)?;
    let key = load_or_create_key(key_path(state_dir))
        .map_err(|e| io::Error::other(format!("broker key: {e}")))?;
    let log = AuditLog::open(audit_dir(state_dir))
        .map_err(|e| io::Error::other(format!("audit log: {e}")))?;
    let failed = Rc::new(Cell::new(false));
    let sink = TrackingSink { inner: log, failed: Rc::clone(&failed) };
    let secrets = SecretStore::load(secrets_path(state_dir));
    let mut broker = Broker::new().with_key(key).with_sink(Box::new(sink));

    let listener = Listener::bind(state_dir)?;
    let pid = std::process::id();
    std::fs::write(pid_path(state_dir), pid.to_string())?;
    eprintln!("delulu broker: serving on `{}` (pid {pid}, state `{}`)", listener.address(), state_dir.display());

    // The serve loop never propagates an error via `?` (transient accept/frame errors `continue`;
    // a `Shutdown` request `break`s), so no IIFE is needed to guarantee the pid-file cleanup below.
    loop {
        let mut conn = match listener.accept() {
            Ok(c) => c,
            Err(e) => {
                // A transient accept error (e.g. a peer that vanished) should not kill the daemon.
                eprintln!("delulu broker: accept error (continuing): {e}");
                continue;
            }
        };
        // Read exactly one request; a malformed/short frame closes this connection (fail closed)
        // without taking down the daemon.
        let req: Request = match read_frame(&mut conn) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("delulu broker: bad frame (dropping connection): {e}");
                continue;
            }
        };
        let (resp, stop) = handle(&mut broker, &secrets, &failed, pid, req);
        if let Err(e) = write_frame(&mut conn, &resp) {
            eprintln!("delulu broker: response write failed: {e}");
        }
        drop(conn);
        if stop {
            break;
        }
    }

    let _ = std::fs::remove_file(pid_path(state_dir));
    Ok(())
}

// ----- the `delulu broker` subcommands -----------------------------------------------------------

/// One quick request/response round-trip to a running daemon (each op is its own short connection,
/// so a `run` in progress and a concurrent `revoke` interleave at request granularity).
pub fn request(state_dir: &Path, body: ReqBody) -> io::Result<Response> {
    let mut conn = broker_transport::connect(state_dir)?;
    write_frame(&mut conn, &Request::new(body))?;
    read_frame(&mut conn)
}

fn parse_state_dir(rest: &[String]) -> Option<String> {
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--state-dir" if i + 1 < rest.len() => return Some(rest[i + 1].clone()),
            s if s.starts_with("--state-dir=") => return Some(s["--state-dir=".len()..].to_string()),
            _ => {}
        }
        i += 1;
    }
    None
}

/// `delulu broker start|status|stop|rotate-key` (spec §2). `start --foreground` runs the serve loop
/// in this process (tests + the detached child); bare `start` spawns a detached child.
pub fn cmd_broker(rest: &[String]) -> i32 {
    let Some(sub) = rest.first().map(String::as_str) else {
        eprintln!("error: `broker` needs a subcommand: start [--foreground] | status | stop | rotate-key [--state-dir DIR]");
        return 2;
    };
    let args = &rest[1..];
    let state_flag = parse_state_dir(args);
    let Some(state_dir) = resolve_state_dir(state_flag.as_deref()) else {
        eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE) — pass --state-dir DIR");
        return 2;
    };
    let json = args.iter().any(|a| a == "--json");

    match sub {
        "start" => {
            let foreground = args.iter().any(|a| a == "--foreground");
            if foreground {
                match serve(&state_dir) {
                    Ok(()) => 0,
                    Err(e) => {
                        eprintln!("error: broker serve loop failed: {e}");
                        2
                    }
                }
            } else {
                start_detached(&state_dir, json)
            }
        }
        "status" => match request(&state_dir, ReqBody::Status) {
            Ok(Response::Status { pid, nodes, epoch }) => {
                if json {
                    println!("{}", serde_json::json!({ "running": true, "pid": pid, "nodes": nodes, "epoch": epoch, "custody": "daemon" }));
                } else {
                    println!("broker running — pid {pid}, {nodes} node(s), epoch {epoch}, custody: daemon");
                }
                0
            }
            Ok(other) => {
                eprintln!("error: unexpected status response: {other:?}");
                2
            }
            Err(_) => {
                if json {
                    println!("{}", serde_json::json!({ "running": false }));
                } else {
                    println!("broker not running (state `{}`) — start it with `delulu broker start`", state_dir.display());
                }
                1
            }
        },
        "stop" => match request(&state_dir, ReqBody::Shutdown) {
            Ok(_) => {
                if !json {
                    eprintln!("ok: broker shutdown requested");
                }
                0
            }
            Err(_) => {
                eprintln!("broker not running (nothing to stop)");
                1
            }
        },
        "rotate-key" => match request(&state_dir, ReqBody::RotateKey) {
            Ok(Response::Ok) => {
                if !json {
                    eprintln!("ok: broker key rotated — outstanding lease tokens are now invalid (DL1407)");
                }
                0
            }
            Ok(other) => {
                eprintln!("error: unexpected rotate-key response: {other:?}");
                2
            }
            Err(_) => {
                eprintln!("broker not running — start it with `delulu broker start`");
                1
            }
        },
        other => {
            eprintln!("error: unknown broker subcommand `{other}` (start | status | stop | rotate-key)");
            2
        }
    }
}

/// Spawn a detached child running the serve loop, then wait until it answers `Status`.
fn start_detached(state_dir: &Path, json: bool) -> i32 {
    // If one is already running, do nothing (idempotent).
    if request(state_dir, ReqBody::Status).is_ok() {
        if !json {
            eprintln!("ok: broker already running (state `{}`)", state_dir.display());
        }
        return 0;
    }
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: cannot locate the delulu executable to spawn the daemon: {e}");
            return 2;
        }
    };
    let mut cmd = std::process::Command::new(exe);
    cmd.args(["broker", "start", "--foreground", "--state-dir"]).arg(state_dir);
    // Give the long-lived daemon a STABLE working directory (the state dir it owns) rather than
    // inheriting the caller's cwd — a process holds a lock on its working directory on Windows, so
    // inheriting the caller's cwd would prevent that directory from being deleted for as long as the
    // broker runs. The daemon needs no relative paths of its own (programs perform their own file
    // effects in their own process after the broker authorizes).
    cmd.current_dir(state_dir);
    // The detached daemon must NOT inherit our stdin/stdout/stderr. A caller that captures our
    // output — `Command::output()`, a shell pipe, CI — reads the inherited pipe until EOF, and EOF
    // never arrives while the long-lived daemon holds the write handle open: the caller hangs
    // forever. Send the daemon's own diagnostics to `<state>/broker.log`; null on any open failure.
    cmd.stdin(std::process::Stdio::null());
    match std::fs::OpenOptions::new().create(true).append(true).open(state_dir.join("broker.log")) {
        Ok(log) => match log.try_clone() {
            Ok(log2) => {
                cmd.stdout(std::process::Stdio::from(log)).stderr(std::process::Stdio::from(log2));
            }
            Err(_) => {
                cmd.stdout(std::process::Stdio::from(log)).stderr(std::process::Stdio::null());
            }
        },
        Err(_) => {
            cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
        }
    }
    detach(&mut cmd);
    let child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: cannot spawn the broker daemon: {e}");
            return 2;
        }
    };
    // Wait (bounded) for the daemon to come up.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while std::time::Instant::now() < deadline {
        if request(state_dir, ReqBody::Status).is_ok() {
            if json {
                println!("{}", serde_json::json!({ "running": true, "pid": child.id(), "custody": "daemon" }));
            } else {
                eprintln!("ok: broker started (pid {}, state `{}`)", child.id(), state_dir.display());
            }
            return 0;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    eprintln!("error: broker daemon did not come up within 5s");
    2
}

#[cfg(windows)]
fn detach(cmd: &mut std::process::Command) {
    use std::os::windows::process::CommandExt as _;
    // Mark THIS process's own std handles non-inheritable before the detached child spawns. Rust
    // spawns with `bInheritHandles = TRUE` (it must, to pass the child's redirected stdio), which
    // would otherwise hand the long-lived daemon copies of OUR stdin/stdout/stderr — including the
    // pipe a caller uses to capture our output (`Command::output()`, a shell pipe, CI). The daemon
    // holding that pipe's write-end open means the caller waits for an EOF that never comes and
    // hangs. (The child's OWN stdio is set explicitly to the log file / null in `start_detached`;
    // those handles are passed independently and are unaffected by this.)
    unsafe { clear_std_handle_inheritance() };
    // DETACHED_PROCESS (0x8) | CREATE_NEW_PROCESS_GROUP (0x200): no inherited console, survives the
    // launching shell.
    cmd.creation_flags(0x0000_0008 | 0x0000_0200);
}

/// Clear the `HANDLE_FLAG_INHERIT` bit on this process's std handles so a subsequently-spawned
/// detached child cannot inherit them (see [`detach`]). Best-effort: a NUL/redirected/invalid std
/// handle simply has nothing to clear.
#[cfg(windows)]
pub(crate) unsafe fn clear_std_handle_inheritance() {
    use windows_sys::Win32::Foundation::{
        SetHandleInformation, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE,
    };
    use windows_sys::Win32::System::Console::{
        GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };
    for id in [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        let h = GetStdHandle(id);
        if !h.is_null() && h != INVALID_HANDLE_VALUE {
            SetHandleInformation(h, HANDLE_FLAG_INHERIT, 0);
        }
    }
}

#[cfg(unix)]
fn detach(_cmd: &mut std::process::Command) {
    // v0.5: the child is a separate process and outlives the parent. Full session detachment
    // (setsid) is a documented post-chunk-3 nicety (no libc dependency pulled in).
}

// ==================================================================================================
// In-process daemon tests: a REAL serve loop on the real transport (pipe/socket), driven from
// another thread of the same process — no spawned process (the one process-spawning test lives in
// tests/broker_cli.rs, playbook 5f). Same-user by construction, so the peer-SID check passes.
// ==================================================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::broker_client::BrokerClientCustody;
    use crate::broker_transport;
    use delulu_broker::Authority;
    use delulu_runtime::{Custody, CustodyDecision};
    use std::collections::BTreeSet;

    fn temp_state(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("delulu_brokerd_{}_{}", std::process::id(), tag));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Start a real serve loop on its own thread; wait (bounded) until it answers Status.
    fn start_daemon(state: &Path) -> std::thread::JoinHandle<()> {
        let dir = state.to_path_buf();
        let handle = std::thread::spawn(move || {
            let _ = serve(&dir);
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while std::time::Instant::now() < deadline {
            if request(state, ReqBody::Status).is_ok() {
                return handle;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        panic!("in-process daemon did not come up within 10s");
    }

    fn stop_daemon(state: &Path, handle: std::thread::JoinHandle<()>) {
        let _ = request(state, ReqBody::Shutdown);
        let _ = handle.join();
    }

    fn spec(effects: &[&str]) -> AuthoritySpec {
        AuthoritySpec {
            effects: effects.iter().map(|s| s.to_string()).collect(),
            fs_write: vec!["./out".to_string()],
            secrets: vec!["API_KEY".to_string()],
            holder_kind: "process".to_string(),
            holder_desc: "brokerd-test".to_string(),
            ..Default::default()
        }
    }

    /// The wire protocol version gate (spec §8): a mismatched version string is answered DL1406 and
    /// never acted on.
    #[test]
    fn version_mismatch_is_dl1406() {
        let state = temp_state("dl1406");
        let handle = start_daemon(&state);

        // Hand-rolled frame with a DELIBERATELY wrong version (`request()` always sends broker/1).
        let mut conn = broker_transport::connect(&state).unwrap();
        let bad = Request { version: "broker/0".to_string(), body: ReqBody::Status };
        write_frame(&mut conn, &bad).unwrap();
        let resp: Response = read_frame(&mut conn).unwrap();
        drop(conn);
        match resp {
            Response::Error { code, message, requires_human } => {
                assert_eq!(code, "DL1406");
                assert!(message.contains("broker/1") && message.contains("broker/0"), "{message}");
                assert!(requires_human);
            }
            other => panic!("expected DL1406, got {other:?}"),
        }

        // The correct version still works on the next connection.
        assert!(matches!(request(&state, ReqBody::Status), Ok(Response::Status { .. })));
        stop_daemon(&state, handle);
        let _ = std::fs::remove_dir_all(&state);
    }

    /// The full daemon-mode custody story in one process: issue → synchronous allow → revoke (as a
    /// concurrent revoker would) → the NEXT synchronous op is DL1403 with the revoking seq; the
    /// epoch class follows within one refresh; after daemon shutdown the next op is DL1401 — fast,
    /// and NEVER an embedded fallback (invariant 27, playbook trap 4).
    #[test]
    fn revoke_mid_run_then_daemon_death_fail_closed() {
        let state = temp_state("midrun");
        let handle = start_daemon(&state);

        let mut custody = BrokerClientCustody::issue_root(
            state.clone(),
            spec(&["Write", "Read"]),
            Some(1), // 1 ms epoch so the refresh window closes immediately in-test (no sleeps > 1ms)
        )
        .expect("issue against the live daemon");

        // Synchronous class allowed while live.
        assert!(matches!(custody.check(Op::FsWrite, Some("./out/a.txt")), CustodyDecision::Allow));

        // A concurrent human/orchestrator revokes the node (spec §3.2: a caller may revoke itself).
        let node = custody.node().as_str().to_string();
        let resp = request(&state, ReqBody::Revoke { caller: node.clone(), target: node.clone() }).unwrap();
        let Response::Revoked { by_seq, .. } = resp else { panic!("expected Revoked, got {resp:?}") };

        // The VERY NEXT synchronous op fails DL1403 carrying the revoking audit seq (spec §4.3).
        match custody.check(Op::FsWrite, Some("./out/a.txt")) {
            CustodyDecision::Deny(d) => {
                assert_eq!(d.code, "DL1403");
                assert!(d.message.contains(&by_seq.to_string()), "DL1403 carries the revoking seq: {}", d.message);
            }
            CustodyDecision::Allow => panic!("a revoked lease must not allow"),
        }
        // The epoch class observes the revocation within ≤ one interval (1 ms here — honest §4.2
        // bound; the refresh happens on the next check, no sleeps needed).
        std::thread::sleep(std::time::Duration::from_millis(3));
        assert!(matches!(custody.check(Op::FsRead, Some("./data")), CustodyDecision::Deny(_)));

        // Kill the daemon mid-run: the next effectful op is DL1401 — fast, never a hang, and there
        // is no embedded path to fall back to.
        stop_daemon(&state, handle);
        let started = std::time::Instant::now();
        match custody.check(Op::FsWrite, Some("./out/a.txt")) {
            CustodyDecision::Deny(d) => {
                assert_eq!(d.code, "DL1401");
                assert!(d.message.contains("delulu broker start"), "exact start command: {}", d.message);
            }
            CustodyDecision::Allow => panic!("invariant 27: a dead broker must never allow"),
        }
        assert!(started.elapsed() < std::time::Duration::from_secs(10), "DL1401 must be fast, not a hang");
        assert_eq!(custody.mode(), "daemon", "mode never silently becomes embedded");
        let _ = std::fs::remove_dir_all(&state);
    }

    /// Delegate → redeem → the redeemed node checks; plus expose through the daemon (phase 5g):
    /// bytes come back ONLY from the broker store, and denied secrets stay denied.
    #[test]
    fn delegate_redeem_and_expose_over_the_wire() {
        let state = temp_state("delegate_expose");
        // Seed the broker-resident secret store BEFORE the daemon starts (it loads at startup).
        SecretStore::load(state.join("secrets.json")).set("API_KEY", "hunter2").unwrap();
        let handle = start_daemon(&state);

        // Issue an orchestrator with Declassify over API_KEY.
        let resp = request(&state, ReqBody::Issue(spec(&["Write", "Declassify"]))).unwrap();
        let Response::Issued { node: orch } = resp else { panic!("expected Issued, got {resp:?}") };

        // Delegate a slice + redeem the token (the orchestration primitive, spec §3.3).
        let child_spec = AuthoritySpec {
            effects: vec!["Write".to_string()],
            fs_write: vec!["./out".to_string()],
            holder_kind: "delegate".to_string(),
            holder_desc: "agent-1".to_string(),
            ..Default::default()
        };
        let resp = request(&state, ReqBody::Delegate { parent: orch.clone(), authority: child_spec, multi: false }).unwrap();
        let Response::Delegated { node: child, token } = resp else { panic!("expected Delegated, got {resp:?}") };
        let resp = request(&state, ReqBody::Redeem { token: token.clone(), peer: "pid:9".to_string() }).unwrap();
        assert!(matches!(&resp, Response::Redeemed { node } if node == &child), "redeem binds the delegated node: {resp:?}");
        // Second redemption of the single-use token → DL1407 over the wire.
        let resp = request(&state, ReqBody::Redeem { token, peer: "pid:10".to_string() }).unwrap();
        assert!(matches!(&resp, Response::Error { code, .. } if code == "DL1407"), "{resp:?}");

        // The redeemed child's synchronous op validates against live tree state.
        let resp = request(&state, ReqBody::Check { node: child.clone(), op: "FsWrite".to_string(), arg: Some("./out/x".to_string()) }).unwrap();
        assert!(matches!(resp, Response::Decision { allow: true, .. }));

        // Expose through the daemon: the orchestrator (Declassify + API_KEY scope) gets the bytes;
        // the child (no Declassify) is denied — bytes never cross for it.
        let resp = request(&state, ReqBody::Expose { node: orch.clone(), name: "API_KEY".to_string(), span: Some("t.delulu:1:2".to_string()) }).unwrap();
        assert!(matches!(&resp, Response::Exposed { bytes } if bytes == "hunter2"), "{resp:?}");
        let resp = request(&state, ReqBody::Expose { node: child, name: "API_KEY".to_string(), span: None }).unwrap();
        assert!(matches!(&resp, Response::Error { .. }), "a node without Declassify+scope must not expose: {resp:?}");

        // SecretMap runs broker-side and the derivative exposes for the authorized node.
        let resp = request(&state, ReqBody::SecretMap { node: orch.clone(), name: "API_KEY".to_string(), op: "upper".to_string(), arg: None }).unwrap();
        let Response::Mapped { name: derived } = resp else { panic!("expected Mapped, got {resp:?}") };
        let resp = request(&state, ReqBody::Expose { node: orch, name: derived, span: None }).unwrap();
        assert!(matches!(&resp, Response::Exposed { bytes } if bytes == "HUNTER2"), "{resp:?}");

        stop_daemon(&state, handle);
        // The audit chain the daemon wrote verifies end-to-end, and the expose records carry spans.
        let stats = delulu_broker::verify(state.join("audit")).expect("daemon audit chain verifies");
        assert!(stats.records >= 8, "issue+delegate+redeems+check+exposes+map all recorded: {stats:?}");
        let exposes = delulu_broker::query(
            state.join("audit"),
            &delulu_broker::QueryFilter { action: Some("expose".to_string()), ..Default::default() },
        )
        .unwrap();
        assert!(exposes.iter().any(|r| r.span.as_deref() == Some("t.delulu:1:2")), "expose audited with the calling span");
        let _ = std::fs::remove_dir_all(&state);
    }

    /// The epoch snapshot path over the wire: NodeState reconstructs a snapshot the client checks
    /// locally (spec §4.1); `for_node` binds custody to an existing node.
    #[test]
    fn epoch_class_over_the_wire_uses_the_cached_snapshot() {
        let state = temp_state("epoch");
        let handle = start_daemon(&state);

        let resp = request(&state, ReqBody::Issue(spec(&["Read"]))).unwrap();
        let Response::Issued { node } = resp else { panic!("expected Issued") };
        let mut authority = Authority::default();
        authority.effects = ["Read"].iter().map(|n| delulu_check::Effect::core_from_name(n).unwrap()).collect::<BTreeSet<_>>();
        authority.scopes.fs_read = ["./data".to_string()].into_iter().collect();

        let mut custody = BrokerClientCustody::for_node(
            state.clone(),
            delulu_broker::GrantId::from_trusted(node),
            authority,
            Some(250), // a LONG window: after the first refresh, checks are local (no round-trip)
        );
        assert!(matches!(custody.check(Op::FsRead, Some("./data/x")), CustodyDecision::Allow));
        // Out-of-scope epoch arg denied LOCALLY by the same Snapshot::check the broker uses.
        assert!(matches!(custody.check(Op::FsRead, Some("./elsewhere")), CustodyDecision::Deny(_)));

        stop_daemon(&state, handle);
        // Within the still-fresh window the cached snapshot answers (the honest ≤ one-interval
        // bound, spec §4.2) — then a stale cache + dead broker is DL1401 (fail closed).
        assert!(matches!(custody.check(Op::FsRead, Some("./data/x")), CustodyDecision::Allow), "fresh cache answers within the epoch window");
        std::thread::sleep(std::time::Duration::from_millis(300)); // let the 250 ms window lapse
        match custody.check(Op::FsRead, Some("./data/x")) {
            CustodyDecision::Deny(d) => assert_eq!(d.code, "DL1401"),
            CustodyDecision::Allow => panic!("a stale cache with a dead broker must fail closed"),
        }
        let _ = std::fs::remove_dir_all(&state);
    }

    /// **Acceptance criterion 2 (spec §9), measured.** Epoch class at the spec-default 50 ms
    /// interval: after a revocation, a tight epoch-class check loop observes the denial within one
    /// epoch interval — asserted with the spec-mandated ×3 CI-jitter tolerance (≤ 150 ms). The
    /// deterministic mechanism twin (no clocks) is `delulu-broker`'s
    /// `epoch_op_passes_on_stale_snapshot_then_fails_after_refresh`; this test adds the real-time
    /// measurement over the real transport. The loop is hard-bounded so a regression is a clean
    /// failure, never a hang.
    #[test]
    fn criterion_2_epoch_revocation_latency_within_one_interval_x3() {
        let state = temp_state("criterion2");
        let handle = start_daemon(&state);

        let mut custody = BrokerClientCustody::issue_root(
            state.clone(),
            AuthoritySpec {
                effects: vec!["Read".to_string()],
                fs_read: vec!["./data".to_string()],
                holder_kind: "process".to_string(),
                holder_desc: "criterion2".to_string(),
                ..Default::default()
            },
            None, // the spec DEFAULT epoch interval (50 ms) — the bound under test
        )
        .expect("issue against the live daemon");

        // Prime the epoch cache so the worst case (a snapshot taken the instant before the revoke)
        // is in play — the honest bound is "within one interval," not "immediately."
        assert!(matches!(custody.check(Op::FsRead, Some("./data/x")), CustodyDecision::Allow));

        // Revoke from "another terminal" (a separate connection), then time the tight loop.
        let node = custody.node().as_str().to_string();
        let resp = request(&state, ReqBody::Revoke { caller: node.clone(), target: node }).unwrap();
        assert!(matches!(resp, Response::Revoked { .. }), "{resp:?}");
        let revoked_at = std::time::Instant::now();

        let deadline = revoked_at + std::time::Duration::from_secs(5); // hard bound: fail, don't hang
        let denial = loop {
            match custody.check(Op::FsRead, Some("./data/x")) {
                CustodyDecision::Deny(d) => break d,
                CustodyDecision::Allow => {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "epoch-class op still allowed 5 s after revocation — the epoch refresh is broken"
                    );
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
            }
        };
        let elapsed = revoked_at.elapsed();
        assert_eq!(denial.code, "DL1403", "the denial is the revocation, not a transport error: {}", denial.message);
        assert!(
            elapsed <= std::time::Duration::from_millis(150),
            "criterion 2: revocation must reach the epoch class within one 50 ms interval (×3 CI \
             jitter tolerance = 150 ms); took {elapsed:?}"
        );

        stop_daemon(&state, handle);
        let _ = std::fs::remove_dir_all(&state);
    }

    /// Phase 5j: the `grants list`/`inspect` read surface over the wire — node detail round-trips,
    /// unknown ids fail closed, and revocation state is folded in.
    #[test]
    fn list_and_inspect_over_the_wire() {
        let state = temp_state("list_inspect");
        let handle = start_daemon(&state);

        let resp = request(&state, ReqBody::Issue(spec(&["Write"]))).unwrap();
        let Response::Issued { node: root } = resp else { panic!("expected Issued, got {resp:?}") };
        let child_spec = AuthoritySpec {
            effects: vec!["Write".to_string()],
            fs_write: vec!["./out".to_string()],
            holder_kind: "delegate".to_string(),
            holder_desc: "agent-7".to_string(),
            ..Default::default()
        };
        let resp = request(&state, ReqBody::Attenuate { parent: root.clone(), authority: child_spec }).unwrap();
        let Response::Issued { node: child } = resp else { panic!("expected Issued, got {resp:?}") };

        // List: both nodes, sorted by id, with parent linkage.
        let resp = request(&state, ReqBody::List).unwrap();
        let Response::Listed { nodes } = resp else { panic!("expected Listed, got {resp:?}") };
        assert_eq!(nodes.len(), 2);
        assert!(nodes.windows(2).all(|w| w[0].id <= w[1].id), "sorted by id");
        let c = nodes.iter().find(|n| n.id == child).unwrap();
        assert_eq!(c.parent.as_deref(), Some(root.as_str()));
        assert_eq!(c.state, "live");
        assert_eq!(c.holder_desc, "agent-7");
        assert_eq!(c.fs_write, vec!["./out".to_string()]);

        // Inspect an unknown id → fail closed (DL0904), never an empty success.
        let resp = request(&state, ReqBody::Inspect { node: "g_0000000000000000000000000000dead".into() }).unwrap();
        assert!(matches!(&resp, Response::Error { code, .. } if code == "DL0904"), "{resp:?}");

        // Revoke the child; inspect folds the effective state + the revoking seq.
        let resp = request(&state, ReqBody::Revoke { caller: child.clone(), target: child.clone() }).unwrap();
        let Response::Revoked { by_seq, .. } = resp else { panic!("expected Revoked, got {resp:?}") };
        let resp = request(&state, ReqBody::Inspect { node: child }).unwrap();
        let Response::Inspected { node } = resp else { panic!("expected Inspected, got {resp:?}") };
        assert_eq!(node.state, "revoked");
        assert_eq!(node.by_seq, Some(by_seq), "inspect carries the revoking audit seq (spec §4.3)");

        stop_daemon(&state, handle);
        let _ = std::fs::remove_dir_all(&state);
    }
}
