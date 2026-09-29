//! PS-E-01 (`docs/DELULULANG_V2/V2_OPENSHELL_STUDY.md` §4.1): a guest's boundary is confirmed before
//! its program is sent — by construction, not by the order of lines in `guest.rs`.
//!
//! A launched guest's channel passes through three states, each consuming the one before:
//!
//! 1. [`open`] sends the [`Open`] frame — the protocol's version and this run's generation, and no
//!    program — and gives an [`Opened`];
//! 2. [`Opened::confirm`] reads the guest's first request, which must be its confinement report for
//!    that generation, has the host accept it, and is the ONLY constructor of a [`Confirmed`];
//! 3. [`Confirmed::send_program`] is the only way the [`Program`] frame is sent, and it hands back the
//!    channel to serve.
//!
//! The fields are private to this module, so nothing else can build a `Confirmed` or reach the channel
//! of an `Opened`: the host cannot send a program to a guest that has not confirmed, however the code
//! around it is arranged. Until `delulu-sandbox-channel/3` the host's first frame WAS the program, so a
//! guest that never confined itself had already been handed it (witnessed by
//! `tests/sandbox_confirm_cli.rs`, red on `ff701ae`).
//!
//! What a confirmation establishes is the guest's own report, in the checked words of
//! `channel::SELF_APPLIED`, for this run; with the host's launch words it answers [`PROPERTIES`] — the
//! five properties a run reports (`sandbox.properties`). What each profile REQUIRES of them, and the
//! refusal when one is missing, is PS-E-01's next step (§4.1's "required set").

use std::io::{self, Read, Write};

use delulu_runtime::channel::{read_frame, write_frame, HostChannel, Open, Program, ReqBody, Request, Response, CHANNEL_VERSION};

/// A guest that has been told its generation and has not confirmed its boundary. It holds the channel
/// so that nothing else can write to it. (The generation it must echo is the host's: `HostChannel`
/// checks it.)
pub(crate) struct Opened<C> {
    conn: C,
}

/// A guest whose boundary report the host accepted, for this run's generation. Built only by
/// [`Opened::confirm`].
pub(crate) struct Confirmed<C> {
    conn: C,
    applied: Vec<&'static str>,
}

/// Open the conversation: this run's generation, and nothing to run.
pub(crate) fn open<C: Read + Write>(mut conn: C, generation: &str) -> io::Result<Opened<C>> {
    write_frame(&mut conn, &Open { version: CHANNEL_VERSION.to_string(), generation: generation.to_string() })?;
    Ok(Opened { conn })
}

/// What this boundary must establish before the program is sent (PS-E-01, D-V2-59): the profile's
/// required properties, and the words the host applied at launch (the guest's own come with its report).
pub(crate) struct Requirement<'a> {
    pub profile: crate::policy::Profile,
    pub launch: &'a [&'static str],
    pub measured_by_host: bool,
}

/// A run refused because its boundary lacks a property its profile requires — DL1408's refusal, carried
/// through `io::Error` so the caller can tell it from a channel that failed.
#[derive(Debug)]
pub(crate) struct Refused(pub String);

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Refused {}

/// Is this error a [`Refused`]?
pub(crate) fn is_refusal(e: &io::Error) -> bool {
    e.get_ref().is_some_and(|inner| inner.is::<Refused>())
}

/// Why a guest's boundary was not confirmed, in words an operator can act on.
fn unconfirmed(why: impl std::fmt::Display) -> io::Error {
    // What the guest sent may be quoted in `why` (a decoder's message, a refused word): shown bounded.
    let why = delulu_runtime::channel::shown(&why.to_string(), 512);
    io::Error::other(format!("the guest never confirmed its boundary, so it was not sent the program: {why}"))
}

impl<C: Read + Write> Opened<C> {
    /// Read the guest's first request. Only a confinement report that `host` accepts — known words,
    /// once, first, for this generation (`HostChannel::answer`) — confirms the guest; anything else ends
    /// the conversation with the program unsent. A first request of any other kind is NOT answered: an
    /// answer to a `RootMethod` would be an effect performed for a guest nobody confirmed.
    pub(crate) fn confirm<S: delulu_runtime::sink::EffectSink>(
        mut self,
        host: &mut HostChannel<S>,
        need: &Requirement<'_>,
    ) -> io::Result<Confirmed<C>> {
        let req: Request = read_frame(&mut self.conn).map_err(|e| {
            unconfirmed(match e.kind() {
                io::ErrorKind::UnexpectedEof => "it closed the channel first".to_string(),
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => "it said nothing within the channel's deadline".to_string(),
                _ => e.to_string(),
            })
        })?;
        if !matches!(req.body, ReqBody::Confined { .. }) {
            host.record_unanswered("DL1401 on a first request that was not the guest's confinement report (not answered)");
            return Err(unconfirmed("its first request was not its confinement report"));
        }
        let resp = host.answer(&req);
        match resp {
            Response::Ok(_) => {
                // D-V2-59: the report is accepted; does the boundary it completes have what the profile
                // requires? Answered from the launch's words and the guest's, as the run report answers it.
                let mut words: Vec<&str> = need.launch.to_vec();
                words.extend(host.self_applied().iter().copied());
                let props = properties(&words, need.measured_by_host);
                let missing: Vec<String> = need
                    .profile
                    .required()
                    .iter()
                    .filter(|p| props[**p]["state"] != "established")
                    .map(|p| format!("{p} ({})", props[*p]["why"].as_str().unwrap_or("not established")))
                    .collect();
                if !missing.is_empty() {
                    let why = format!(
                        "the `{}` profile requires a boundary that establishes all five properties, and this one does \
                         not: {}. Nothing was sent to the guest. Ways out: `--isolation microvm` (Linux with KVM), an \
                         external launcher whose attester vouches for it, or — a person's choice, never made for \
                         you — a weaker profile (`--sandbox-profile contained`) [see `delulu explain DL1408`]",
                        need.profile.name(),
                        missing.join("; ")
                    );
                    let _ = write_frame(&mut self.conn, &Response::Error { code: "DL1408".into(), message: why.clone() });
                    return Err(io::Error::other(Refused(why)));
                }
                write_frame(&mut self.conn, &resp)?;
                Ok(Confirmed { conn: self.conn, applied: host.self_applied().to_vec() })
            }
            Response::Fault { ref message, .. } | Response::Error { ref message, .. } => {
                let why = message.clone();
                // The guest is told why, if it is still listening; the refusal stands either way.
                let _ = write_frame(&mut self.conn, &resp);
                Err(unconfirmed(format!("its confinement report was refused ({why})")))
            }
        }
    }
}

impl<C: Read + Write> Confirmed<C> {
    /// What the guest reported applying to itself, in `channel::SELF_APPLIED`'s words.
    pub(crate) fn applied(&self) -> &[&'static str] {
        &self.applied
    }

    /// Send the program — the only way it is ever sent — and hand back the channel to serve.
    pub(crate) fn send_program(mut self, program: &Program) -> io::Result<C> {
        write_frame(&mut self.conn, program)?;
        Ok(self.conn)
    }
}

/// PS-E-01 (§4.1): the five properties a sandboxed run's boundary has — or has not — each answered
/// from the posture the host already derives from what was APPLIED (the host's launch words and the
/// guest's accepted report), so the properties and the posture cannot disagree: there is one source.
///
/// Reported, not yet required: which of them each profile requires, and the refusal when one is
/// absent, follow once every operating system's answers have been read from CI (a required set that no
/// macOS host can meet would refuse every macOS run — `jail.rs` claims no memory ceiling there).
pub(crate) const PROPERTIES: [&str; 5] =
    ["filesystem_confinement", "egress_confinement", "privilege_floor", "host_loss_ends_guest", "resource_ceiling"];

/// `measured_by_host` is false for an external launcher (L3): its wall is the operator's, and what its
/// guest says of itself is the word of a binary the launcher chose — every property is `unknown`.
pub(crate) fn properties(guarantees: &[&str], measured_by_host: bool) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    if !measured_by_host {
        for p in PROPERTIES {
            out.insert(
                p.to_string(),
                serde_json::json!({
                    "state": "unknown",
                    "why": "an external launcher's wall: DeluluLang measured none of it, and its guest's report is \
                            the word of a binary the launcher chose",
                }),
            );
        }
        return serde_json::Value::Object(out);
    }
    let (posture, _) = crate::policy::SandboxPolicy::posture(guarantees);
    let row = |q: &str| posture[q].as_str().unwrap_or("not confined").to_string();
    let held = |q: &str| row(q) != "not confined";
    let answer = |rows: &[&str]| rows.iter().map(|q| format!("{q}: {}", row(q))).collect::<Vec<_>>().join("; ");
    let separate = row("identity") != "same OS user";
    let killed = guarantees.contains(&"killed with the host");
    let cases: [(&str, bool, String); 5] = [
        ("filesystem_confinement", held("filesystem_writes") && held("filesystem_reads"), answer(&["filesystem_writes", "filesystem_reads"])),
        ("egress_confinement", held("network"), answer(&["network"])),
        (
            "privilege_floor",
            held("privilege_escalation") || separate,
            format!("{}; identity: {}", answer(&["privilege_escalation"]), row("identity")),
        ),
        (
            "host_loss_ends_guest",
            killed,
            if killed {
                "killed with the host".to_string()
            } else {
                "nothing on this host ends the guest when its host dies (PS-E-02)".to_string()
            },
        ),
        ("resource_ceiling", held("memory") && held("processor_time"), answer(&["memory", "processor_time"])),
    ];
    for (name, established, words) in cases {
        out.insert(
            name.to_string(),
            if established {
                serde_json::json!({ "state": "established", "by": words })
            } else {
                serde_json::json!({ "state": "absent", "why": words })
            },
        );
    }
    serde_json::Value::Object(out)
}

// `#[cfg(test)]` first and alone: the thread-stack gate (`actors.rs`) cuts each file at that exact
// attribute, so the scripted guest thread below is read as test code, which it is.
#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;
    use delulu_runtime::channel::ChannelSink;
    use delulu_runtime::sink::LocalSink;
    use std::os::unix::net::UnixStream;

    const GEN: &str = "5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a5a";
    const NEED: Requirement<'static> =
        Requirement { profile: crate::policy::Profile::Contained, launch: &[], measured_by_host: true };

    fn program() -> Program {
        let text = "module p\n\nfn main(root: Root) {\n}\n".to_string();
        let hash = blake3::hash(text.as_bytes()).to_hex().to_string();
        Program { program: text, hash, seed: 1, fixed_clock_ms: None }
    }

    /// Run a scripted guest on the other end: it reads the `Open` frame, then does `act` with the channel.
    fn with_guest(act: impl FnOnce(UnixStream, Open) + Send + 'static) -> (UnixStream, std::thread::JoinHandle<()>) {
        let (host_end, guest_end) = UnixStream::pair().unwrap();
        guest_end.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
        host_end.set_read_timeout(Some(std::time::Duration::from_secs(5))).unwrap();
        let t = std::thread::spawn(move || {
            let mut g = guest_end;
            let open: Open = read_frame(&mut g).expect("the host opens first");
            act(g, open);
        });
        (host_end, t)
    }

    /// Everything the guest hears after the host's answer to its report, until the host hangs up.
    fn rest(mut g: UnixStream) -> Vec<u8> {
        let mut all = Vec::new();
        let _ = g.read_to_end(&mut all);
        all
    }

    #[test]
    fn an_honest_guest_is_confirmed_and_only_then_sent_the_program() {
        let (tx, rx) = std::sync::mpsc::channel();
        let (conn, t) = with_guest(move |g, open| {
            assert_eq!(open.version, CHANNEL_VERSION);
            let sink = ChannelSink::new(g);
            sink.confined(&["no new programs"], &open.generation).expect("the host takes an honest report");
            let p: Program = sink.receive().expect("and then sends the program");
            tx.send((open.generation, p)).unwrap();
        });
        let mut host = HostChannel::new(LocalSink).with_generation(GEN);
        let confirmed = open(conn, GEN).unwrap().confirm(&mut host, &NEED).expect("confirmed");
        assert_eq!(confirmed.applied(), ["no new programs"]);
        let _conn = confirmed.send_program(&program()).unwrap();
        t.join().unwrap();
        let (generation, p) = rx.recv().unwrap();
        assert_eq!(generation, GEN, "the guest was told this run's generation");
        assert_eq!(p, program());
    }

    /// Each way of not confirming ends the conversation with the program unsent — and a first request
    /// that would perform something is never answered at all.
    #[test]
    fn a_guest_that_does_not_confirm_is_never_sent_the_program() {
        type Guest = Box<dyn FnOnce(&mut UnixStream, &Open) + Send>;
        let cases: Vec<(&str, Guest, &str)> = vec![
            ("hangs up", Box::new(|_, _| {}), "closed the channel first"),
            (
                "another run's generation",
                Box::new(|g, _| {
                    let req = Request {
                        version: CHANNEL_VERSION.into(),
                        seq: 1,
                        body: ReqBody::Confined { applied: vec!["no new programs".into()], generation: "00".repeat(32) },
                    };
                    write_frame(g, &req).unwrap();
                }),
                "another run's generation",
            ),
            (
                "a word nobody applies",
                Box::new(|g, open| {
                    let req = Request {
                        version: CHANNEL_VERSION.into(),
                        seq: 1,
                        body: ReqBody::Confined { applied: vec!["trust me".into()], generation: open.generation.clone() },
                    };
                    write_frame(g, &req).unwrap();
                }),
                "not a boundary a guest applies",
            ),
            (
                "asks for the console first",
                Box::new(|g, _| {
                    let req = Request {
                        version: CHANNEL_VERSION.into(),
                        seq: 1,
                        body: ReqBody::RootMethod { method: "console".into(), args: vec![], file: 0, start: 0, end: 1 },
                    };
                    write_frame(g, &req).unwrap();
                }),
                "not its confinement report",
            ),
        ];
        for (name, act, says) in cases {
            let (tx, rx) = std::sync::mpsc::channel();
            let (conn, t) = with_guest(move |mut g, open| {
                act(&mut g, &open);
                g.shutdown(std::net::Shutdown::Write).ok();
                tx.send(rest(g)).unwrap();
            });
            // A root that DOES grant the console, so an unconfirmed guest's request for it would be
            // performed if it were answered at all.
            let mut grants = delulu_runtime::broker::Grants::default();
            grants.add("console").unwrap();
            let mut host = HostChannel::new(LocalSink).with_generation(GEN).with_root(std::rc::Rc::new(crate::cli::build_root(&grants)));
            let err = match open(conn, GEN).unwrap().confirm(&mut host, &NEED) {
                Ok(_) => panic!("{name}: confirmed"),
                Err(e) => e.to_string(),
            };
            t.join().unwrap();
            assert!(err.contains("was not sent the program") && err.contains(says), "{name}: {err}");
            let heard = rx.recv().unwrap();
            let heard = String::from_utf8_lossy(&heard);
            assert!(!heard.contains("module p"), "{name}: the program reached the guest: {heard:?}");
            if name == "asks for the console first" {
                assert!(heard.is_empty(), "{name}: an unconfirmed guest's request was answered: {heard:?}");
                assert_eq!(host.denied().1, 1, "{name}: the unanswered request is on the record");
            }
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    fn states(v: &serde_json::Value) -> Vec<(&'static str, String)> {
        PROPERTIES.iter().map(|p| (*p, v[*p]["state"].as_str().unwrap().to_string())).collect()
    }

    fn state_of(v: &serde_json::Value, p: &str) -> String {
        v[p]["state"].as_str().unwrap().to_string()
    }

    /// Each platform's words, as its launch applies them (`jail.rs`, `identity.rs`, `microvm.rs`) and its
    /// guest reports them (`channel::SELF_APPLIED`) — so the answers below are the ones a real run of that
    /// kind reports, and a platform's gap shows as `absent` with its reason, never as `established`.
    #[test]
    fn each_platforms_words_answer_the_five_properties() {
        // Linux L1 with Landlock, as this VM's run reports it (no second identity here).
        let linux = [
            "memory ceiling", "processor-time ceiling", "no privilege escalation", "no core dump", "killed with the host",
            "no file writes", "reads only from the system paths", "no TCP bind or connect", "no new programs",
            "no debugger", "no namespace or module tricks", "no sockets but the channel",
        ];
        let v = properties(&linux, true);
        assert!(states(&v).iter().all(|(_, s)| s == "established"), "{v}");
        assert!(v["filesystem_confinement"]["by"].as_str().unwrap().contains("confined to the system paths"), "{v}");

        // Linux on a kernel with no Landlock: its files are open, but its filter still refuses every new
        // socket (PS-E-03 H2), so its network is the channel alone.
        let old_kernel = [
            "memory ceiling", "processor-time ceiling", "no privilege escalation", "no core dump", "killed with the host",
            "no new programs", "no sockets but the channel",
        ];
        let v = properties(&old_kernel, true);
        assert_eq!(state_of(&v, "filesystem_confinement"), "absent", "{v}");
        assert_eq!(state_of(&v, "egress_confinement"), "established", "{v}");
        assert_eq!(state_of(&v, "resource_ceiling"), "established", "{v}");
        // Landlock's TCP rule alone is NOT the channel alone: UDP, netlink and Unix sockets stayed open
        // under it (GUEST-SOCKET-1), so without the filter's word the network is not confined.
        let tcp_only = ["no file writes", "reads only from the system paths", "no TCP bind or connect"];
        let v = properties(&tcp_only, true);
        assert_eq!(state_of(&v, "egress_confinement"), "absent", "{v}");
        assert!(v["egress_confinement"]["why"].as_str().unwrap().contains("network: not confined"), "{v}");

        // macOS L1: Seatbelt denies writes and the network, but reads stay open and no memory ceiling is
        // claimed (RLIMIT_DATA is refused there). Since PS-E-02 (D-V2-60) a watcher outside the guest ends
        // it with its host, claimed once the watcher is armed; a run whose watcher did not arm says so.
        let macos = [
            "deny by default", "no file writes", "no network but the channel", "no new programs", "no Mach services",
            "no signals or process info beyond itself", "processor-time ceiling", "no core dump", "killed with the host",
        ];
        let v = properties(&macos, true);
        assert_eq!(state_of(&v, "filesystem_confinement"), "absent", "{v}");
        assert_eq!(state_of(&v, "egress_confinement"), "established", "{v}");
        assert_eq!(state_of(&v, "privilege_floor"), "established", "{v}");
        assert_eq!(state_of(&v, "host_loss_ends_guest"), "established", "{v}");
        assert_eq!(state_of(&v, "resource_ceiling"), "absent", "{v}");
        assert!(v["resource_ceiling"]["why"].as_str().unwrap().contains("memory: not confined"), "{v}");
        let unwatched: Vec<&str> = macos.iter().copied().filter(|w| *w != "killed with the host").collect();
        let v = properties(&unwatched, true);
        assert_eq!(state_of(&v, "host_loss_ends_guest"), "absent", "{v}");
        assert!(v["host_loss_ends_guest"]["why"].as_str().unwrap().contains("PS-E-02"), "{v}");

        // Windows L1: the Job Object and a per-run AppContainer — the identity is the privilege floor.
        let windows = ["one process only", "memory ceiling", "processor-time ceiling", "killed with the host", crate::identity::WINDOWS_GUARANTEE];
        let v = properties(&windows, true);
        assert!(states(&v).iter().all(|(_, s)| s == "established"), "{v}");
        assert!(v["privilege_floor"]["by"].as_str().unwrap().contains("AppContainer"), "{v}");

        // L2 (Linux only): the microVM's own words and its guest's.
        #[cfg(target_os = "linux")]
        {
            let mut vm: Vec<&str> = crate::microvm::GUARANTEES.to_vec();
            vm.extend(["no new programs", "no network stack in its kernel"]);
            let v = properties(&vm, true);
            assert!(states(&v).iter().all(|(_, s)| s == "established"), "{v}");
        }

        // Nothing applied at all: nothing established.
        let v = properties(&[], true);
        assert!(states(&v).iter().all(|(_, s)| s == "absent"), "{v}");

        // L3: whatever words arrive, nothing is established by them.
        let v = properties(&linux, false);
        assert!(states(&v).iter().all(|(_, s)| s == "unknown"), "{v}");
    }
}
