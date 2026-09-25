//! The egress client (PS-B-02, owner ruling D-V2-30): the first code in DeluluLang that sends a byte
//! over the network, and the ONLY code that does.
//!
//! # Where it runs, and why that is the whole design
//!
//! `prim::call_cap_method` is the single host-side place an effect happens. At L0 the interpreter
//! reaches it through `LocalSink`; a sandboxed guest reaches it through `HostChannel::decide`, which
//! performs the guest's `CapMethod` request **in the host**. So calling this module from the
//! `(Http, "get")` arm serves L0 and every guest from one implementation *by construction* — there is
//! no second path to keep in step, and a guest has no resolver for the same reason it has no socket:
//! it never reaches this code.
//!
//! # What one request is held to
//!
//! Every hop — the first, and every redirect — goes through the WHOLE of [`get_with`]:
//!
//! 1. **The spelling.** The URL is parsed by [`parse_target`], which accepts only a form that no URL
//!    parser would rewrite: `https://` exactly, printable ASCII, no backslash, no userinfo, a host in
//!    lower-case LDH form or a canonical address literal, a canonical port. Anything else is refused,
//!    not normalized. This matters because the HTTP client normalizes host names itself (IDNA, via
//!    `url` — the ICU stack D-V2-30 recorded), *after* the allowlist check would otherwise have been
//!    made on the string the program wrote. That is the P22 search key exactly: a security decision
//!    made on one spelling and acted on under another.
//! 2. **The allowlist**, on that parsed host, with the C85 dot-boundary rule ([`crate::prim::host_matches`],
//!    the one function the capability check also uses).
//! 3. **Resolution, once, here.** Every address the name resolves to is classified with
//!    [`crate::netclass::addr_class`]. If ANY of them is special-use and the host was not granted with
//!    `net.special=`, the request is refused — a name that resolves partly into a private range is a
//!    name somebody pointed there.
//! 4. **The pin.** The client is handed the classified addresses and cannot resolve anything itself:
//!    its resolver refuses every lookup, and the pinned addresses are an override for exactly the
//!    checked host. So the host that was checked is the host that is connected to *by construction*,
//!    and the TLS server name and `Host` header are that same checked name.
//! 5. **Real TLS.** rustls, the platform trust store, verification always on. TLS is never
//!    implemented here (house rule 5), and there is no plain-HTTP path at all (D-V2-30).
//! 6. **Redirects are followed by this loop, not by the client**, so a redirect to another host, to
//!    `http:`, or to a name that resolves into a private range is refused exactly as a first request
//!    would be.
//! 7. **The body is bounded on what the program receives.** No decompressor is compiled in (the
//!    refused features in `Cargo.toml`), so the bytes counted are the bytes delivered.
//!
//! # What the program learns, and what only the host learns
//!
//! The program sees `NetErr`'s three variants and nothing else. `Refused` is deliberately OPAQUE:
//! telling a guest "that name resolved into a special-use range" would hand it DNS results it has no
//! resolver for — a resolver oracle rebuilt out of error messages. The machine-readable [`Reason`]
//! goes to the host-side record ([`take_log`]), which the run report and the sandbox channel's
//! `denied[]` carry. `Other` carries only a fixed vocabulary chosen here, never text from the far end
//! or from a resolver.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The response-size ceiling a request is held to unless the operator's limits say otherwise.
/// Eight MiB: large enough for an API response or a page, small enough that a hostile server cannot
/// make the host hold an unbounded string on the program's behalf.
pub const DEFAULT_MAX_BODY: u64 = 8 * 1024 * 1024;
/// Redirect hops followed before the request is refused. Browsers follow twenty; a program asking
/// one API for one resource has no business with more than a handful, and every hop is a fresh
/// resolution and a fresh connection.
pub const DEFAULT_MAX_REDIRECTS: u32 = 5;
/// The whole request — every resolution, connection, handshake and body read, across every hop.
pub const DEFAULT_DEADLINE: Duration = Duration::from_secs(30);
/// How many egress records a run keeps. Past this the counts still rise but nothing more is stored,
/// so a program cannot make the host allocate without limit by making requests in a loop.
pub const MAX_RECORDED: usize = 64;

/// The ceilings and the authority one request is held to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EgressPolicy {
    /// Host patterns the capability names — each exact, or `*.suffix` with the dot boundary.
    pub allow: Vec<String>,
    /// The subset of `allow` granted with `net.special=`: the only hosts that may be, or resolve to,
    /// a special-use address.
    pub allow_special: Vec<String>,
    pub max_body: u64,
    pub max_redirects: u32,
    pub deadline: Duration,
}

impl EgressPolicy {
    /// The policy a `Cap[Http]` carries, with the default ceilings.
    pub fn for_capability(allow: &[String], special: &[String]) -> Self {
        EgressPolicy {
            allow: allow.to_vec(),
            allow_special: special.to_vec(),
            max_body: DEFAULT_MAX_BODY,
            max_redirects: DEFAULT_MAX_REDIRECTS,
            deadline: DEFAULT_DEADLINE,
        }
    }
}

/// Why a request did not deliver a body — machine-readable, host-side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reason {
    /// Not `https://` — including `http:`, and a redirect that tried to downgrade to it.
    Scheme,
    /// The authority carries userinfo (`user@host`). Refused outright rather than parsed: in RFC 3986
    /// everything before the last `@` is userinfo, and C86 was a host check fooled by exactly that.
    Userinfo,
    /// A spelling some URL parser would rewrite; the payload names the rule.
    Malformed(&'static str),
    /// The host is not in the capability's allowlist (a redirect's target, typically — the program's
    /// own first request is refused earlier, as a `DL0904` fault).
    NotAllowlisted,
    /// The host is, or resolved to, a special-use address, and was not granted with `net.special=`.
    SpecialUse(&'static str),
    /// The name resolved to nothing.
    NoAddress,
    TooManyRedirects,
    /// A 3xx answer with no usable `Location`.
    RedirectWithoutLocation,
    /// The response was larger than the policy allows.
    BodyTooLarge,
    /// The body is not UTF-8, and `http.get` answers a `Str`.
    NotUtf8,
    /// The server answered, with a status that is neither success nor a redirect.
    Status(u16),
    /// The TLS handshake failed — an untrusted or invalid certificate among the causes.
    Tls,
    /// The TCP connection could not be made.
    Connect,
    Timeout,
    /// The server spoke something that is not HTTP/1.1 as the client understands it.
    Protocol,
    /// The client would have connected to something other than what was checked. Impossible by
    /// construction; checked anyway, because "impossible" is a claim and this is where it is tested.
    HostMismatch,
    /// This build has no network client (`--no-default-features`).
    NoClient,
}

/// What the program is told — `NetErr`'s three variants, and a body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Body(String),
    Refused,
    Timeout,
    /// A fixed phrase chosen here; never text from the server or a resolver.
    Other(String),
}

impl Reason {
    /// The stable, machine-readable code — what the run report and the channel's `denied[]` carry.
    pub fn code(&self) -> &'static str {
        match self {
            Reason::Scheme => "scheme",
            Reason::Userinfo => "userinfo",
            Reason::Malformed(_) => "malformed",
            Reason::NotAllowlisted => "not-allowlisted",
            Reason::SpecialUse(_) => "special-use",
            Reason::NoAddress => "no-address",
            Reason::TooManyRedirects => "too-many-redirects",
            Reason::RedirectWithoutLocation => "redirect-without-location",
            Reason::BodyTooLarge => "body-too-large",
            Reason::NotUtf8 => "not-utf8",
            Reason::Status(_) => "status",
            Reason::Tls => "tls",
            Reason::Connect => "connect",
            Reason::Timeout => "timeout",
            Reason::Protocol => "protocol",
            Reason::HostMismatch => "host-mismatch",
            Reason::NoClient => "no-client",
        }
    }

    /// A sentence for the operator. Host-side only — the program is told [`Reason::answer`].
    pub fn explain(&self) -> String {
        match self {
            Reason::Scheme => "only `https://` is fetched; there is no plain-HTTP path".into(),
            Reason::Userinfo => "the URL carries userinfo (`user@host`), which is refused rather than parsed".into(),
            Reason::Malformed(why) => format!("the URL is refused rather than normalized: {why}"),
            Reason::NotAllowlisted => "the host is not in the capability's allowlist".into(),
            Reason::SpecialUse(class) => format!(
                "the host is, or resolved to, a special-use address ({class}); a host reaches one only when it is granted with `--grant net.special=HOST`"
            ),
            Reason::NoAddress => "the host name resolved to no address".into(),
            Reason::TooManyRedirects => format!("more than {DEFAULT_MAX_REDIRECTS} redirects"),
            Reason::RedirectWithoutLocation => "a redirect with no usable `Location` header".into(),
            Reason::BodyTooLarge => "the response is larger than the size limit".into(),
            Reason::NotUtf8 => "the response body is not UTF-8".into(),
            Reason::Status(s) => format!("the server answered HTTP status {s}"),
            Reason::Tls => "the TLS handshake failed (an untrusted or invalid certificate among the causes)".into(),
            Reason::Connect => "the connection could not be made".into(),
            Reason::Timeout => format!("the request did not finish within {} s", DEFAULT_DEADLINE.as_secs()),
            Reason::Protocol => "the server's answer is not HTTP/1.1 this client understands".into(),
            Reason::HostMismatch => "the client would have reached a host other than the one checked".into(),
            Reason::NoClient => "this build has no network client (built without the `net` feature)".into(),
        }
    }

    /// What the program sees. Every POLICY decision is the opaque `Refused`, so special-use and
    /// no-address are indistinguishable to it (the resolver-oracle argument above).
    pub fn answer(&self) -> Answer {
        match self {
            Reason::Scheme
            | Reason::Userinfo
            | Reason::Malformed(_)
            | Reason::NotAllowlisted
            | Reason::SpecialUse(_)
            | Reason::NoAddress
            | Reason::TooManyRedirects
            | Reason::HostMismatch
            | Reason::NoClient => Answer::Refused,
            Reason::Timeout => Answer::Timeout,
            Reason::RedirectWithoutLocation => Answer::Other("redirect without a location".into()),
            Reason::BodyTooLarge => Answer::Other("response too large".into()),
            Reason::NotUtf8 => Answer::Other("response is not UTF-8".into()),
            Reason::Status(s) => Answer::Other(format!("HTTP status {s}")),
            Reason::Tls => Answer::Other("TLS handshake failed".into()),
            Reason::Connect => Answer::Other("connection failed".into()),
            Reason::Protocol => Answer::Other("protocol error".into()),
        }
    }
}

// ----- the spelling ----------------------------------------------------------------------------

/// A URL that passed [`parse_target`]: exactly the string requested, and its parts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    /// The URL as written — nothing in it would be rewritten by a URL parser.
    pub url: String,
    /// The host as it is compared and connected: lower-case LDH, a dotted-quad, or `[v6]` bracketed
    /// (the spelling `prim::host_of` and the `url` crate also produce).
    pub host: String,
    pub port: u16,
    /// Present when the host is an address literal — nothing is resolved for it.
    pub ip: Option<IpAddr>,
    /// Byte offset in `url` where the authority ends (the path, query or fragment begins).
    authority_end: usize,
}

/// Parse `url` into a [`Target`], refusing every spelling a URL parser would rewrite.
pub fn parse_target(url: &str) -> Result<Target, Reason> {
    // Order matters only for which reason is reported; every branch refuses.
    let Some(rest) = url.strip_prefix("https://") else {
        return Err(Reason::Scheme);
    };
    if let Some(b) = url.bytes().find(|b| !(0x21..=0x7e).contains(b)) {
        return Err(Reason::Malformed(if b < 0x21 || b == 0x7f {
            "it contains a space or a control character, which URL parsers strip or reject"
        } else {
            "it contains a non-ASCII byte; percent-encode it, and spell an international host in its `xn--` form"
        }));
    }
    if url.contains('\\') {
        return Err(Reason::Malformed("it contains a backslash, which URL parsers read as `/`"));
    }
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..end];
    if authority.contains('@') {
        return Err(Reason::Userinfo);
    }
    if authority.is_empty() {
        return Err(Reason::Malformed("the host is empty"));
    }
    let (host_text, port_text) = if let Some(inner) = authority.strip_prefix('[') {
        let Some(close) = inner.find(']') else {
            return Err(Reason::Malformed("an IPv6 literal is missing its `]`"));
        };
        let after = &inner[close + 1..];
        let port = match after.strip_prefix(':') {
            Some(p) => Some(p),
            None if after.is_empty() => None,
            None => return Err(Reason::Malformed("text follows an IPv6 literal")),
        };
        (&authority[..close + 2], port)
    } else {
        match authority.split_once(':') {
            Some((h, p)) => (h, Some(p)),
            None => (authority, None),
        }
    };
    let port = match port_text {
        None => 443,
        Some(p) => parse_port(p)?,
    };
    let ip = parse_host(host_text)?;
    Ok(Target {
        url: url.to_string(),
        host: host_text.to_string(),
        port,
        ip,
        authority_end: "https://".len() + end,
    })
}

fn parse_port(p: &str) -> Result<u16, Reason> {
    const WHY: &str = "the port is not a canonical number from 1 to 65535";
    if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) || (p.len() > 1 && p.starts_with('0')) {
        return Err(Reason::Malformed(WHY));
    }
    match p.parse::<u16>() {
        Ok(0) | Err(_) => Err(Reason::Malformed(WHY)),
        Ok(n) => Ok(n),
    }
}

/// `Some(ip)` for an address literal, `None` for a name; `Err` for any spelling that would be rewritten.
fn parse_host(h: &str) -> Result<Option<IpAddr>, Reason> {
    if let Some(inner) = h.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        let Ok(v6) = inner.parse::<Ipv6Addr>() else {
            return Err(Reason::Malformed("the bracketed host is not an IPv6 address (a zone id is refused too)"));
        };
        if v6.to_string() != inner {
            return Err(Reason::Malformed("the IPv6 literal is not in canonical (RFC 5952) form"));
        }
        return Ok(Some(IpAddr::V6(v6)));
    }
    if h.len() > 253 {
        return Err(Reason::Malformed("the host name is longer than 253 characters"));
    }
    if h.ends_with('.') {
        return Err(Reason::Malformed("the host ends in a dot"));
    }
    if h.bytes().any(|b| b.is_ascii_uppercase()) {
        return Err(Reason::Malformed("the host has upper-case letters; write it in lower case"));
    }
    let labels: Vec<&str> = h.split('.').collect();
    for l in &labels {
        if l.is_empty() || l.len() > 63 {
            return Err(Reason::Malformed("a label of the host name is empty or longer than 63 characters"));
        }
        if !l.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_') {
            return Err(Reason::Malformed(
                "the host has a character outside letters, digits, `-` and `_` (an international name is written in its `xn--` form)",
            ));
        }
    }
    // A host whose LAST label is numeric is an IPv4 address to every URL parser (the WHATWG "ends in
    // a number" rule), in any of `inet_aton`'s spellings — `127.1`, `2130706433`, `0x7f.1`. Only the
    // canonical dotted quad is accepted, so the address checked is the address dialled.
    let last = labels.last().copied().unwrap_or("");
    if last.bytes().all(|b| b.is_ascii_digit()) || last.starts_with("0x") {
        return match h.parse::<Ipv4Addr>() {
            Ok(v4) if v4.to_string() == h => Ok(Some(IpAddr::V4(v4))),
            _ => Err(Reason::Malformed(
                "a number-shaped host that is not a canonical dotted-quad IPv4 address",
            )),
        };
    }
    Ok(None)
}

/// Resolve a redirect's `Location` against the URL that answered it (RFC 3986 §5.2, the four forms
/// a server sends). The result is only a candidate: it goes back through [`parse_target`] and the
/// whole check, so this function decides nothing.
fn join_location(base: &Target, location: &str) -> String {
    let origin = &base.url[..base.authority_end];
    if location.contains("://") {
        return location.to_string();
    }
    if let Some(rest) = location.strip_prefix("//") {
        return format!("https://{rest}");
    }
    if location.starts_with('/') {
        return format!("{origin}{location}");
    }
    let tail = &base.url[base.authority_end..];
    let path_end = tail.find(['?', '#']).unwrap_or(tail.len());
    let path = &tail[..path_end];
    if location.starts_with('?') {
        let path = if path.is_empty() { "/" } else { path };
        return format!("{origin}{path}{location}");
    }
    if location.starts_with('#') || location.is_empty() {
        return base.url.clone();
    }
    // A relative path: replace the last segment of the base path.
    let dir = match path.rfind('/') {
        Some(i) => &path[..=i],
        None => "/",
    };
    format!("{origin}{dir}{location}")
}

// ----- resolution and transport (the two seams a test replaces) --------------------------------

/// Turns a granted host name into addresses. The real one asks the operating system; a test's
/// answers whatever the test needs, which is what makes the policy testable offline.
pub trait Resolver {
    fn resolve(&self, host: &str, port: u16, within: Duration) -> Result<Vec<IpAddr>, Reason>;
}

/// One request, already checked: the URL, the checked host it must be sent as, and the addresses it
/// may be connected to — and no others.
#[derive(Debug)]
pub struct PinnedRequest<'a> {
    pub url: &'a str,
    pub host: &'a str,
    pub port: u16,
    pub addrs: &'a [SocketAddr],
}

/// What came back on one hop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fetched {
    pub status: u16,
    /// For a 3xx: the `Location` header, if it was readable text.
    pub location: Option<String>,
    /// For anything else: the body, already held to the size bound.
    pub body: Vec<u8>,
    /// The address actually connected to, when the client reports it.
    pub peer: Option<SocketAddr>,
}

/// Performs one pinned request. The real one is reqwest over rustls; a test's records what it was
/// asked to do.
pub trait Transport {
    /// False for a build with no client. Asked BEFORE resolution, so a build that cannot connect
    /// never sends a DNS query either.
    fn available(&self) -> bool {
        true
    }
    fn fetch(&self, req: &PinnedRequest<'_>, max_body: u64, within: Duration) -> Result<Fetched, Reason>;
}

/// The transport of a build without the `net` feature: refuses, and is asked nothing else.
pub struct NoTransport;

impl Transport for NoTransport {
    fn available(&self) -> bool {
        false
    }
    fn fetch(&self, _: &PinnedRequest<'_>, _: u64, _: Duration) -> Result<Fetched, Reason> {
        Err(Reason::NoClient)
    }
}

/// The operating system's resolver, bounded by the request's deadline. `getaddrinfo` cannot be
/// cancelled, so it runs on its own thread and is abandoned — not waited for — when the deadline
/// passes; the thread ends whenever the system call does.
pub struct SystemResolver;

impl Resolver for SystemResolver {
    fn resolve(&self, host: &str, port: u16, within: Duration) -> Result<Vec<IpAddr>, Reason> {
        use std::net::ToSocketAddrs;
        let (tx, rx) = std::sync::mpsc::channel();
        let name = host.to_string();
        std::thread::Builder::new()
            .name("delulu-egress-resolve".into())
            .spawn(move || {
                let r = (name.as_str(), port).to_socket_addrs().map(|it| it.map(|a| a.ip()).collect::<Vec<_>>());
                let _ = tx.send(r);
            })
            .map_err(|_| Reason::NoAddress)?;
        match rx.recv_timeout(within) {
            Ok(Ok(addrs)) if !addrs.is_empty() => Ok(addrs),
            Ok(_) => Err(Reason::NoAddress),
            Err(_) => Err(Reason::Timeout),
        }
    }
}

// ----- the record --------------------------------------------------------------------------------

/// One hop, as the host saw it. The addresses never reach the program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HopRecord {
    pub url: String,
    pub host: String,
    pub port: u16,
    /// Every address the host resolved to (or the literal), in the order they were pinned.
    pub addrs: Vec<String>,
    /// The address actually connected to, when known.
    pub peer: Option<String>,
    pub status: Option<u16>,
}

/// One `http.get`, start to finish.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EgressRecord {
    /// The URL exactly as the program wrote it.
    pub url: String,
    pub hops: Vec<HopRecord>,
    /// `Ok((status, bytes delivered))`, or why nothing was delivered.
    pub outcome: Result<(u16, u64), Reason>,
}

impl EgressRecord {
    fn refused(url: &str, reason: Reason) -> Self {
        EgressRecord { url: url.to_string(), hops: Vec::new(), outcome: Err(reason) }
    }

    /// The record as the run report and `--json` carry it.
    pub fn to_json(&self) -> serde_json::Value {
        let hops: Vec<serde_json::Value> = self
            .hops
            .iter()
            .map(|h| {
                serde_json::json!({
                    "url": h.url, "host": h.host, "port": h.port, "addrs": h.addrs,
                    "peer": h.peer, "status": h.status,
                })
            })
            .collect();
        match &self.outcome {
            Ok((status, bytes)) => serde_json::json!({
                "url": self.url, "delivered": true, "status": status, "bytes": bytes, "hops": hops,
            }),
            Err(r) => serde_json::json!({
                "url": self.url, "delivered": false, "reason": r.code(), "explain": r.explain(), "hops": hops,
            }),
        }
    }
}

/// The run's egress evidence: kept records (bounded) and the true counts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EgressLog {
    pub records: Vec<EgressRecord>,
    pub total: u64,
    pub refused: u64,
}

impl EgressLog {
    fn push(&mut self, rec: EgressRecord) {
        self.total += 1;
        if rec.outcome.is_err() {
            self.refused += 1;
        }
        if self.records.len() < MAX_RECORDED {
            self.records.push(rec);
        }
    }

    /// The `egress` object of the run report.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "requests": self.total,
            "not_delivered": self.refused,
            "records": self.records.iter().map(EgressRecord::to_json).collect::<Vec<_>>(),
            "records_kept": self.records.len(),
        })
    }
}

/// Process-wide, not per thread: an actor's `http.get` runs on a worker thread, and evidence that
/// only the main thread's requests reach is evidence with a hole in it.
static LOG: Mutex<EgressLog> = Mutex::new(EgressLog { records: Vec::new(), total: 0, refused: 0 });

fn log(rec: EgressRecord) {
    if let Ok(mut l) = LOG.lock() {
        l.push(rec);
    }
}

/// Take the run's egress evidence, leaving the log empty.
pub fn take_log() -> EgressLog {
    LOG.lock().map(|mut l| std::mem::take(&mut *l)).unwrap_or_default()
}

/// The run's egress evidence so far, leaving it in place.
pub fn snapshot() -> EgressLog {
    LOG.lock().map(|l| l.clone()).unwrap_or_default()
}

/// A point in the log: both counters, so a window after it can be counted exactly even when its
/// records were past the bound and not kept.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mark {
    total: u64,
    refused: u64,
}

/// Where the log stands now — a mark for [`records_since`].
pub fn mark() -> Mark {
    LOG.lock().map(|l| Mark { total: l.total, refused: l.refused }).unwrap_or_default()
}

/// The kept records logged after `m`, and EXACTLY how many requests were refused after it — which is
/// more than the refusals among the kept records once the log is full.
pub fn records_since(m: Mark) -> (Vec<EgressRecord>, u64) {
    let Ok(l) = LOG.lock() else { return (Vec::new(), 0) };
    let kept_before = (m.total as usize).min(l.records.len());
    (l.records[kept_before..].to_vec(), l.refused.saturating_sub(m.refused))
}

/// Record a refusal the primitive table made before the request reached [`get_for_program`] (the
/// scheme check, which predates this module and answers first).
pub fn note_refusal(url: &str, reason: Reason) {
    log(EgressRecord::refused(url, reason));
}

// ----- the request -------------------------------------------------------------------------------

/// Serve one `http.get` for a program: the real resolver and transport, the default ceilings, the
/// record logged. Called only from the primitive table.
pub fn get_for_program(url: &str, allow: &[String], special: &[String]) -> Answer {
    let policy = EgressPolicy::for_capability(allow, special);
    let (rec, body) = get_with(&policy, url, &SystemResolver, system_transport());
    let answer = match (&rec.outcome, body) {
        (Ok(_), Some(b)) => Answer::Body(b),
        (Err(r), _) => r.answer(),
        // `get_with` pairs a delivered outcome with its body; anything else is a bug here, answered
        // as the refusal it would have to be rather than as an empty success.
        (Ok(_), None) => Answer::Refused,
    };
    log(rec);
    answer
}

/// The whole check, every hop, with the resolver and transport given. Returns the record and, when
/// the request delivered, the body — beside the record, so the record never holds a copy of it.
pub fn get_with(
    policy: &EgressPolicy,
    url: &str,
    resolver: &dyn Resolver,
    transport: &dyn Transport,
) -> (EgressRecord, Option<String>) {
    let started = Instant::now();
    let deadline = started + policy.deadline;
    let mut rec = EgressRecord { url: url.to_string(), hops: Vec::new(), outcome: Err(Reason::Protocol) };
    let mut current = url.to_string();
    let mut redirects = 0u32;
    loop {
        match hop(policy, &current, resolver, transport, deadline, &mut rec) {
            Err(reason) => {
                rec.outcome = Err(reason);
                return (rec, None);
            }
            Ok(HopEnd::Redirect(next)) => {
                redirects += 1;
                if redirects > policy.max_redirects {
                    rec.outcome = Err(Reason::TooManyRedirects);
                    return (rec, None);
                }
                current = next;
            }
            Ok(HopEnd::Body(status, bytes)) => {
                let n = bytes.len() as u64;
                return match String::from_utf8(bytes) {
                    Ok(s) => {
                        rec.outcome = Ok((status, n));
                        (rec, Some(s))
                    }
                    Err(_) => {
                        rec.outcome = Err(Reason::NotUtf8);
                        (rec, None)
                    }
                };
            }
        }
    }
}

enum HopEnd {
    Redirect(String),
    Body(u16, Vec<u8>),
}

fn remaining(deadline: Instant) -> Result<Duration, Reason> {
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        Err(Reason::Timeout)
    } else {
        Ok(left)
    }
}

/// One hop: the full check, the pinned request, and what it came back with.
fn hop(
    policy: &EgressPolicy,
    url: &str,
    resolver: &dyn Resolver,
    transport: &dyn Transport,
    deadline: Instant,
    rec: &mut EgressRecord,
) -> Result<HopEnd, Reason> {
    let t = parse_target(url)?;
    if !policy.allow.iter().any(|p| crate::prim::host_matches(&t.host, p)) {
        return Err(Reason::NotAllowlisted);
    }
    // Asked before anything touches the network, DNS included.
    if !transport.available() {
        return Err(Reason::NoClient);
    }
    let special_ok = policy.allow_special.iter().any(|p| crate::prim::host_matches(&t.host, p));
    // The NAME first: `localhost` and the metadata names are special whatever they resolve to.
    if let Some(class) = crate::netclass::special_use_class(&t.host) {
        if !special_ok {
            return Err(Reason::SpecialUse(class));
        }
    }
    let addrs: Vec<IpAddr> = match t.ip {
        Some(ip) => vec![ip],
        None => resolver.resolve(&t.host, t.port, remaining(deadline)?)?,
    };
    if addrs.is_empty() {
        return Err(Reason::NoAddress);
    }
    // EVERY candidate, before any is dialled.
    for a in &addrs {
        if let Some(class) = crate::netclass::addr_class(*a) {
            if !special_ok {
                return Err(Reason::SpecialUse(class));
            }
        }
    }
    let mut pinned: Vec<SocketAddr> = Vec::with_capacity(addrs.len());
    for a in &addrs {
        let s = SocketAddr::new(a.to_canonical(), t.port);
        if !pinned.contains(&s) {
            pinned.push(s);
        }
    }
    rec.hops.push(HopRecord {
        url: t.url.clone(),
        host: t.host.clone(),
        port: t.port,
        addrs: pinned.iter().map(|a| a.ip().to_string()).collect(),
        peer: None,
        status: None,
    });
    let req = PinnedRequest { url: &t.url, host: &t.host, port: t.port, addrs: &pinned };
    let got = transport.fetch(&req, policy.max_body, remaining(deadline)?)?;
    if let Some(h) = rec.hops.last_mut() {
        h.status = Some(got.status);
        h.peer = got.peer.map(|p| p.ip().to_canonical().to_string());
    }
    // Belt to the construction's braces: a peer outside the pinned set means the client connected
    // somewhere nobody checked.
    if let Some(peer) = got.peer {
        if !pinned.iter().any(|p| p.ip() == peer.ip().to_canonical()) {
            return Err(Reason::HostMismatch);
        }
    }
    if got.body.len() as u64 > policy.max_body {
        return Err(Reason::BodyTooLarge);
    }
    match got.status {
        301 | 302 | 303 | 307 | 308 => match got.location {
            Some(loc) if !loc.is_empty() => Ok(HopEnd::Redirect(join_location(&t, &loc))),
            _ => Err(Reason::RedirectWithoutLocation),
        },
        200..=299 => Ok(HopEnd::Body(got.status, got.body)),
        other => Err(Reason::Status(other)),
    }
}

// ----- the real transport ------------------------------------------------------------------------

/// The transport this build performs requests with.
pub fn system_transport() -> &'static dyn Transport {
    #[cfg(feature = "net")]
    {
        static REAL: std::sync::OnceLock<real::ReqwestTransport> = std::sync::OnceLock::new();
        REAL.get_or_init(real::ReqwestTransport::system)
    }
    #[cfg(not(feature = "net"))]
    {
        &NoTransport
    }
}

/// What `doctor` reports about the client: whether one is compiled in, and how many trust roots the
/// platform store gave it — a host with none cannot complete any HTTPS request, and that must be
/// legible rather than mysterious.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientStatus {
    pub compiled: bool,
    pub trust_roots: usize,
    /// Certificates in the platform store that could not be used (unparsable, or unreadable).
    pub unusable_roots: usize,
}

pub fn client_status() -> ClientStatus {
    #[cfg(feature = "net")]
    {
        let (store, unusable) = real::native_roots();
        ClientStatus { compiled: true, trust_roots: store.len(), unusable_roots: unusable }
    }
    #[cfg(not(feature = "net"))]
    {
        ClientStatus { compiled: false, trust_roots: 0, unusable_roots: 0 }
    }
}

#[cfg(feature = "net")]
pub(crate) mod real {
    use super::{Fetched, PinnedRequest, Reason, Transport};
    use std::io::Read as _;
    use std::sync::Arc;
    use std::time::Duration;

    /// The platform trust store, as rustls roots, and how many of its certificates were unusable.
    pub(crate) fn native_roots() -> (rustls::RootCertStore, usize) {
        let loaded = rustls_native_certs::load_native_certs();
        let mut store = rustls::RootCertStore::empty();
        let (_added, ignored) = store.add_parsable_certificates(loaded.certs);
        (store, ignored + loaded.errors.len())
    }

    /// The TLS configuration every request is made with: rustls on `ring`, the safe default protocol
    /// versions (1.2 and 1.3), the given roots, no client certificate, HTTP/1.1 only.
    pub(crate) fn tls_config(roots: rustls::RootCertStore) -> Result<rustls::ClientConfig, String> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let mut cfg = rustls::ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(|e| e.to_string())?
            .with_root_certificates(roots)
            .with_no_client_auth();
        cfg.alpn_protocols = vec![b"http/1.1".to_vec()];
        Ok(cfg)
    }

    /// A resolver that answers nothing. The client is given the checked addresses as an override for
    /// the checked host; ANY other lookup — a host the override does not cover — fails here instead
    /// of reaching the network. That is what makes "the host checked is the host connected" a
    /// property of the construction rather than of the URL parser agreeing with ours.
    struct NoDns;

    impl reqwest::dns::Resolve for NoDns {
        fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
            let n = name.as_str().to_string();
            Box::pin(async move {
                let e: Box<dyn std::error::Error + Send + Sync> =
                    format!("delulu egress: `{n}` was not checked and pinned, so it is not resolved").into();
                Err(e)
            })
        }
    }

    pub(crate) struct ReqwestTransport {
        tls: Result<rustls::ClientConfig, String>,
    }

    impl ReqwestTransport {
        /// The platform's trust store. Loaded once per process.
        pub(crate) fn system() -> Self {
            ReqwestTransport { tls: tls_config(native_roots().0) }
        }

        /// Explicit roots. For the tests' self-signed server only — there is no flag, variable or
        /// file that reaches this from a release build, because a trust root an environment can
        /// inject is a hole, not a seam.
        #[cfg(test)]
        pub(crate) fn with_roots(roots: rustls::RootCertStore) -> Self {
            ReqwestTransport { tls: tls_config(roots) }
        }
    }

    /// Does the error chain carry a rustls error? That is the TLS failure, whatever the layer that
    /// wrapped it called itself.
    ///
    /// Not just `source()`: hyper reports a certificate refusal as an `io::Error` wrapping an
    /// `io::Error` wrapping the `rustls::Error`, and `io::Error::source()` answers the WRAPPED
    /// error's source rather than the wrapped error — so a walk over `source()` alone steps straight
    /// past the one value that says "TLS". Found by the test below classifying an untrusted
    /// certificate as `connect`. Every nested `io::Error` is therefore opened with `get_ref()`.
    fn is_tls(e: &(dyn std::error::Error + 'static)) -> bool {
        let mut cur: Option<&(dyn std::error::Error + 'static)> = Some(e);
        while let Some(err) = cur {
            if err.downcast_ref::<rustls::Error>().is_some() {
                return true;
            }
            if let Some(io) = err.downcast_ref::<std::io::Error>() {
                let mut inner = io.get_ref();
                while let Some(i) = inner {
                    if i.downcast_ref::<rustls::Error>().is_some() {
                        return true;
                    }
                    inner = i.downcast_ref::<std::io::Error>().and_then(|x| x.get_ref());
                }
            }
            cur = err.source();
        }
        false
    }

    fn classify(e: &reqwest::Error) -> Reason {
        if e.is_timeout() {
            Reason::Timeout
        } else if is_tls(e) {
            Reason::Tls
        } else if e.is_connect() {
            Reason::Connect
        } else {
            Reason::Protocol
        }
    }

    impl Transport for ReqwestTransport {
        fn fetch(&self, req: &PinnedRequest<'_>, max_body: u64, within: Duration) -> Result<Fetched, Reason> {
            let tls = self.tls.clone().map_err(|_| Reason::Tls)?;
            let mut b = reqwest::blocking::Client::builder()
                .use_preconfigured_tls(tls)
                // `HTTPS_PROXY` and friends are ambient authority: a proxy resolves the name itself,
                // which would bypass the pin and every check made on the address.
                .no_proxy()
                .redirect(reqwest::redirect::Policy::none())
                .https_only(true)
                .referer(false)
                .dns_resolver(Arc::new(NoDns))
                .pool_max_idle_per_host(0)
                .timeout(within)
                .connect_timeout(within)
                .user_agent(concat!("delulu/", env!("CARGO_PKG_VERSION")));
            if !req.host.starts_with('[') && req.host.parse::<std::net::Ipv4Addr>().is_err() {
                b = b.resolve_to_addrs(req.host, req.addrs);
            }
            let client = b.build().map_err(|e| classify(&e))?;
            let resp = client.get(req.url).send().map_err(|e| classify(&e))?;
            let status = resp.status().as_u16();
            let peer = resp.remote_addr();
            if (300..400).contains(&status) {
                let location = resp
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string);
                return Ok(Fetched { status, location, body: Vec::new(), peer });
            }
            if resp.content_length().is_some_and(|n| n > max_body) {
                return Err(Reason::BodyTooLarge);
            }
            let mut body = Vec::new();
            resp.take(max_body.saturating_add(1)).read_to_end(&mut body).map_err(|e| {
                if e.kind() == std::io::ErrorKind::TimedOut {
                    Reason::Timeout
                } else {
                    Reason::Protocol
                }
            })?;
            if body.len() as u64 > max_body {
                return Err(Reason::BodyTooLarge);
            }
            Ok(Fetched { status, location: None, body, peer })
        }
    }
}

#[cfg(test)]
mod tests;
