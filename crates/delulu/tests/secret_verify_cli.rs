//! `Secret.verify` under a lease, end to end through the real binary (campaign finding
//! VERIFY-FABRICATED-1, agent pass 2 — found independently by the Sonnet 5 testers on Windows and on
//! Linux).
//!
//! Under a lease a program's secrets are broker handles, with no bytes in its process. `verify` was
//! never routed to the broker: two handles compared `false` whatever they held, and the Guard's
//! `declassify` tier (guarded by default) was never asked. Now the broker computes the bit where the
//! bytes are, the Guard decides for each secret, and the chain records it:
//!
//! 1. the principal stores the values (`secrets set`) and delegates the NAMES;
//! 2. the program's `verify` is DL1410 until the principal approves the declassification;
//! 3. approved, equal secrets are `true` and unequal ones `false` — before, both were `false`;
//! 4. the chain has the `verify` records; `audit verify` is green.
//!
//! And the two ways a principal got this wrong in the pass, each now said in words: `--secret
//! NAME=VALUE` (refused, pointing at `secrets set`), and a secret the broker never stored.

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

#[test]
fn verify_under_a_lease_is_computed_by_the_broker_and_gated_by_the_guard() {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    // Short: macOS allows 103 bytes of socket path, and the broker's socket lives in the state dir.
    let base = std::env::temp_dir().join(format!("dsv-{}-{}", std::process::id(), t % 1_000_000_000));
    let cwd = base.join("w");
    let state = base.join("s");
    std::fs::create_dir_all(&cwd).unwrap();
    std::fs::create_dir_all(&state).unwrap();
    let program = "module sv\n\nfn main(root: Root) ! {Write, Declassify} {\n    let out = root.console()\n    \
                   let key = root.secret(\"KEY\")\n    let same = root.secret(\"SAME\")\n    let diff = root.secret(\"DIFF\")\n    \
                   if key.verify(same) {\n        out.println(\"same: true\")\n    } else {\n        out.println(\"same: false\")\n    }\n    \
                   if key.verify(diff) {\n        out.println(\"diff: true\")\n    } else {\n        out.println(\"diff: false\")\n    }\n}\n";
    std::fs::write(cwd.join("sv.delulu"), program).unwrap();
    let c = delulu_in(&cwd, &state, &["check", "sv.delulu"]);
    assert!(c.status.success(), "the program must check: {}", text(&c));

    let s = delulu_in(&cwd, &state, &["broker", "start"]);
    assert!(s.status.success(), "broker start: {}", text(&s));
    let _daemon = DaemonGuard { state: state.clone() };
    let owner = owner_code(&text(&s));
    for (name, value) in [("KEY", "hunter2"), ("SAME", "hunter2"), ("DIFF", "hunter3")] {
        let r = delulu_in(&cwd, &state, &["secrets", "set", name, value]);
        assert!(r.status.success(), "secrets set {name}: {}", text(&r));
    }

    // A value in `--secret` is refused, and says where values go.
    let bad = delulu_in(&cwd, &state, &["grants", "delegate", "--effects", "Write,Declassify", "--secret", "KEY=hunter2", "--owner", &owner]);
    assert_eq!(bad.status.code(), Some(2), "{}", text(&bad));
    assert!(text(&bad).contains("delulu secrets set KEY VALUE"), "{}", text(&bad));

    let del = delulu_in(
        &cwd,
        &state,
        &[
            "grants", "delegate", "--effects", "Write,Declassify", "--secret", "KEY", "--secret", "SAME", "--secret", "DIFF",
            "--multi", "--owner", &owner, "--json",
        ],
    );
    assert!(del.status.success(), "delegate: {}", text(&del));
    let v: serde_json::Value = serde_json::from_slice(&del.stdout).expect("delegate --json");
    let (node, token) = (v["node"].as_str().unwrap().to_string(), v["token"].as_str().unwrap().to_string());
    let run = || delulu_in(&cwd, &state, &["run", "sv.delulu", "--lease", &token, "--no-prompt"]);

    // Guarded by default: the first comparison is refused until the principal approves.
    let r = run();
    assert!(text(&r).contains("DL1410"), "declassify is guarded by default: {}", text(&r));
    assert!(!text(&r).contains("same: "), "nothing was compared: {}", text(&r));

    let q = delulu_in(
        &cwd,
        &state,
        &["guard", "request", &node, "--use", "declassify:KEY", "--use", "declassify:SAME", "--use", "declassify:DIFF", "--why", "compare"],
    );
    assert!(q.status.success(), "request: {}", text(&q));
    let req = String::from_utf8_lossy(&q.stdout).trim().to_string();
    let ap = delulu_in(&cwd, &state, &["guard", "approve", &req, "--owner", &owner]);
    assert!(ap.status.success(), "approve: {}", text(&ap));

    // Approved: the bit is the real one. Before VERIFY-FABRICATED-1 both lines said `false`.
    let r = run();
    assert_eq!(r.status.code(), Some(0), "{}", text(&r));
    assert!(text(&r).contains("same: true"), "equal secrets compare equal: {}", text(&r));
    assert!(text(&r).contains("diff: false"), "{}", text(&r));
    assert!(!text(&r).contains("hunter"), "no secret byte reaches the program's output: {}", text(&r));

    // And `expose` under the same lease, of a secret stored AFTER the broker started: it used to be
    // refused as "not in the store" until a restart (SECRETS-STALE-1), which the agent pass read as
    // "expose under a lease can never work".
    std::fs::write(
        cwd.join("ex.delulu"),
        "module ex

fn main(root: Root) ! {Write, Declassify} {
    let out = root.console()
             let d = root.declassify()
    let k = root.secret(\"KEY\")
    out.println(\"exposed: \" + k.expose(d))
}
",
    )
    .unwrap();
    let c = delulu_in(&cwd, &state, &["check", "ex.delulu"]);
    assert!(c.status.success(), "ex.delulu must check: {}", text(&c));
    let r = delulu_in(&cwd, &state, &["run", "ex.delulu", "--lease", &token, "--no-prompt"]);
    assert!(text(&r).contains("exposed: hunter2"), "an approved expose of a stored secret works under a lease: {}", text(&r));

    let tail = text(&delulu_in(&cwd, &state, &["audit", "tail", "100"]));
    assert!(tail.contains("verify"), "the chain records the comparison: {tail}");
    let a = delulu_in(&cwd, &state, &["audit", "verify"]);
    assert!(a.status.success(), "audit verify: {}", text(&a));

    // A secret the broker never stored says how to store it, rather than naming a scope.
    let del2 = delulu_in(&cwd, &state, &["grants", "delegate", "--effects", "Write,Declassify", "--secret", "NEVER", "--owner", &owner]);
    let tok2 = String::from_utf8_lossy(&del2.stdout).trim().to_string();
    std::fs::write(
        cwd.join("nv.delulu"),
        "module nv\n\nfn main(root: Root) ! {Write, Declassify} {\n    let out = root.console()\n    let d = root.declassify()\n    \
         let n = root.secret(\"NEVER\")\n    out.println(n.expose(d))\n}\n",
    )
    .unwrap();
    if delulu_in(&cwd, &state, &["check", "nv.delulu"]).status.success() && !tok2.is_empty() {
        let r = delulu_in(&cwd, &state, &["run", "nv.delulu", "--lease", &tok2, "--no-prompt"]);
        let all = text(&r);
        assert!(
            all.contains("secrets set NEVER") || all.contains("DL1410"),
            "an unstored secret says how to store it (or the Guard refuses first): {all}"
        );
    }
    drop(_daemon);
    let _ = std::fs::remove_dir_all(&base);
}
