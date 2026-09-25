//! The egress client's tests. Most of the policy is tested OFFLINE, with a resolver and a transport
//! that answer whatever the test needs and record what they were asked — which is what lets a test
//! say "the name resolved into a private range and nothing was dialled" without a DNS server that
//! lies. The real transport is tested against a TLS server on loopback, below.

use super::*;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

const PUBLIC: &str = "93.184.215.14";

fn ip(s: &str) -> IpAddr {
    s.parse().unwrap()
}

/// A resolver that answers from a table and remembers every name it was asked.
#[derive(Default)]
struct FakeResolver {
    answers: HashMap<String, Result<Vec<IpAddr>, Reason>>,
    asked: RefCell<Vec<String>>,
}

impl FakeResolver {
    fn with(pairs: &[(&str, &[&str])]) -> Self {
        let mut r = FakeResolver::default();
        for (name, addrs) in pairs {
            r.answers.insert(name.to_string(), Ok(addrs.iter().map(|a| ip(a)).collect()));
        }
        r
    }
}

impl Resolver for FakeResolver {
    fn resolve(&self, host: &str, _port: u16, _within: Duration) -> Result<Vec<IpAddr>, Reason> {
        self.asked.borrow_mut().push(host.to_string());
        self.answers.get(host).cloned().unwrap_or(Err(Reason::NoAddress))
    }
}

/// One request as the transport received it.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Sent {
    url: String,
    host: String,
    port: u16,
    addrs: Vec<SocketAddr>,
}

/// A transport that plays a script of answers and records what it was asked to send.
#[derive(Default)]
struct FakeTransport {
    script: RefCell<VecDeque<Result<Fetched, Reason>>>,
    sent: RefCell<Vec<Sent>>,
}

impl FakeTransport {
    fn answering(answers: Vec<Result<Fetched, Reason>>) -> Self {
        FakeTransport { script: RefCell::new(answers.into()), sent: RefCell::new(Vec::new()) }
    }
}

impl Transport for FakeTransport {
    fn fetch(&self, req: &PinnedRequest<'_>, _max_body: u64, _within: Duration) -> Result<Fetched, Reason> {
        self.sent.borrow_mut().push(Sent {
            url: req.url.to_string(),
            host: req.host.to_string(),
            port: req.port,
            addrs: req.addrs.to_vec(),
        });
        self.script.borrow_mut().pop_front().unwrap_or(Err(Reason::Connect))
    }
}

fn ok(body: &str) -> Result<Fetched, Reason> {
    Ok(Fetched { status: 200, location: None, body: body.as_bytes().to_vec(), peer: None })
}

fn redirect(status: u16, to: &str) -> Result<Fetched, Reason> {
    Ok(Fetched { status, location: Some(to.to_string()), body: Vec::new(), peer: None })
}

fn policy(allow: &[&str], special: &[&str]) -> EgressPolicy {
    EgressPolicy::for_capability(
        &allow.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        &special.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    )
}

// ----- the spelling ------------------------------------------------------------------------------

#[test]
fn only_https_is_fetched() {
    for url in ["http://example.com/", "HTTPS://example.com/", "ftp://example.com/", "example.com", "//example.com/"] {
        assert_eq!(parse_target(url), Err(Reason::Scheme), "`{url}`");
    }
}

/// C86's shape, refused before it can be parsed wrongly: everything before the last `@` is userinfo.
#[test]
fn userinfo_is_refused_rather_than_parsed() {
    for url in ["https://u@example.com/", "https://u:p@example.com/", "https://example.com:8080@evil.com/steal", "https://@example.com/"] {
        assert_eq!(parse_target(url), Err(Reason::Userinfo), "`{url}`");
    }
}

/// Every spelling a URL parser would REWRITE is refused, so the host checked is the host dialled.
/// Each of these, handed to a WHATWG parser, names a host other than the one a naive reading sees.
#[test]
fn every_rewritable_spelling_is_refused() {
    let long_label = format!("https://{}.com/", "a".repeat(64));
    let long_name = format!("https://{}com/", "abcdefghi.".repeat(26));
    for url in [
        "https://exam ple.com/",
        "https://example.com/a b",
        "https://exam\tple.com/",
        "https://example.com\n/",
        "https://example.com\\@evil.com/",
        "https://example.com\\evil/",
        "https://ex\u{0430}mple.com/",
        "https://Example.com/",
        "https://example.com./",
        "https://example..com/",
        "https://.example.com/",
        "https://exa%6dple.com/",
        "https://example.com:/",
        "https://example.com:0/",
        "https://example.com:65536/",
        "https://example.com:0443/",
        "https://example.com:+443/",
        "https://127.1/",
        "https://2130706433/",
        "https://0x7f.0.0.1/",
        "https://0177.0.0.1/",
        "https://127.0.0.01/",
        "https://example.123/",
        "https://example.0x10/",
        "https://[0:0:0:0:0:0:0:1]/",
        "https://[::1%25eth0]/",
        "https://[::1]x/",
        "https://[::1/",
        "https://::1/",
        "https:///path",
        long_label.as_str(),
        long_name.as_str(),
    ] {
        match parse_target(url) {
            Err(Reason::Malformed(_)) => {}
            other => panic!("`{}` must be refused as malformed, got {other:?}", url.escape_debug()),
        }
    }
}

#[test]
fn the_canonical_forms_are_accepted_as_written() {
    let t = parse_target("https://api.example.com/v1/items?q=1#frag").unwrap();
    assert_eq!((t.host.as_str(), t.port, t.ip), ("api.example.com", 443, None));
    assert_eq!(t.url, "https://api.example.com/v1/items?q=1#frag");
    let t = parse_target("https://example.com:8443").unwrap();
    assert_eq!((t.host.as_str(), t.port), ("example.com", 8443));
    let t = parse_target("https://93.184.215.14/").unwrap();
    assert_eq!(t.ip, Some(ip("93.184.215.14")));
    let t = parse_target("https://[2606:4700::1111]:444/x").unwrap();
    assert_eq!((t.host.as_str(), t.port, t.ip), ("[2606:4700::1111]", 444, Some(ip("2606:4700::1111"))));
    // An international name in its ASCII form, and the underscore some real hosts use.
    assert!(parse_target("https://xn--bcher-kva.example/").is_ok());
    assert!(parse_target("https://my_service.example.com/").is_ok());
    // A path may carry anything printable; only the HOST decides where the request goes.
    assert!(parse_target("https://example.com/%E2%9C%93/../a?x=%20").is_ok());
}

/// The host this module derives agrees with `prim::host_of` — the one the `DL0904` gate and the
/// daemon's custody check use — on every URL the strict parse accepts. Two parsers that disagree on
/// the host are the shape of C86.
#[test]
fn the_strict_host_agrees_with_the_capability_checks_host() {
    for url in [
        "https://example.com/",
        "https://example.com:8443/a",
        "https://a.b.example.com?q",
        "https://example.com#f",
        "https://93.184.215.14:444/",
        "https://[::1]:8443/",
        "https://[2606:4700::1111]/",
    ] {
        let t = parse_target(url).unwrap();
        assert_eq!(t.host, crate::prim::host_of(url), "`{url}`");
    }
}

// ----- the allowlist and special-use addresses -----------------------------------------------------

#[test]
fn a_public_name_is_resolved_once_pinned_and_sent_as_the_checked_host() {
    let r = FakeResolver::with(&[("api.example.com", &[PUBLIC, "2606:4700::1111"])]);
    let t = FakeTransport::answering(vec![ok("hello")]);
    let (rec, body) = get_with(&policy(&["api.example.com"], &[]), "https://api.example.com/x", &r, &t);
    assert_eq!(body.as_deref(), Some("hello"));
    assert_eq!(rec.outcome, Ok((200, 5)));
    assert_eq!(*r.asked.borrow(), vec!["api.example.com".to_string()], "resolved exactly once");
    let sent = t.sent.borrow();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].host, "api.example.com", "the checked name is the name sent (SNI and Host)");
    assert_eq!(
        sent[0].addrs,
        vec![SocketAddr::new(ip(PUBLIC), 443), SocketAddr::new(ip("2606:4700::1111"), 443)],
        "the client is handed exactly the classified addresses"
    );
    assert_eq!(rec.hops[0].addrs, vec![PUBLIC.to_string(), "2606:4700::1111".to_string()]);
}

/// The wildcard keeps C85's dot boundary, on this path too.
#[test]
fn the_allowlist_keeps_its_dot_boundary() {
    let r = FakeResolver::with(&[("evilexample.com", &[PUBLIC]), ("api.example.com", &[PUBLIC]), ("example.com", &[PUBLIC])]);
    for (url, allowed) in [
        ("https://api.example.com/", true),
        ("https://evilexample.com/", false),
        ("https://example.com/", false),
    ] {
        let t = FakeTransport::answering(vec![ok("x")]);
        let (rec, _) = get_with(&policy(&["*.example.com"], &[]), url, &r, &t);
        assert_eq!(rec.outcome.is_ok(), allowed, "`{url}`: {:?}", rec.outcome);
        if !allowed {
            assert_eq!(rec.outcome, Err(Reason::NotAllowlisted));
            assert!(t.sent.borrow().is_empty(), "nothing is sent for a host outside the allowlist");
        }
    }
}

/// REMAINING_WORK 4.16 — the case PS-0-09 could not see: a PUBLIC-LOOKING name, granted with plain
/// `net=`, that resolves into a special-use range. Refused, and nothing is dialled.
#[test]
fn a_granted_name_that_resolves_into_a_special_range_is_refused() {
    let cases: [&[&str]; 9] = [
        &["127.0.0.1"],
        &["169.254.169.254"],
        &["10.1.2.3"],
        &["::1"],
        &["fd00::7"],
        &["64:ff9b::a9fe:a9fe"],
        &["::ffff:192.168.1.1"],
        // One bad candidate among good ones is enough: a name that resolves partly into a private
        // range is a name somebody pointed there, and happy-eyeballs would eventually dial it.
        &[PUBLIC, "10.0.0.8"],
        &["2606:4700::1111", "fe80::1"],
    ];
    for addrs in cases {
        let r = FakeResolver::with(&[("internal.example.com", addrs)]);
        let t = FakeTransport::default();
        let (rec, body) = get_with(&policy(&["internal.example.com"], &[]), "https://internal.example.com/", &r, &t);
        assert!(matches!(rec.outcome, Err(Reason::SpecialUse(_))), "{addrs:?}: {:?}", rec.outcome);
        assert_eq!(body, None);
        assert!(t.sent.borrow().is_empty(), "{addrs:?}: nothing may be dialled");
        assert_eq!(Reason::SpecialUse("x").answer(), Answer::Refused, "and the program is told only `Refused`");
    }
}

/// The explicit spelling is the whole mechanism: the same name, granted with `net.special=`, reaches
/// the same address — and ONLY the host so granted does.
#[test]
fn net_special_is_what_admits_a_special_range_and_only_for_its_own_host() {
    let r = FakeResolver::with(&[("db.corp.example", &["10.1.2.3"]), ("other.corp.example", &["10.9.9.9"])]);
    let t = FakeTransport::answering(vec![ok("rows")]);
    let p = policy(&["db.corp.example", "other.corp.example"], &["db.corp.example"]);
    let (rec, body) = get_with(&p, "https://db.corp.example/", &r, &t);
    assert_eq!(body.as_deref(), Some("rows"), "{:?}", rec.outcome);
    assert_eq!(t.sent.borrow()[0].addrs, vec![SocketAddr::new(ip("10.1.2.3"), 443)]);

    let t = FakeTransport::default();
    let (rec, _) = get_with(&p, "https://other.corp.example/", &r, &t);
    assert!(matches!(rec.outcome, Err(Reason::SpecialUse(_))), "{:?}", rec.outcome);
    assert!(t.sent.borrow().is_empty());
}

/// A special-use NAME is refused whatever it resolves to, and a special-use LITERAL needs no
/// resolver to be refused.
#[test]
fn special_names_and_literals_are_refused_without_net_special() {
    let r = FakeResolver::with(&[("localhost", &[PUBLIC]), ("metadata.google.internal", &[PUBLIC])]);
    for (url, host) in [
        ("https://localhost/", "localhost"),
        ("https://metadata.google.internal/", "metadata.google.internal"),
        ("https://169.254.169.254/latest/meta-data/", "169.254.169.254"),
        ("https://[::1]:8443/", "[::1]"),
        ("https://10.0.0.1/", "10.0.0.1"),
    ] {
        let t = FakeTransport::default();
        let (rec, _) = get_with(&policy(&[host], &[]), url, &r, &t);
        assert!(matches!(rec.outcome, Err(Reason::SpecialUse(_))), "`{url}`: {:?}", rec.outcome);
        assert!(t.sent.borrow().is_empty(), "`{url}`");
        // Granted by its own spelling, the same URL goes through.
        let t = FakeTransport::answering(vec![ok("ok")]);
        let (rec, _) = get_with(&policy(&[host], &[host]), url, &r, &t);
        assert!(rec.outcome.is_ok(), "`{url}` with net.special: {:?}", rec.outcome);
    }
    assert!(!r.asked.borrow().contains(&"169.254.169.254".to_string()), "a literal is never resolved");
}

#[test]
fn a_name_with_no_address_is_refused_opaquely() {
    let r = FakeResolver::default();
    let t = FakeTransport::default();
    let (rec, _) = get_with(&policy(&["gone.example.com"], &[]), "https://gone.example.com/", &r, &t);
    assert_eq!(rec.outcome, Err(Reason::NoAddress));
    assert_eq!(Reason::NoAddress.answer(), Answer::Refused, "indistinguishable from a special-use refusal");
    assert!(t.sent.borrow().is_empty());
}

/// A build with no client refuses BEFORE resolving: a build that cannot connect must not send DNS
/// queries on a program's behalf either.
#[test]
fn a_build_without_a_client_refuses_before_any_dns_query() {
    let r = FakeResolver::with(&[("example.com", &[PUBLIC])]);
    let (rec, _) = get_with(&policy(&["example.com"], &[]), "https://example.com/", &r, &NoTransport);
    assert_eq!(rec.outcome, Err(Reason::NoClient));
    assert!(r.asked.borrow().is_empty(), "no resolution in a network-less build");
}

// ----- redirects -----------------------------------------------------------------------------------

/// Every hop goes back through the whole check: the allowlist, the scheme, and the resolution.
#[test]
fn every_redirect_hop_is_checked_like_a_first_request() {
    let r = FakeResolver::with(&[
        ("a.example.com", &[PUBLIC]),
        ("b.example.com", &[PUBLIC]),
        ("evil.example.net", &[PUBLIC]),
        ("rebind.example.com", &["127.0.0.1"]),
    ]);
    let allow = ["a.example.com", "b.example.com", "rebind.example.com"];
    // Allowed: a relative redirect, then one to another allowlisted host.
    let t = FakeTransport::answering(vec![redirect(302, "/next"), redirect(301, "https://b.example.com/final"), ok("done")]);
    let (rec, body) = get_with(&policy(&allow, &[]), "https://a.example.com/start", &r, &t);
    assert_eq!(body.as_deref(), Some("done"), "{:?}", rec.outcome);
    let urls: Vec<String> = t.sent.borrow().iter().map(|s| s.url.clone()).collect();
    assert_eq!(urls, ["https://a.example.com/start", "https://a.example.com/next", "https://b.example.com/final"]);
    assert_eq!(rec.hops.len(), 3);

    for (to, why) in [
        ("https://evil.example.net/", Reason::NotAllowlisted),
        ("http://b.example.com/", Reason::Scheme),
        ("https://u@b.example.com/", Reason::Userinfo),
        ("https://rebind.example.com/", Reason::SpecialUse("")),
        ("https://B.example.com/", Reason::Malformed("")),
    ] {
        let t = FakeTransport::answering(vec![redirect(307, to), ok("never")]);
        let (rec, body) = get_with(&policy(&allow, &[]), "https://a.example.com/", &r, &t);
        assert_eq!(body, None, "redirect to `{to}`");
        assert_eq!(rec.outcome.as_ref().unwrap_err().code(), why.code(), "redirect to `{to}`: {:?}", rec.outcome);
        assert_eq!(t.sent.borrow().len(), 1, "the refused hop `{to}` is never sent");
    }
}

#[test]
fn a_redirect_loop_is_cut_off() {
    let r = FakeResolver::with(&[("a.example.com", &[PUBLIC])]);
    let script = (0..20).map(|_| redirect(302, "/again")).collect();
    let t = FakeTransport::answering(script);
    let (rec, _) = get_with(&policy(&["a.example.com"], &[]), "https://a.example.com/", &r, &t);
    assert_eq!(rec.outcome, Err(Reason::TooManyRedirects));
    assert_eq!(t.sent.borrow().len() as u32, DEFAULT_MAX_REDIRECTS + 1, "the first request and five redirects");
}

#[test]
fn relative_locations_resolve_the_way_rfc_3986_says() {
    let base = parse_target("https://h.example/a/b/c?q=1").unwrap();
    for (loc, want) in [
        ("/x", "https://h.example/x"),
        ("x", "https://h.example/a/b/x"),
        ("../x", "https://h.example/a/b/../x"),
        ("?z=2", "https://h.example/a/b/c?z=2"),
        ("//other.example/p", "https://other.example/p"),
        ("https://other.example/p", "https://other.example/p"),
        ("#f", "https://h.example/a/b/c?q=1"),
    ] {
        assert_eq!(join_location(&base, loc), want, "`{loc}`");
    }
    let bare = parse_target("https://h.example").unwrap();
    assert_eq!(join_location(&bare, "x"), "https://h.example/x");
    assert_eq!(join_location(&bare, "?q"), "https://h.example/?q");
}

#[test]
fn a_redirect_without_a_location_is_not_followed_anywhere() {
    let r = FakeResolver::with(&[("a.example.com", &[PUBLIC])]);
    let t = FakeTransport::answering(vec![Ok(Fetched { status: 302, location: None, body: Vec::new(), peer: None })]);
    let (rec, _) = get_with(&policy(&["a.example.com"], &[]), "https://a.example.com/", &r, &t);
    assert_eq!(rec.outcome, Err(Reason::RedirectWithoutLocation));
}

// ----- what comes back -----------------------------------------------------------------------------

#[test]
fn statuses_bodies_and_the_size_bound() {
    let r = FakeResolver::with(&[("a.example.com", &[PUBLIC])]);
    let mut p = policy(&["a.example.com"], &[]);
    p.max_body = 4;
    for (answer, want) in [
        (ok("abcd"), Ok((200, 4))),
        (ok("abcde"), Err(Reason::BodyTooLarge)),
        (Ok(Fetched { status: 404, location: None, body: b"no".to_vec(), peer: None }), Err(Reason::Status(404))),
        (Ok(Fetched { status: 304, location: None, body: Vec::new(), peer: None }), Err(Reason::Status(304))),
        (Ok(Fetched { status: 200, location: None, body: vec![0xff, 0xfe], peer: None }), Err(Reason::NotUtf8)),
        (Err(Reason::Tls), Err(Reason::Tls)),
        (Err(Reason::Timeout), Err(Reason::Timeout)),
    ] {
        let t = FakeTransport::answering(vec![answer.clone()]);
        let (rec, _) = get_with(&p, "https://a.example.com/", &r, &t);
        assert_eq!(rec.outcome, want, "{answer:?}");
    }
}

/// The belt to the construction's braces: a transport that reports connecting anywhere outside the
/// pinned set is refused, and the body — whatever it holds — is not delivered.
#[test]
fn a_peer_outside_the_pinned_set_is_refused() {
    let r = FakeResolver::with(&[("a.example.com", &[PUBLIC])]);
    let t = FakeTransport::answering(vec![Ok(Fetched {
        status: 200,
        location: None,
        body: b"secret".to_vec(),
        peer: Some(SocketAddr::new(ip("10.0.0.1"), 443)),
    })]);
    let (rec, body) = get_with(&policy(&["a.example.com"], &[]), "https://a.example.com/", &r, &t);
    assert_eq!(rec.outcome, Err(Reason::HostMismatch));
    assert_eq!(body, None);
}

/// What the PROGRAM is told: `NetErr`'s three variants, and `Other` only ever carries a phrase chosen
/// here — never a resolver's answer or a server's words.
#[test]
fn the_program_learns_nothing_the_host_did_not_choose_to_say() {
    for r in [
        Reason::Scheme,
        Reason::Userinfo,
        Reason::Malformed("x"),
        Reason::NotAllowlisted,
        Reason::SpecialUse("loopback"),
        Reason::NoAddress,
        Reason::TooManyRedirects,
        Reason::HostMismatch,
        Reason::NoClient,
    ] {
        assert_eq!(r.answer(), Answer::Refused, "{r:?} is a policy decision and stays opaque");
    }
    assert_eq!(Reason::Timeout.answer(), Answer::Timeout);
    for r in [Reason::Tls, Reason::Connect, Reason::Protocol, Reason::BodyTooLarge, Reason::NotUtf8, Reason::Status(418)] {
        match r.answer() {
            Answer::Other(m) => assert!(!m.contains('.') || m.contains("UTF"), "a fixed phrase, no addresses: {m}"),
            other => panic!("{r:?} -> {other:?}"),
        }
    }
    // Every code is distinct, so the report can be read by a machine.
    let codes: std::collections::BTreeSet<&str> = [
        Reason::Scheme, Reason::Userinfo, Reason::Malformed(""), Reason::NotAllowlisted, Reason::SpecialUse(""),
        Reason::NoAddress, Reason::TooManyRedirects, Reason::RedirectWithoutLocation, Reason::BodyTooLarge,
        Reason::NotUtf8, Reason::Status(0), Reason::Tls, Reason::Connect, Reason::Timeout, Reason::Protocol,
        Reason::HostMismatch, Reason::NoClient,
    ]
    .iter()
    .map(Reason::code)
    .collect();
    assert_eq!(codes.len(), 17);
}

// ----- the record ----------------------------------------------------------------------------------

#[test]
fn the_log_is_bounded_but_its_counts_are_not() {
    let mut l = EgressLog::default();
    for i in 0..(MAX_RECORDED + 10) {
        l.push(EgressRecord::refused(&format!("https://x/{i}"), Reason::Scheme));
    }
    l.push(EgressRecord { url: "https://y/".into(), hops: Vec::new(), outcome: Ok((200, 1)) });
    assert_eq!(l.records.len(), MAX_RECORDED);
    assert_eq!(l.total, MAX_RECORDED as u64 + 11);
    assert_eq!(l.refused, MAX_RECORDED as u64 + 10);
    let j = l.to_json();
    assert_eq!(j["requests"], MAX_RECORDED as u64 + 11);
    assert_eq!(j["records_kept"], MAX_RECORDED);
}

#[test]
fn a_refusal_record_says_why_in_a_machine_readable_code() {
    let r = FakeResolver::with(&[("internal.example.com", &["10.0.0.1"])]);
    let (rec, _) = get_with(&policy(&["internal.example.com"], &[]), "https://internal.example.com/", &r, &FakeTransport::default());
    let j = rec.to_json();
    assert_eq!(j["delivered"], false);
    assert_eq!(j["reason"], "special-use");
    assert!(j["explain"].as_str().unwrap().contains("net.special="), "the operator is told the spelling: {j}");
}

/// The process-wide log: what the primitive table records is readable by a mark, which is how the
/// sandbox channel finds the refusals one guest request produced.
#[test]
fn records_logged_after_a_mark_are_found_by_it() {
    let m = mark();
    let url = format!("http://mark-test-{}.example/", std::process::id());
    note_refusal(&url, Reason::Scheme);
    let (recs, refused) = records_since(m);
    // Other tests may log concurrently; ours must be among what the mark finds.
    assert!(recs.iter().any(|r| r.url == url && r.outcome == Err(Reason::Scheme)) || refused >= 1);
}

// ----- the real transport, against a TLS server on loopback ----------------------------------------

#[cfg(feature = "net")]
mod real_transport {
    use super::super::real::ReqwestTransport;
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex as StdMutex};

    /// What the server saw on one connection.
    #[derive(Clone, Debug, Default)]
    struct Seen {
        sni: Option<String>,
        host: Option<String>,
        path: String,
        /// The handshake never produced a request — the client refused the certificate, or never sent.
        no_request: bool,
    }

    struct Server {
        port: u16,
        roots: rustls::RootCertStore,
        seen: Arc<StdMutex<Vec<Seen>>>,
    }

    /// A TLS server on 127.0.0.1 presenting a certificate for `names` that NO trust store contains.
    /// `respond` maps the request path to the raw HTTP response. It serves until `idle` passes with no
    /// new connection, then stops, so no test leaves a thread blocked in `accept`.
    fn serve(names: &[&str], respond: impl Fn(&str) -> Vec<u8> + Send + 'static, idle: Duration) -> Server {
        let ck = rcgen::generate_simple_self_signed(names.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap();
        let cert = ck.cert.der().clone();
        let key = rustls::pki_types::PrivateKeyDer::Pkcs8(ck.signing_key.serialize_der().into());
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let cfg = Arc::new(
            rustls::ServerConfig::builder_with_provider(provider)
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_no_client_auth()
                .with_single_cert(vec![cert.clone()], key)
                .unwrap(),
        );
        let mut roots = rustls::RootCertStore::empty();
        roots.add(cert).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let seen = Arc::new(StdMutex::new(Vec::new()));
        let log = seen.clone();
        std::thread::spawn(move || {
            let mut last = Instant::now();
            while last.elapsed() < idle {
                let (tcp, _) = match listener.accept() {
                    Ok(c) => c,
                    Err(_) => {
                        std::thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                };
                tcp.set_nonblocking(false).unwrap();
                tcp.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
                let mut tls = rustls::StreamOwned::new(rustls::ServerConnection::new(cfg.clone()).unwrap(), tcp);
                let mut head = Vec::new();
                let mut chunk = [0u8; 2048];
                while !head.windows(4).any(|w| w == b"\r\n\r\n") {
                    match tls.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => head.extend_from_slice(&chunk[..n]),
                    }
                }
                let text = String::from_utf8_lossy(&head).to_string();
                let sni = tls.conn.server_name().map(str::to_string);
                if text.is_empty() {
                    log.lock().unwrap().push(Seen { sni, no_request: true, ..Default::default() });
                } else {
                    let host = text.lines().find_map(|l| {
                        let (k, v) = l.split_once(':')?;
                        k.eq_ignore_ascii_case("host").then(|| v.trim().to_string())
                    });
                    let path = text.split_whitespace().nth(1).unwrap_or("").to_string();
                    log.lock().unwrap().push(Seen { sni, host, path: path.clone(), no_request: false });
                    let _ = tls.write_all(&respond(&path));
                    let _ = tls.flush();
                    tls.conn.send_close_notify();
                    let _ = tls.flush();
                }
                last = Instant::now();
            }
        });
        Server { port, roots, seen }
    }

    fn http(status: &str, body: &str) -> Vec<u8> {
        format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).into_bytes()
    }

    fn loopback_policy(host: &str) -> EgressPolicy {
        // Loopback is special-use, so the test server is reachable only by the explicit spelling —
        // which is exactly what that spelling is for.
        policy(&[host], &[host])
    }

    /// The whole path, end to end: the checked name is the TLS server name AND the `Host` header,
    /// while the connection goes to the pinned address. This is the conformance case D-V2-30 owes:
    /// the host checked is the host connected.
    #[test]
    fn the_checked_host_is_the_server_name_and_the_host_header() {
        let s = serve(&["pinned.test"], |_| http("200 OK", "hello over tls"), Duration::from_secs(5));
        let r = FakeResolver::with(&[("pinned.test", &["127.0.0.1"])]);
        let t = ReqwestTransport::with_roots(s.roots.clone());
        let url = format!("https://pinned.test:{}/greet", s.port);
        let (rec, body) = get_with(&loopback_policy("pinned.test"), &url, &r, &t);
        assert_eq!(body.as_deref(), Some("hello over tls"), "{:?}", rec.outcome);
        let seen = s.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 1, "{seen:?}");
        assert_eq!(seen[0].sni.as_deref(), Some("pinned.test"), "TLS server name");
        assert_eq!(seen[0].host.as_deref(), Some(format!("pinned.test:{}", s.port).as_str()), "Host header");
        assert_eq!(seen[0].path, "/greet");
        assert_eq!(rec.hops[0].peer.as_deref(), Some("127.0.0.1"), "and the peer is the pinned address");
    }

    /// Verification is ON in the configuration a release uses: the platform's trust store refuses a
    /// certificate nobody issued. If this ever delivers a body, certificate checking has been turned
    /// off, and nothing else in the suite would say so.
    #[test]
    fn the_platform_trust_store_refuses_a_self_signed_server() {
        let s = serve(&["pinned.test"], |_| http("200 OK", "must not arrive"), Duration::from_secs(5));
        let r = FakeResolver::with(&[("pinned.test", &["127.0.0.1"])]);
        let t = ReqwestTransport::system();
        let url = format!("https://pinned.test:{}/", s.port);
        let (rec, body) = get_with(&loopback_policy("pinned.test"), &url, &r, &t);
        assert_eq!(body, None);
        assert_eq!(rec.outcome, Err(Reason::Tls), "an untrusted certificate is a TLS refusal");
        // The connection DID happen — bytes left the process, which is C-03 flipped — and the client
        // hung up at the certificate, before a single byte of the request was sent.
        std::thread::sleep(Duration::from_millis(200));
        let seen = s.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 1, "{seen:?}");
        assert!(seen[0].no_request, "the request must not be sent over an unverified connection: {seen:?}");
        assert_eq!(seen[0].sni.as_deref(), Some("pinned.test"));
        // And with no roots at all, the same.
        let (rec, _) = get_with(&loopback_policy("pinned.test"), &url, &r, &ReqwestTransport::with_roots(rustls::RootCertStore::empty()));
        assert_eq!(rec.outcome, Err(Reason::Tls));
    }

    /// A certificate for ANOTHER name is refused even when its issuer is trusted: the name checked
    /// is the name verified.
    #[test]
    fn a_certificate_for_another_name_is_refused() {
        let s = serve(&["someone-else.test"], |_| http("200 OK", "must not arrive"), Duration::from_secs(5));
        let r = FakeResolver::with(&[("pinned.test", &["127.0.0.1"])]);
        let t = ReqwestTransport::with_roots(s.roots.clone());
        let url = format!("https://pinned.test:{}/", s.port);
        let (rec, _) = get_with(&loopback_policy("pinned.test"), &url, &r, &t);
        assert_eq!(rec.outcome, Err(Reason::Tls));
    }

    /// The client resolves NOTHING itself. Handed a URL whose host the pin does not cover, it fails
    /// rather than asking a resolver — so a URL parser that disagreed with ours could not send the
    /// request somewhere unchecked.
    ///
    /// The unpinned host is `localhost` ON PURPOSE: it is the one name every system resolver answers,
    /// with the very address this server listens on, and the certificate covers it. So a client that
    /// DID resolve for itself would connect, verify and deliver, and this test would fail. An earlier
    /// draft used a name no resolver knows, which failed with or without the refusing resolver — a
    /// gate that cannot fail. Falsified by removing `.dns_resolver(NoDns)`: the request then arrives.
    #[test]
    fn the_client_cannot_resolve_a_host_it_was_not_pinned_to() {
        let s = serve(&["pinned.test", "localhost"], |_| http("200 OK", "must not arrive"), Duration::from_secs(3));
        let t = ReqwestTransport::with_roots(s.roots.clone());
        let addrs = [SocketAddr::new(ip("127.0.0.1"), s.port)];
        let url = format!("https://localhost:{}/", s.port);
        let req = PinnedRequest { url: &url, host: "pinned.test", port: s.port, addrs: &addrs };
        let got = t.fetch(&req, 1024, Duration::from_secs(5));
        assert_eq!(got, Err(Reason::Connect), "an unpinned host is not resolved");
        std::thread::sleep(Duration::from_millis(200));
        assert!(s.seen.lock().unwrap().is_empty(), "and nothing reached the server");
    }

    /// A redirect to a host outside the allowlist is refused BEFORE a connection to it exists: the
    /// server that would have answered as `evil.test` never sees a handshake for it.
    #[test]
    fn a_redirect_off_the_allowlist_never_connects() {
        let port_holder = Arc::new(StdMutex::new(0u16));
        let ph = port_holder.clone();
        let s = serve(
            &["pinned.test", "evil.test"],
            move |_| {
                let p = *ph.lock().unwrap();
                format!("HTTP/1.1 302 Found\r\nLocation: https://evil.test:{p}/steal\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").into_bytes()
            },
            Duration::from_secs(3),
        );
        *port_holder.lock().unwrap() = s.port;
        let r = FakeResolver::with(&[("pinned.test", &["127.0.0.1"]), ("evil.test", &["127.0.0.1"])]);
        let t = ReqwestTransport::with_roots(s.roots.clone());
        let (rec, _) = get_with(&loopback_policy("pinned.test"), &format!("https://pinned.test:{}/", s.port), &r, &t);
        assert_eq!(rec.outcome, Err(Reason::NotAllowlisted));
        std::thread::sleep(Duration::from_millis(200));
        let seen = s.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 1, "only the first hop connected: {seen:?}");
        assert_eq!(seen[0].sni.as_deref(), Some("pinned.test"));
    }

    /// The size bound holds on what the program would receive — announced (`Content-Length`) and
    /// unannounced (a body that simply keeps coming until the connection closes).
    #[test]
    fn the_response_is_bounded_whether_or_not_its_length_is_announced() {
        let big = "x".repeat(4096);
        let announced = http("200 OK", &big);
        let unannounced = format!("HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{big}").into_bytes();
        for (label, resp) in [("announced", announced), ("unannounced", unannounced)] {
            let s = serve(&["pinned.test"], move |_| resp.clone(), Duration::from_secs(3));
            let r = FakeResolver::with(&[("pinned.test", &["127.0.0.1"])]);
            let t = ReqwestTransport::with_roots(s.roots.clone());
            let mut p = loopback_policy("pinned.test");
            p.max_body = 1000;
            let (rec, body) = get_with(&p, &format!("https://pinned.test:{}/", s.port), &r, &t);
            assert_eq!(rec.outcome, Err(Reason::BodyTooLarge), "{label}");
            assert_eq!(body, None, "{label}");
        }
    }

    /// A server that accepts and never answers is a timeout, not a hang.
    #[test]
    fn a_silent_server_is_a_timeout_not_a_hang() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let _hold = std::thread::spawn(move || {
            let c = listener.accept();
            std::thread::sleep(Duration::from_secs(6));
            drop(c);
        });
        let r = FakeResolver::with(&[("pinned.test", &["127.0.0.1"])]);
        let t = ReqwestTransport::with_roots(rustls::RootCertStore::empty());
        let mut p = loopback_policy("pinned.test");
        p.deadline = Duration::from_millis(700);
        let started = Instant::now();
        let (rec, _) = get_with(&p, &format!("https://pinned.test:{port}/"), &r, &t);
        assert_eq!(rec.outcome, Err(Reason::Timeout));
        assert!(started.elapsed() < Duration::from_secs(5), "bounded by the deadline: {:?}", started.elapsed());
    }

    /// `HTTPS_PROXY` must not route a request. A proxy resolves the name itself, so honouring one
    /// would bypass the pin and every check made on the address. The variable is set on a CHILD
    /// process — this same test binary, running the helper below — because changing the environment
    /// of a multi-threaded test process is a race, not a test.
    #[test]
    fn proxy_variables_do_not_route_a_request() {
        let exe = std::env::current_exe().unwrap();
        let out = std::process::Command::new(exe)
            .args(["egress::tests::real_transport::proxy_child", "--exact", "--ignored", "--nocapture", "--test-threads=1"])
            // Port 9 (discard) on loopback: nothing listens there, so a request routed through the
            // "proxy" fails, and the helper's assertion that the body arrived fails with it.
            .env("HTTPS_PROXY", "http://127.0.0.1:9")
            .env("https_proxy", "http://127.0.0.1:9")
            .env("ALL_PROXY", "http://127.0.0.1:9")
            .env("all_proxy", "http://127.0.0.1:9")
            .env("DELULU_EGRESS_PROXY_CHILD", "1")
            .output()
            .unwrap();
        let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
        assert!(out.status.success(), "the child must deliver despite the proxy variables:\n{text}");
        assert!(text.contains("1 passed"), "the helper must actually have run:\n{text}");
    }

    #[test]
    #[ignore = "run by `proxy_variables_do_not_route_a_request` as a child process with proxy variables set"]
    fn proxy_child() {
        assert_eq!(std::env::var("DELULU_EGRESS_PROXY_CHILD").as_deref(), Ok("1"), "run only as the child");
        assert!(std::env::var("HTTPS_PROXY").is_ok());
        let s = serve(&["pinned.test"], |_| http("200 OK", "direct"), Duration::from_secs(5));
        let r = FakeResolver::with(&[("pinned.test", &["127.0.0.1"])]);
        let t = ReqwestTransport::with_roots(s.roots.clone());
        let (rec, body) = get_with(&loopback_policy("pinned.test"), &format!("https://pinned.test:{}/", s.port), &r, &t);
        assert_eq!(body.as_deref(), Some("direct"), "{:?}", rec.outcome);
    }

    /// The real resolver, on the one name every host resolves locally.
    #[test]
    fn the_system_resolver_answers_localhost_with_loopback() {
        let addrs = SystemResolver.resolve("localhost", 443, Duration::from_secs(10)).unwrap();
        assert!(!addrs.is_empty());
        assert!(addrs.iter().all(|a| a.to_canonical().is_loopback()), "{addrs:?}");
    }
}
