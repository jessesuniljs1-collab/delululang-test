//! The runtime primitive table (spec §7.3): the execution half of the effect truth. Every
//! capability operation validates its scope host-side on every use (invariant 7 / §5.15.6),
//! independent of the compile-time check. Closure-taking methods (`map`) are handled in the
//! interpreter, which can call back into evaluation; everything else lives here.

use std::path::{Component, Path, PathBuf};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use delulu_check::ResourceKind;
use delulu_diag::Span;

use crate::value::{CapScope, CapVal, Fault, RootVal, SecretVal, Value};

/// A granted filesystem root, resolved to an absolute normalized path (used by the broker).
pub fn granted_root(rel: &str) -> PathBuf {
    normalize(&std::env::current_dir().unwrap_or_default().join(rel))
}

/// Resolve `rel` inside `root` with the SAME lexical `.`/`..` normalization the interpreter and the
/// broker use — no filesystem access (Stage 5 phase 5f). Public so the custody gate can present the
/// broker the exact resolved-path string its node's fs scope was granted against.
pub fn resolve_norm(root: &Path, rel: &str) -> PathBuf {
    normalize(&root.join(rel))
}

/// Lexically normalize a path, resolving `.` and `..` without touching the filesystem.
fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Resolve `rel` inside a capability's filesystem scope, refusing escapes (DL0904).
fn resolve_in_scope(root: &Path, rel: &str, span: Span) -> Result<PathBuf, Fault> {
    let joined = normalize(&root.join(rel));
    if joined.starts_with(root) {
        Ok(joined)
    } else {
        Err(Fault::at("DL0904", format!("path `{rel}` escapes the granted scope"), span))
    }
}

fn host_allowed(url: &str, allow: &[String]) -> bool {
    // Extract host from an https URL; match exact or `*.suffix` patterns.
    let host = url.strip_prefix("https://").unwrap_or(url);
    let host = host.split(['/', ':']).next().unwrap_or(host);
    allow.iter().any(|pat| {
        if let Some(suffix) = pat.strip_prefix("*.") {
            host.ends_with(suffix)
        } else {
            host == pat
        }
    })
}

// ----- Root: mint capabilities (attenuation — no effect) -------------------

pub fn call_root_method(root: &RootVal, method: &str, args: &[Value], span: Span) -> Result<Value, Fault> {
    // DL0703 is the first error nearly every newcomer meets: zero ambient authority means the
    // very first program that prints anything fails until a human grants the console. The
    // refusal is correct and is the point of the language — but a refusal that does not say
    // what to type teaches nothing, so every message below names the exact grant. (Prose only:
    // the code, the span, and the `--json` envelope are unchanged, per `for-agents.md`.)
    let refused = |what: &str, grant: &str| {
        Fault::at("DL0703", format!("`{what}` was not granted to this program — pass `--grant {grant}`"), span)
    };
    match method {
        "console" => {
            if root.console {
                Ok(cap(ResourceKind::Console, CapScope::Console))
            } else {
                Err(refused("console", "console"))
            }
        }
        "fs_read" => {
            let p = str_arg(args, 0, span)?;
            let want = normalize(&std::env::current_dir().unwrap_or_default().join(&p));
            match root.fs_read.iter().find(|granted| want.starts_with(granted.as_path())) {
                Some(_) => Ok(cap(ResourceKind::FsRead, CapScope::Fs { root: want, write: false })),
                None => Err(Fault::at("DL0703", format!("filesystem read of `{p}` was not granted — pass `--grant fs.read={p}`"), span)),
            }
        }
        "fs_write" => {
            let p = str_arg(args, 0, span)?;
            let want = normalize(&std::env::current_dir().unwrap_or_default().join(&p));
            match root.fs_write.iter().find(|granted| want.starts_with(granted.as_path())) {
                Some(_) => Ok(cap(ResourceKind::FsWrite, CapScope::Fs { root: want, write: true })),
                None => Err(Fault::at("DL0703", format!("filesystem write of `{p}` was not granted — pass `--grant fs.write={p}`"), span)),
            }
        }
        "http" => {
            let hosts = list_str_arg(args, 0, span)?;
            if hosts.iter().all(|h| root.net.iter().any(|g| g == h)) {
                Ok(cap(ResourceKind::Http, CapScope::Net { allow: hosts }))
            } else {
                Err(refused("network host", "net=HOST"))
            }
        }
        "clock" => {
            if root.clock {
                Ok(cap(ResourceKind::Clock, CapScope::Clock))
            } else {
                Err(refused("clock", "clock"))
            }
        }
        "rand" => {
            if root.rand {
                Ok(cap(ResourceKind::Rand, CapScope::Rand))
            } else {
                Err(refused("rand", "rand"))
            }
        }
        "declassify" => {
            if root.declassify {
                let names = root.secrets.keys().cloned().collect();
                Ok(cap(ResourceKind::Declassify, CapScope::Declassify { names }))
            } else {
                Err(refused("declassify", "declassify"))
            }
        }
        "secret" => {
            let name = str_arg(args, 0, span)?;
            // Daemon mode (Stage 5 phase 5g): a broker-held secret returns an opaque HANDLE — no
            // bytes cross into this process here (invariant 23); `expose` fetches them later.
            if root.broker_secrets.iter().any(|n| n == &name) {
                return Ok(Value::Secret(Rc::new(SecretVal::handle(name))));
            }
            match root.secrets.get(&name) {
                Some(v) => Ok(Value::Secret(Rc::new(SecretVal::new(v.clone())))),
                None => Err(Fault::at("DL0703", format!("secret `{name}` was not granted — pass `--grant secret:{name}=VALUE` (or `secret:{name}=env:VAR`)"), span)),
            }
        }
        "foreign_load" => {
            if root.foreign_load {
                Ok(cap(ResourceKind::ForeignLoad, CapScope::ForeignLoad))
            } else {
                Err(Fault::at("DL0703", "foreign loading was not granted (grant a `foreign.c` lib or `foreign.python`)", span))
            }
        }
        // T-Py binding (spec §5.1): `root.python(load: Cap[ForeignLoad]) -> Result[Cap[Python],
        // ForeignErr]`, PURE like `root.foreign` — deriving the handle is not an effect; *using* it
        // is. An ungranted `foreign.python` is `Err(NotGranted)` (DL1303's runtime face, behind the
        // CLI startup refusal); an interpreter that will not start is `Err(Unavailable)` (DL1307).
        // The interpreter is prepared lazily HERE, on this first grant-checked call.
        "python" => {
            if root.python_allowlist.is_empty() {
                return Ok(Value::err(Value::variant("NotGranted", vec![])));
            }
            match crate::python::ensure_available() {
                Ok(()) => Ok(Value::ok(cap(
                    ResourceKind::Python,
                    CapScope::Python { allowlist: root.python_allowlist.clone() },
                ))),
                Err(reason) => Ok(Value::err(Value::variant("Unavailable", vec![Value::str(reason)]))),
            }
        }
        // Stage 10 (10e): physical-device mints. Both are pure attenuation (deriving the handle is
        // not an effect; *using* it is), and both are deny-by-default: no matching grant, no cap —
        // the envelope/device list on `RootVal` is the whole authority story (spec §5.1).
        "actuator" => {
            let d = str_arg(args, 0, span)?;
            match root.actuators.iter().find(|e| e.device == d) {
                Some(e) => Ok(cap(ResourceKind::Actuator, CapScope::Actuator(e.clone()))),
                None => Err(Fault::at("DL0703", format!("actuator `{d}` was not granted — pass `--grant \"actuator={d}:DIM=LO..HI\"` with the envelope this machine may move in"), span)),
            }
        }
        "sensor" => {
            let d = str_arg(args, 0, span)?;
            if root.sensors.iter().any(|s| s == &d) {
                Ok(cap(ResourceKind::Sensor, CapScope::Sensor { device: d }))
            } else {
                Err(Fault::at("DL0703", format!("sensor `{d}` was not granted — pass `--grant sensor={d}`"), span))
            }
        }
        // Stage 10 (10h): the same shape for an accelerator. Deriving the handle is pure; the
        // `ForeignCall` effect is in dispatching through it.
        "compute" => {
            let d = str_arg(args, 0, span)?;
            match root.computes.iter().find(|e| e.device == d) {
                Some(e) => Ok(cap(ResourceKind::Compute, CapScope::Compute(e.clone()))),
                None => Err(Fault::at("DL0703", format!("compute device `{d}` was not granted — pass `--grant \"compute={d}:memory_bytes=N,...\"`"), span)),
            }
        }
        "plugin_host" => Err(Fault::at("DL0703", "plugin hosting is not available in the Stage-1 runtime", span)),
        _ => Err(Fault::at("DL0907", format!("unknown Root method `{method}` (checker bug)"), span)),
    }
}

fn cap(kind: ResourceKind, scope: CapScope) -> Value {
    Value::Cap(Rc::new(CapVal { kind, scope }))
}

// ----- capability operations (the effects themselves) ----------------------

pub fn call_cap_method(capv: &CapVal, method: &str, args: &[Value], span: Span) -> Result<Value, Fault> {
    match (capv.kind, method) {
        (ResourceKind::Console, "println") => {
            emit_console(&str_arg(args, 0, span)?, true);
            Ok(Value::Unit)
        }
        (ResourceKind::Console, "print") => {
            emit_console(&str_arg(args, 0, span)?, false);
            Ok(Value::Unit)
        }
        (ResourceKind::Console, "readline") => {
            let mut line = String::new();
            match std::io::stdin().read_line(&mut line) {
                Ok(0) => Ok(Value::err(io_err("NotFound"))),
                Ok(_) => Ok(Value::ok(Value::str(line.trim_end_matches(['\n', '\r']).to_string()))),
                Err(_) => Ok(Value::err(io_err("Other"))),
            }
        }
        (ResourceKind::FsRead, "read_text") => {
            let CapScope::Fs { root, .. } = &capv.scope else { return Err(scope_bug(span)) };
            let p = resolve_in_scope(root, &str_arg(args, 0, span)?, span)?;
            match std::fs::read_to_string(&p) {
                Ok(s) => Ok(Value::ok(Value::str(s))),
                Err(e) => Ok(Value::err(io_err_for(&e))),
            }
        }
        (ResourceKind::FsRead, "list_dir") => {
            let CapScope::Fs { root, .. } = &capv.scope else { return Err(scope_bug(span)) };
            let p = resolve_in_scope(root, &str_arg(args, 0, span)?, span)?;
            match std::fs::read_dir(&p) {
                Ok(rd) => {
                    let names: Vec<Value> = rd
                        .filter_map(|e| e.ok())
                        .map(|e| Value::str(e.file_name().to_string_lossy().to_string()))
                        .collect();
                    Ok(Value::ok(Value::List(Rc::new(std::cell::RefCell::new(names)))))
                }
                Err(e) => Ok(Value::err(io_err_for(&e))),
            }
        }
        (ResourceKind::FsRead, "narrow") => {
            let CapScope::Fs { root, write } = &capv.scope else { return Err(scope_bug(span)) };
            let sub = resolve_in_scope(root, &str_arg(args, 0, span)?, span)?;
            Ok(cap(ResourceKind::FsRead, CapScope::Fs { root: sub, write: *write }))
        }
        (ResourceKind::FsWrite, "write_text") | (ResourceKind::FsWrite, "append_text") => {
            let CapScope::Fs { root, .. } = &capv.scope else { return Err(scope_bug(span)) };
            let p = resolve_in_scope(root, &str_arg(args, 0, span)?, span)?;
            let body = str_arg(args, 1, span)?;
            let res = if method == "append_text" {
                use std::io::Write as _;
                std::fs::OpenOptions::new().create(true).append(true).open(&p).and_then(|mut f| f.write_all(body.as_bytes()))
            } else {
                std::fs::write(&p, body.as_bytes())
            };
            match res {
                Ok(()) => Ok(Value::ok(Value::Unit)),
                Err(e) => Ok(Value::err(io_err_for(&e))),
            }
        }
        (ResourceKind::Http, "get") => {
            let CapScope::Net { allow } = &capv.scope else { return Err(scope_bug(span)) };
            let url = str_arg(args, 0, span)?;
            if !url.starts_with("https://") {
                return Ok(Value::err(net_err("Refused")));
            }
            if !host_allowed(&url, allow) {
                return Err(Fault::at("DL0904", format!("host of `{url}` is not in the granted allowlist"), span));
            }
            // Stage-1 runtime bundles no network client; the authority path is what matters.
            Ok(Value::err(net_err("Refused")))
        }
        (ResourceKind::Clock, "now_ms") => {
            let ms = FIXED_CLOCK.with(|c| c.get()).unwrap_or_else(|| {
                SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
            });
            Ok(Value::Int(ms))
        }
        (ResourceKind::Rand, "int") => {
            let lo = int_arg(args, 0, span)?;
            let hi = int_arg(args, 1, span)?;
            if hi <= lo {
                return Err(Fault::at("DL0904", "rand.int requires lo < hi", span));
            }
            Ok(Value::Int(lo + (next_rand() % (hi - lo) as u64) as i64))
        }
        (ResourceKind::Rand, "float") => Ok(Value::Float((next_rand() as f64) / (u64::MAX as f64))),
        // Stage 10 (10e): the null sensor adapter — a read that cannot observe anything says so
        // honestly: `Err(NoDevice)`, never a fabricated measurement (invariant 50). 10f gave the
        // interpreter its own `Sensor` arm so a bound adapter can answer, and that arm returns
        // exactly this when no adapter is attached. This one stays as the floor for any caller
        // that reaches `call_cap_method` without a broker: the fallback for "no adapter" must be
        // absence in every path that can produce a reading, not just the one we remembered.
        (ResourceKind::Sensor, "read") => Ok(Value::err(Value::variant("NoDevice", vec![]))),
        _ => Err(Fault::at("DL0907", format!("unknown capability method `{method}` (checker bug)"), span)),
    }
}

pub fn call_secret_method(secret: &SecretVal, method: &str, args: &[Value], span: Span) -> Result<Value, Fault> {
    match method {
        "verify" => match args.first() {
            Some(Value::Secret(other)) => Ok(Value::Bool(secret.verify(other))),
            _ => Err(Fault::at("DL0907", "Secret.verify expects a Secret argument", span)),
        },
        // The only reader: expose. Reaching here means the checker admitted Cap[Declassify].
        "expose" => Ok(Value::str(secret.reveal())),
        _ => Err(Fault::at("DL0907", format!("unknown Secret method `{method}` (checker bug)"), span)),
    }
}

pub fn call_str_method(s: &str, method: &str, args: &[Value], span: Span) -> Result<Value, Fault> {
    match method {
        "len" => Ok(Value::Int(s.chars().count() as i64)),
        "trim" => Ok(Value::str(s.trim().to_string())),
        "contains" => Ok(Value::Bool(s.contains(&str_arg(args, 0, span)?))),
        "starts_with" => Ok(Value::Bool(s.starts_with(&str_arg(args, 0, span)?))),
        "split" => {
            let sep = str_arg(args, 0, span)?;
            let parts: Vec<Value> = if sep.is_empty() {
                s.chars().map(|c| Value::str(c.to_string())).collect()
            } else {
                s.split(&sep).map(|p| Value::str(p.to_string())).collect()
            };
            Ok(Value::List(Rc::new(std::cell::RefCell::new(parts))))
        }
        "slice" => {
            let lo = int_arg(args, 0, span)?.max(0) as usize;
            let hi = int_arg(args, 1, span)?.max(0) as usize;
            let chars: Vec<char> = s.chars().collect();
            let lo = lo.min(chars.len());
            let hi = hi.min(chars.len()).max(lo);
            Ok(Value::str(chars[lo..hi].iter().collect::<String>()))
        }
        _ => Err(Fault::at("DL0907", format!("unknown Str method `{method}`"), span)),
    }
}

pub fn call_list_method(list: &Rc<std::cell::RefCell<Vec<Value>>>, method: &str, args: &[Value], span: Span) -> Result<Value, Fault> {
    match method {
        "len" => Ok(Value::Int(list.borrow().len() as i64)),
        "get" => {
            let i = int_arg(args, 0, span)?;
            match list.borrow().get(i as usize) {
                Some(v) => Ok(Value::variant("Some", vec![v.clone()])),
                None => Ok(Value::variant("None", vec![])),
            }
        }
        "push" => {
            if let Some(v) = args.first() {
                list.borrow_mut().push(v.clone());
            }
            Ok(Value::Unit)
        }
        _ => Err(Fault::at("DL0907", format!("unknown List method `{method}`"), span)),
    }
}

/// Free builtins (§11). Returns None when `name` is not a builtin.
pub fn call_builtin(name: &str, args: &[Value], span: Span) -> Option<Result<Value, Fault>> {
    Some(match name {
        "Ok" => Ok(Value::ok(args.first().cloned().unwrap_or(Value::Unit))),
        "Err" => Ok(Value::err(args.first().cloned().unwrap_or(Value::Unit))),
        "Some" => Ok(Value::variant("Some", vec![args.first().cloned().unwrap_or(Value::Unit)])),
        "None" => Ok(Value::variant("None", vec![])),
        "str" => Ok(Value::str(args.first().map(|v| v.display()).unwrap_or_default())),
        "len" => Ok(match args.first() {
            Some(Value::Str(s)) => Value::Int(s.chars().count() as i64),
            Some(Value::List(l)) => Value::Int(l.borrow().len() as i64),
            _ => Value::Int(0),
        }),
        "int" => Ok(match args.first() {
            Some(Value::Float(f)) => Value::Int(*f as i64),
            Some(Value::Int(i)) => Value::Int(*i),
            _ => Value::Int(0),
        }),
        "float" => Ok(match args.first() {
            Some(Value::Int(i)) => Value::Float(*i as f64),
            Some(Value::Float(f)) => Value::Float(*f),
            _ => Value::Float(0.0),
        }),
        "parse_int" => Ok(match args.first() {
            Some(Value::Str(s)) => match s.trim().parse::<i64>() {
                Ok(i) => Value::variant("Some", vec![Value::Int(i)]),
                Err(_) => Value::variant("None", vec![]),
            },
            _ => Value::variant("None", vec![]),
        }),
        // The float half of `parse_int`, and deliberately the SAME rule the lexer applies to a
        // literal (`delulu_syntax::num`) rather than a second one: text that names a magnitude
        // `Float` cannot hold is `None`, not a silent `inf` or `0.0`. That also closes the way in
        // for the non-finite words — without it, a data file containing `inf` could put infinity
        // into a program whose source is not allowed to write it.
        "parse_float" => Ok(match args.first() {
            Some(Value::Str(s)) => match delulu_syntax::num::float_from_text(s.trim()) {
                delulu_syntax::num::FloatText::Value(f) => {
                    Value::variant("Some", vec![Value::Float(f)])
                }
                _ => Value::variant("None", vec![]),
            },
            _ => Value::variant("None", vec![]),
        }),
        "range" => {
            let lo = args.first().and_then(as_int).unwrap_or(0);
            let hi = args.get(1).and_then(as_int).unwrap_or(0);
            let items: Vec<Value> = (lo..hi).map(Value::Int).collect();
            Ok(Value::List(Rc::new(std::cell::RefCell::new(items))))
        }
        "push" => {
            if let (Some(Value::List(l)), Some(v)) = (args.first(), args.get(1)) {
                l.borrow_mut().push(v.clone());
                Ok(Value::Unit)
            } else {
                Err(Fault::at("DL0907", "push expects (List, value)", span))
            }
        }
        // Stage 8 (phase 8a, spec §2): assertion failure is a PANIC — the fault message
        // carries the compared values, the span carries file/line (the 8g runner lifts both
        // into the structured JSON failure payload). Only non-opaque values reach here: the
        // checker refuses `assert_eq` on opaque types (DL0605, R-5) before anything runs.
        "assert" => match args.first() {
            Some(Value::Bool(true)) => Ok(Value::Unit),
            Some(Value::Bool(false)) => Err(Fault::at("DL1707", "assertion failed", span)),
            _ => Err(Fault::at("DL0907", "assert expects a Bool condition", span)),
        },
        "assert_eq" => match (args.first(), args.get(1)) {
            (Some(a), Some(b)) if a.eq(b) => Ok(Value::Unit),
            (Some(a), Some(b)) => Err(Fault::at(
                "DL1707",
                format!("assertion failed: `{}` != `{}`", a.display(), b.display()),
                span,
            )),
            _ => Err(Fault::at("DL0907", "assert_eq expects two values", span)),
        },
        _ => return None,
    })
}

// ----- argument helpers ----------------------------------------------------

fn str_arg(args: &[Value], i: usize, span: Span) -> Result<String, Fault> {
    match args.get(i) {
        Some(Value::Str(s)) => Ok(s.to_string()),
        _ => Err(Fault::at("DL0907", format!("expected a Str argument at position {i}"), span)),
    }
}
fn int_arg(args: &[Value], i: usize, span: Span) -> Result<i64, Fault> {
    match args.get(i) {
        Some(Value::Int(n)) => Ok(*n),
        _ => Err(Fault::at("DL0907", format!("expected an Int argument at position {i}"), span)),
    }
}
fn list_str_arg(args: &[Value], i: usize, span: Span) -> Result<Vec<String>, Fault> {
    match args.get(i) {
        Some(Value::List(l)) => Ok(l.borrow().iter().map(|v| v.display()).collect()),
        _ => Err(Fault::at("DL0907", "expected a List[Str] argument", span)),
    }
}
fn as_int(v: &Value) -> Option<i64> {
    match v {
        Value::Int(n) => Some(*n),
        _ => None,
    }
}

fn io_err(variant: &str) -> Value {
    Value::variant(variant, if variant == "Other" { vec![Value::str("error")] } else { vec![] })
}
fn net_err(variant: &str) -> Value {
    Value::variant(variant, vec![])
}
fn io_err_for(e: &std::io::Error) -> Value {
    use std::io::ErrorKind::*;
    match e.kind() {
        NotFound => io_err("NotFound"),
        PermissionDenied => io_err("Denied"),
        _ => Value::variant("Other", vec![Value::str(e.to_string())]),
    }
}
fn scope_bug(span: Span) -> Fault {
    Fault::at("DL0907", "capability scope mismatch (checker bug)", span)
}

// A small, non-cryptographic xorshift PRNG for `Cap[Rand]`. Deterministic replay (spec §6.2):
// `set_rand_seed` reseeds this thread-local generator so two runs with the same seed produce
// identical `Cap[Rand]` sequences (required by the fuzz harness and agent debugging loops).
use std::cell::Cell;
thread_local! {
    static RNG: Cell<u64> = Cell::new(seed());
    static FIXED_CLOCK: Cell<Option<i64>> = const { Cell::new(None) };
    /// When `Some`, `Cap[Console]` output is captured into this buffer instead of stdout — used by
    /// tests and by the WASM backend's two-engine parity harness (§9). Off by default.
    static CAPTURE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Turn Console output capture on (fresh empty buffer) or off (restore stdout).
pub fn set_capture(on: bool) {
    CAPTURE.with(|c| *c.borrow_mut() = if on { Some(String::new()) } else { None });
}

/// Take and clear the captured Console output, if capture is on.
pub fn take_capture() -> Option<String> {
    CAPTURE.with(|c| c.borrow_mut().take())
}

/// Emit a Console string: to the capture buffer if capturing, else to real stdout.
fn emit_console(s: &str, newline: bool) {
    let captured = CAPTURE.with(|c| {
        let mut b = c.borrow_mut();
        if let Some(buf) = b.as_mut() {
            buf.push_str(s);
            if newline {
                buf.push('\n');
            }
            true
        } else {
            false
        }
    });
    if !captured {
        if newline {
            println!("{s}");
        } else {
            print!("{s}");
        }
    }
}
fn seed() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x9E3779B9) | 1
}
fn next_rand() -> u64 {
    RNG.with(|r| {
        let mut x = r.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        r.set(x);
        x
    })
}

/// Deterministic replay (spec §6.2): reseed `Cap[Rand]`'s thread-local xorshift generator. Two
/// runs on the same thread that call `set_rand_seed` with the same value produce identical
/// `rand.int`/`rand.float` sequences; different seeds (almost always) diverge. The xorshift
/// generator gets stuck at zero, so a zero seed is mapped to a fixed nonzero value, matching the
/// `| 1` bias `seed()` already applies for the non-deterministic default.
pub fn set_rand_seed(seed: u64) {
    let s = if seed == 0 { 0x9E3779B9 } else { seed } | 1;
    RNG.with(|r| r.set(s));
}

/// Deterministic replay (spec §6.2): fix `Cap[Clock].now_ms()` to always return `ms` on this
/// thread. `None` restores the real wall clock.
pub fn set_fixed_clock_ms(ms: Option<i64>) {
    FIXED_CLOCK.with(|c| c.set(ms));
}

#[cfg(test)]
mod determinism_tests {
    use super::*;

    #[test]
    fn same_seed_reproduces_the_same_rand_sequence() {
        set_rand_seed(42);
        let a: Vec<u64> = (0..8).map(|_| next_rand()).collect();
        set_rand_seed(42);
        let b: Vec<u64> = (0..8).map(|_| next_rand()).collect();
        assert_eq!(a, b, "same seed must reproduce the same xorshift sequence");
    }

    #[test]
    fn different_seeds_diverge() {
        set_rand_seed(1);
        let a: Vec<u64> = (0..8).map(|_| next_rand()).collect();
        set_rand_seed(2);
        let b: Vec<u64> = (0..8).map(|_| next_rand()).collect();
        assert_ne!(a, b, "different seeds should (overwhelmingly likely) diverge");
    }

    #[test]
    fn zero_seed_is_mapped_to_a_nonzero_deterministic_seed() {
        // The xorshift generator is a fixed point at zero (0 ^ ... == 0 forever); a literal
        // `--seed 0` must not silently produce an all-zero, non-random-looking sequence.
        set_rand_seed(0);
        let first = next_rand();
        assert_ne!(first, 0);
        set_rand_seed(0);
        let again = next_rand();
        assert_eq!(first, again, "seed 0 must still be deterministic");
    }

    #[test]
    fn fixed_clock_overrides_the_cap_clock_now_ms_primitive() {
        set_fixed_clock_ms(Some(1_700_000_000_123));
        let cap = CapVal { kind: ResourceKind::Clock, scope: CapScope::Clock };
        let span = Span::new(0, 0, 0);
        let v = call_cap_method(&cap, "now_ms", &[], span).expect("now_ms should not fault");
        set_fixed_clock_ms(None);
        match v {
            Value::Int(ms) => assert_eq!(ms, 1_700_000_000_123),
            other => panic!("expected Int, got {other:?}"),
        }
    }

    #[test]
    fn clearing_the_fixed_clock_restores_the_wall_clock() {
        set_fixed_clock_ms(Some(1));
        set_fixed_clock_ms(None);
        let cap = CapVal { kind: ResourceKind::Clock, scope: CapScope::Clock };
        let span = Span::new(0, 0, 0);
        let before = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        let v = call_cap_method(&cap, "now_ms", &[], span).expect("now_ms should not fault");
        match v {
            Value::Int(ms) => assert!((ms - before).abs() < 60_000, "expected a real wall-clock value, got {ms}"),
            other => panic!("expected Int, got {other:?}"),
        }
    }
}
