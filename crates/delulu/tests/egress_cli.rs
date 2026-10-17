//! PS-B-02 at the command line: the network family (`SANDBOX_TEST_PLAN.md` §5.3).
//!
//! **C-03, flipped.** The characterization test pinned that "no bytes leave the process — a listening
//! socket on the granted host sees nothing", because `http.get` had no client behind it (NE-17). It
//! now has one, so the same listener must see a request arrive, and it must arrive as TLS: the first
//! bytes are a handshake record, never an HTTP request line in the clear.
//!
//! And the property PS-B-02 is built around, observed from outside: an L0 run and a sandboxed guest
//! reach the network through ONE host-side implementation. The listener cannot tell them apart,
//! because it is the same code talking to it.
//!
//! Everything here stays on loopback. Loopback is special-use, so each grant is `net.special=`, which
//! is what that spelling exists for.

use std::io::Read;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

fn isolated_state() -> PathBuf {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let d = std::env::temp_dir().join(format!("delulu-teststate-egress-{}-{t}", std::process::id()));
        let _ = std::fs::create_dir_all(d.join("audit"));
        d
    })
    .clone()
}

fn tmp(name: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let d = std::env::temp_dir().join(format!("delulu-egress-{name}-{}-{t}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("a temp directory");
    d
}

fn delulu(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(dir)
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .env("DELULU_STATE_DIR", isolated_state())
        .output()
        .expect("the delulu binary runs")
}

fn out(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

/// A program that fetches `url` from the host list `hosts` and says whether a body arrived.
fn fetcher(dir: &Path, hosts: &str, url: &str) {
    std::fs::write(
        dir.join("fetch.delulu"),
        format!(
            "module fetch\n\nfn main(root: Root) ! {{Net, Write}} {{\n    \
             let out = root.console()\n    \
             let h = root.http([\"{hosts}\"])\n    \
             match h.get(\"{url}\") {{\n        \
             Ok(body) => out.println(\"delivered\")\n        \
             Err(e) => out.println(\"not delivered\")\n    \
             }}\n}}\n"
        ),
    )
    .unwrap();
}

/// A loopback listener that keeps the first bytes of the first connection, then hangs up. It gives
/// up after `wait` with no connection, so a run that never connects cannot hang the test.
fn listen(wait: Duration) -> (u16, std::thread::JoinHandle<Option<Vec<u8>>>) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    l.set_nonblocking(true).unwrap();
    let port = l.local_addr().unwrap().port();
    let h = std::thread::spawn(move || {
        let start = Instant::now();
        loop {
            match l.accept() {
                Ok((mut s, _)) => {
                    s.set_nonblocking(false).unwrap();
                    s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
                    let mut buf = vec![0u8; 5];
                    let mut got = 0;
                    while got < buf.len() {
                        match s.read(&mut buf[got..]) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => got += n,
                        }
                    }
                    buf.truncate(got);
                    return Some(buf);
                }
                Err(_) if start.elapsed() < wait => std::thread::sleep(Duration::from_millis(5)),
                Err(_) => return None,
            }
        }
    });
    (port, h)
}

fn report(dir: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(dir.join("rep.json")).expect("the run report");
    serde_json::from_str(text.trim()).expect("one JSON value")
}

/// C-03 flipped, at L0 and under `--sandbox`: the bytes leave, and they leave as TLS.
#[test]
fn a_granted_request_leaves_the_process_as_tls_at_l0_and_from_a_guest() {
    for sandbox in [false, true] {
        let dir = tmp(if sandbox { "tls-guest" } else { "tls-l0" });
        let (port, seen) = listen(Duration::from_secs(20));
        fetcher(&dir, "127.0.0.1", &format!("https://127.0.0.1:{port}/probe"));
        let mut args = vec!["run", "fetch.delulu", "--grant", "console", "--grant", "net.special=127.0.0.1", "--report-out", "rep.json"];
        if sandbox {
            args.push("--sandbox");
        }
        let o = delulu(&dir, &args);
        let first = seen.join().unwrap().unwrap_or_else(|| panic!("sandbox={sandbox}: nothing connected:\n{}", out(&o)));
        assert!(
            first.len() >= 3 && first[0] == 0x16 && first[1] == 0x03,
            "sandbox={sandbox}: the first bytes must be a TLS handshake record, got {first:02x?}:\n{}",
            out(&o)
        );
        // The listener hung up without a certificate, so nothing was delivered — and the program was
        // told so as a value, not a crash.
        assert_eq!(o.status.code(), Some(0), "sandbox={sandbox}:\n{}", out(&o));
        assert!(String::from_utf8_lossy(&o.stdout).contains("not delivered"), "sandbox={sandbox}:\n{}", out(&o));
        // The host's record: one request, pinned to the address the grant named.
        let v = report(&dir);
        assert_eq!(v["egress"]["requests"], 1, "sandbox={sandbox}: {v}");
        let rec = &v["egress"]["records"][0];
        assert_eq!(rec["delivered"], false, "{v}");
        assert_eq!(rec["hops"][0]["addrs"], serde_json::json!(["127.0.0.1"]), "{v}");
        assert!(
            ["connect", "tls", "protocol"].contains(&rec["reason"].as_str().unwrap_or("")),
            "a handshake the server abandoned is a transport failure: {v}"
        );
    }
}

/// A refusal carries its machine-readable reason to the operator — on stderr, in the run report, and
/// for a guest in `denied[]` too — while the program is told only `Refused`.
#[test]
fn a_refused_request_says_why_to_the_operator_and_nothing_to_the_program() {
    for sandbox in [false, true] {
        let dir = tmp(if sandbox { "scheme-guest" } else { "scheme-l0" });
        fetcher(&dir, "example.com", "http://example.com/plain");
        let mut args = vec!["run", "fetch.delulu", "--grant", "console", "--grant", "net=example.com", "--report-out", "rep.json"];
        if sandbox {
            args.push("--sandbox");
        }
        let o = delulu(&dir, &args);
        assert_eq!(o.status.code(), Some(0), "sandbox={sandbox}:\n{}", out(&o));
        let stdout = String::from_utf8_lossy(&o.stdout);
        assert!(stdout.contains("not delivered"), "sandbox={sandbox}:\n{}", out(&o));
        assert!(!stdout.contains("scheme"), "the program's own output learns nothing of the reason:\n{stdout}");
        assert!(
            String::from_utf8_lossy(&o.stderr).contains("[egress: scheme]"),
            "sandbox={sandbox}: the operator is told why on stderr:\n{}",
            out(&o)
        );
        let v = report(&dir);
        assert_eq!(v["egress"]["records"][0]["reason"], "scheme", "sandbox={sandbox}: {v}");
        if sandbox {
            let denied = v["sandbox"]["denied"].as_array().expect("denied[]");
            assert!(
                denied.iter().any(|d| d.as_str().is_some_and(|s| s.contains("scheme") && s.contains("http://example.com/plain"))),
                "the guest's refused request is in denied[]: {v}"
            );
        }
    }
}

/// A program that fetches `url` with `headers` through `get_with` (P9-04 (a), D-V2-128).
fn fetcher_with(dir: &Path, hosts: &str, url: &str, headers: &[(&str, &str)]) {
    let inserts: String = headers.iter().map(|(n, v)| format!("    hs.insert(\"{n}\", \"{v}\")\n")).collect();
    std::fs::write(
        dir.join("fetch.delulu"),
        format!(
            "module fetch\n\nfn main(root: Root) ! {{Net, Write}} {{\n    \
             let out = root.console()\n    \
             let h = root.http([\"{hosts}\"])\n    \
             let hs = Map()\n{inserts}    \
             match h.get_with(\"{url}\", hs) {{\n        \
             Ok(body) => out.println(\"delivered\")\n        \
             Err(e) => out.println(\"not delivered\")\n    \
             }}\n}}\n"
        ),
    )
    .unwrap();
}

/// P9-04 (a): a request WITH HEADERS leaves as TLS at L0 and from a guest — whose map of headers crosses
/// the channel for the host to send — and the report names the headers each hop carried, never a value.
#[test]
fn a_request_with_headers_leaves_as_tls_at_l0_and_from_a_guest_and_the_report_names_them() {
    for sandbox in [false, true] {
        let dir = tmp(if sandbox { "hdr-guest" } else { "hdr-l0" });
        let (port, seen) = listen(Duration::from_secs(20));
        fetcher_with(&dir, "127.0.0.1", &format!("https://127.0.0.1:{port}/probe"), &[("X-Api-Version", "7"), ("Accept", "text/plain")]);
        let mut args = vec!["run", "fetch.delulu", "--grant", "console", "--grant", "net.special=127.0.0.1", "--report-out", "rep.json"];
        if sandbox {
            args.push("--sandbox");
        }
        let o = delulu(&dir, &args);
        let first = seen.join().unwrap().unwrap_or_else(|| panic!("sandbox={sandbox}: nothing connected:\n{}", out(&o)));
        assert!(first.len() >= 3 && first[0] == 0x16 && first[1] == 0x03, "sandbox={sandbox}: TLS, got {first:02x?}:\n{}", out(&o));
        assert_eq!(o.status.code(), Some(0), "sandbox={sandbox}:\n{}", out(&o));
        assert!(String::from_utf8_lossy(&o.stdout).contains("not delivered"), "sandbox={sandbox}:\n{}", out(&o));
        let v = report(&dir);
        assert_eq!(v["egress"]["requests"], 1, "sandbox={sandbox}: {v}");
        let hop = &v["egress"]["records"][0]["hops"][0];
        assert_eq!(hop["headers"], serde_json::json!(["accept", "x-api-version"]), "sandbox={sandbox}: the names, in the map's order: {v}");
        assert!(!v.to_string().contains("text/plain"), "sandbox={sandbox}: no header value in the report: {v}");
        // And the report, `headers` and all, is still a `run-report` (a field the schema had never seen is a field
        // nothing would have held to it — routine run 21's lesson).
        let o = delulu(&dir, &["schema", "validate", "run-report", "rep.json", "--json"]);
        let check: serde_json::Value = serde_json::from_slice(&o.stdout).expect("JSON");
        assert_eq!(check["validate"]["valid"], true, "sandbox={sandbox}: {}\n{v}", check["validate"]["errors"]);
    }
}

/// A header the egress client refuses (here `Host`, which would carry the request to another site on
/// the checked address) is refused before anything connects: the program is told `Refused`, the
/// operator why — on stderr, in the report, and for a guest in `denied[]`.
#[test]
fn a_refused_header_connects_nowhere_and_says_why_to_the_operator() {
    for sandbox in [false, true] {
        let dir = tmp(if sandbox { "hdr-refused-guest" } else { "hdr-refused-l0" });
        let (port, seen) = listen(Duration::from_secs(3));
        fetcher_with(&dir, "127.0.0.1", &format!("https://127.0.0.1:{port}/probe"), &[("Host", "evil.example.net")]);
        let mut args = vec!["run", "fetch.delulu", "--grant", "console", "--grant", "net.special=127.0.0.1", "--report-out", "rep.json"];
        if sandbox {
            args.push("--sandbox");
        }
        let o = delulu(&dir, &args);
        assert_eq!(o.status.code(), Some(0), "sandbox={sandbox}:\n{}", out(&o));
        assert!(String::from_utf8_lossy(&o.stdout).contains("not delivered"), "sandbox={sandbox}:\n{}", out(&o));
        assert!(String::from_utf8_lossy(&o.stderr).contains("[egress: header]"), "sandbox={sandbox}:\n{}", out(&o));
        let v = report(&dir);
        let rec = &v["egress"]["records"][0];
        assert_eq!(rec["reason"], "header", "sandbox={sandbox}: {v}");
        assert_eq!(rec["hops"], serde_json::json!([]), "sandbox={sandbox}: no hop was made: {v}");
        if sandbox {
            let denied = v["sandbox"]["denied"].as_array().expect("denied[]");
            assert!(denied.iter().any(|d| d.as_str().is_some_and(|s| s.contains("header"))), "sandbox={sandbox}: {v}");
        }
        assert!(seen.join().unwrap().is_none(), "sandbox={sandbox}: nothing connected for a refused header");
    }
}

/// The bytes the secret witnesses grant — a marker no report, trace or message may ever hold.
const SECRET_BYTES: &str = "s3cr3t-wq7z-never-printed";
/// The part of [`SECRET_BYTES`] searched for. Not hex: a sandboxed run's report carries 64-hex-digit digests, and a
/// hex canary can be spelled by one by chance (2026-10-17: `8f2c` was, in an audit chain — broker_cli.rs).
const SECRET_CANARY: &str = "wq7z";

/// A program that sends the secret `API_TOKEN` as the `Authorization` header to `url` (P9-04 (b)/(c), D-V2-131),
/// through `Secret.map` — `"Bearer " + s` — so the derived secret's audience is the one under test.
fn secret_sender(dir: &Path, url: &str) {
    std::fs::write(
        dir.join("send.delulu"),
        format!(
            "module fetch\n\nfn main(root: Root) ! {{Net, Write}} {{\n    \
             let out = root.console()\n    \
             let h = root.http([\"127.0.0.1\"])\n    \
             let tok = root.secret(\"API_TOKEN\")\n    \
             let sent = tok.map(fn(s: Str) -> Str {{ \"Bearer \" + s }})\n    \
             let hs = Map()\n    \
             hs.insert(\"Accept\", \"text/plain\")\n    \
             match h.get_with_secret(\"{url}\", hs, \"Authorization\", sent) {{\n        \
             Ok(body) => out.println(\"delivered\")\n        \
             Err(e) => out.println(\"not delivered\")\n    \
             }}\n}}\n"
        ),
    )
    .unwrap();
}

/// P9-04 (b)/(c): a secret leaves in a header — as TLS — to the host and port its grant's audience names, and the
/// report says which secret each hop carried, never its bytes; the same request with the audience naming another
/// PORT, or with no audience at all, connects nowhere and says why to the operator. `Secret.map`'s result carries
/// the audience (a mapped secret granted with an audience is sent); a guest holds no secret, so nothing is sent from
/// one. No byte of the secret reaches the report, the trace, stdout or stderr on any path.
#[test]
fn a_secret_is_sent_to_its_audience_and_nowhere_else() {
    let run = |dir: &Path, audience: &str, sandbox: bool| -> Output {
        let grant = format!("secret:API_TOKEN{audience}={SECRET_BYTES}");
        let mut args = vec!["run", "send.delulu", "--grant", "console", "--grant", "net.special=127.0.0.1", "--grant", &grant, "--report-out", "rep.json"];
        // A sandboxed run refuses the trace flags (it does not apply them yet) — the trace is read at L0.
        args.extend(if sandbox { ["--sandbox"].as_slice() } else { ["--trace-effects", "--trace-out", "trace.txt"].as_slice() });
        delulu(dir, &args)
    };
    let leaked = |dir: &Path, o: &Output| {
        let mut all = out(o);
        for f in ["rep.json", "trace.txt"] {
            all.push_str(&std::fs::read_to_string(dir.join(f)).unwrap_or_default());
        }
        all.contains(SECRET_CANARY)
    };
    // Inside the audience: sent.
    let dir = tmp("secret-in");
    let (port, seen) = listen(Duration::from_secs(20));
    secret_sender(&dir, &format!("https://127.0.0.1:{port}/probe"));
    let o = run(&dir, &format!("@127.0.0.1:{port}"), false);
    let first = seen.join().unwrap().unwrap_or_else(|| panic!("nothing connected:\n{}", out(&o)));
    assert!(first.len() >= 3 && first[0] == 0x16 && first[1] == 0x03, "TLS, got {first:02x?}:\n{}", out(&o));
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let v = report(&dir);
    let hop = &v["egress"]["records"][0]["hops"][0];
    assert_eq!(hop["secret"], "API_TOKEN", "the hop names the secret it carried: {v}");
    assert_eq!(hop["headers"], serde_json::json!(["accept", "authorization"]), "{v}");
    assert!(!leaked(&dir, &o), "the secret's bytes are nowhere:\n{}\n{v}", out(&o));
    // And the trace that was searched is there: the request is in it, its detail redacted.
    let trace = std::fs::read_to_string(dir.join("trace.txt")).unwrap_or_default();
    assert!(trace.contains("\"get_with_secret\"") && trace.contains("«opaque»"), "{trace}");
    let o = delulu(&dir, &["schema", "validate", "run-report", "rep.json", "--json"]);
    let check: serde_json::Value = serde_json::from_slice(&o.stdout).expect("JSON");
    assert_eq!(check["validate"]["valid"], true, "a hop's `secret` is a field the schema holds: {}", check["validate"]["errors"]);
    // Outside it — the audience names another port of the same host — and with none: nothing connects.
    for (label, audience_of) in [("another port", Some(1u16)), ("no audience", None)] {
        let dir = tmp("secret-out");
        let (port, seen) = listen(Duration::from_secs(3));
        secret_sender(&dir, &format!("https://127.0.0.1:{port}/probe"));
        let audience = audience_of.map(|d| format!("@127.0.0.1:{}", port.wrapping_add(d).max(1))).unwrap_or_default();
        let o = run(&dir, &audience, false);
        assert_eq!(o.status.code(), Some(0), "{label}:\n{}", out(&o));
        assert!(String::from_utf8_lossy(&o.stdout).contains("not delivered"), "{label}:\n{}", out(&o));
        assert!(String::from_utf8_lossy(&o.stderr).contains("[egress: audience]"), "{label}:\n{}", out(&o));
        let v = report(&dir);
        assert_eq!(v["egress"]["records"][0]["reason"], "audience", "{label}: {v}");
        assert_eq!(v["egress"]["records"][0]["hops"], serde_json::json!([]), "{label}: no hop was made: {v}");
        assert!(!leaked(&dir, &o), "{label}: the secret's bytes are nowhere:\n{}", out(&o));
        assert!(seen.join().unwrap().is_none(), "{label}: nothing connected");
    }
    // A guest holds no secret (the channel carries none), so a sandboxed run sends nothing — refused in words.
    let dir = tmp("secret-guest");
    let (port, seen) = listen(Duration::from_secs(3));
    secret_sender(&dir, &format!("https://127.0.0.1:{port}/probe"));
    let o = run(&dir, &format!("@127.0.0.1:{port}"), true);
    assert_ne!(o.status.code(), Some(0), "{}", out(&o));
    assert!(out(&o).contains("DL1401") && out(&o).contains("secret"), "{}", out(&o));
    assert!(!out(&o).contains(SECRET_CANARY), "{}", out(&o));
    assert!(seen.join().unwrap().is_none(), "a guest sent nothing");
}

/// `--json` keeps the operator's prose off stderr — the same records are in the report.
#[test]
fn under_json_the_reason_is_in_the_report_and_not_in_prose() {
    let dir = tmp("json");
    fetcher(&dir, "example.com", "http://example.com/plain");
    let o = delulu(&dir, &["run", "fetch.delulu", "--grant", "console", "--grant", "net=example.com", "--json", "--report-out", "rep.json"]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    assert!(!String::from_utf8_lossy(&o.stderr).contains("[egress:"), "{}", out(&o));
    assert_eq!(report(&dir)["egress"]["records"][0]["reason"], "scheme");
}

/// A run that makes no request still reports `egress`, so its absence never has to be interpreted.
#[test]
fn a_run_without_requests_reports_zero_rather_than_nothing() {
    let dir = tmp("none");
    std::fs::write(dir.join("hi.delulu"), "module hi\n\nfn main(root: Root) ! {Write} {\n    root.console().println(\"hi\")\n}\n").unwrap();
    let o = delulu(&dir, &["run", "hi.delulu", "--grant", "console", "--report-out", "rep.json"]);
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let v = report(&dir);
    assert_eq!(v["egress"]["requests"], 0, "{v}");
    assert_eq!(v["egress"]["records"], serde_json::json!([]), "{v}");
}
