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
//! What a confirmation establishes today is the guest's own report, in the checked words of
//! `channel::SELF_APPLIED`, for this run. What each profile REQUIRES of it — the five properties and the
//! refusal when one is missing — is PS-E-01's next step (§4.1's "required set").

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

/// Why a guest's boundary was not confirmed, in words an operator can act on.
fn unconfirmed(why: impl std::fmt::Display) -> io::Error {
    io::Error::other(format!("the guest never confirmed its boundary, so it was not sent the program: {why}"))
}

impl<C: Read + Write> Opened<C> {
    /// Read the guest's first request. Only a confinement report that `host` accepts — known words,
    /// once, first, for this generation (`HostChannel::answer`) — confirms the guest; anything else ends
    /// the conversation with the program unsent. A first request of any other kind is NOT answered: an
    /// answer to a `RootMethod` would be an effect performed for a guest nobody confirmed.
    pub(crate) fn confirm<S: delulu_runtime::sink::EffectSink>(mut self, host: &mut HostChannel<S>) -> io::Result<Confirmed<C>> {
        let req: Request = read_frame(&mut self.conn).map_err(|e| {
            unconfirmed(match e.kind() {
                io::ErrorKind::UnexpectedEof => "it closed the channel first".to_string(),
                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut => "it said nothing within the channel's deadline".to_string(),
                _ => e.to_string(),
            })
        })?;
        if !matches!(req.body, ReqBody::Confined { .. }) {
            return Err(unconfirmed("its first request was not its confinement report"));
        }
        let resp = host.answer(&req);
        match resp {
            Response::Ok(_) => {
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
        let confirmed = open(conn, GEN).unwrap().confirm(&mut host).expect("confirmed");
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
            let err = match open(conn, GEN).unwrap().confirm(&mut host) {
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
            }
        }
    }
}
