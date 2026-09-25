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
