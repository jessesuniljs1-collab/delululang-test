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
    let refused = |what: &str| Fault::at("DL0703", format!("`{what}` was not granted to this program"), span);
    match method {
        "console" => {
            if root.console {
                Ok(cap(ResourceKind::Console, CapScope::Console))
            } else {
                Err(refused("console"))
            }
        }
        "fs_read" => {
            let p = str_arg(args, 0, span)?;
            let want = normalize(&std::env::current_dir().unwrap_or_default().join(&p));
            match root.fs_read.iter().find(|granted| want.starts_with(granted.as_path())) {
                Some(_) => Ok(cap(ResourceKind::FsRead, CapScope::Fs { root: want, write: false })),
                None => Err(Fault::at("DL0703", format!("filesystem read of `{p}` was not granted"), span)),
            }
        }
        "fs_write" => {
            let p = str_arg(args, 0, span)?;
            let want = normalize(&std::env::current_dir().unwrap_or_default().join(&p));
            match root.fs_write.iter().find(|granted| want.starts_with(granted.as_path())) {
                Some(_) => Ok(cap(ResourceKind::FsWrite, CapScope::Fs { root: want, write: true })),
                None => Err(Fault::at("DL0703", format!("filesystem write of `{p}` was not granted"), span)),
            }
        }
        "http" => {
            let hosts = list_str_arg(args, 0, span)?;
            if hosts.iter().all(|h| root.net.iter().any(|g| g == h)) {
                Ok(cap(ResourceKind::Http, CapScope::Net { allow: hosts }))
            } else {
                Err(refused("network host"))
            }
        }
        "clock" => {
            if root.clock {
                Ok(cap(ResourceKind::Clock, CapScope::Clock))
            } else {
                Err(refused("clock"))
            }
        }
        "rand" => {
            if root.rand {
                Ok(cap(ResourceKind::Rand, CapScope::Rand))
            } else {
                Err(refused("rand"))
            }
        }
        "declassify" => {
            if root.declassify {
                let names = root.secrets.keys().cloned().collect();
                Ok(cap(ResourceKind::Declassify, CapScope::Declassify { names }))
            } else {
                Err(refused("declassify"))
            }
        }
        "secret" => {
            let name = str_arg(args, 0, span)?;
            match root.secrets.get(&name) {
                Some(v) => Ok(Value::Secret(Rc::new(SecretVal::new(v.clone())))),
                None => Err(Fault::at("DL0703", format!("secret `{name}` was not granted"), span)),
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
            println!("{}", str_arg(args, 0, span)?);
            Ok(Value::Unit)
        }
        (ResourceKind::Console, "print") => {
            print!("{}", str_arg(args, 0, span)?);
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
            let ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
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

// A small, non-cryptographic xorshift PRNG for `Cap[Rand]` (deterministic seeding lands Stage 2).
use std::cell::Cell;
thread_local! {
    static RNG: Cell<u64> = Cell::new(seed());
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
