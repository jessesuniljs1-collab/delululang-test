//! The `delulu` CLI (spec §9.5). Terminal-first: everything the language can do is reachable
//! here, with `--json` for machine consumers (agents) and human-rendered output otherwise
//! (Constitution §8.4). Exit codes are part of the stable contract: 0 ok / 1 diagnostics /
//! 2 internal.

mod advisories;
mod atlas_chain;
mod attest;
mod boundary;
mod breakglass;
mod broker_client;
mod broker_ipc;
mod broker_transport;
mod brokerd;
mod budget;
mod ceiling;
mod cert_crypto;
mod cli;
mod completions;
mod deploy;
mod device_cmd;
mod doctor;
mod edit;
mod examples;
mod fix;
mod guest;
mod identity;
mod jail;
mod launcher;
mod monitor_cmd;
// Used by the Windows contained guest today; its tests run everywhere, so the module is not gated.
mod fleet;
mod foreign_worker;
mod locale;
mod lsp;
mod mcp;
mod morph_file;
mod new;
#[cfg_attr(not(windows), allow(dead_code))]
mod pipe_channel;
mod policy;
mod signing;
// Phase 5i / PS-C: the microVM profile is Linux-only (spec §6); the probe, the launcher and the
// in-VM guest compile only there. Every other platform refuses `--isolation microvm` with DL1408 in `cli.rs` (trap 8 —
// never fake a weaker platform's isolation as equivalent).
#[cfg(target_os = "linux")]
mod microvm;
mod openshell;
mod repl;
mod run_cmd;
mod sandbox;
mod schema;
mod toolchain;
mod within;

use std::process::ExitCode;

/// SANDBOX-STOP-1: a sandbox guest whose allocation is refused at its memory ceiling ends with a
/// status the host can name, instead of Rust's abort text (`ceiling.rs`). Unchanged outside a guest.
#[global_allocator]
static ALLOCATOR: ceiling::CeilingAware = ceiling::CeilingAware;

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
///
/// **RW 3.29: the reservation walks a ladder, and the bound follows the stack it got.** A host that caps address space
/// (`ulimit -v`, the sandbox's own memory ceiling — `hostile-agent`'s 256 MiB) refuses the full reservation. This used to
/// fall back to the main stack (8 MiB on Linux) with the full bound, so a recursion 100,000 deep, DL0905 without the
/// cap, aborted the process (*"thread 'main' has overflowed its stack"*, exit 134, no report). Now the CLI runs on the
/// largest rung of `INTERPRETER_STACK_LADDER` the host grants, and every interpreter it builds takes the bound that fits
/// that stack (`delulu_runtime::on_laddered_thread`), so the guard still fires first.
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // A worker panic becomes exit 2 (internal), never a pretended success. Rust has already printed the panic itself.
    // **This mapping is load-bearing and must not be "simplified" into something that loses it**: `json_contract.rs` once
    // keyed its no-panic sweep on exit 101 and could therefore not see *any* crash in the path where all the work happens
    // (campaign finding C49, ruling D44c).
    let code = delulu_runtime::on_laddered_thread(move || cli::run(&args)).unwrap_or(2);
    ExitCode::from(code as u8)
}
