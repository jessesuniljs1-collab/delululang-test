//! The `delulu` CLI (spec §9.5). Terminal-first: everything the language can do is reachable
//! here, with `--json` for machine consumers (agents) and human-rendered output otherwise
//! (Constitution §8.4). Exit codes are part of the stable contract: 0 ok / 1 diagnostics /
//! 2 internal.

mod advisories;
mod broker_client;
mod broker_ipc;
mod broker_transport;
mod brokerd;
mod breakglass;
mod budget;
mod cert_crypto;
mod cli;
mod completions;
mod deploy;
mod doctor;
mod examples;
mod fix;
mod guest;
mod identity;
mod jail;
// Used by the Windows contained guest today; its tests run everywhere, so the module is not gated.
#[cfg_attr(not(windows), allow(dead_code))]
mod pipe_channel;
mod policy;
mod fleet;
mod foreign_worker;
mod locale;
mod lsp;
mod mcp;
mod morph_file;
mod new;
mod signing;
// Phase 5i: the microVM profile is Linux-first (spec §6); the probe/launch module compiles only
// there. Every other platform refuses `--isolation microvm` with DL1408 in `cli.rs` (trap 8 —
// never fake a weaker platform's isolation as equivalent).
#[cfg(target_os = "linux")]
mod microvm;
mod repl;
mod run_cmd;
mod sandbox;
mod schema;
mod toolchain;

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
///
/// **The number now comes from `delulu-runtime`, which owns the depth bound it pays for** (ruling
/// D67). It was defined here, privately, so the rule *"a thread that runs a DeluluLang program
/// reserves a stack sized for the depth bound"* existed at exactly one site — and the actor
/// scheduler, which runs the same interpreter on its own worker threads, was the site that never
/// learned it.
use delulu_runtime::INTERPRETER_STACK_BYTES;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let worker = std::thread::Builder::new()
        .name("delulu-main".into())
        .stack_size(INTERPRETER_STACK_BYTES)
        .spawn(move || cli::run(&args));
    let code = match worker {
        // A worker panic becomes exit 2 (internal), never a pretended success. Rust has already
        // printed the panic itself. **This mapping is load-bearing and must not be "simplified"
        // into something that loses it**: `json_contract.rs` once keyed its no-panic sweep on exit
        // 101 and could therefore not see *any* crash in the path where all the work happens
        // (campaign finding C49, ruling D44c).
        Ok(handle) => handle.join().unwrap_or(2),
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
