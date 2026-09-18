//! The sandbox guest (PS-A-03): `delulu __guest`, the child that runs a program while holding no
//! authority of its own.
//!
//! The host sends one [`Hello`] frame — the program, the hash it must match, the seed and the clock
//! — and then answers the guest's requests on `delulu-sandbox-channel/1` until the guest says it is
//! done. The guest's own root is EMPTY: every capability it uses is a handle the host minted, so
//! "the guest performs no effects" is true by construction rather than by policy.
//!
//! Standard input and standard output belong to the CHANNEL here. The program's own output is
//! performed by the host, on the host's streams, which is also what keeps a guest from forging the
//! run's report (D-V2-21).
//!
//! Like the foreign worker, this is an INTERNAL subcommand: dispatched by constant through a guard
//! arm, so it is never offered in `--help` or in completions, and never typed by a caller. The gates
//! that read the dispatch see literal-string arms only, which is exactly why the worker uses this
//! shape. What covers it instead is `tests/guest_cli.rs`, which drives the real binary end to end,
//! including its refusal to run without a hello.

use std::io::{self, Read, Write};
use std::rc::Rc;

use delulu_runtime::channel::{read_frame, write_frame, ChannelSink, Hello, HostChannel, CHANNEL_VERSION};
use delulu_runtime::interp::Interp;
use delulu_runtime::sink::LocalSink;
use delulu_runtime::value::{RootVal, Value};

/// The guest's two halves of one conversation: it reads replies from standard input and writes
/// requests to standard output.
struct Stdio {
    r: io::Stdin,
    w: io::Stdout,
}

impl Read for Stdio {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.r.lock().read(buf)
    }
}

impl Write for Stdio {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.w.lock().write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.w.lock().flush()
    }
}

/// The internal subcommand name. Never advertised: the host passes it when it spawns the child.
pub const GUEST_SUBCOMMAND: &str = "__guest";

/// Run as the guest. Returns the process exit status.
pub fn run_guest() -> i32 {
    let mut io_pair = Stdio { r: io::stdin(), w: io::stdout() };
    let hello: Hello = match read_frame(&mut io_pair) {
        Ok(h) => h,
        // No hello, no run: a guest started by anything other than its host does nothing at all.
        Err(e) => {
            eprintln!("error: the sandbox guest was started without a hello frame ({e})");
            return 2;
        }
    };
    if hello.version != CHANNEL_VERSION {
        eprintln!("error: the host speaks `{}`, this guest speaks `{CHANNEL_VERSION}`", hello.version);
        return 2;
    }
    // The hash pins WHAT runs: a program swapped in flight is refused, not executed.
    let got = blake3::hash(hello.program.as_bytes()).to_hex().to_string();
    if got != hello.hash {
        eprintln!("error: the program does not match the hash the host sent — refused");
        return 2;
    }

    let checked = delulu_check::check_source(0, &hello.program);
    if checked.diagnostics.iter().any(|d| d.is_error()) {
        eprintln!("error: the host sent a program that does not check");
        return 1;
    }

    delulu_runtime::prim::set_rand_seed(hello.seed);
    delulu_runtime::prim::set_fixed_clock_ms(hello.fixed_clock_ms);

    let sink = Rc::new(ChannelSink::new(io_pair));
    let interp = Interp::new(&checked.module).with_effect_sink(sink.clone());
    // The guest's root grants NOTHING. Every capability the program obtains is minted by the host,
    // over the channel, from the root the operator actually granted.
    let exit = match interp.run_main(Value::Root(Rc::new(RootVal::default()))) {
        Ok(_) => 0,
        Err(f) => {
            eprintln!("error[{}]: {}", f.code, f.message);
            1
        }
    };
    // Say goodbye even on failure, so the host stops serving rather than waiting on a dead guest.
    let _ = sink.done(exit);
    exit
}

/// The host half: launch a guest, hand it the program, and serve its effects under `root`.
///
/// Not yet reachable from the CLI: `--sandbox` and the profiles are PS-A3's, and wiring a half-built
/// flag would be worse than leaving the function here with its tests. `tests/guest_cli.rs` drives the
/// same arrangement against the real binary today.
#[allow(dead_code)]
///
/// Today the guest is an ordinary child process: it has no OS jail yet, which PS-A2 adds. What it
/// already has is no capability of its own — it can only ask.
pub fn spawn_and_serve(
    program: &str,
    root: Rc<RootVal>,
    seed: u64,
    fixed_clock_ms: Option<i64>,
) -> io::Result<i32> {
    let exe = std::env::current_exe()?;
    let mut child = std::process::Command::new(exe)
        .arg("__guest")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    let mut to_guest = child.stdin.take().expect("the guest's standard input is piped");
    let mut from_guest = child.stdout.take().expect("the guest's standard output is piped");

    let hello = Hello {
        version: CHANNEL_VERSION.to_string(),
        program: program.to_string(),
        hash: blake3::hash(program.as_bytes()).to_hex().to_string(),
        seed,
        fixed_clock_ms,
    };
    write_frame(&mut to_guest, &hello)?;

    let mut host = HostChannel::new(LocalSink).with_root(root);
    let served = host.serve(&mut from_guest, &mut to_guest);
    // Whatever happened on the channel, the child is not left running.
    let status = child.wait()?;
    match served {
        Ok(exit) => Ok(exit),
        // A guest that dies without saying goodbye is a failure, never a silent success.
        Err(e) if status.success() => Err(io::Error::new(io::ErrorKind::UnexpectedEof, format!("the guest stopped mid-conversation: {e}"))),
        Err(_) => Ok(status.code().unwrap_or(1)),
    }
}
