//! Stage 5 acceptance criterion 8 (spec §9), RESTATED for the no-NIC guest (T9; PS-C-04).
//!
//! The original contract imagined a guest WITH a network device behind a default-deny egress proxy, and
//! a guest filesystem assembled from virtio-fs mounts matching the granted scopes. PS-C built the guest
//! the V2 security model rules for instead (D-NE-23, `V2_SECURITY_MODEL.md` §7): no network device and
//! no filesystem device at all, and every effect a channel request that the host authorizes and
//! performs. The criterion's claims, restated for that guest:
//!
//! 1. **egress-deny** — a program in the guest that asks for a host the grant does not name gets NO
//!    bytes out, although the host machine can reach that address (a control connects to it after the
//!    run); a request to the granted host does leave — from the HOST, as TLS;
//! 2. **the granted path is readable**;
//! 3. **a sibling path is not** — refused by the host's containment, and its contents never reach the
//!    guest;
//! 4. **T9 itself** — the guest's own kernel refuses to create an IPv4 or an IPv6 socket at all ("no
//!    network stack in its kernel", which the guest reports only when BOTH attempts are refused as an
//!    unsupported family), and the VM was configured with no network device and no filesystem device.
//!
//! Gated on `cfg(delulu_kvm)`: the KVM CI job compiles with it and provides the image and the VMM
//! (`tests/microvm_cli.rs` says how). A host that enables the gate without them fails the test.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

fn isolated_state() -> PathBuf {
    static DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    DIR.get_or_init(|| {
        let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let d = std::env::temp_dir().join(format!("delulu-teststate-c8-{}-{t}", std::process::id()));
        let _ = std::fs::create_dir_all(d.join("audit"));
        d
    })
    .clone()
}

fn delulu(dir: &Path, args: &[&str]) -> Output {
    let provisioned = std::env::var_os("DELULU_MICROVM_IMAGE").is_some() && std::env::var_os("DELULU_FIRECRACKER").is_some();
    assert!(provisioned, "criterion 8 is enabled (cfg delulu_kvm) but DELULU_MICROVM_IMAGE and DELULU_FIRECRACKER are not set");
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

/// A loopback listener that records the first bytes of every connection until `stop` is dropped.
fn listen(addr: &str) -> (u16, std::sync::mpsc::Sender<()>, std::thread::JoinHandle<Vec<Vec<u8>>>) {
    let l = TcpListener::bind(format!("{addr}:0")).unwrap();
    l.set_nonblocking(true).unwrap();
    let port = l.local_addr().unwrap().port();
    let (stop, stopped) = std::sync::mpsc::channel::<()>();
    let h = std::thread::spawn(move || {
        let mut seen = Vec::new();
        loop {
            match l.accept() {
                Ok((mut s, _)) => {
                    s.set_nonblocking(false).unwrap();
                    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                    let mut buf = vec![0u8; 5];
                    let mut got = 0;
                    while got < buf.len() {
                        match s.read(&mut buf[got..]) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => got += n,
                        }
                    }
                    buf.truncate(got);
                    seen.push(buf);
                }
                Err(_) => {
                    if let Err(std::sync::mpsc::TryRecvError::Disconnected) = stopped.try_recv() {
                        return seen;
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }
    });
    (port, stop, h)
}

fn report(dir: &Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(dir.join("rep.json")).expect("the run report")).unwrap()
}

#[test]
#[cfg_attr(
    not(delulu_kvm),
    ignore = "criterion 8 boots a microVM: needs Linux, /dev/kvm, Firecracker and a guest image — run on a KVM \
              host with RUSTFLAGS=\"--cfg delulu_kvm\", DELULU_MICROVM_IMAGE and DELULU_FIRECRACKER"
)]
fn criterion_8_restated_for_the_no_nic_guest() {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let base = std::env::temp_dir().join(format!("delulu_microvm_c8_{}_{t}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("granted")).unwrap();
    std::fs::create_dir_all(base.join("sibling")).unwrap();
    std::fs::write(base.join("granted/ok.txt"), "inside").unwrap();
    std::fs::write(base.join("sibling/secret.txt"), "outside-secret").unwrap();
    let grants = ["--grant", "console", "--grant", "fs.read=./granted", "--grant", "net.special=127.0.0.1"];

    // ---- 1 + 2: the granted path, and a request to the granted host, which leaves from the HOST --------
    let (granted_port, stop_a, seen_a) = listen("127.0.0.1");
    std::fs::write(
        base.join("allowed.delulu"),
        format!(
            "module c8\n\nfn main(root: Root) ! {{Read, Net, Write}} {{\n  let out = root.console()\n  \
             let fr = root.fs_read(\"./granted\")\n  \
             match fr.read_text(\"ok.txt\") {{\n    Ok(t) => out.println(t)\n    Err(e) => out.println(\"granted read refused\")\n  }}\n  \
             let h = root.http([\"127.0.0.1\"])\n  \
             match h.get(\"https://127.0.0.1:{granted_port}/c8\") {{\n    Ok(b) => out.println(\"delivered\")\n    Err(e) => out.println(\"not delivered\")\n  }}\n}}\n"
        ),
    )
    .unwrap();
    let mut args = vec!["run", "allowed.delulu", "--isolation", "microvm", "--report-out", "rep.json"];
    args.extend(grants);
    let o = delulu(&base, &args);
    drop(stop_a);
    let seen_a = seen_a.join().unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", out(&o));
    let stdout = String::from_utf8_lossy(&o.stdout);
    assert!(stdout.contains("inside"), "the granted path must be readable: {}", out(&o));
    // The listener hung up without a certificate, so nothing came back — but the bytes LEFT, as TLS.
    assert!(stdout.contains("not delivered"), "{}", out(&o));
    assert_eq!(seen_a.len(), 1, "one connection to the granted host: {seen_a:?}");
    assert!(seen_a[0].len() >= 3 && seen_a[0][0] == 0x16 && seen_a[0][1] == 0x03, "a TLS handshake: {:02x?}", seen_a[0]);
    // T9, from inside: the guest's kernel has no IP stack, and the VM had no network or filesystem device.
    let v = report(&base);
    let g: Vec<&str> = v["sandbox"]["host_guarantees"].as_array().unwrap().iter().filter_map(|x| x.as_str()).collect();
    for word in ["no network stack in its kernel", "no network device", "no filesystem device"] {
        assert!(g.contains(&word), "`{word}` missing: {g:?}");
    }
    assert_eq!(v["sandbox"]["level"], 2, "{v}");
    // The request was made by the host, and recorded there, pinned to the granted address.
    assert_eq!(v["egress"]["requests"], 1, "{v}");
    assert_eq!(v["egress"]["records"][0]["hops"][0]["addrs"], serde_json::json!(["127.0.0.1"]), "{v}");

    // ---- 1: egress-deny — a host the grant does not name gets nothing, though the host can reach it --
    let (other_port, stop_b, seen_b) = listen("127.0.0.2");
    std::fs::write(
        base.join("denied.delulu"),
        format!(
            "module c8\n\nfn main(root: Root) ! {{Net, Write}} {{\n  let out = root.console()\n  \
             let h = root.http([\"127.0.0.2\"])\n  \
             match h.get(\"https://127.0.0.2:{other_port}/c8\") {{\n    Ok(b) => out.println(\"delivered\")\n    Err(e) => out.println(\"not delivered\")\n  }}\n}}\n"
        ),
    )
    .unwrap();
    let mut args = vec!["run", "denied.delulu", "--isolation", "microvm", "--report-out", "rep.json"];
    args.extend(grants);
    let o = delulu(&base, &args);
    assert_ne!(o.status.code(), Some(0), "an ungranted host must be refused: {}", out(&o));
    let v = report(&base);
    assert!(v["sandbox"]["denied_total"].as_u64().unwrap_or(0) >= 1, "the refusal is recorded: {v}");
    // The control: the address IS reachable from this machine, so the silence above was the refusal.
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut control = loop {
        match TcpStream::connect(format!("127.0.0.2:{other_port}")) {
            Ok(s) => break s,
            Err(e) if Instant::now() >= deadline => panic!("the control could not reach 127.0.0.2:{other_port}: {e}"),
            Err(_) => std::thread::sleep(Duration::from_millis(20)),
        }
    };
    control.write_all(b"CTRL!").unwrap();
    drop(control);
    std::thread::sleep(Duration::from_millis(200));
    drop(stop_b);
    let seen_b = seen_b.join().unwrap();
    assert_eq!(seen_b, vec![b"CTRL!".to_vec()], "only the control may have connected: {seen_b:?}\n{}", out(&o));

    // ---- 3: a sibling path is refused, and its contents never reach the guest ---------------------
    std::fs::write(
        base.join("sibling.delulu"),
        "module c8\n\nfn main(root: Root) ! {Read, Write} {\n  let out = root.console()\n  \
         let fr = root.fs_read(\"./granted\")\n  \
         match fr.read_text(\"../sibling/secret.txt\") {\n    Ok(t) => out.println(t)\n    Err(e) => out.println(\"sibling refused\")\n  }\n}\n",
    )
    .unwrap();
    let mut args = vec!["run", "sibling.delulu", "--isolation", "microvm", "--report-out", "rep.json"];
    args.extend(grants);
    let o = delulu(&base, &args);
    assert_ne!(o.status.code(), Some(0), "an escaping path must be refused: {}", out(&o));
    assert!(!out(&o).contains("outside-secret"), "the sibling's contents reached the guest: {}", out(&o));
    assert!(report(&base)["sandbox"]["denied_total"].as_u64().unwrap_or(0) >= 1);

    let _ = std::fs::remove_dir_all(&base);
}
