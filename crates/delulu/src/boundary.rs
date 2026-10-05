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
//! five properties a run reports (`sandbox.properties`). What each profile REQUIRES of them is
//! `Profile::required` (D-V2-59), refused in [`Opened::confirm`] before the program is sent; at level 3 a
//! property is met only by the claim of the attester the run pinned that names it (D-V2-87).

use std::io::{self, Read, Write};

use delulu_runtime::channel::{
    read_frame, write_frame, FrameTooSlow, HostChannel, Open, Program, ReqBody, Request, Response, Within, CHANNEL_VERSION,
    OUTER_FILTER,
};

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
    // A guest already gone — a launcher that exits at once — fails this first write; said in words, as the
    // reads below are, never as "Broken pipe (os error 32)" (macOS CI, `3ec690b`'s push run).
    write_frame(&mut conn, &Open { version: CHANNEL_VERSION.to_string(), generation: generation.to_string() })
        .map_err(|e| unconfirmed(gone_before(&e, "the host opened it")))?;
    Ok(Opened { conn })
}

/// A write to a guest that has closed its end, in words: `when` says at which step.
fn gone_before(e: &io::Error, when: &str) -> String {
    match e.kind() {
        io::ErrorKind::BrokenPipe | io::ErrorKind::ConnectionReset | io::ErrorKind::UnexpectedEof => {
            format!("it had closed the channel before {when}")
        }
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => "it took nothing within the channel's deadline".to_string(),
        _ => e.to_string(),
    }
}

/// What this boundary must establish before the program is sent (PS-E-01, D-V2-59): the profile's
/// required properties, and the words the host applied at launch (the guest's own come with its report).
pub(crate) struct Requirement<'a> {
    pub profile: crate::policy::Profile,
    pub launch: &'a [&'static str],
    pub measured_by_host: bool,
    /// D-V2-87: the statement of the attester this run pinned, verified before the guest was opened —
    /// at level 3 the only thing that can meet a required property (by a claim that names it).
    pub attested: Option<&'a crate::attest::Attested>,
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
        // RW 4.32: its report is a frame like any other, owed whole within the host's frame deadline.
        let within = host.frame_deadline();
        let req: Request = read_frame(&mut Within::from_first_byte(&mut self.conn, within)).map_err(|e| {
            unconfirmed(match e.kind() {
                io::ErrorKind::UnexpectedEof => "it closed the channel first".to_string(),
                _ if FrameTooSlow::is(&e) => format!("it {}", e.get_ref().map_or_else(String::new, |x| x.to_string())),
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
            // D-V2-83: a guest standing under an outer wall's syscall filter in place of its own is one an
            // external launcher started and declared that wall for. A guest the HOST started was given no such
            // wall and no such word: its own filter is part of the boundary this host measures and claims.
            Response::Ok(_) if need.measured_by_host && host.self_applied().contains(&OUTER_FILTER) => {
                let why = format!(
                    "`{OUTER_FILTER}` is a word only a guest an external launcher started may report (D-V2-83), and \
                     this guest was started by this host"
                );
                let _ = write_frame(&mut self.conn, &Response::Error { code: "DL1401".into(), message: why.clone() });
                Err(unconfirmed(format!("its confinement report was refused ({why})")))
            }
            Response::Ok(_) => {
                // D-V2-59: the report is accepted; does the boundary it completes have what the profile
                // requires? Answered from the launch's words and the guest's, as the run report answers it.
                let mut words: Vec<&str> = need.launch.to_vec();
                words.extend(host.self_applied().iter().copied());
                let props = properties(&words, need.measured_by_host, need.attested);
                let missing: Vec<String> = need
                    .profile
                    .required()
                    .iter()
                    .filter(|p| !met(&props[**p]))
                    .map(|p| format!("{p} ({})", props[*p]["why"].as_str().unwrap_or("not established")))
                    .collect();
                if !missing.is_empty() {
                    let why = format!(
                        "the `{}` profile requires a boundary with all five properties — each established by this \
                         host or, at level 3, vouched for by name by the attester the run pinned — and this one \
                         lacks: {}. Nothing was sent to the guest. Ways out: `--isolation microvm` (Linux with KVM), \
                         an external launcher whose attester vouches for each property by name (`--require-attestation \
                         KEY`, and a claim `PROPERTY: how` for each), or — a person's choice, never made for you — a \
                         weaker profile (`--sandbox-profile contained`) [see `delulu explain DL1408`]",
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
        write_frame(&mut self.conn, program).map_err(|e| {
            let why = gone_before(&e, "it was sent the program");
            io::Error::new(e.kind(), format!("the guest confirmed its boundary but was not sent the program: {why}"))
        })?;
        Ok(self.conn)
    }
}

/// PS-E-01 (§4.1): the five properties a sandboxed run's boundary has — or has not — each answered
/// from the posture the host already derives from what was APPLIED (the host's launch words and the
/// guest's accepted report), so the properties and the posture cannot disagree: there is one source.
///
/// Which of them each profile requires is `Profile::required` (D-V2-59): `hostile-agent` all five. A
/// `contained` set that no macOS host can meet would refuse every macOS run — macOS's reads stay open
/// (its memory ceiling is the host's sampler since D-V2-90).
pub(crate) const PROPERTIES: [&str; 5] =
    ["filesystem_confinement", "egress_confinement", "privilege_floor", "host_loss_ends_guest", "resource_ceiling"];

/// Does this answer meet a profile's requirement? Established by the host — or, at level 3 only (where
/// [`properties`] alone writes it), vouched for by name by the attester the run pinned (D-V2-87).
fn met(property: &serde_json::Value) -> bool {
    property["state"] == "established" || property.get("attested").is_some()
}

/// `measured_by_host` is false for an external launcher (L3): its wall is the operator's, and what its
/// guest says of itself is the word of a binary the launcher chose — every property is `unknown`. Where
/// the run's pinned attester names a property (`PROPERTY: how`, D-V2-87), its claim is kept beside that
/// `unknown` as `attested` — the attester's word, never the host's measurement. A run the host measured
/// takes no attester's word for anything: `attested` is read only at level 3.
pub(crate) fn properties(
    guarantees: &[&str],
    measured_by_host: bool,
    attested: Option<&crate::attest::Attested>,
) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    if !measured_by_host {
        for p in PROPERTIES {
            let mut answer = serde_json::json!({
                "state": "unknown",
                "why": "an external launcher's wall: DeluluLang measured none of it, and its guest's report is the \
                        word of a binary the launcher chose",
            });
            match attested.map(|a| (a, a.vouches_for(p))) {
                Some((a, Some(how))) => answer["attested"] = serde_json::json!({ "attester": a.attester, "by": how }),
                Some((a, None)) => {
                    answer["why"] = serde_json::json!(format!(
                        "an external launcher's wall: DeluluLang measured none of it, and the attester `{}` did not \
                         vouch for it by name (a claim `{p}: how`)",
                        a.attester
                    ))
                }
                None => {}
            }
            out.insert(p.to_string(), answer);
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
        Requirement { profile: crate::policy::Profile::Contained, launch: &[], measured_by_host: true, attested: None };

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

    /// D-V2-83: "an outer syscall filter, not its own" is an external launcher's guest's word. A guest this
    /// host started is refused it — its own filter is part of the boundary the host claims — and is never
    /// sent the program; at L3 the same report is confirmed, the word kept as the guest's.
    #[test]
    fn an_outer_filter_is_an_external_guests_word_alone() {
        for measured_by_host in [true, false] {
            let (tx, rx) = std::sync::mpsc::channel();
            let (conn, t) = with_guest(move |g, open| {
                let sink = ChannelSink::new(g);
                let taken = sink.confined(&["no file writes", OUTER_FILTER], &open.generation).is_ok();
                let sent = taken && sink.receive::<Program>().is_ok();
                tx.send((taken, sent)).unwrap();
            });
            let mut host = HostChannel::new(LocalSink).with_generation(GEN);
            let need = Requirement { profile: crate::policy::Profile::Contained, launch: &[], measured_by_host, attested: None };
            match open(conn, GEN).unwrap().confirm(&mut host, &need) {
                Ok(confirmed) => {
                    assert!(!measured_by_host, "a guest this host started was confirmed with an outer filter");
                    assert_eq!(confirmed.applied(), ["no file writes", OUTER_FILTER]);
                    let _conn = confirmed.send_program(&program()).unwrap();
                }
                Err(e) => {
                    assert!(measured_by_host, "an external launcher's guest was refused its word: {e}");
                    assert!(e.to_string().contains("only a guest an external launcher started may report"), "{e}");
                }
            }
            t.join().unwrap();
            let (taken, sent) = rx.recv().unwrap();
            assert_eq!((taken, sent), (!measured_by_host, !measured_by_host), "measured_by_host = {measured_by_host}");
        }
    }

    /// D-V2-83: the word establishes nothing — the posture a guest's words answer is the same with it as
    /// without it, so no future posture needle can be matched inside it by accident.
    #[test]
    fn an_outer_filter_establishes_nothing() {
        let (without, _) = crate::policy::SandboxPolicy::posture(&[]);
        let (with, _) = crate::policy::SandboxPolicy::posture(&[OUTER_FILTER]);
        assert_eq!(with, without);
        assert_eq!(properties(&[OUTER_FILTER], true, None), properties(&[], true, None));
    }

    /// RW 4.32, FRAME-DRIP-1: the guest's confinement report is a frame like any other — begun, it is
    /// owed whole within the host's frame deadline, or the guest is not confirmed. Each byte here comes
    /// well inside the channel's five-second read deadline; the report as a whole does not.
    #[test]
    fn a_confinement_report_dripped_past_the_frame_deadline_does_not_confirm() {
        use std::io::Write as _;
        let (conn, t) = with_guest(|mut g, open| {
            let mut frame = Vec::new();
            let req = Request {
                version: CHANNEL_VERSION.into(),
                seq: 1,
                body: ReqBody::Confined { applied: vec!["no new programs".into()], generation: open.generation.clone() },
            };
            write_frame(&mut frame, &req).unwrap();
            for b in frame {
                if g.write_all(&[b]).is_err() {
                    return;
                }
                std::thread::sleep(std::time::Duration::from_millis(40));
            }
            let _ = rest(g);
        });
        let mut host = HostChannel::new(LocalSink).with_generation(GEN).with_frame_deadline(std::time::Duration::from_millis(200));
        let said = open(conn, GEN).unwrap().confirm(&mut host, &NEED).err().expect("not confirmed").to_string();
        assert!(said.contains("did not send one whole frame within 200ms"), "{said}");
        t.join().unwrap();
    }

    /// A guest gone before the host opens the channel — a launcher that exits at once — fails the host's
    /// FIRST write, and the operator is told so in words, never "Broken pipe (os error 32)". Red on
    /// `dd2a542`; on macOS CI (`3ec690b`'s push run `36527491801`) the watcher's start let such a launcher
    /// die before the host wrote, and the raw error reached the operator.
    #[test]
    fn a_guest_gone_before_the_host_opens_the_channel_is_told_in_words() {
        let (host_end, guest_end) = UnixStream::pair().unwrap();
        drop(guest_end);
        let said = open(host_end, GEN).err().expect("there is no guest to open the channel to").to_string();
        assert!(!said.contains("os error"), "in words: {said}");
        assert!(said.contains("before the host opened it"), "{said}");
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
    /// D-V2-87: at level 3, `hostile-agent`'s five properties are met only by the claims of the attester the
    /// run pinned that NAME them; one it leaves out refuses before the program is sent. A guest this host
    /// measured takes no attester's word: the same statement meets nothing there.
    #[test]
    fn at_level_3_only_an_attesters_named_claims_meet_a_required_property() {
        let all: Vec<String> = PROPERTIES.iter().map(|p| format!("{p}: by the image")).collect();
        let attested = |claims: &[String]| crate::attest::Attested {
            key: "00".repeat(32),
            attester: "ci".into(),
            guarantees: claims.to_vec(),
            launcher_blake3: None,
        };
        let four = all[..4].to_vec();
        for (name, claims, measured_by_host, confirmed) in [
            ("all five named, level 3", all.clone(), false, true),
            ("four named, level 3", four, false, false),
            ("all five named, a guest this host measured", all.clone(), true, false),
        ] {
            let (tx, rx) = std::sync::mpsc::channel();
            let (conn, t) = with_guest(move |g, open| {
                let sink = ChannelSink::new(g);
                let taken = sink.confined(&["no new programs"], &open.generation).is_ok();
                tx.send(taken && sink.receive::<Program>().is_ok()).unwrap();
            });
            let mut host = HostChannel::new(LocalSink).with_generation(GEN);
            let a = attested(&claims);
            let need =
                Requirement { profile: crate::policy::Profile::HostileAgent, launch: &[], measured_by_host, attested: Some(&a) };
            match open(conn, GEN).unwrap().confirm(&mut host, &need) {
                Ok(c) => {
                    assert!(confirmed, "{name}: confirmed");
                    let _conn = c.send_program(&program()).unwrap();
                }
                Err(e) => {
                    assert!(!confirmed, "{name}: refused: {e}");
                    assert!(is_refusal(&e), "{name}: DL1408's refusal, not a channel's failure: {e}");
                    assert!(e.to_string().contains("resource_ceiling"), "{name}: {e}");
                }
            }
            t.join().unwrap();
            assert_eq!(rx.recv().unwrap(), confirmed, "{name}: the program reached the guest only when confirmed");
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
        let v = properties(&linux, true, None);
        assert!(states(&v).iter().all(|(_, s)| s == "established"), "{v}");
        assert!(v["filesystem_confinement"]["by"].as_str().unwrap().contains("confined to the system paths"), "{v}");

        // Linux on a kernel with no Landlock: its files are open, but its filter still refuses every new
        // socket (PS-E-03 H2), so its network is the channel alone.
        let old_kernel = [
            "memory ceiling", "processor-time ceiling", "no privilege escalation", "no core dump", "killed with the host",
            "no new programs", "no sockets but the channel",
        ];
        let v = properties(&old_kernel, true, None);
        assert_eq!(state_of(&v, "filesystem_confinement"), "absent", "{v}");
        assert_eq!(state_of(&v, "egress_confinement"), "established", "{v}");
        assert_eq!(state_of(&v, "resource_ceiling"), "established", "{v}");
        // PS-E-03 H5: a kernel whose Landlock predates ABI 3 cannot stop truncation, so the guest says "no file
        // writes but truncation" — and that is NOT writes denied: filesystem confinement is absent, and
        // `hostile-agent`, which requires it, refuses. H5 asked for "ABI >= 3" as a requirement; it already is.
        let before_abi3 = [
            "no file writes but truncation", "reads only from the system paths", "no new programs", "no sockets but the channel",
            "memory ceiling", "processor-time ceiling", "no privilege escalation", "killed with the host",
        ];
        let v = properties(&before_abi3, true, None);
        assert_eq!(state_of(&v, "filesystem_confinement"), "absent", "{v}");
        assert!(v["filesystem_confinement"]["why"].as_str().unwrap().contains("filesystem_writes: not confined"), "{v}");
        assert!(crate::policy::Profile::HostileAgent.required().contains(&"filesystem_confinement"));
        // Landlock's TCP rule alone is NOT the channel alone: UDP, netlink and Unix sockets stayed open
        // under it (GUEST-SOCKET-1), so without the filter's word the network is not confined.
        let tcp_only = ["no file writes", "reads only from the system paths", "no TCP bind or connect"];
        let v = properties(&tcp_only, true, None);
        assert_eq!(state_of(&v, "egress_confinement"), "absent", "{v}");
        assert!(v["egress_confinement"]["why"].as_str().unwrap().contains("network: not confined"), "{v}");

        // macOS L1: Seatbelt denies writes and the network, but reads stay open. RLIMIT_DATA is refused
        // there, so since D-V2-90 the memory ceiling is the host's sampler, claimed once the kernel answered
        // its first reading; a run whose sampler had no answer claims none and says so. Since PS-E-02
        // (D-V2-60) a watcher outside the guest ends it with its host, claimed once the watcher is armed; a
        // run whose watcher did not arm says so.
        let macos = [
            "deny by default", "no file writes", "no network but the channel", "no new programs", "no Mach services",
            "no signals or process info beyond itself", "processor-time ceiling", "no core dump", "killed with the host",
            crate::jail::SAMPLED_MEMORY_CEILING,
        ];
        let v = properties(&macos, true, None);
        assert_eq!(state_of(&v, "filesystem_confinement"), "absent", "{v}");
        assert_eq!(state_of(&v, "egress_confinement"), "established", "{v}");
        assert_eq!(state_of(&v, "privilege_floor"), "established", "{v}");
        assert_eq!(state_of(&v, "host_loss_ends_guest"), "established", "{v}");
        assert_eq!(state_of(&v, "resource_ceiling"), "established", "{v}");
        let unsampled: Vec<&str> = macos.iter().copied().filter(|w| *w != crate::jail::SAMPLED_MEMORY_CEILING).collect();
        let v = properties(&unsampled, true, None);
        assert_eq!(state_of(&v, "resource_ceiling"), "absent", "{v}");
        assert!(v["resource_ceiling"]["why"].as_str().unwrap().contains("memory: not confined"), "{v}");
        let unwatched: Vec<&str> = macos.iter().copied().filter(|w| *w != "killed with the host").collect();
        let v = properties(&unwatched, true, None);
        assert_eq!(state_of(&v, "host_loss_ends_guest"), "absent", "{v}");
        assert!(v["host_loss_ends_guest"]["why"].as_str().unwrap().contains("PS-E-02"), "{v}");

        // Windows L1: the Job Object and a per-run AppContainer — the identity is the privilege floor.
        let windows = ["one process only", "memory ceiling", "processor-time ceiling", "killed with the host", crate::identity::WINDOWS_GUARANTEE];
        let v = properties(&windows, true, None);
        assert!(states(&v).iter().all(|(_, s)| s == "established"), "{v}");
        assert!(v["privilege_floor"]["by"].as_str().unwrap().contains("AppContainer"), "{v}");

        // L2 (Linux only): the microVM's own words and its guest's.
        #[cfg(target_os = "linux")]
        {
            let mut vm: Vec<&str> = crate::microvm::GUARANTEES.to_vec();
            vm.extend(["no new programs", "no network stack in its kernel"]);
            let v = properties(&vm, true, None);
            assert!(states(&v).iter().all(|(_, s)| s == "established"), "{v}");
        }

        // Nothing applied at all: nothing established.
        let v = properties(&[], true, None);
        assert!(states(&v).iter().all(|(_, s)| s == "absent"), "{v}");

        // L3: whatever words arrive, nothing is established by them.
        let v = properties(&linux, false, None);
        assert!(states(&v).iter().all(|(_, s)| s == "unknown"), "{v}");
    }

    /// D-V2-87: an L3 property the run's pinned attester names carries its claim beside the `unknown` — the
    /// attester's name and how it says the property holds — and one it does not name says so in `why`. The
    /// state never moves: DeluluLang measured none of an external wall. A measured run takes no attester's
    /// word, so the same statement changes nothing there.
    #[test]
    fn an_attesters_named_claim_is_kept_beside_the_unknown_at_level_3_and_nowhere_else() {
        let a = crate::attest::Attested {
            key: "00".repeat(32),
            attester: "ci-image".into(),
            guarantees: vec![
                "egress_confinement: runsc --network=none".into(),
                "egress_confinement:no NIC".into(),
                "privilege_floor".into(),
                "gVisor".into(),
                "Resource_Ceiling: cgroup".into(),
            ],
            launcher_blake3: None,
        };
        let v = properties(&[], false, Some(&a));
        assert!(states(&v).iter().all(|(_, s)| s == "unknown"), "{v}");
        assert_eq!(v["egress_confinement"]["attested"], serde_json::json!({ "attester": "ci-image", "by": "runsc --network=none; no NIC" }));
        assert_eq!(v["privilege_floor"]["attested"]["by"], "the attester did not say how", "{v}");
        for p in ["filesystem_confinement", "host_loss_ends_guest", "resource_ceiling"] {
            assert!(v[p].get("attested").is_none(), "{p}: {v}");
            let why = v[p]["why"].as_str().unwrap();
            assert!(why.contains("`ci-image` did not vouch for it") && why.contains(&format!("`{p}: how`")), "{p}: {why}");
        }
        assert!(met(&v["egress_confinement"]) && met(&v["privilege_floor"]) && !met(&v["resource_ceiling"]), "{v}");
        // The host's own measurement stands alone where the host measured.
        let measured = properties(&[], true, Some(&a));
        assert_eq!(measured, properties(&[], true, None));
        assert!(PROPERTIES.iter().all(|p| !met(&measured[*p])), "{measured}");
    }
}
