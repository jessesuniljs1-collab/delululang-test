//! RW 4.36 (the red-team pass on FRAME-DRIP-1, F3): `delulu-registry serve` is reachable by anyone who can
//! reach its port, with no token needed to be read, and it gave every connection all the time and memory it
//! asked for. One connection at a time, a request line and headers read with no deadline and no cap, and a
//! body buffer allocated at whatever `Content-Length` said before a byte of it was read. So one idle
//! connection held every other client, and one number took the server down.
//!
//! What must hold: a connection that says nothing, or says it slowly, delays no other client; a
//! `Content-Length` past the bound is refused before anything is allocated, and the server goes on
//! serving; a line with no end is cut at its bound; a request that does not arrive whole within its bound
//! is dropped.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::Value;

use delulu_registry::Registry;

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-rb-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn recompute(_: &[u8]) -> Option<Value> {
    None
}

fn start(tag: &str) -> (SocketAddr, PathBuf) {
    let root = scratch(tag);
    let reg = Arc::new(Registry::open(&root).unwrap());
    let (addr, _h) = delulu_registry::serve(reg, "127.0.0.1:0", Arc::new(recompute)).unwrap();
    (addr, root)
}

/// One GET, with a deadline of our own: the status line, or what went wrong and when.
fn get(addr: SocketAddr, within: Duration) -> Result<String, String> {
    let t = Instant::now();
    let mut s = TcpStream::connect_timeout(&addr, within).map_err(|e| format!("connect: {e}"))?;
    s.set_read_timeout(Some(within)).unwrap();
    s.write_all(b"GET /index/nothing-here HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n").map_err(|e| format!("write: {e}"))?;
    let mut out = String::new();
    match s.read_to_string(&mut out) {
        Ok(_) => Ok(out.lines().next().unwrap_or("").to_string()),
        Err(e) => Err(format!("no answer after {:?}: {e}", t.elapsed())),
    }
}

/// A connection that sends nothing held every other client: the server read it first, and for ever.
#[test]
fn an_idle_connection_delays_no_other_client() {
    let (addr, root) = start("idle");
    let _idle = TcpStream::connect(addr).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let t = Instant::now();
    let r = get(addr, Duration::from_secs(3));
    assert_eq!(r.as_deref(), Ok("HTTP/1.1 404 Not Found"), "a client behind an idle connection ({:?})", t.elapsed());
    let _ = std::fs::remove_dir_all(&root);
}

/// `Content-Length: 18446744073709551615` was a `vec!` of that many bytes: the server panicked and every
/// later client was refused. It is answered 413 before anything is allocated, and the server goes on.
#[test]
fn a_content_length_past_the_bound_is_refused_and_the_server_goes_on() {
    let (addr, root) = start("huge");
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    s.write_all(b"POST /publish HTTP/1.1\r\nHost: x\r\nContent-Length: 18446744073709551615\r\n\r\n").unwrap();
    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    assert!(out.starts_with("HTTP/1.1 413"), "refused, in the protocol's own words: {out:?}");
    drop(s);
    assert_eq!(get(addr, Duration::from_secs(3)).as_deref(), Ok("HTTP/1.1 404 Not Found"), "and it still serves");
    let _ = std::fs::remove_dir_all(&root);
}

/// A request line with no end was read for as long as it came, into memory: it is cut at its bound, and the
/// connection answered and closed, not read on.
#[test]
fn a_line_with_no_end_is_cut_at_its_bound() {
    let (addr, root) = start("line");
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    let t = Instant::now();
    // 1 MiB of a request line, and no line break; a server that keeps reading takes all of it. One that cuts
    // it answers 431 and closes — and a close with our bytes still unread reaches us as a reset, which can
    // cut the answer short. Either is the line refused; reading on and saying nothing is not.
    let sent = s.write_all(&vec![b'A'; 1 << 20]);
    let mut out = String::new();
    let read = s.read_to_string(&mut out).map(|_| ());
    let reset = |r: &std::io::Result<()>| {
        matches!(r, Err(e) if matches!(e.kind(), std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe))
    };
    assert!(out.starts_with("HTTP/1.1 431") || reset(&sent) || reset(&read), "cut, not read on: {out:?} {sent:?} {read:?}");
    assert!(t.elapsed() < Duration::from_secs(5), "{:?}", t.elapsed());
    assert_eq!(get(addr, Duration::from_secs(3)).as_deref(), Ok("HTTP/1.1 404 Not Found"), "and it still serves");
    let _ = std::fs::remove_dir_all(&root);
}

/// A request dribbled a byte at a time, each inside the read bound, is dropped once the whole-request bound
/// has passed — FRAME-DRIP-1's shape, on the registry's port (with `Limits` a test can wait for).
#[test]
fn a_request_dribbled_past_its_bound_is_dropped() {
    let root = scratch("drip");
    let reg = Arc::new(Registry::open(&root).unwrap());
    let limits = delulu_registry::Limits {
        read: Duration::from_secs(2),
        request: Duration::from_secs(1),
        ..delulu_registry::Limits::default()
    };
    let (addr, _h) = delulu_registry::serve_with(reg, "127.0.0.1:0", Arc::new(recompute), limits).unwrap();
    let mut s = TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_millis(50))).unwrap();
    let t = Instant::now();
    let mut dropped = None;
    'drip: for b in b"GET /index/nothing-here HTTP/1.1\r\nHost: x\r\nX-Padding: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\r\n\r\n" {
        if s.write_all(&[*b]).is_err() {
            dropped = Some(t.elapsed());
            break;
        }
        // Between bytes, see whether the server has closed on us.
        let until = Instant::now() + Duration::from_millis(200);
        while Instant::now() < until {
            let mut buf = [0u8; 64];
            match s.read(&mut buf) {
                Ok(_) => {
                    dropped = Some(t.elapsed());
                    break 'drip;
                }
                Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
                Err(_) => {
                    dropped = Some(t.elapsed());
                    break 'drip;
                }
            }
        }
    }
    let dropped = dropped.unwrap_or_else(|| panic!("the whole dribbled request was read, {:?} after its bound began", t.elapsed()));
    assert!(dropped < Duration::from_secs(4), "dropped at {dropped:?}, against a 1 s bound and 2 s reads");
    let _ = std::fs::remove_dir_all(&root);
}
