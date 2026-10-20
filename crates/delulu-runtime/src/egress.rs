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
//! 8. **A program's headers** (`get_with`, P9-04 (a), D-V2-128) are judged by [`check_headers`] before
//!    anything is resolved: each name an HTTP token, each value printable ASCII with no line break, no
//!    two names that differ only in case, a bound on their number and size — and none of the names the
//!    client owns ([`RESERVED_HEADERS`]: `Host`, which would carry the request to another site on the
//!    checked address, the framing and connection headers, `Proxy-*`, `Accept-Encoding`). They are sent
//!    only to the ORIGIN the program named: a redirect to another host or port drops them, so what a
//!    program addressed to one server is never handed to the next. Their names are recorded per hop;
//!    their values never are, anywhere.
//! 9. **A secret in a header** (`get_with_secret`, P9-04 (b)/(c), D-V2-131) goes only to its
//!    AUDIENCE — the hosts and ports its grant names (`--grant secret:NAME@HOST[:PORT]=VALUE`). The
//!    URL's origin is judged against the audience before anything is resolved, and every hop that would
//!    carry the secret is judged again, beside the origin rule above: two walls, so a change to either
//!    cannot alone send the bytes somewhere the grant never named. A secret granted with no audience is
//!    sent nowhere. The record names the secret (its grant name) on each hop that carried it, never its
//!    bytes.
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
/// How many headers one request may carry (P9-04 (a)). A request to an API carries a handful; the bound
/// is on what a program can make the host format and send on its behalf.
pub const MAX_HEADERS: usize = 32;
/// The longest header name accepted.
pub const MAX_HEADER_NAME: usize = 128;
/// The longest header value accepted.
pub const MAX_HEADER_VALUE: usize = 8 * 1024;
/// Every name and value of one request together.
pub const MAX_HEADER_BYTES: usize = 16 * 1024;

/// The header names a program may not set, lower-case, each with why: the egress client owns them, and a
/// program's would re-route, re-frame or re-negotiate the request that was checked.
pub const RESERVED_HEADERS: &[(&str, &str)] = &[
    ("host", "`Host` names the checked host and is the client's to send; a program's would carry the request to another site served from the checked address"),
    ("content-length", "`Content-Length` frames the request, and the client frames it"),
    ("transfer-encoding", "`Transfer-Encoding` frames the request, and the client frames it"),
    ("te", "`TE` negotiates the response's framing, which is the client's"),
    ("trailer", "`Trailer` announces fields after the body, and a GET has none"),
    ("expect", "`Expect` changes how the request is sent, which is the client's"),
    ("connection", "`Connection` governs the connection, which is the client's"),
    ("keep-alive", "`Keep-Alive` governs the connection, which is the client's"),
    ("upgrade", "`Upgrade` would switch the connection to another protocol"),
    ("proxy-authorization", "`Proxy-Authorization` is for a proxy, and there is none"),
    ("proxy-connection", "`Proxy-Connection` is for a proxy, and there is none"),
    ("accept-encoding", "`Accept-Encoding` asks for a content coding this client cannot decode; the body would arrive as bytes the program cannot read"),
];

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
    /// The host, or under a path scope the path, is not in the capability's allowlist (a redirect's target, typically — the program's
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
    /// A request header the program gave was refused before anything was resolved (P9-04 (a)); the
    /// payload names the rule, never the value.
    Header(&'static str),
    /// A secret was to be sent to an origin outside the audience its grant names (P9-04 (c),
    /// D-V2-131) — `credential_endpoint_mismatch`'s twin. Refused before anything is resolved, so its
    /// bytes reach no one; the payload names the rule, never the secret's value.
    Audience(&'static str),
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
            Reason::Header(_) => "header",
            Reason::Audience(_) => "audience",
        }
    }

    /// A sentence for the operator. Host-side only — the program is told [`Reason::answer`].
    pub fn explain(&self) -> String {
        match self {
            Reason::Scheme => "only `https://` is fetched; there is no plain-HTTP path".into(),
            Reason::Userinfo => "the URL carries userinfo (`user@host`), which is refused rather than parsed".into(),
            Reason::Malformed(why) => format!("the URL is refused rather than normalized: {why}"),
            Reason::NotAllowlisted => "the host — or, under a path scope, the path — is not in the capability's allowlist".into(),
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
            Reason::Header(why) => format!("a request header is refused: {why}"),
            Reason::Audience(why) => format!("a secret is not sent: {why}"),
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
            | Reason::NoClient
            | Reason::Header(_)
            | Reason::Audience(_) => Answer::Refused,
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

impl Target {
    /// The path as sent — the text after the authority up to a `?` or `#`; empty when the URL names none.
    pub fn path(&self) -> &str {
        let rest = &self.url[self.authority_end..];
        if !rest.starts_with('/') {
            return "";
        }
        &rest[..rest.find(['?', '#']).unwrap_or(rest.len())]
    }
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
    Ok(Target { url: url.to_string(), host: host_text.to_string(), port, ip, authority_end: "https://".len() + end })
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
            _ => Err(Reason::Malformed("a number-shaped host that is not a canonical dotted-quad IPv4 address")),
        };
    }
    Ok(None)
}

/// One origin a secret may be sent to (P9-04 (c), D-V2-131): a host in exactly the spelling
/// [`parse_target`] compares and connects (lower-case LDH, a canonical dotted quad, or `[v6]`), and a
/// port — 443 unless the grant names another. Compared by equality with each hop's parsed host and
/// port, so the audience and the request are judged in one spelling: there is no second one to walk past.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Audience {
    pub host: String,
    pub port: u16,
}

impl std::fmt::Display for Audience {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

/// Parse one member of a secret's audience, `HOST[:PORT]`, by the URL parser's own rules (it is
/// handed to [`parse_target`] as the authority of an `https://` URL), and refuse what that parse would
/// accept around the authority: a path, a query, a fragment (a path in an audience waits for P9-05's
/// path scopes, and is refused rather than ignored), a wildcard (an audience names hosts exactly), and
/// userinfo. A spelling refused for a URL is refused here in the same words.
pub fn parse_audience(entry: &str) -> Result<Audience, String> {
    if entry.is_empty() {
        return Err("an audience member is empty (write HOST or HOST:PORT)".to_string());
    }
    if entry.contains(['/', '?', '#']) {
        return Err(format!(
            "`{entry}`: an audience names a host and port only — a path in an audience waits for path scopes (P9-05), and is refused rather than ignored"
        ));
    }
    if entry.contains('*') {
        return Err(format!("`{entry}`: an audience names each host exactly; a wildcard is refused"));
    }
    match parse_target(&format!("https://{entry}")) {
        Ok(t) => Ok(Audience { host: t.host, port: t.port }),
        Err(Reason::Userinfo) => {
            Err(format!("`{entry}`: a second `@` — an audience member is HOST[:PORT], and a secret's name holds no `@`"))
        }
        Err(r) => Err(format!("`{entry}`: {}", r.explain().trim_start_matches("the URL is refused rather than normalized: "))),
    }
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
pub struct PinnedRequest<'a> {
    pub url: &'a str,
    pub host: &'a str,
    pub port: u16,
    pub addrs: &'a [SocketAddr],
    /// The program's headers for THIS hop — checked by [`check_headers`], lower-case names, and empty on
    /// a hop to another origin than the program named.
    pub headers: &'a [(String, String)],
}

impl std::fmt::Debug for PinnedRequest<'_> {
    // A header's value may be a secret's bytes (P9-04 (b)/(c)): the NAMES only, here as in every record.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names: Vec<&str> = self.headers.iter().map(|(n, _)| n.as_str()).collect();
        f.debug_struct("PinnedRequest")
            .field("url", &self.url)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("addrs", &self.addrs)
            .field("headers", &names)
            .finish()
    }
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
    /// The NAMES of the program's headers sent on this hop, lower-case — never their values. Empty when
    /// it sent none, and on a hop to another origin, where they are dropped.
    pub headers: Vec<String>,
    /// The grant NAME of the secret this hop carried (P9-04 (b)/(c)) — never its bytes. `None` on a hop
    /// that carried no secret.
    pub secret: Option<String>,
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
                let mut j = serde_json::json!({
                    "url": h.url, "host": h.host, "port": h.port, "addrs": h.addrs,
                    "peer": h.peer, "status": h.status,
                });
                // Only where the program sent some, so a report of a plain `get` is what it always was.
                if !h.headers.is_empty() {
                    j["headers"] = serde_json::json!(h.headers);
                }
                // The same rule: only on a hop that carried one.
                if let Some(name) = &h.secret {
                    j["secret"] = serde_json::json!(name);
                }
                j
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

thread_local! {
    /// RW 7.20: what THIS thread logged, bounded and counted as [`LOG`] is — the log a [`mark`] reads. The sandbox host
    /// performs a guest's request on the thread that answers it, so a window opened and read there holds that request's
    /// refusals and nothing of another thread's: a second host in the process, or a test beside it, whose refusal
    /// landed in the window of the process-wide log had been counted as this guest's.
    static MINE: std::cell::RefCell<EgressLog> = std::cell::RefCell::new(EgressLog::default());
}

fn log(rec: EgressRecord) {
    MINE.with(|m| m.borrow_mut().push(rec.clone()));
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

/// A point in this thread's log: both counters, so a window after it can be counted exactly even when
/// its records were past the bound and not kept.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Mark {
    total: u64,
    refused: u64,
}

/// Where this thread's log stands now — a mark for [`records_since`] on the same thread.
pub fn mark() -> Mark {
    MINE.with(|l| {
        let l = l.borrow();
        Mark { total: l.total, refused: l.refused }
    })
}

/// The kept records this thread logged after `m`, and EXACTLY how many of its requests were refused after
/// it — which is more than the refusals among the kept records once the log is full. Another thread's
/// requests are in neither (RW 7.20).
pub fn records_since(m: Mark) -> (Vec<EgressRecord>, u64) {
    MINE.with(|l| {
        let l = l.borrow();
        let kept_before = (m.total as usize).min(l.records.len());
        (l.records[kept_before..].to_vec(), l.refused.saturating_sub(m.refused))
    })
}

thread_local! {
    /// RW 4.46: the refusal of the last request THIS thread made through [`get_for_program`], kept whatever
    /// the evidence log's bound. The log keeps its first [`MAX_RECORDED`] records only, and the first form of
    /// this fix read refusals from it — so after 64 requests of any kind a run's refusals reached no chain, and a
    /// `special-use` monitor could be walked past by spending the 64 first (routine run 18's red team, F1/F2).
    static LAST_REFUSAL: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}

/// RW 4.46: begin a fetch the custody allowed — forget this thread's last refusal.
pub fn begin_fetch() {
    LAST_REFUSAL.with(|c| c.set(None));
}

/// RW 4.46: the egress client's REFUSAL of the fetch this thread made since [`begin_fetch`], by its
/// machine-readable reason — for the custody that allowed the use of its host, whose chain must then say
/// it was refused. A request it refused rather than failed (its [`Reason::answer`] is `Refused`); a network
/// failure (a timeout, a TLS or connection error, an HTTP status) is not a refusal. From the call's own
/// outcome, on the calling thread: never from the bounded log, never another thread's (an actor's) request.
pub fn fetch_refusal() -> Option<&'static str> {
    LAST_REFUSAL.with(|c| c.take())
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
    get_for_program_with(url, &[], allow, special)
}

/// Serve one `http.get_with` for a program (P9-04 (a)): [`get_for_program`] with the program's headers,
/// which [`request_with`] checks before anything is resolved.
pub fn get_for_program_with(url: &str, headers: &[(String, String)], allow: &[String], special: &[String]) -> Answer {
    get_for_program_with_secret(url, headers, None, allow, special)
}

/// Serve one `http.get_with_secret` for a program (P9-04 (b)/(c)): [`get_for_program_with`] carrying one
/// secret in one header, sent only inside its audience ([`request_with_secret`]).
pub fn get_for_program_with_secret(
    url: &str,
    headers: &[(String, String)],
    secret: Option<&SecretHeader<'_>>,
    allow: &[String],
    special: &[String],
) -> Answer {
    let policy = EgressPolicy::for_capability(allow, special);
    let (rec, body) = request_with_secret(&policy, url, headers, secret, &SystemResolver, system_transport());
    let answer = match (&rec.outcome, body) {
        (Ok(_), Some(b)) => Answer::Body(b),
        (Err(r), _) => r.answer(),
        // `get_with` pairs a delivered outcome with its body; anything else is a bug here, answered
        // as the refusal it would have to be rather than as an empty success.
        (Ok(_), None) => Answer::Refused,
    };
    if let Err(r) = &rec.outcome {
        if r.answer() == Answer::Refused {
            LAST_REFUSAL.with(|c| c.set(Some(r.code())));
        }
    }
    log(rec);
    answer
}

/// P9-04 (d), D-V2-134: what [`request_with_secret`] judges before anything is resolved, asked BEFORE a broker-held
/// secret's bytes exist — so the broker is asked for them only for a request that would carry them: every header
/// (the secret's with a stand-in value; its real value is judged when it arrives, before a byte is sent), the URL's
/// own strict parse, and its origin in the secret's audience. Returns that origin — the one the broker is asked to
/// send to, and the only one the bytes it answers may then go to.
pub fn prejudge_secret(url: &str, headers: &[(String, String)], header: &str, audience: &[Audience]) -> Result<Audience, Reason> {
    let mut given = headers.to_vec();
    given.push((header.to_string(), "x".to_string()));
    check_headers(&given)?;
    let t = parse_target(url)?;
    if audience.is_empty() {
        return Err(Reason::Audience(
            "its grant names no audience — grant it as `secret:NAME@HOST[:PORT]=VALUE` to say where it may be sent",
        ));
    }
    if !audience.iter().any(|a| a.host == t.host && a.port == t.port) {
        return Err(Reason::Audience("the URL's host and port are outside the audience its grant names"));
    }
    Ok(Audience { host: t.host, port: t.port })
}

/// A request refused before anything was resolved — by [`prejudge_secret`] — recorded and answered exactly as
/// [`get_for_program_with_secret`] records and answers one it refuses: the run's evidence holds it, and a refusal after
/// the custody allowed the host's use is said to the custody (RW 4.46).
pub fn refuse_unsent(url: &str, reason: Reason) -> Answer {
    let answer = reason.answer();
    if answer == Answer::Refused {
        LAST_REFUSAL.with(|c| c.set(Some(reason.code())));
    }
    log(EgressRecord::refused(url, reason));
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
    request_with(policy, url, &[], resolver, transport)
}

/// Judge a program's headers (P9-04 (a), D-V2-128) before anything is resolved: the names lower-cased
/// for sending, in the order given. Every rule refuses rather than normalizes — a value with a space at
/// its end is not trimmed, a name in two cases is not merged — because a header the program wrote one
/// way and the server read another is a request nobody checked.
pub fn check_headers(given: &[(String, String)]) -> Result<Vec<(String, String)>, Reason> {
    if given.len() > MAX_HEADERS {
        return Err(Reason::Header("more than 32 headers"));
    }
    let mut total = 0usize;
    let mut out: Vec<(String, String)> = Vec::with_capacity(given.len());
    for (name, value) in given {
        // RFC 9110 §5.1: a field name is a `token` — `tchar`s, at least one.
        let tchar = |b: u8| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b);
        if name.is_empty() || !name.bytes().all(tchar) {
            return Err(Reason::Header("a header name is not an HTTP token (RFC 9110 §5.1)"));
        }
        if name.len() > MAX_HEADER_NAME {
            return Err(Reason::Header("a header name is longer than 128 bytes"));
        }
        let lower = name.to_ascii_lowercase();
        if let Some((_, why)) = RESERVED_HEADERS.iter().find(|(n, _)| *n == lower) {
            return Err(Reason::Header(why));
        }
        // Printable ASCII, space and tab: no CR, LF or NUL, which would end the header and begin
        // another (request splitting), and no byte past ASCII, which servers read differently.
        if !value.bytes().all(|b| (0x20..=0x7e).contains(&b) || b == b'\t') {
            return Err(Reason::Header(
                "a header value holds a byte other than printable ASCII, space or tab (a line break would begin another header)",
            ));
        }
        if value.starts_with([' ', '\t']) || value.ends_with([' ', '\t']) {
            return Err(Reason::Header("a header value begins or ends with whitespace, which servers strip"));
        }
        if value.len() > MAX_HEADER_VALUE {
            return Err(Reason::Header("a header value is longer than 8 KiB"));
        }
        if out.iter().any(|(n, _)| *n == lower) {
            return Err(Reason::Header("two header names differ only in case, and a header name has no case"));
        }
        total += name.len() + value.len();
        if total > MAX_HEADER_BYTES {
            return Err(Reason::Header("the headers together are larger than 16 KiB"));
        }
        out.push((lower, value.clone()));
    }
    Ok(out)
}

/// [`get_with`] carrying the program's headers: checked first, then sent on each hop whose origin —
/// host and port, the scheme being `https` always — is the one the program named, and on no other.
pub fn request_with(
    policy: &EgressPolicy,
    url: &str,
    headers: &[(String, String)],
    resolver: &dyn Resolver,
    transport: &dyn Transport,
) -> (EgressRecord, Option<String>) {
    request_with_secret(policy, url, headers, None, resolver, transport)
}

/// A secret one request carries in one header (P9-04 (b)/(c), D-V2-131): the header's name as the
/// program wrote it, the secret's grant name (what the record says was sent), its bytes, and the
/// audience its grant names — empty when the grant named none, and then it is sent nowhere.
pub struct SecretHeader<'a> {
    pub header: &'a str,
    pub name: &'a str,
    pub value: &'a str,
    pub audience: &'a [Audience],
}

impl std::fmt::Debug for SecretHeader<'_> {
    // The bytes are never formatted, here or anywhere.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretHeader")
            .field("header", &self.header)
            .field("name", &self.name)
            .field("audience", &self.audience)
            .finish()
    }
}

/// [`request_with`] carrying, beside the program's headers, one secret in one header. The secret's header
/// is held to every rule a program's header is ([`check_headers`], over the whole set — so it cannot
/// repeat a plain header's name in another case, nor be `Host`); then the URL's origin must be in the
/// secret's audience, or the request is refused before anything is resolved. On each hop the secret is
/// sent only where the program's headers are (the origin it named) AND that origin is in its audience:
/// the second condition is implied by the first today, and is checked anyway, so that neither rule can
/// be loosened alone into sending a secret where its grant never pointed.
pub fn request_with_secret(
    policy: &EgressPolicy,
    url: &str,
    headers: &[(String, String)],
    secret: Option<&SecretHeader<'_>>,
    resolver: &dyn Resolver,
    transport: &dyn Transport,
) -> (EgressRecord, Option<String>) {
    let started = Instant::now();
    let deadline = started + policy.deadline;
    let mut rec = EgressRecord { url: url.to_string(), hops: Vec::new(), outcome: Err(Reason::Protocol) };
    let mut given: Vec<(String, String)> = headers.to_vec();
    if let Some(s) = secret {
        given.push((s.header.to_string(), s.value.to_string()));
    }
    let mut headers = match check_headers(&given) {
        Ok(h) => h,
        Err(reason) => {
            rec.outcome = Err(reason);
            return (rec, None);
        }
    };
    // The secret's header, checked with the rest, is kept apart from them: it rides only where its
    // audience allows.
    let secret_header: Option<(String, String)> = if secret.is_some() { headers.pop() } else { None };
    let headers = headers;
    // The origin the program named. A URL that does not parse has none, and its first hop refuses it.
    let origin = parse_target(url).ok().map(|t| (t.host, t.port));
    if let Some(s) = secret {
        if s.audience.is_empty() {
            rec.outcome = Err(Reason::Audience(
                "its grant names no audience — grant it as `secret:NAME@HOST[:PORT]=VALUE` to say where it may be sent",
            ));
            return (rec, None);
        }
        // A URL that does not parse is refused by its first hop, in its own words, and nothing is sent.
        if let Some((host, port)) = &origin {
            if !s.audience.iter().any(|a| a.host == *host && a.port == *port) {
                rec.outcome = Err(Reason::Audience("the URL's host and port are outside the audience its grant names"));
                return (rec, None);
            }
        }
    }
    let mut current = url.to_string();
    let mut redirects = 0u32;
    loop {
        let here = parse_target(&current).ok().map(|t| (t.host, t.port));
        let send = match (&origin, &here) {
            (Some(o), Some(h)) => o == h,
            _ => false,
        };
        let carry_secret = send
            && secret
                .is_some_and(|s| here.as_ref().is_some_and(|(h, p)| s.audience.iter().any(|a| a.host == *h && a.port == *p)));
        let mut hop_headers: Vec<(String, String)> = if send { headers.clone() } else { Vec::new() };
        let mut secret_name: Option<&str> = None;
        if let (true, Some(sh), Some(s)) = (carry_secret, &secret_header, secret) {
            hop_headers.push(sh.clone());
            secret_name = Some(s.name);
        }
        match hop(policy, &current, &hop_headers, secret_name, resolver, transport, deadline, &mut rec) {
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
#[allow(clippy::too_many_arguments)]
fn hop(
    policy: &EgressPolicy,
    url: &str,
    headers: &[(String, String)],
    secret: Option<&str>,
    resolver: &dyn Resolver,
    transport: &dyn Transport,
    deadline: Instant,
    rec: &mut EgressRecord,
) -> Result<HopEnd, Reason> {
    let t = parse_target(url)?;
    // P9-05 (a), D-V2-138: the host, and where an entry names a path pattern, this hop's path — a redirect out of the
    // granted path is refused here as one to an ungranted host is; a path a server could read two ways, before DNS.
    match delulu_broker::netscope::judge_all(&policy.allow, &t.host, t.path()) {
        delulu_broker::netscope::Verdict::Allow => {}
        delulu_broker::netscope::Verdict::Ambiguous(why) => return Err(Reason::Malformed(why)),
        delulu_broker::netscope::Verdict::OtherHost | delulu_broker::netscope::Verdict::OutsidePath => {
            return Err(Reason::NotAllowlisted);
        }
    }
    // Asked before anything touches the network, DNS included.
    if !transport.available() {
        return Err(Reason::NoClient);
    }
    let special_ok = policy.allow_special.iter().any(|p| crate::prim::host_matches(&t.host, delulu_broker::netscope::host_of(p)));
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
        headers: headers.iter().map(|(n, _)| n.clone()).collect(),
        secret: secret.map(str::to_string),
    });
    let req = PinnedRequest { url: &t.url, host: &t.host, port: t.port, addrs: &pinned, headers };
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
            // The program's headers, already held to `check_headers`. A request header replaces the
            // client's default of the same name (`User-Agent`, `Accept`); `Host` is never among them.
            let mut rb = client.get(req.url);
            for (name, value) in req.headers {
                rb = rb.header(name.as_str(), value.as_str());
            }
            let resp = rb.send().map_err(|e| classify(&e))?;
            let status = resp.status().as_u16();
            let peer = resp.remote_addr();
            if (300..400).contains(&status) {
                let location = resp.headers().get(reqwest::header::LOCATION).and_then(|v| v.to_str().ok()).map(str::to_string);
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
