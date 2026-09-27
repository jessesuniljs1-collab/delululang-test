//! A sandboxed guest under the Guard, end to end through the real binary (REMAINING_WORK 4.20's open
//! half, and campaign finding GUARD-SCOPE-1).
//!
//! Until 2026-09-27 a sandboxed run REFUSED `--lease` and `--broker`: the guest's effects were
//! performed by the host in embedded custody, so a program run in the sandbox could not be run under
//! a delegated node, and nothing the Guard said applied to it. The owner asked for exactly that —
//! authority configured with `delulu authority`, validated by the Guard, the program confined — so the
//! host now authorizes every operation a guest asks for through the broker before performing it,
//! with the interpreter's own op mapping. This test is the story, told once for the guest and checked
//! against the ordinary lease run at each step that matters:
//!
//! 1. `delulu authority --grants` names what the program needs; the principal delegates exactly that
//!    as a lease, and SEALS one directory inside the read scope and GUARDS the write scope.
//! 2. The guest's write is refused DL1410 with the exact `guard request` command; nothing is written.
//! 3. The request (with `--why`) makes a retry DL1411; a denial makes it DL1412; a permit for ONE use
//!    lets exactly one write through and the next is DL1410 again; a standing approval then writes.
//! 4. A read inside the sealed directory is DL1413 — for the guest AND for the ordinary run (before
//!    GUARD-SCOPE-1 both READ it: a rule on a directory sealed only the directory entry). A file beside
//!    it is still readable.
//!    The same sealed file reached through a link inside the grant, another case, or a Windows 8.3
//!    name is DL1413 too (GUARD-ALIAS-1: the Guard decides on the path the use resolves to).
//! 5. A local `--grant` beside the lease is refused; revoking the node stops the guest.
//! 6. The report names the node the guest ran under; `audit verify` is green over all of it.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn delulu_in(cwd: &Path, state: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(cwd)
        .env("DELULU_STATE_DIR", state)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .args(args)
        .output()
        .expect("failed to run delulu")
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

/// Stops the daemon on drop, from a stable cwd, never panicking.
struct DaemonGuard {
    state: PathBuf,
}
impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(std::env::temp_dir())
            .env("DELULU_STATE_DIR", &self.state)
            .args(["broker", "stop"])
            .output();
    }
}

fn owner_code(t: &str) -> String {
    let i = t.find("gow1_").expect("the owner code is printed once at start");
    let tail = &t[i..];
    let end = tail.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(tail.len());
    tail[..end].to_string()
}

/// A directory link any user can make: a junction on Windows (a symlink there needs privilege), a
/// symlink elsewhere. `false` when this host will not make one.
fn make_dir_link(target: &Path, link: &Path) -> bool {
    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link).is_ok()
    }
}

/// The Windows 8.3 short name of `dir/name`, when the volume generates them (many do not).
fn short_name(dir: &Path, name: &str) -> Option<String> {
    if !cfg!(windows) {
        return None;
    }
    let o = Command::new("cmd").args(["/C", "dir", "/x", "/ad"]).current_dir(dir).output().ok()?;
    String::from_utf8_lossy(&o.stdout).lines().find_map(|l| {
        let f: Vec<&str> = l.split_whitespace().collect();
        (f.len() >= 5 && f[f.len() - 1] == name && f[f.len() - 2].contains('~')).then(|| f[f.len() - 2].to_string())
    })
}

/// The path as a program and a rule spell it: absolute, forward slashes.
fn fwd(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

#[test]
fn a_sandboxed_guest_runs_under_a_lease_and_the_guard_decides_its_every_use() {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let base = std::env::temp_dir().join(format!("delulu-sbx-guard-{}-{t}", std::process::id()));
    let cwd = base.join("work");
    let state = base.join("state");
    let data = cwd.join("data");
    let out = cwd.join("out");
    std::fs::create_dir_all(data.join("secret")).unwrap();
    std::fs::create_dir_all(&out).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    std::fs::write(data.join("public.txt"), "public words").unwrap();
    std::fs::write(data.join("secret").join("key.txt"), "THE-SEALED-KEY").unwrap();
    let (d, o) = (fwd(&data), fwd(&out));

    // The two programs an untrusted agent might hand over.
    let writer = format!(
        "module writer\n\nfn main(root: Root) ! {{Read, Write}} {{\n    let fr = root.fs_read(\"{d}\")\n    \
         let fw = root.fs_write(\"{o}\")\n    let out = root.console()\n    \
         match fr.read_text(\"public.txt\") {{\n        Ok(v) => {{\n            match fw.write_text(\"result.txt\", v) {{\n                \
         Ok(_) => out.println(\"wrote\"),\n                Err(_) => out.println(\"write refused\")\n            }}\n        }}\n        \
         Err(_) => out.println(\"read refused\")\n    }}\n}}\n"
    );
    let reader = format!(
        "module reader\n\nfn main(root: Root) ! {{Read, Write}} {{\n    let out = root.console()\n    \
         let fr = root.fs_read(\"{d}\")\n    match fr.read_text(\"public.txt\") {{\n        Ok(v) => out.println(\"public: \" + v),\n        \
         Err(_) => out.println(\"public refused\")\n    }}\n    match fr.read_text(\"secret/key.txt\") {{\n        \
         Ok(v) => out.println(\"secret: \" + v),\n        Err(_) => out.println(\"secret refused\")\n    }}\n}}\n"
    );
    std::fs::write(cwd.join("writer.delulu"), &writer).unwrap();
    std::fs::write(cwd.join("reader.delulu"), &reader).unwrap();
    // Both programs must check first, so a test-program mistake reads as that and not as a Guard
    // verdict.
    for p in ["writer.delulu", "reader.delulu"] {
        let c = delulu_in(&cwd, &state, &["check", p]);
        assert!(c.status.success(), "{p} must check: {}", text(&c));
    }

    // 1. What the program needs, from the toolchain; the principal delegates exactly that.
    let a = delulu_in(&cwd, &state, &["authority", "writer.delulu", "--grants"]);
    let need = text(&a);
    assert!(need.contains(&format!("--grant fs.read={d}")) && need.contains(&format!("--grant fs.write={o}")), "{need}");
    let s = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(s.status.success(), "broker start: {}", text(&s));
    let _daemon = DaemonGuard { state: state.clone() };
    let owner = owner_code(&text(&s));
    for (rule, tier) in [(format!("fs_read:{d}/secret"), "sealed"), (format!("fs_write:{o}"), "guarded")] {
        let p = delulu_in(&cwd, &state, &["guard", "policy", "set", &rule, tier, "--owner", &owner]);
        assert!(p.status.success(), "{rule}: {}", text(&p));
    }
    let del = delulu_in(
        &cwd,
        &state,
        &["grants", "delegate", "--effects", "Read,Write", "--fs-read", &d, "--fs-write", &o, "--multi", "--owner", &owner, "--json"],
    );
    assert!(del.status.success(), "delegate: {}", text(&del));
    let v: serde_json::Value = serde_json::from_slice(&del.stdout).expect("delegate --json");
    let (node, token) = (v["node"].as_str().unwrap().to_string(), v["token"].as_str().unwrap().to_string());

    // 2. The guest's write is guarded: DL1410 with the exact request command, and nothing written.
    let run = |prog: &str, sandbox: bool, extra: &[&str]| {
        let mut args = vec!["run", prog, "--lease", token.as_str(), "--no-prompt"];
        if sandbox {
            args.push("--sandbox");
        }
        args.extend_from_slice(extra);
        delulu_in(&cwd, &state, &args)
    };
    let r = run("writer.delulu", true, &[]);
    assert_eq!(r.status.code(), Some(1), "{}", text(&r));
    assert!(text(&r).contains("DL1410"), "a guarded write, refused for the guest: {}", text(&r));
    assert!(text(&r).contains(&format!("delulu guard request {node} --use fs_write:")), "{}", text(&r));
    assert!(text(&r).contains("custody: daemon"), "{}", text(&r));
    assert!(!out.join("result.txt").exists(), "nothing written under a guard block");

    // 3. Request → DL1411 → DENY → DL1412; a new request → approve ONE use → exactly one write, and the
    //    next is refused again; the report names the node.
    let request = || {
        let q = delulu_in(&cwd, &state, &["guard", "request", &node, "--use", &format!("fs_write:{o}"), "--why", "write the result"]);
        assert!(q.status.success(), "request: {}", text(&q));
        String::from_utf8_lossy(&q.stdout).trim().to_string()
    };
    let req = request();
    let r = run("writer.delulu", true, &[]);
    assert!(text(&r).contains("DL1411") && text(&r).contains(&req), "pending: {}", text(&r));
    let dn = delulu_in(&cwd, &state, &["guard", "deny", &req, "--owner", &owner, "--comment", "not this one"]);
    assert!(dn.status.success(), "deny: {}", text(&dn));
    let r = run("writer.delulu", true, &[]);
    assert!(text(&r).contains("DL1412"), "a denied request refuses the guest: {}", text(&r));
    assert!(!out.join("result.txt").exists(), "nothing written after a denial");
    let req = request();
    let ap = delulu_in(&cwd, &state, &["guard", "approve", &req, "--owner", &owner, "--uses", "1", "--comment", "once"]);
    assert!(ap.status.success(), "approve: {}", text(&ap));
    let r = run("writer.delulu", true, &[]);
    assert_eq!(r.status.code(), Some(0), "the single-use permit lets the guest write once: {}", text(&r));
    std::fs::remove_file(out.join("result.txt")).unwrap();
    let r = run("writer.delulu", true, &[]);
    assert!(text(&r).contains("DL1410"), "the permit was used up: the next write is guarded again: {}", text(&r));
    assert!(!out.join("result.txt").exists(), "no second write on a spent permit");
    let req = request();
    let ap = delulu_in(&cwd, &state, &["guard", "approve", &req, "--owner", &owner, "--comment", "fine"]);
    assert!(ap.status.success(), "approve: {}", text(&ap));
    let report = base.join("report.json");
    let r = run("writer.delulu", true, &["--report-out", report.to_str().unwrap()]);
    assert_eq!(r.status.code(), Some(0), "the permit lets the guest write: {}", text(&r));
    assert_eq!(std::fs::read_to_string(out.join("result.txt")).unwrap(), "public words");
    let rep: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&report).unwrap()).unwrap();
    assert_eq!(rep["custody"]["mode"], "daemon", "{rep}");
    assert_eq!(rep["custody"]["node"], node.as_str(), "{rep}");

    // 4. The sealed directory seals what is IN it — for the guest and for the ordinary run alike.
    for sandbox in [true, false] {
        let r = run("reader.delulu", sandbox, &[]);
        let all = text(&r);
        assert!(all.contains("public: public words"), "sandbox={sandbox}: the file beside the seal is readable: {all}");
        assert!(all.contains("DL1413"), "sandbox={sandbox}: a read inside a sealed directory is DL1413: {all}");
        assert!(!all.contains("THE-SEALED-KEY"), "sandbox={sandbox}: the sealed bytes never reach the program: {all}");
    }

    // 4b. GUARD-ALIAS-1: the seal holds whatever the path is CALLED. A link inside the grant aimed at
    //     the sealed directory (a junction on Windows, which any user can make; a symlink elsewhere),
    //     another case where the filesystem folds case, and the Windows 8.3 short name when the volume
    //     makes one. Before the fix the Guard decided on the path as spelled, and the alias read the key.
    let mut spellings = vec!["secret/key.txt".to_string()];
    if make_dir_link(&data.join("secret"), &data.join("alias")) {
        spellings.push("alias/key.txt".to_string());
    }
    if cfg!(any(windows, target_os = "macos")) {
        spellings.push("SECRET/key.txt".to_string());
    }
    if let Some(short) = short_name(&data, "secret") {
        spellings.push(format!("{short}/key.txt"));
    }
    for rel in &spellings {
        let prog = format!(
            "module spell

fn main(root: Root) ! {{Read, Write}} {{
    let out = root.console()
                 let fr = root.fs_read(\"{d}\")
    match fr.read_text(\"{rel}\") {{
                     Ok(v) => out.println(\"read: \" + v),
        Err(_) => out.println(\"refused\")
    }}
}}
"
        );
        std::fs::write(cwd.join("spell.delulu"), &prog).unwrap();
        for sandbox in [true, false] {
            let r = run("spell.delulu", sandbox, &[]);
            let all = text(&r);
            assert!(all.contains("DL1413"), "`{rel}` (sandbox={sandbox}) must meet the seal: {all}");
            assert!(!all.contains("THE-SEALED-KEY"), "`{rel}` (sandbox={sandbox}) read the sealed key: {all}");
        }
    }

    // 5. A local grant beside a lease is refused, for the guest as for the ordinary run; revocation
    //    stops the guest.
    let r = run("reader.delulu", true, &["--grant", "console"]);
    assert_eq!(r.status.code(), Some(2), "{}", text(&r));
    assert!(text(&r).contains("derives its authority from the delegated node"), "{}", text(&r));
    let rv = delulu_in(&cwd, &state, &["grants", "revoke", &node]);
    assert!(rv.status.success(), "revoke: {}", text(&rv));
    let r = run("reader.delulu", true, &[]);
    assert_ne!(r.status.code(), Some(0), "a revoked node runs nothing: {}", text(&r));
    assert!(!text(&r).contains("public: "), "no effect after revocation: {}", text(&r));

    // 6. The chain holds over the whole story.
    let a = delulu_in(&cwd, &state, &["audit", "verify"]);
    assert!(a.status.success(), "audit verify: {}", text(&a));
    let tail = text(&delulu_in(&cwd, &state, &["audit", "tail", "200"]));
    for event in ["guard_block", "guard_request", "guard_deny", "guard_approve", "guard_permit_use", "sandbox-launch"] {
        assert!(tail.contains(event), "the chain records `{event}`: {tail}");
    }
    drop(_daemon);
    let _ = std::fs::remove_dir_all(&base);
}
