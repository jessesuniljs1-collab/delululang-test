//! The DeluluLang package registry (Stage 9g, spec §5).
//!
//! ## The one rule that matters
//! **A publisher cannot claim an authority they do not carry.** The index line's authority summary
//! is **recomputed server-side from the uploaded artifact** — never copied from what the client
//! sent. A registry that trusts the publisher's own authority claim is a registry whose authority
//! column is decoration, and the whole point of `delulu add` showing an authority diff *before*
//! downloading anything is that the column is load-bearing.
//!
//! ## Policies implemented here
//! - **Scoped, revocable tokens.** A token names the packages it may publish. No token grants yank
//!   on someone else's package, ever.
//! - **Yank ≠ delete.** A yanked version stops being resolvable for *new* requirements and keeps
//!   resolving from existing lockfiles. Deleting a version breaks builds that were working; yanking
//!   stops the bleeding without doing that.
//! - **Mandatory signature on publish.** An unsigned artifact is refused.
//! - **Server-side authority recomputation** (above), including a doctored index line submitted by
//!   a client being discarded rather than corrected — the server never merges attacker-supplied
//!   fields into a record it vouches for.
//! - **Outage degrades to lockfiles.** The registry being down must never break an existing build;
//!   that property lives in the client and is tested there.
//!
//! ## Why the HTTP is hand-rolled
//! Dependency austerity (house rule 7). The surface needed is small and completely specified: five
//! routes, one verb each, JSON bodies. Stage 8's LSP made the same call for JSON-RPC. A web
//! framework here would be a large dependency added to a security-critical component in exchange
//! for saving about two hundred lines.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

pub mod token;

/// How the registry re-derives an artifact's authority. `None` means "cannot tell", which the
/// registry treats as a refusal — see [`Registry::publish`].
pub type Recompute = Arc<dyn Fn(&[u8]) -> Option<Value> + Send + Sync>;

/// The registry's on-disk state: a sparse index plus the artifacts it vouches for.
pub struct Registry {
    root: PathBuf,
    tokens: Mutex<token::Store>,
}

/// One publish attempt, as it arrives from a client.
///
/// Grouped into a struct rather than passed as seven positional arguments, because the two that
/// matter most — `artifact` and `claimed` — are `&[u8]` and `Option<&Value>` and would be easy to
/// transpose at a call site. The type makes the mistake unwritable.
#[derive(Clone, Copy)]
pub struct Submission<'a> {
    pub token: &'a str,
    pub name: &'a str,
    pub version: &'a str,
    /// The bytes the registry will vouch for. **The only evidence** for the authority it records.
    pub artifact: &'a [u8],
    pub signature: Option<&'a [u8]>,
    /// What the publisher *says* the authority is. Compared against the truth, never trusted as it.
    pub claimed: Option<&'a Value>,
}

/// What a publish attempt resolved to.
#[derive(Debug, PartialEq, Eq)]
pub enum PublishOutcome {
    Accepted { name: String, version: String },
    /// Refused, with the diagnostic code and the reason a human needs.
    Refused { code: &'static str, reason: String },
}

impl Registry {
    pub fn open(root: impl Into<PathBuf>) -> std::io::Result<Registry> {
        let root = root.into();
        std::fs::create_dir_all(root.join("index"))?;
        std::fs::create_dir_all(root.join("artifacts"))?;
        std::fs::create_dir_all(root.join("advisories"))?;
        let tokens = token::Store::load(&root.join("tokens.jsonl"))?;
        Ok(Registry { root, tokens: Mutex::new(tokens) })
    }

    pub fn index_dir(&self) -> PathBuf {
        self.root.join("index")
    }

    /// Every line for a package, oldest first. Absent package = empty.
    pub fn lines(&self, name: &str) -> Vec<Value> {
        let path = self.index_dir().join(name);
        let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect()
    }

    /// The line a *new* requirement should resolve to: the latest **unyanked** version.
    ///
    /// Yanked versions are skipped here and still readable through [`Registry::lines`], which is
    /// exactly the yank ≠ delete split: new resolutions move on, existing lockfiles still find
    /// what they pinned.
    pub fn resolve(&self, name: &str) -> Option<Value> {
        self.lines(name).into_iter().rfind(|l| !l["yanked"].as_bool().unwrap_or(false))
    }

    /// A specific version, yanked or not — the path a lockfile takes.
    pub fn resolve_exact(&self, name: &str, version: &str) -> Option<Value> {
        self.lines(name).into_iter().find(|l| l["version"].as_str() == Some(version))
    }

    /// Publish an artifact from a [`Submission`].
    ///
    /// `submission.claimed` is whatever the client sent. It is used for **nothing** except being
    /// compared against the truth, so a mismatch can be reported. The record the registry writes is
    /// built only from what `recompute` derives from the artifact.
    pub fn publish_submission(
        &self,
        sub: &Submission<'_>,
        recompute: impl Fn(&[u8]) -> Option<Value>,
    ) -> PublishOutcome {
        let Submission { token: token_str, name, version, artifact, signature, claimed } = *sub;
        // ----- authorization -------------------------------------------------
        let tokens = self.tokens.lock().expect("token store");
        match tokens.authorize(token_str, name) {
            token::Authorization::Ok => {}
            token::Authorization::Unknown => {
                return PublishOutcome::Refused {
                    code: "DL1706",
                    reason: "unknown or revoked token".into(),
                }
            }
            token::Authorization::OutOfScope => {
                return PublishOutcome::Refused {
                    code: "DL1706",
                    reason: format!("this token is not scoped to publish `{name}`"),
                }
            }
        }
        drop(tokens);

        // ----- mandatory signature -------------------------------------------
        let Some(sig) = signature else {
            return PublishOutcome::Refused {
                code: "DL1705",
                reason: "publish requires a signature; the artifact is unsigned".into(),
            };
        };
        match delulu_runtime::verify_detached(artifact, sig) {
            delulu_runtime::SignatureStatus::Valid { .. } => {}
            delulu_runtime::SignatureStatus::Unsigned => {
                return PublishOutcome::Refused {
                    code: "DL1705",
                    reason: "publish requires a signature; the artifact is unsigned".into(),
                }
            }
            delulu_runtime::SignatureStatus::Invalid { reason } => {
                return PublishOutcome::Refused {
                    code: "DL1705",
                    reason: format!("signature does not verify: {reason}"),
                }
            }
        }

        // ----- SERVER-SIDE AUTHORITY RECOMPUTATION ---------------------------
        // The artifact is the only evidence. If the server cannot derive an authority from it, the
        // publish is REFUSED — never accepted with the publisher's claim as a stand-in. "Could not
        // tell" must fail closed, or the recomputation is theatre.
        let Some(truth) = recompute(artifact) else {
            return PublishOutcome::Refused {
                code: "DL1706",
                reason: "the registry could not derive an authority from the uploaded artifact; \
                         publish refused rather than trusting the submitted claim"
                    .into(),
            };
        };

        // A mismatch is reported, because a publisher sending a different authority than their
        // artifact carries is worth knowing about — but the recomputed value is what gets stored
        // either way.
        if let Some(claimed) = claimed {
            if claimed.get("effects") != truth.get("effects") {
                return PublishOutcome::Refused {
                    code: "DL1706",
                    reason: format!(
                        "submitted authority {} does not match the authority recomputed from the \
                         artifact {}",
                        claimed.get("effects").unwrap_or(&json!([])),
                        truth.get("effects").unwrap_or(&json!([]))
                    ),
                };
            }
        }

        // ----- immutability ---------------------------------------------------
        // Checked BEFORE the semver rule: a version that already exists is refused for being
        // taken, whatever its authority. Ordering matters for the message a publisher reads —
        // "authority widened" would send them to bump the version they are already colliding with.
        if self.resolve_exact(name, version).is_some() {
            return PublishOutcome::Refused {
                code: "DL1706",
                reason: format!("{name} {version} is already published; versions are immutable"),
            };
        }

        // ----- semver-authority ----------------------------------------------
        if let Some(prior) = self.resolve(name) {
            let prior_effects = effect_set(&prior);
            let new_effects = effect_set(&truth);
            let widened: Vec<&String> =
                new_effects.iter().filter(|e| !prior_effects.contains(*e)).collect();
            let prior_v = prior["version"].as_str().unwrap_or("0.0.0");
            if !widened.is_empty() && same_major(prior_v, version) {
                return PublishOutcome::Refused {
                    code: "DL1003",
                    reason: format!(
                        "version {version} adds authority {widened:?} over {prior_v} without a \
                         major bump — authority widening is semver-major"
                    ),
                };
            }
        }

        // ----- write ----------------------------------------------------------
        let line = json!({
            "name": name,
            "version": version,
            "effects": truth.get("effects").cloned().unwrap_or(json!([])),
            "capabilities": truth.get("capabilities").cloned().unwrap_or(json!([])),
            "secrets": truth.get("secrets").cloned().unwrap_or(json!([])),
            "yanked": false,
            "authority_source": "recomputed-server-side",
        });
        let index_path = self.index_dir().join(name);
        let mut existing = std::fs::read_to_string(&index_path).unwrap_or_default();
        if !existing.is_empty() && !existing.ends_with('\n') {
            existing.push('\n');
        }
        existing.push_str(&line.to_string());
        existing.push('\n');
        if std::fs::write(&index_path, existing).is_err() {
            return PublishOutcome::Refused { code: "DL1706", reason: "index write failed".into() };
        }
        let _ = std::fs::write(self.root.join("artifacts").join(format!("{name}-{version}.dwx")), artifact);
        let _ = std::fs::write(
            self.root.join("artifacts").join(format!("{name}-{version}.dwx.sig")),
            sig,
        );

        PublishOutcome::Accepted { name: name.into(), version: version.into() }
    }

    /// Yank a version. **Never deletes.** The line stays, flagged, so lockfiles keep resolving.
    pub fn yank(&self, token_str: &str, name: &str, version: &str, yanked: bool) -> PublishOutcome {
        let tokens = self.tokens.lock().expect("token store");
        match tokens.authorize(token_str, name) {
            token::Authorization::Ok => {}
            token::Authorization::Unknown => {
                return PublishOutcome::Refused { code: "DL1706", reason: "unknown or revoked token".into() }
            }
            // The policy in one line: a token never grants yank on someone else's package.
            token::Authorization::OutOfScope => {
                return PublishOutcome::Refused {
                    code: "DL1706",
                    reason: format!("this token is not scoped to `{name}`; yank refused"),
                }
            }
        }
        drop(tokens);

        let path = self.index_dir().join(name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            return PublishOutcome::Refused { code: "DL1706", reason: "no such package".into() };
        };
        let mut found = false;
        let mut out = String::new();
        for l in text.lines().filter(|l| !l.trim().is_empty()) {
            let mut v: Value = match serde_json::from_str(l) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if v["version"].as_str() == Some(version) {
                v["yanked"] = json!(yanked);
                found = true;
            }
            out.push_str(&v.to_string());
            out.push('\n');
        }
        if !found {
            return PublishOutcome::Refused { code: "DL1706", reason: "no such version".into() };
        }
        let _ = std::fs::write(&path, out);
        PublishOutcome::Accepted { name: name.into(), version: version.into() }
    }

    pub fn issue_token(&self, owner: &str, scopes: Vec<String>) -> String {
        let mut t = self.tokens.lock().expect("token store");
        t.issue(owner, scopes, &self.root.join("tokens.jsonl"))
    }

    pub fn revoke_token(&self, tok: &str) -> bool {
        let mut t = self.tokens.lock().expect("token store");
        t.revoke(tok, &self.root.join("tokens.jsonl"))
    }

    // ----- the advisory feed (Stage 10 phase 10k, Track C, spec §4) --------------------------
    //
    // An advisory ties a package to a set of affected version strings and, where there is one, the
    // patched version to upgrade to. Stored per package as JSONL under `advisories/<name>`,
    // mirroring the index exactly — one append-only file per package, one JSON record per line.
    //
    // `delulu build` reads this feed (synced to a local file) and warns (DL1903) when a resolved
    // dependency is on an affected version; `--deny-advisories` turns the warning into a CI gate.
    // The registry is the *source* of that feed; the build is deliberately offline against a local
    // copy, so the registry being down never silences an advisory a build already holds.

    /// Every advisory recorded for a package, oldest first. Absent package = empty.
    pub fn advisories(&self, name: &str) -> Vec<Value> {
        let path = self.root.join("advisories").join(name);
        let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect()
    }

    /// Every advisory across every package, for exporting a whole-feed file a build can consult.
    pub fn all_advisories(&self) -> Vec<Value> {
        let mut out = Vec::new();
        let Ok(entries) = std::fs::read_dir(self.root.join("advisories")) else { return out };
        let mut names: Vec<String> = entries
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .collect();
        names.sort(); // stable export order — a feed file diffs cleanly
        for name in names {
            out.extend(self.advisories(&name));
        }
        out
    }

    /// File an advisory against a package. Requires a token **scoped to that package** — the same
    /// standing `yank` requires, and for the same reason: the feed is load-bearing, and a token
    /// with no authority over a package must not be able to plant (or, by extension, shape) an
    /// advisory that will fail other people's builds. The record is written from only the fields
    /// the registry vouches for; a submission missing `id`, `package`, or a non-empty `affected`
    /// list is refused rather than stored as a half-record a build would then have to guess about.
    pub fn file_advisory(&self, token_str: &str, advisory: &Value) -> PublishOutcome {
        let Some(package) = advisory.get("package").and_then(Value::as_str).filter(|s| !s.is_empty())
        else {
            return PublishOutcome::Refused {
                code: "DL1706",
                reason: "an advisory must name a `package`".into(),
            };
        };

        let tokens = self.tokens.lock().expect("token store");
        match tokens.authorize(token_str, package) {
            token::Authorization::Ok => {}
            token::Authorization::Unknown => {
                return PublishOutcome::Refused { code: "DL1706", reason: "unknown or revoked token".into() }
            }
            token::Authorization::OutOfScope => {
                return PublishOutcome::Refused {
                    code: "DL1706",
                    reason: format!("this token is not scoped to `{package}`; advisory refused"),
                }
            }
        }
        drop(tokens);

        let id = advisory.get("id").and_then(Value::as_str).filter(|s| !s.is_empty());
        let affected = advisory
            .get("affected")
            .and_then(Value::as_array)
            .filter(|xs| !xs.is_empty() && xs.iter().all(Value::is_string));
        let (Some(id), Some(affected)) = (id, affected) else {
            return PublishOutcome::Refused {
                code: "DL1706",
                reason: "an advisory needs a non-empty `id` and a non-empty `affected` list of \
                         version strings"
                    .into(),
            };
        };

        // Only the vouched-for fields are stored — an unknown extra field a client sends is dropped,
        // never merged into a record the registry signs its name to (the same discipline `publish`
        // takes toward a client-supplied index line).
        let line = json!({
            "id": id,
            "package": package,
            "affected": affected,
            "patched": advisory.get("patched").and_then(Value::as_str),
            "severity": advisory.get("severity").and_then(Value::as_str).unwrap_or("unknown"),
            "summary": advisory.get("summary").and_then(Value::as_str).unwrap_or(""),
        });

        let path = self.root.join("advisories").join(package);
        let mut existing = std::fs::read_to_string(&path).unwrap_or_default();
        if !existing.is_empty() && !existing.ends_with('\n') {
            existing.push('\n');
        }
        existing.push_str(&line.to_string());
        existing.push('\n');
        if std::fs::write(&path, existing).is_err() {
            return PublishOutcome::Refused { code: "DL1706", reason: "advisory write failed".into() };
        }
        PublishOutcome::Accepted { name: package.to_string(), version: id.to_string() }
    }
}

fn effect_set(line: &Value) -> Vec<String> {
    line.get("effects")
        .and_then(Value::as_array)
        .map(|xs| xs.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// Same major (semver), with cargo's 0.x rule: under 0.x the MINOR is the compatibility axis.
pub fn same_major(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> (u64, u64) {
        let mut it = v.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
        (it.next().unwrap_or(0), it.next().unwrap_or(0))
    };
    let (am, an) = parts(a);
    let (bm, bn) = parts(b);
    if am == 0 || bm == 0 { am == bm && an == bn } else { am == bm }
}

// ===== the HTTP surface =====================================================

/// What one connection may take (RW 4.36, the red-team pass on FRAME-DRIP-1). The port is reachable by
/// anyone who can reach it — no token is needed to be read — and the server used to give each connection
/// all the time and memory it asked for: one at a time, lines read with no deadline or cap, and a body
/// buffer allocated at whatever `Content-Length` said. One idle connection held every other client, and
/// one number (`18446744073709551615`) took the server down.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// The longest one read may wait: a client that goes quiet is dropped after it.
    pub read: std::time::Duration,
    /// The whole request — line, headers and body — from the connection's acceptance.
    pub request: std::time::Duration,
    /// The longest request line or header line, in bytes.
    pub line: u64,
    /// How many headers a request may carry.
    pub headers: usize,
    /// The largest body. A `Content-Length` past it is answered 413 before anything is allocated.
    pub body: usize,
    /// Connections served at once; one past it is closed at once.
    pub connections: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            read: std::time::Duration::from_secs(10),
            request: std::time::Duration::from_secs(60),
            line: 8 * 1024,
            headers: 64,
            // As the sandbox channel's frame bound: a package's artifact, in a JSON string, and room.
            body: 16 * 1024 * 1024,
            connections: 64,
        }
    }
}

/// Serve with the default [`Limits`]. Returns the bound address so a test can talk to it without
/// guessing a port.
pub fn serve(
    reg: Arc<Registry>,
    addr: &str,
    recompute: Recompute,
) -> std::io::Result<(std::net::SocketAddr, std::thread::JoinHandle<()>)> {
    serve_with(reg, addr, recompute, Limits::default())
}

/// Serve until the listener fails. Each connection is read on a thread of its own, within `limits`, so a
/// slow or silent client delays nobody else; the requests themselves are handled one at a time, under
/// one lock, as they always were — a publish reads and rewrites a package's index, and two at once
/// would race.
pub fn serve_with(
    reg: Arc<Registry>,
    addr: &str,
    recompute: Recompute,
    limits: Limits,
) -> std::io::Result<(std::net::SocketAddr, std::thread::JoinHandle<()>)> {
    let listener = TcpListener::bind(addr)?;
    let bound = listener.local_addr()?;
    let handling = Arc::new(Mutex::new(()));
    let live = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let handle = std::thread::spawn(move || {
        for stream in listener.incoming() {
            let s = match stream {
                Ok(s) => s,
                Err(_) => break,
            };
            if live.fetch_add(1, std::sync::atomic::Ordering::SeqCst) >= limits.connections {
                live.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                drop(s);
                continue;
            }
            let (reg, recompute, handling, done) = (reg.clone(), recompute.clone(), handling.clone(), live.clone());
            let spawned = std::thread::Builder::new().name("registry-conn".into()).spawn(move || {
                // A panic stays on its connection's thread; the server goes on.
                let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    let _ = handle_conn(&reg, s, &recompute, &handling, &limits);
                }));
                done.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            });
            if spawned.is_err() {
                live.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            }
        }
    });
    Ok((bound, handle))
}

/// A refusal of the request's shape, answered in the protocol's own words before the connection closes.
fn refuse(stream: &mut TcpStream, status: &str, why: &str) -> std::io::Result<()> {
    let body = json!({ "code": "DL1706", "error": why }).to_string();
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    stream.flush()
}

/// One line, at most `cap` bytes: `Ok(None)` when it ran past the cap with no end.
fn bounded_line(reader: &mut impl BufRead, cap: u64) -> std::io::Result<Option<String>> {
    let mut raw = Vec::new();
    let n = reader.take(cap).read_until(b'\n', &mut raw)?;
    if n as u64 == cap && raw.last() != Some(&b'\n') {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&raw).into_owned()))
}

fn handle_conn(
    reg: &Registry,
    mut stream: TcpStream,
    recompute: &Recompute,
    handling: &Mutex<()>,
    limits: &Limits,
) -> std::io::Result<()> {
    stream.set_read_timeout(Some(limits.read))?;
    stream.set_write_timeout(Some(limits.read))?;
    let mut raw = stream.try_clone()?;
    // The whole request within its bound, not each read of it (FRAME-DRIP-1's lesson).
    let mut within = delulu_runtime::channel::Within::from_now(&mut raw, limits.request);
    let mut reader = BufReader::new(&mut within);
    let Some(request_line) = bounded_line(&mut reader, limits.line)? else {
        return refuse(&mut stream, "431 Request Header Fields Too Large", "the request line is longer than the registry reads");
    };
    if request_line.is_empty() {
        return Ok(());
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("/").to_string();

    let mut headers: BTreeMap<String, String> = BTreeMap::new();
    let mut count = 0usize;
    loop {
        let Some(line) = bounded_line(&mut reader, limits.line)? else {
            return refuse(&mut stream, "431 Request Header Fields Too Large", "a header line is longer than the registry reads");
        };
        let t = line.trim_end();
        if t.is_empty() {
            break;
        }
        count += 1;
        if count > limits.headers {
            return refuse(&mut stream, "431 Request Header Fields Too Large", "more headers than the registry reads");
        }
        if let Some((k, v)) = t.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    let len: usize = match headers.get("content-length") {
        None => 0,
        Some(v) => match v.parse::<u64>() {
            Ok(n) if n <= limits.body as u64 => n as usize,
            Ok(_) => return refuse(&mut stream, "413 Payload Too Large", "the body is larger than the registry takes"),
            Err(_) => return refuse(&mut stream, "400 Bad Request", "Content-Length is not a length"),
        },
    };
    let mut body = vec![0u8; len];
    if len > 0 {
        reader.read_exact(&mut body)?;
    }
    let token = headers.get("authorization").map(|a| a.trim_start_matches("Bearer ").to_string()).unwrap_or_default();

    let (status, payload) = {
        let _one_at_a_time = handling.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        route(reg, &method, &path, &body, &token, recompute)
    };
    let body_bytes = payload.to_string().into_bytes();
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body_bytes.len()
    )?;
    stream.write_all(&body_bytes)?;
    stream.flush()
}

fn route(
    reg: &Registry,
    method: &str,
    path: &str,
    body: &[u8],
    token: &str,
    recompute: &Recompute,
) -> (&'static str, Value) {
    match (method, path) {
        // The sparse index: exactly the Stage-8 format, one JSON line per version.
        ("GET", p) if p.starts_with("/index/") => {
            let name = p.trim_start_matches("/index/");
            let lines = reg.lines(name);
            if lines.is_empty() {
                return ("404 Not Found", json!({"code": "DL1706", "error": "no such package"}));
            }
            ("200 OK", json!({ "name": name, "lines": lines }))
        }
        ("POST", "/publish") => {
            let Ok(req) = serde_json::from_slice::<Value>(body) else {
                return ("400 Bad Request", json!({"code": "DL1706", "error": "malformed request"}));
            };
            let name = req["name"].as_str().unwrap_or("");
            let version = req["version"].as_str().unwrap_or("");
            let artifact = req["artifact"].as_str().unwrap_or("").as_bytes().to_vec();
            let sig = req["signature"].as_str().and_then(hex_decode);
            let claimed = req.get("authority");
            let sub = Submission {
                token,
                name,
                version,
                artifact: &artifact,
                signature: sig.as_deref(),
                claimed,
            };
            match reg.publish_submission(&sub, |a| recompute(a)) {
                PublishOutcome::Accepted { name, version } => {
                    ("200 OK", json!({"published": name, "version": version}))
                }
                PublishOutcome::Refused { code, reason } => {
                    ("400 Bad Request", json!({"code": code, "error": reason}))
                }
            }
        }
        ("POST", "/yank") => {
            let Ok(req) = serde_json::from_slice::<Value>(body) else {
                return ("400 Bad Request", json!({"code": "DL1706", "error": "malformed request"}));
            };
            let yanked = req["yanked"].as_bool().unwrap_or(true);
            match reg.yank(token, req["name"].as_str().unwrap_or(""), req["version"].as_str().unwrap_or(""), yanked) {
                PublishOutcome::Accepted { name, version } => {
                    ("200 OK", json!({"yanked": yanked, "name": name, "version": version}))
                }
                PublishOutcome::Refused { code, reason } => {
                    ("400 Bad Request", json!({"code": code, "error": reason}))
                }
            }
        }
        // The advisory feed (10k). GET is public — a build must be able to fetch advisories with
        // no token, exactly as it fetches the index — and returns `[]` for a package with none
        // (not 404: "this package has no advisories" is a first-class, safe answer a build acts on,
        // unlike "no such package" which the index reserves 404 for).
        ("GET", p) if p.starts_with("/advisories/") => {
            let name = p.trim_start_matches("/advisories/");
            ("200 OK", json!({ "name": name, "advisories": reg.advisories(name) }))
        }
        ("POST", "/advisory") => {
            let Ok(req) = serde_json::from_slice::<Value>(body) else {
                return ("400 Bad Request", json!({"code": "DL1706", "error": "malformed request"}));
            };
            match reg.file_advisory(token, &req) {
                PublishOutcome::Accepted { name, version } => {
                    ("200 OK", json!({"advisory": version, "package": name}))
                }
                PublishOutcome::Refused { code, reason } => {
                    ("400 Bad Request", json!({"code": code, "error": reason}))
                }
            }
        }
        ("GET", "/health") => ("200 OK", json!({"status": "ok"})),
        _ => ("404 Not Found", json!({"code": "DL1706", "error": "no such route"})),
    }
}

pub fn hex_decode(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

pub fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Read an index line from a local directory (the offline path a client falls back to).
pub fn read_local_line(index_dir: &Path, name: &str) -> Option<Value> {
    let text = std::fs::read_to_string(index_dir.join(name)).ok()?;
    text.lines().rfind(|l| !l.trim().is_empty()).and_then(|l| serde_json::from_str(l).ok())
}

#[cfg(test)]
mod tests;
