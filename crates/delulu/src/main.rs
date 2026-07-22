//! The `delulu` CLI (spec §9.5). Terminal-first: everything the language can do is reachable
//! here, with `--json` for machine consumers (agents) and human-rendered output otherwise
//! (Constitution §8.4). Exit codes are part of the stable contract: 0 ok / 1 diagnostics /
//! 2 internal.

mod advisories;
mod broker_client;
mod broker_ipc;
mod broker_transport;
mod brokerd;
mod cert_crypto;
mod cli;
mod deploy;
mod fleet;
mod foreign_worker;
mod locale;
mod lsp;
mod signing;
// Phase 5i: the microVM profile is Linux-first (spec §6); the probe/launch module compiles only
// there. Every other platform refuses `--isolation microvm` with DL1408 in `cli.rs` (trap 8 —
// never fake a weaker platform's isolation as equivalent).
#[cfg(target_os = "linux")]
mod microvm;
mod repl;

use std::process::ExitCode;

/// The interpreter is a tree-walker: one DeluluLang call costs several native frames, and a debug
/// build's frames are large. On a default 1 MiB main stack, recursion a few hundred deep exhausted
/// the native stack and the process died with a raw stack-overflow abort — no diagnostic, no exit
/// code, nothing a caller could act on. That is a violation of `ref.rule.runtime.faults-are-
/// diagnostics`: a runtime fault must be a diagnostic (`DL0905`), never a host crash.
///
/// So the whole CLI runs on a thread with a stack large enough for the interpreter's own depth
/// bound (`MAX_DEPTH`) to be the limit that actually fires. The reservation is virtual — pages are
/// committed only as they are touched — so this costs nothing for the programs that never recurse.
///
/// Found by Study C: `fib(24)` crashed the process instead of reporting anything.
const INTERPRETER_STACK_BYTES: usize = 512 * 1024 * 1024;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let worker = std::thread::Builder::new()
        .name("delulu-main".into())
        .stack_size(INTERPRETER_STACK_BYTES)
        .spawn(move || cli::run(&args));
    let code = match worker {
        Ok(handle) => match handle.join() {
            Ok(code) => code,
            // The worker panicked. Rust already printed the panic; exit 2 (internal) rather than
            // pretending success.
            Err(_) => 2,
        },
        // If the thread cannot be spawned, fall back to the main stack rather than refusing to
        // run at all — a shallow program still works, and a deep one now hits DL0905 or the same
        // crash it would have hit anyway.
        Err(_) => {
            let args: Vec<String> = std::env::args().skip(1).collect();
            cli::run(&args)
        }
    };
    ExitCode::from(code as u8)
}
