//! PS-B-06: BREAK-GLASS through the real binary — the owner's principle D-V2-09 as a sequence of
//! transitions (`V2_SECURITY_MODEL.md` §5): strict → required → break-glass → strict → released.
//!
//! Every step runs `delulu` as an operator would, with its own state directory and its own key
//! directory, and every refusal is paired with the control that shows the same command succeeding
//! once the refusal's cause is removed — so no refusal here can pass by the command simply failing.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Host {
    dir: PathBuf,
}

impl Host {
    fn new(tag: &str) -> Host {
        let dir = std::env::temp_dir().join(format!("delulu-bg-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for d in ["state", "home", "work"] {
            std::fs::create_dir_all(dir.join(d)).unwrap();
        }
        let hello = "module m\n\nfn main(root: Root) ! {Write} {\n    let out = root.console()\n    out.println(\"started\")\n}\n";
        std::fs::write(dir.join("work").join("hello.delulu"), hello).unwrap();
        std::fs::write(dir.join("work").join("other.delulu"), hello.replace("started", "other")).unwrap();
        Host { dir }
    }

    fn delulu(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_delulu"))
            .current_dir(self.dir.join("work"))
            .env("DELULU_NO_FIRST_RUN", "1")
            .env("DELULU_NO_COLOR", "1")
            .env("DELULU_STATE_DIR", self.dir.join("state"))
            .env("DELULU_HOME", self.dir.join("home"))
            .args(args)
            .output()
            .expect("the delulu binary runs")
    }

    fn path(&self, rel: &str) -> String {
        self.dir.join(rel).display().to_string()
    }

    /// `delulu keygen` into this host's key directory; returns (private key path, public key hex).
    fn keygen(&self, name: &str) -> (String, String) {
        let o = self.delulu(&["keygen", "--name", name, "--json"]);
        assert!(o.status.success(), "keygen: {}", text(&o));
        let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("keygen --json");
        (v["key"].as_str().unwrap().to_string(), v["public_key"].as_str().unwrap().to_string())
    }

    fn ticket(&self, key: &str, what: &[&str], out: &str) {
        let mut a = vec!["sandbox", "ticket", "--key", key, "--ttl", "15m", "--reason", "the pump controller is down", "--out", out];
        a.extend_from_slice(what);
        let o = self.delulu(&a);
        assert!(o.status.success(), "ticket: {}", text(&o));
    }

    fn audit(&self, decision: &str) -> usize {
        let o = self.delulu(&["audit", "query", "--action", "break-glass", "--json"]);
        let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap_or_default();
        v["records"].as_array().map_or(0, |r| r.iter().filter(|x| x["decision"] == decision).count())
    }
}

fn text(o: &Output) -> String {
    format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr))
}

fn ran(o: &Output) -> bool {
    o.status.success() && String::from_utf8_lossy(&o.stdout).contains("started")
}

#[test]
fn the_glass_is_broken_only_by_a_signed_ticket_once_for_one_program_and_then_the_host_is_strict_again() {
    let h = Host::new("flow");
    let run = ["run", "hello.delulu", "--grant", "console"];

    // Before any policy: an ordinary run runs, and a ticket has nothing to break.
    assert!(ran(&h.delulu(&run)), "the control: no policy, the program runs");
    let (key, public) = h.keygen("bg");

    // The operator requires the sandbox.
    let o = h.delulu(&["sandbox", "require", "--break-glass-key", &public]);
    assert!(o.status.success(), "{}", text(&o));

    // Strict now: nothing runs outside the sandbox — not `run`, not `test`, not `repl`.
    for cmd in [&run[..], &["run", "hello.delulu", "--grant", "console", "--sandbox=off"], &["test", "hello.delulu"], &["repl"]] {
        let o = h.delulu(cmd);
        assert_eq!(o.status.code(), Some(2), "`{cmd:?}` ran outside a required sandbox: {}", text(&o));
        assert!(text(&o).contains("requires the sandbox"), "{}", text(&o));
        assert!(!String::from_utf8_lossy(&o.stdout).contains("started"));
    }
    // …and inside it, everything still works: the policy restricts where, not what.
    let o = h.delulu(&["run", "hello.delulu", "--grant", "console", "--sandbox"]);
    assert!(ran(&o), "a sandboxed run is what the policy asks for: {}", text(&o));

    // A ticket for THIS program opens the glass once: loud, reported, audited.
    let t = h.path("hello.ticket");
    h.ticket(&key, &["--program", &h.path("work/hello.delulu")], &t);
    let rep = h.path("rep.json");
    let o = h.delulu(&["run", "hello.delulu", "--grant", "console", "--break-glass", &t, "--report-out", &rep]);
    assert!(ran(&o), "{}", text(&o));
    let err = String::from_utf8_lossy(&o.stderr).to_string();
    assert!(err.contains("BREAK-GLASS") && err.contains("the pump controller is down"), "the banner: {err}");
    let r: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&rep).unwrap()).unwrap();
    assert_eq!(r["sandbox"]["break_glass"], true, "{r}");
    assert_eq!(r["sandbox"]["break_glass_ticket"]["reason"], "the pump controller is down", "{r}");
    assert_eq!(h.audit("allow"), 1, "the use is in the audit chain");

    // Spent: the same ticket again is refused, and that is recorded too.
    let o = h.delulu(&["run", "hello.delulu", "--grant", "console", "--break-glass", &t]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("already used"), "{}", text(&o));
    assert_eq!(h.audit("deny"), 1, "the refused replay is recorded");

    // break-glass → strict: with the ticket spent, the host is exactly as strict as before.
    assert_eq!(h.delulu(&run).status.code(), Some(2), "after a break-glass run the host is strict again");

    // A ticket names its program: a fresh one for hello.delulu does not open other.delulu.
    let t2 = h.path("hello2.ticket");
    h.ticket(&key, &["--program", &h.path("work/hello.delulu")], &t2);
    let o = h.delulu(&["run", "other.delulu", "--grant", "console", "--break-glass", &t2]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("different program"), "{}", text(&o));

    // A ticket signed by a key the policy does not pin is refused, however valid its signature.
    let (stranger, _) = h.keygen("stranger");
    let t3 = h.path("stranger.ticket");
    h.ticket(&stranger, &["--program", &h.path("work/hello.delulu")], &t3);
    let o = h.delulu(&["run", "hello.delulu", "--grant", "console", "--break-glass", &t3]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("not one this host's policy accepts"), "{}", text(&o));

    // A tampered ticket is refused, and a wrong use never burns a good one: t2 still opens hello.
    let tampered = std::fs::read_to_string(&t2).unwrap().replace("the pump controller is down", "just because");
    let t4 = h.path("tampered.ticket");
    std::fs::write(&t4, tampered).unwrap();
    let o = h.delulu(&["run", "hello.delulu", "--grant", "console", "--break-glass", &t4]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("does not verify"), "{}", text(&o));
    assert!(ran(&h.delulu(&["run", "hello.delulu", "--grant", "console", "--break-glass", &t2])), "t2 was not spent by the wrong uses");

    // A sandbox ticket cannot release the policy, and a ticket with `--sandbox` is a contradiction.
    let t5 = h.path("hello5.ticket");
    h.ticket(&key, &["--program", &h.path("work/hello.delulu")], &t5);
    assert_eq!(h.delulu(&["sandbox", "release", "--break-glass", &t5]).status.code(), Some(2));
    assert_eq!(h.delulu(&["run", "hello.delulu", "--grant", "console", "--sandbox", "--break-glass", &t5]).status.code(), Some(2));

    // doctor reads all of this back, and notices the private key left on the host.
    let o = h.delulu(&["doctor"]);
    let d = text(&o);
    assert!(d.contains("armed — this host requires the sandbox"), "{d}");
    assert!(d.contains("break-glass key custody"), "the private half is on this host, and doctor says so: {d}");

    // Released only by a `policy-off` ticket; then a plain run runs, and a ticket has nothing to break.
    let rel = h.path("release.ticket");
    h.ticket(&key, &["--release"], &rel);
    let o = h.delulu(&["sandbox", "release", "--break-glass", &rel]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(ran(&h.delulu(&run)), "released: the program runs as before the policy");
    let o = h.delulu(&["run", "hello.delulu", "--grant", "console", "--break-glass", &t5]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("nothing for a ticket to open"), "{}", text(&o));

    // Every record kept the chain whole.
    let o = h.delulu(&["audit", "verify"]);
    assert!(o.status.success(), "the audit chain verifies after every break-glass record: {}", text(&o));
    let _ = std::fs::remove_dir_all(&h.dir);
}

/// "Fully audited" is a precondition, not a hope: a use that cannot be recorded does not happen.
#[test]
fn a_break_glass_use_that_cannot_be_recorded_does_not_happen() {
    let h = Host::new("unrecorded");
    let (key, public) = h.keygen("bg");
    assert!(h.delulu(&["sandbox", "require", "--break-glass-key", &public]).status.success());
    let t = h.path("hello.ticket");
    h.ticket(&key, &["--program", &h.path("work/hello.delulu")], &t);
    // The audit chain's directory replaced by a file: nothing can be appended.
    let audit = h.dir.join("state").join("audit");
    let _ = std::fs::remove_dir_all(&audit);
    std::fs::write(&audit, b"not a directory").unwrap();
    let o = h.delulu(&["run", "hello.delulu", "--grant", "console", "--break-glass", &t]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("could not be recorded"), "{}", text(&o));
    assert!(!String::from_utf8_lossy(&o.stdout).contains("started"), "nothing ran");
    let _ = std::fs::remove_dir_all(&h.dir);
}

/// A damaged policy is not a way out: it reads as required with no key.
#[test]
fn a_damaged_policy_keeps_the_host_strict_and_doctor_says_so() {
    let h = Host::new("damaged");
    std::fs::write(h.dir.join("state").join("sandbox_policy.json"), "{ half written").unwrap();
    let o = h.delulu(&["run", "hello.delulu", "--grant", "console"]);
    assert_eq!(o.status.code(), Some(2), "{}", text(&o));
    assert!(text(&o).contains("cannot be read"), "{}", text(&o));
    let d = text(&h.delulu(&["doctor"]));
    assert!(d.contains("the host policy cannot be read"), "{d}");
    let _ = Path::new(&h.dir);
    let _ = std::fs::remove_dir_all(&h.dir);
}
