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
    /// The headers it was asked to send, as given.
    headers: Vec<(String, String)>,
    /// Its method and body (P9-05 (d), D-V2-183).
    method: String,
    body: Option<String>,
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
            headers: req.headers.to_vec(),
            method: req.method.to_string(),
            body: req.body.map(str::to_string),
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
    for url in
        ["https://u@example.com/", "https://u:p@example.com/", "https://example.com:8080@evil.com/steal", "https://@example.com/"]
    {
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
    for (url, allowed) in
        [("https://api.example.com/", true), ("https://evilexample.com/", false), ("https://example.com/", false)]
    {
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

/// P9-05 (d), D-V2-183: a POST sends its method and body, judged by its method on every hop. A 301, 302 or 303 after it
/// is followed as a GET with no body (judged as a GET, so a grant naming POST alone refuses it); a 307 or 308 keeps the
/// method and the body, to the origin the program named only. A body over the limit is refused before anything is sent.
#[test]
fn a_post_sends_its_body_by_its_method_and_a_redirect_keeps_or_drops_them_by_its_status() {
    let r = FakeResolver::with(&[("a.example.com", &[PUBLIC]), ("b.example.com", &[PUBLIC])]);
    let post = |allow: &[&str], t: &FakeTransport| {
        request(&policy(allow, &[]), "POST", "https://a.example.com/x", &[], Some("{\"k\":1}"), None, &r, t)
    };
    let sent = |t: &FakeTransport| -> Vec<(String, String, Option<String>)> {
        t.sent.borrow().iter().map(|s| (s.method.clone(), s.url.clone(), s.body.clone())).collect()
    };
    let body = Some("{\"k\":1}".to_string());

    let t = FakeTransport::answering(vec![ok("made")]);
    let (rec, got) = post(&["a.example.com:POST"], &t);
    assert_eq!(got.as_deref(), Some("made"), "{:?}", rec.outcome);
    assert_eq!(sent(&t), [("POST".to_string(), "https://a.example.com/x".to_string(), body.clone())]);
    assert_eq!(rec.to_json()["hops"][0]["method"], "POST", "the record names a POST");

    // Under an entry that names no methods — GET alone — the POST is refused before anything is resolved.
    let t = FakeTransport::answering(vec![ok("never")]);
    let asked = r.asked.borrow().len();
    let (rec, got) = post(&["a.example.com"], &t);
    assert_eq!((got, rec.outcome.clone()), (None, Err(Reason::NotAllowlisted)));
    assert!(t.sent.borrow().is_empty() && r.asked.borrow().len() == asked, "nothing resolved, nothing sent");

    // 303: followed as a GET with no body — refused under an entry naming POST alone, taken where GET is granted too.
    let t = FakeTransport::answering(vec![redirect(303, "/done"), ok("seen")]);
    let (rec, got) = post(&["a.example.com:POST"], &t);
    assert_eq!((got, rec.outcome.clone()), (None, Err(Reason::NotAllowlisted)), "the GET after a 303 is judged as one");
    let t = FakeTransport::answering(vec![redirect(303, "/done"), ok("seen")]);
    let (_, got) = post(&["a.example.com:GET,POST"], &t);
    assert_eq!(got.as_deref(), Some("seen"));
    assert_eq!(sent(&t)[1], ("GET".to_string(), "https://a.example.com/done".to_string(), None), "no body after a 303");
    for status in [301, 302] {
        let t = FakeTransport::answering(vec![redirect(status, "/done"), ok("seen")]);
        let _ = post(&["a.example.com:GET,POST"], &t);
        assert_eq!(sent(&t)[1].0, "GET", "{status}");
        assert_eq!(sent(&t)[1].2, None, "{status}");
    }

    // 307 and 308: the method and the body again, to the same origin; to another, refused before it is sent.
    for status in [307, 308] {
        let t = FakeTransport::answering(vec![redirect(status, "/again"), ok("kept")]);
        let (_, got) = post(&["a.example.com:POST"], &t);
        assert_eq!(got.as_deref(), Some("kept"), "{status}");
        assert_eq!(sent(&t)[1], ("POST".to_string(), "https://a.example.com/again".to_string(), body.clone()), "{status}");
        let t = FakeTransport::answering(vec![redirect(status, "https://b.example.com/x"), ok("never")]);
        let (rec, got) = post(&["a.example.com:POST", "b.example.com:POST"], &t);
        assert_eq!((got, rec.outcome.clone()), (None, Err(Reason::BodyElsewhere)), "{status}");
        assert_eq!(t.sent.borrow().len(), 1, "{status}: the body is never sent elsewhere");
    }

    let big = "x".repeat(MAX_REQUEST_BODY + 1);
    let t = FakeTransport::answering(vec![ok("never")]);
    let (rec, got) = request(&policy(&["a.example.com:POST"], &[]), "POST", "https://a.example.com/x", &[], Some(&big), None, &r, &t);
    assert_eq!((got, rec.outcome), (None, Err(Reason::RequestTooLarge)));
    assert!(t.sent.borrow().is_empty());
    // A GET's record is what it always was: no `method`.
    let t = FakeTransport::answering(vec![ok("plain")]);
    let (rec, _) = get_with(&policy(&["a.example.com"], &[]), "https://a.example.com/", &r, &t);
    assert!(rec.to_json()["hops"][0].get("method").is_none());
}

/// P9-05 (a), D-V2-138: a path-scoped entry is judged on every hop — a redirect out of the granted path is refused as a
/// redirect to an ungranted host is, and a path a server could read two ways is refused before anything is resolved. The
/// query is not the path; a host granted whole reaches every path.
#[test]
fn a_path_scope_is_judged_on_every_hop() {
    let r = FakeResolver::with(&[("a.example.com", &[PUBLIC]), ("b.example.com", &[PUBLIC])]);
    let allow = ["a.example.com/v1/**", "b.example.com"];
    let t = FakeTransport::answering(vec![
        redirect(302, "next"),
        redirect(301, "https://b.example.com/anything/../else"),
        ok("done"),
    ]);
    let (rec, body) = get_with(&policy(&allow, &[]), "https://a.example.com/v1/start?q=/v2", &r, &t);
    assert_eq!(body.as_deref(), Some("done"), "{:?}", rec.outcome);
    let urls: Vec<String> = t.sent.borrow().iter().map(|s| s.url.clone()).collect();
    assert_eq!(
        urls,
        ["https://a.example.com/v1/start?q=/v2", "https://a.example.com/v1/next", "https://b.example.com/anything/../else"]
    );
    for (first, to, why) in [
        ("https://a.example.com/v2/x", None, Reason::NotAllowlisted),
        ("https://a.example.com/", None, Reason::NotAllowlisted),
        ("https://a.example.com/v1/x", Some("/v2/x"), Reason::NotAllowlisted),
        ("https://a.example.com/v1/x", Some("../v2"), Reason::Malformed("")),
        ("https://a.example.com/v1/..%2Fv2", None, Reason::Malformed("")),
        ("https://a.example.com/v1/..;/v2", None, Reason::Malformed("")),
        ("https://a.example.com/v1//x", None, Reason::Malformed("")),
    ] {
        let script = match to {
            Some(to) => vec![redirect(307, to), ok("never")],
            None => vec![ok("never")],
        };
        let t = FakeTransport::answering(script);
        let (rec, body) = get_with(&policy(&allow, &[]), first, &r, &t);
        assert_eq!(body, None, "{first} -> {to:?}");
        assert_eq!(rec.outcome.as_ref().unwrap_err().code(), why.code(), "{first} -> {to:?}: {:?}", rec.outcome);
        assert_eq!(t.sent.borrow().len(), usize::from(to.is_some()), "{first} -> {to:?}: the refused hop is never sent");
    }
}

/// A request's headers, as a program's map hands them over.
type Headers = Vec<(String, String)>;

fn hs(pairs: &[(&str, &str)]) -> Headers {
    pairs.iter().map(|(n, v)| (n.to_string(), v.to_string())).collect()
}

/// P9-04 (a), D-V2-128: a program's headers are judged BEFORE anything is resolved or sent, and every
/// rule refuses rather than normalizes — each case below is a header some server would read as a
/// different request from the one the program's text says (a second header begun by a line break, a
/// `Host` that moves the request to another site on the checked address, a name in two cases).
#[test]
fn a_programs_headers_are_checked_before_anything_is_resolved() {
    let long_value = "v".repeat(MAX_HEADER_VALUE + 1);
    let long_name = "n".repeat(MAX_HEADER_NAME + 1);
    let many: Vec<(String, String)> = (0..=MAX_HEADERS).map(|i| (format!("x-h{i}"), "1".to_string())).collect();
    let big: Vec<(String, String)> = (0..3).map(|i| (format!("x-big{i}"), "b".repeat(MAX_HEADER_VALUE - 10))).collect();
    const VALUE: &str = "printable ASCII";
    const EDGE: &str = "begins or ends with whitespace";
    const TOKEN: &str = "not an HTTP token";
    let mut cases: Vec<(&str, Headers, &str)> = vec![
        ("CRLF in a value", hs(&[("x-a", "1\r\nHost: evil.example.net")]), VALUE),
        ("LF in a value", hs(&[("x-a", "1\nx-b: 2")]), VALUE),
        ("CR in a value", hs(&[("x-a", "1\r")]), VALUE),
        ("NUL in a value", hs(&[("x-a", "1\u{0}2")]), VALUE),
        ("DEL in a value", hs(&[("x-a", "1\u{7f}")]), VALUE),
        ("non-ASCII in a value", hs(&[("x-a", "caf\u{e9}")]), VALUE),
        ("a leading space", hs(&[("x-a", " 1")]), EDGE),
        ("a trailing tab", hs(&[("x-a", "1\t")]), EDGE),
        ("an empty name", hs(&[("", "1")]), TOKEN),
        ("a space in a name", hs(&[("x a", "1")]), TOKEN),
        ("a colon in a name", hs(&[("x-a:", "1")]), TOKEN),
        ("a pseudo-header", hs(&[(":authority", "evil.example.net")]), TOKEN),
        ("a line break in a name", hs(&[("x-a\r\nhost", "evil")]), TOKEN),
        ("a non-ASCII name", hs(&[("x-\u{e9}", "1")]), TOKEN),
        // Both ways round: the comparison must be made on the lower-cased name the SECOND spelling becomes,
        // not on how it was written (mutant M351 compared the written name, and the first order alone let it live).
        ("two cases of one name", hs(&[("Accept", "a"), ("accept", "b")]), "differ only in case"),
        ("two cases of one name, the other way", hs(&[("x-api-key", "a"), ("X-Api-Key", "b")]), "differ only in case"),
        ("too long a name", vec![(long_name, "1".into())], "longer than 128 bytes"),
        ("too long a value", vec![("x-a".into(), long_value)], "longer than 8 KiB"),
        ("too many", many, "more than 32 headers"),
        ("too large together", big, "larger than 16 KiB"),
    ];
    for (name, why) in RESERVED_HEADERS {
        let upper = name.to_ascii_uppercase();
        let leaked: &'static str = Box::leak(upper.into_boxed_str());
        cases.push((leaked, hs(&[(leaked, "x")]), why));
        cases.push((name, hs(&[("x-before", "1"), (name, "x")]), why));
    }
    for (label, headers, rule) in cases {
        let r = FakeResolver::with(&[("api.example.com", &[PUBLIC])]);
        let t = FakeTransport::answering(vec![ok("must not arrive")]);
        let (rec, body) = request_with(&policy(&["api.example.com"], &[]), "https://api.example.com/v1", &headers, &r, &t);
        assert_eq!(body, None, "{label}");
        assert!(
            matches!(rec.outcome, Err(Reason::Header(why)) if why.contains(rule)),
            "{label}: refused by the rule `{rule}`, got {:?}",
            rec.outcome
        );
        assert!(r.asked.borrow().is_empty(), "{label}: nothing is resolved for a refused header");
        assert!(t.sent.borrow().is_empty(), "{label}: nothing is sent");
        assert!(rec.hops.is_empty(), "{label}");
        assert_eq!(rec.outcome.as_ref().unwrap_err().answer(), Answer::Refused, "{label}: the program is told `Refused`");
    }
}

/// The accepted forms: every `tchar` a name may hold, an empty value, a tab and a space INSIDE a value,
/// a name the client would otherwise default (`User-Agent`, `Accept`), and exactly the bounds.
#[test]
fn headers_that_follow_the_rules_are_sent_lower_cased_in_the_order_given() {
    let r = FakeResolver::with(&[("api.example.com", &[PUBLIC])]);
    let t = FakeTransport::answering(vec![ok("{}")]);
    let given = hs(&[
        ("X-Api-Version", "7"),
        ("Accept", "application/json"),
        ("User-Agent", "probe/1"),
        ("x-!#$%&'*+-.^_`|~09az", "a\tb c"),
        ("x-empty", ""),
    ]);
    let (rec, body) = request_with(&policy(&["api.example.com"], &[]), "https://api.example.com/v1", &given, &r, &t);
    assert_eq!(body.as_deref(), Some("{}"), "{:?}", rec.outcome);
    let sent = t.sent.borrow();
    assert_eq!(
        sent[0].headers,
        hs(&[
            ("x-api-version", "7"),
            ("accept", "application/json"),
            ("user-agent", "probe/1"),
            ("x-!#$%&'*+-.^_`|~09az", "a\tb c"),
            ("x-empty", ""),
        ]),
        "names lower-cased, values as written, in the order given"
    );
    assert_eq!(rec.hops[0].headers, ["x-api-version", "accept", "user-agent", "x-!#$%&'*+-.^_`|~09az", "x-empty"]);
    // At the bounds exactly: accepted.
    let at: Vec<(String, String)> = (0..MAX_HEADERS).map(|i| (format!("x-{i}"), "1".to_string())).collect();
    let t = FakeTransport::answering(vec![ok("{}")]);
    let (rec, _) = request_with(&policy(&["api.example.com"], &[]), "https://api.example.com/v1", &at, &r, &t);
    assert!(rec.outcome.is_ok(), "{MAX_HEADERS} headers are allowed: {:?}", rec.outcome);
    let t = FakeTransport::answering(vec![ok("{}")]);
    let one = vec![("n".repeat(MAX_HEADER_NAME), "v".repeat(MAX_HEADER_VALUE))];
    let (rec, _) = request_with(&policy(&["api.example.com"], &[]), "https://api.example.com/v1", &one, &r, &t);
    assert!(rec.outcome.is_ok(), "a name and a value at their bounds are allowed: {:?}", rec.outcome);
}

/// What a program addressed to one origin is never handed to the next: a redirect to another host, or
/// to another PORT of the same host, drops the program's headers; a hop back to the origin it named
/// carries them again. Each hop's record names what it carried.
#[test]
fn the_headers_go_to_the_origin_the_program_named_and_to_no_other() {
    let r = FakeResolver::with(&[("a.example.com", &[PUBLIC]), ("b.example.com", &[PUBLIC])]);
    let allow = ["a.example.com", "b.example.com"];
    let t = FakeTransport::answering(vec![
        redirect(302, "/next"),
        redirect(302, "https://a.example.com:443/same-origin-spelled-with-its-port"),
        redirect(302, "https://b.example.com/elsewhere"),
        redirect(302, "https://a.example.com:8443/another-port"),
        redirect(302, "https://a.example.com/back"),
        ok("done"),
    ]);
    let mut p = policy(&allow, &[]);
    p.max_redirects = 6;
    let given = hs(&[("Authorization", "Bearer declassified"), ("Accept", "application/json")]);
    let (rec, body) = request_with(&p, "https://a.example.com/start", &given, &r, &t);
    assert_eq!(body.as_deref(), Some("done"), "{:?}", rec.outcome);
    let carried: Vec<(String, usize)> = t.sent.borrow().iter().map(|s| (s.url.clone(), s.headers.len())).collect();
    assert_eq!(
        carried,
        [
            ("https://a.example.com/start".to_string(), 2),
            ("https://a.example.com/next".to_string(), 2),
            ("https://a.example.com:443/same-origin-spelled-with-its-port".to_string(), 2),
            ("https://b.example.com/elsewhere".to_string(), 0),
            ("https://a.example.com:8443/another-port".to_string(), 0),
            ("https://a.example.com/back".to_string(), 2),
        ]
    );
    let named: Vec<usize> = rec.hops.iter().map(|h| h.headers.len()).collect();
    assert_eq!(named, [2, 2, 2, 0, 0, 2], "each hop's record names what it carried");
    // The record names headers, never their values — in the record and in what the report carries.
    let json = rec.to_json().to_string();
    assert!(json.contains("\"authorization\""), "{json}");
    assert!(!json.contains("declassified") && !json.contains("application/json"), "no value in the record: {json}");
    // A plain `get` sends none and its record says nothing of headers.
    let t = FakeTransport::answering(vec![ok("x")]);
    let (rec, _) = get_with(&policy(&allow, &[]), "https://a.example.com/", &r, &t);
    assert!(t.sent.borrow()[0].headers.is_empty());
    assert!(!rec.to_json().to_string().contains("headers"), "{}", rec.to_json());
}

// ----- a secret in a header (P9-04 (b)/(c), D-V2-131) ---------------------------------------------

const SECRET_BYTES: &str = "tok_8f2c-never-in-a-record";

fn aud(members: &[&str]) -> Vec<Audience> {
    members.iter().map(|m| parse_audience(m).unwrap_or_else(|e| panic!("`{m}`: {e}"))).collect()
}

/// The secret goes where its audience and the origin rule BOTH allow, and nowhere else: a same-origin redirect
/// carries it (the origin spelled with its port too), a redirect to another host does not, and — the second
/// wall — neither does a redirect to another PORT of the same host even though the audience names that port:
/// the program addressed one origin. Back at the origin it is carried again. Each hop's record names the secret
/// it carried, never its bytes.
#[test]
fn a_secret_goes_only_to_its_audience_on_the_origins_hops() {
    let r = FakeResolver::with(&[("a.example.com", &[PUBLIC]), ("b.example.com", &[PUBLIC])]);
    let allow = ["a.example.com", "b.example.com"];
    let t = FakeTransport::answering(vec![
        redirect(302, "/next"),
        redirect(302, "https://a.example.com:443/same-origin-spelled-with-its-port"),
        redirect(302, "https://b.example.com/elsewhere"),
        redirect(302, "https://a.example.com:8443/another-port-in-the-audience"),
        redirect(302, "https://a.example.com/back"),
        ok("done"),
    ]);
    let mut p = policy(&allow, &[]);
    p.max_redirects = 6;
    let to = aud(&["a.example.com", "a.example.com:8443", "b.example.com"]);
    let secret = SecretHeader { header: "Authorization", name: "API_TOKEN", value: SECRET_BYTES, audience: &to };
    let given = hs(&[("Accept", "application/json")]);
    let (rec, body) = request_with_secret(&p, "https://a.example.com/start", &given, Some(&secret), &r, &t);
    assert_eq!(body.as_deref(), Some("done"), "{:?}", rec.outcome);
    let carried: Vec<(String, Vec<(String, String)>)> =
        t.sent.borrow().iter().map(|s| (s.url.clone(), s.headers.clone())).collect();
    let with = hs(&[("accept", "application/json"), ("authorization", SECRET_BYTES)]);
    assert_eq!(
        carried,
        [
            ("https://a.example.com/start".to_string(), with.clone()),
            ("https://a.example.com/next".to_string(), with.clone()),
            ("https://a.example.com:443/same-origin-spelled-with-its-port".to_string(), with.clone()),
            ("https://b.example.com/elsewhere".to_string(), Vec::new()),
            ("https://a.example.com:8443/another-port-in-the-audience".to_string(), Vec::new()),
            ("https://a.example.com/back".to_string(), with.clone()),
        ]
    );
    let named: Vec<Option<&str>> = rec.hops.iter().map(|h| h.secret.as_deref()).collect();
    assert_eq!(named, [Some("API_TOKEN"), Some("API_TOKEN"), Some("API_TOKEN"), None, None, Some("API_TOKEN")]);
    let json = rec.to_json().to_string();
    assert!(json.contains("\"secret\":\"API_TOKEN\"") && json.contains("\"authorization\""), "{json}");
    assert!(!json.contains(SECRET_BYTES), "the record never holds the secret's bytes: {json}");
    assert!(!format!("{secret:?}").contains(SECRET_BYTES), "nor does the carrier's own Debug");
    let pinned = PinnedRequest { url: "https://a.example.com/", host: "a.example.com", port: 443, addrs: &[], headers: &with, method: "GET", body: None };
    let shown = format!("{pinned:?}");
    assert!(shown.contains("authorization") && !shown.contains(SECRET_BYTES), "nor a pinned request's: {shown}");
}

/// Outside its audience, a secret is refused BEFORE anything is resolved or sent — another host, another port of
/// the audience's host, a sub-domain of it, an address the audience's name might resolve to, and a secret whose
/// grant named no audience at all. The program is told `Refused`; the operator `audience`; the bytes appear in no
/// record or reason.
#[test]
fn a_secret_outside_its_audience_is_refused_before_anything_is_resolved() {
    let cases: [(&str, &str, Vec<Audience>, &str); 5] = [
        ("another host", "https://evil.example.net/x", aud(&["api.example.com"]), "outside the audience"),
        ("another port", "https://api.example.com:8443/x", aud(&["api.example.com"]), "outside the audience"),
        ("a sub-domain", "https://v2.api.example.com/x", aud(&["api.example.com"]), "outside the audience"),
        ("an address it resolves to", &format!("https://{PUBLIC}/x"), aud(&["api.example.com"]), "outside the audience"),
        ("no audience", "https://api.example.com/x", Vec::new(), "names no audience"),
    ];
    let allow = ["api.example.com", "evil.example.net", "*.api.example.com", PUBLIC];
    for (label, url, to, rule) in cases {
        let r = FakeResolver::with(&[
            ("api.example.com", &[PUBLIC]),
            ("evil.example.net", &[PUBLIC]),
            ("v2.api.example.com", &[PUBLIC]),
        ]);
        let t = FakeTransport::answering(vec![ok("must not arrive")]);
        let secret = SecretHeader { header: "X-Api-Key", name: "API_TOKEN", value: SECRET_BYTES, audience: &to };
        let (rec, body) = request_with_secret(&policy(&allow, &[]), url, &[], Some(&secret), &r, &t);
        assert_eq!(body, None, "{label}");
        assert!(matches!(&rec.outcome, Err(Reason::Audience(why)) if why.contains(rule)), "{label}: {:?}", rec.outcome);
        assert_eq!(rec.outcome.as_ref().unwrap_err().answer(), Answer::Refused, "{label}: the program is told `Refused`");
        assert_eq!(rec.outcome.as_ref().unwrap_err().code(), "audience", "{label}");
        assert!(r.asked.borrow().is_empty(), "{label}: nothing is resolved");
        assert!(t.sent.borrow().is_empty(), "{label}: nothing is sent");
        assert!(rec.hops.is_empty(), "{label}");
        assert!(!rec.to_json().to_string().contains(SECRET_BYTES), "{label}: {}", rec.to_json());
    }
    // The same request inside the audience is sent — so each refusal above is the audience's, not the URL's.
    let r = FakeResolver::with(&[("api.example.com", &[PUBLIC])]);
    let t = FakeTransport::answering(vec![ok("{}")]);
    let to = aud(&["api.example.com"]);
    let secret = SecretHeader { header: "X-Api-Key", name: "API_TOKEN", value: SECRET_BYTES, audience: &to };
    let (rec, body) = request_with_secret(&policy(&allow, &[]), "https://api.example.com/x", &[], Some(&secret), &r, &t);
    assert_eq!(body.as_deref(), Some("{}"), "{:?}", rec.outcome);
    assert_eq!(t.sent.borrow()[0].headers, hs(&[("x-api-key", SECRET_BYTES)]));
}

/// The secret's header is a header: held to every rule a program's is, over the whole set — it cannot be `Host`,
/// cannot repeat a plain header's name in another case (both ways round, M351's lesson), and bytes that would begin
/// another header are refused — with a reason that never quotes them.
#[test]
fn a_secrets_header_is_held_to_every_header_rule() {
    let to = aud(&["api.example.com"]);
    let cases: [(&str, &str, &str, Headers, &str); 5] = [
        ("Host", "Host", SECRET_BYTES, Vec::new(), "`Host`"),
        (
            "a plain name, then the secret's in another case",
            "X-Api-Key",
            SECRET_BYTES,
            hs(&[("x-api-key", "1")]),
            "differ only in case",
        ),
        ("the other way round", "x-api-key", SECRET_BYTES, hs(&[("X-Api-Key", "1")]), "differ only in case"),
        ("a line break in the bytes", "Authorization", "8f2c\r\nHost: evil.example.net", Vec::new(), "printable ASCII"),
        ("not a token", "Author ization", SECRET_BYTES, Vec::new(), "not an HTTP token"),
    ];
    for (label, header, value, plain, rule) in cases {
        let r = FakeResolver::with(&[("api.example.com", &[PUBLIC])]);
        let t = FakeTransport::answering(vec![ok("must not arrive")]);
        let secret = SecretHeader { header, name: "API_TOKEN", value, audience: &to };
        let (rec, _) =
            request_with_secret(&policy(&["api.example.com"], &[]), "https://api.example.com/x", &plain, Some(&secret), &r, &t);
        assert!(matches!(&rec.outcome, Err(Reason::Header(why)) if why.contains(rule)), "{label}: {:?}", rec.outcome);
        assert!(t.sent.borrow().is_empty() && r.asked.borrow().is_empty(), "{label}");
        let explain = rec.outcome.as_ref().unwrap_err().explain();
        assert!(
            !explain.contains("8f2c") && !explain.contains("evil.example.net"),
            "{label}: the reason never quotes the bytes: {explain}"
        );
    }
}

/// An audience member is spelled as a URL's host is compared — and every other spelling is refused rather than
/// normalized, so the audience and the request can never be judged in two spellings.
#[test]
fn an_audience_is_spelled_as_the_request_is_compared() {
    for (member, host, port) in [
        ("api.example.com", "api.example.com", 443),
        ("api.example.com:8443", "api.example.com", 8443),
        ("93.184.215.14", "93.184.215.14", 443),
        ("[2001:db8::1]:444", "[2001:db8::1]", 444),
    ] {
        assert_eq!(parse_audience(member), Ok(Audience { host: host.to_string(), port }), "`{member}`");
    }
    for (member, says) in [
        ("", "empty"),
        ("api.example.com/repos", "path scopes"),
        ("api.example.com?x", "path scopes"),
        ("*.example.com", "wildcard"),
        ("API.example.com", "lower case"),
        ("api.example.com.", "ends in a dot"),
        ("user@api.example.com", "a second `@`"),
        ("api.example.com:0", "port"),
        ("api.example.com:08443", "port"),
        ("127.1", "dotted-quad"),
        ("[2001:DB8::1]", "canonical"),
        ("exam ple.com", "space"),
    ] {
        match parse_audience(member) {
            Ok(a) => panic!("`{member}` must be refused, got {a:?}"),
            Err(e) => assert!(e.contains(says), "`{member}`: {e}"),
        }
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
        Reason::Scheme,
        Reason::Userinfo,
        Reason::Malformed(""),
        Reason::NotAllowlisted,
        Reason::SpecialUse(""),
        Reason::NoAddress,
        Reason::TooManyRedirects,
        Reason::RedirectWithoutLocation,
        Reason::BodyTooLarge,
        Reason::NotUtf8,
        Reason::Status(0),
        Reason::Tls,
        Reason::Connect,
        Reason::Timeout,
        Reason::Protocol,
        Reason::HostMismatch,
        Reason::NoClient,
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
    let (rec, _) =
        get_with(&policy(&["internal.example.com"], &[]), "https://internal.example.com/", &r, &FakeTransport::default());
    let j = rec.to_json();
    assert_eq!(j["delivered"], false);
    assert_eq!(j["reason"], "special-use");
    assert!(j["explain"].as_str().unwrap().contains("net.special="), "the operator is told the spelling: {j}");
}

/// What the primitive table records is readable by a mark, which is how the sandbox channel finds the
/// refusals one guest request produced — this thread's, exactly, whatever other threads log meanwhile
/// (RW 7.20: the mark had read the process-wide log, and a test beside it was counted as ours).
#[test]
fn records_logged_after_a_mark_are_found_by_it() {
    let m = mark();
    let url = format!("http://mark-test-{}.example/", std::process::id());
    note_refusal(&url, Reason::Scheme);
    std::thread::spawn(|| note_refusal("http://another-thread.example/", Reason::Scheme)).join().unwrap();
    let (recs, refused) = records_since(m);
    assert_eq!(refused, 1, "{recs:?}");
    assert!(matches!(recs.as_slice(), [r] if r.url == url && r.outcome == Err(Reason::Scheme)), "{recs:?}");
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
        /// The request's whole head, as it arrived (P9-04 (a): the headers a program sent).
        head: String,
        /// Its body, read to its `Content-Length` (P9-05 (d), D-V2-183).
        body: String,
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
                // A body: what came after the head, then the rest of its `Content-Length`.
                let split = head.windows(4).position(|w| w == b"\r\n\r\n").map_or(head.len(), |i| i + 4);
                let mut body = head.split_off(split);
                let length = String::from_utf8_lossy(&head).lines().find_map(|l| {
                    let (k, v) = l.split_once(':')?;
                    k.eq_ignore_ascii_case("content-length").then(|| v.trim().parse::<usize>().ok()).flatten()
                });
                while body.len() < length.unwrap_or(0) {
                    match tls.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => body.extend_from_slice(&chunk[..n]),
                    }
                }
                let body = String::from_utf8_lossy(&body).to_string();
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
                    log.lock().unwrap().push(Seen { sni, host, path: path.clone(), head: text.clone(), body, no_request: false });
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

    /// P9-04 (a), D-V2-128, end to end over real TLS: the program's headers arrive as it wrote them
    /// (lower-case names), one of each — a `User-Agent` it set REPLACES the client's rather than joining
    /// it — and the `Host` header is still the checked host's.
    #[test]
    fn a_programs_headers_arrive_over_tls_and_the_host_header_stays_the_checked_one() {
        let s = serve(&["pinned.test"], |_| http("200 OK", "with headers"), Duration::from_secs(5));
        let r = FakeResolver::with(&[("pinned.test", &["127.0.0.1"])]);
        let t = ReqwestTransport::with_roots(s.roots.clone());
        let url = format!("https://pinned.test:{}/h", s.port);
        let given = hs(&[("X-Api-Version", "7"), ("User-Agent", "probe/1"), ("Accept", "application/json")]);
        let (rec, body) = request_with(&loopback_policy("pinned.test"), &url, &given, &r, &t);
        assert_eq!(body.as_deref(), Some("with headers"), "{:?}", rec.outcome);
        let seen = s.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 1, "{seen:?}");
        let lines: Vec<String> = seen[0].head.lines().map(|l| l.to_ascii_lowercase()).collect();
        let count = |prefix: &str| lines.iter().filter(|l| l.starts_with(prefix)).count();
        assert!(lines.contains(&"x-api-version: 7".to_string()), "{lines:?}");
        assert!(lines.contains(&"user-agent: probe/1".to_string()), "{lines:?}");
        assert!(lines.contains(&"accept: application/json".to_string()), "{lines:?}");
        assert_eq!(count("user-agent:"), 1, "the program's User-Agent replaces the client's: {lines:?}");
        assert_eq!(count("accept:"), 1, "{lines:?}");
        assert_eq!(count("host:"), 1, "{lines:?}");
        assert_eq!(seen[0].host.as_deref(), Some(format!("pinned.test:{}", s.port).as_str()), "Host header");
        assert_eq!(rec.hops[0].headers, ["x-api-version", "user-agent", "accept"]);
    }

    /// P9-05 (d), D-V2-183, end to end over real TLS: a POST arrives as a POST, with the program's body whole and its
    /// length, and its headers as written.
    #[test]
    fn a_post_arrives_over_tls_with_its_body() {
        let s = serve(&["pinned.test"], |_| http("201 Created", "made"), Duration::from_secs(5));
        let r = FakeResolver::with(&[("pinned.test", &["127.0.0.1"])]);
        let t = ReqwestTransport::with_roots(s.roots.clone());
        let url = format!("https://pinned.test:{}/items", s.port);
        let given = hs(&[("Content-Type", "application/json")]);
        let sent = "{\"name\":\"tea\",\"qty\":2}";
        let p = policy(&["pinned.test:POST"], &["pinned.test:POST"]);
        let (rec, body) = request(&p, "POST", &url, &given, Some(sent), None, &r, &t);
        assert_eq!(body.as_deref(), Some("made"), "{:?}", rec.outcome);
        let seen = s.seen.lock().unwrap().clone();
        assert_eq!(seen.len(), 1, "{seen:?}");
        assert!(seen[0].head.starts_with("POST /items HTTP/1.1\r\n"), "{:?}", seen[0].head);
        let lines: Vec<String> = seen[0].head.lines().map(|l| l.to_ascii_lowercase()).collect();
        assert!(lines.contains(&format!("content-length: {}", sent.len())), "{lines:?}");
        assert!(lines.contains(&"content-type: application/json".to_string()), "{lines:?}");
        assert_eq!(seen[0].body, sent, "the body arrives whole");
        assert_eq!(rec.hops[0].method, "POST");
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
        let (rec, _) =
            get_with(&loopback_policy("pinned.test"), &url, &r, &ReqwestTransport::with_roots(rustls::RootCertStore::empty()));
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
        let req = PinnedRequest { url: &url, host: "pinned.test", port: s.port, addrs: &addrs, headers: &[], method: "GET", body: None };
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
