//! Command dispatch and the four Stage-1 commands (§9.5).

use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

use delulu_check::{
    authority_report, authority_widened, check_pins, check_program, check_self_authority,
    check_source, check_workspace, compute_lockfile, enforce_semver_law, load_package,
    program_authority, resolve_workspace, verify_locked, Checked, Effect, Lockfile, Program, Row,
    Workspace,
};
use delulu_atlas::{Atlas, BuildInput, ModuleView, PackageView};
use delulu_check::lockfile::api_row_hash;
use delulu_diag::{
    color_enabled, envelope_to_string, render_human_with, ColorChoice, Diagnostic, Palette, Role,
    SourceMap, Theme,
};
use delulu_runtime::{parse_manifest, Grants, Interp, Value};
use delulu_runtime::value::RootVal;
use delulu_syntax::ast::Item;
use serde_json::{json, Value as Json};

// ----- the Palette (Surface addendum §2.5) --------------------------------------------------------
//
// Global `--color`/`--theme` and the `DELULU_COLOR`/`DELULU_THEME`/`NO_COLOR` envs are resolved ONCE
// per process into `SURFACE`; each human-facing surface then asks for a `Palette` scoped to the
// stream it writes to (auto-detection is per-stream TTY). Every `delulu` invocation is a fresh
// process, so a process-global is exactly right here. Color is OFF for non-TTY by default, which is
// why every pre-existing test — all piped — stays byte-identical (criterion 10).

use std::io::IsTerminal;
use std::sync::OnceLock;

struct Surface {
    color_flag: Option<ColorChoice>,
    delulu_color: Option<String>,
    no_color: bool,
    theme: Theme,
}

static SURFACE: OnceLock<Surface> = OnceLock::new();

fn surface() -> &'static Surface {
    // If `init_surface` was never called (e.g. a unit test calling a helper directly), default to
    // "no forced color, default theme" — auto/TTY decides, so tests stay colorless.
    SURFACE.get_or_init(|| Surface {
        color_flag: None,
        delulu_color: None,
        no_color: false,
        theme: Theme::default_theme(),
    })
}

fn palette_for(is_tty: bool) -> Palette {
    let s = surface();
    let on = color_enabled(s.color_flag, s.delulu_color.as_deref(), s.no_color, is_tty);
    Palette::new(on, s.theme.clone())
}

/// The palette for diagnostics / `ok:` lines / banners (all of which write to stderr).
fn palette_stderr() -> Palette {
    palette_for(std::io::stderr().is_terminal())
}

/// The palette for human report bodies written to stdout (grants tree/list, atlas tree).
fn palette_stdout() -> Palette {
    palette_for(std::io::stdout().is_terminal())
}

/// Paint an `ok:`/success line for stderr. A disabled palette returns the text unchanged.
fn ok(msg: impl Into<String>) -> String {
    palette_stderr().paint(Role::Success, &msg.into())
}

/// `ok_line!(fmt, args…)` — an `eprintln!` of a success line painted in the Palette `success` role.
/// Same format arguments as `eprintln!`; with color off it is byte-identical to the original
/// `eprintln!("ok: …")` (criterion 10). Defined before its first use (textual macro scoping).
macro_rules! ok_line {
    ($($arg:tt)*) => {
        eprintln!("{}", ok(format!($($arg)*)))
    };
}

/// Colorize the `g_…` node ids inside broker-rendered grants text (list/tree). Only the ids are
/// wrapped, so a disabled palette leaves the text byte-identical. Safe on any text.
fn colorize_node_ids(text: &str, palette: &Palette) -> String {
    if !palette.is_enabled() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'g' && i + 1 < bytes.len() && bytes[i + 1] == b'_' {
            // Only at a token boundary (start, or preceded by non-identifier char).
            let boundary = i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
            if boundary {
                let mut j = i + 2;
                while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                    j += 1;
                }
                out.push_str(&palette.paint(Role::Authority, &text[i..j]));
                i = j;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Parse and strip the global `--color`/`--theme` flags, resolve the theme, and store `SURFACE`.
/// Returns the cleaned argv (globals removed so the per-command parsers never see them) plus any
/// DL1790 theme warnings to print. Envs (`DELULU_COLOR`/`DELULU_THEME`/`NO_COLOR`) are read here too.
fn init_surface(args: &[String]) -> (Vec<String>, Vec<Diagnostic>) {
    let mut color_flag: Option<ColorChoice> = None;
    let mut theme_flag: Option<String> = None;
    let mut cleaned: Vec<String> = Vec::with_capacity(args.len());
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--color" {
            if i + 1 < args.len() {
                color_flag = ColorChoice::parse(&args[i + 1]);
                i += 1;
            }
        } else if let Some(v) = a.strip_prefix("--color=") {
            color_flag = ColorChoice::parse(v);
        } else if a == "--theme" {
            if i + 1 < args.len() {
                theme_flag = Some(args[i + 1].clone());
                i += 1;
            }
        } else if let Some(v) = a.strip_prefix("--theme=") {
            theme_flag = Some(v.to_string());
        } else {
            cleaned.push(args[i].clone());
        }
        i += 1;
    }

    let delulu_color = std::env::var("DELULU_COLOR").ok();
    // NO_COLOR: present & non-empty ⇒ off (https://no-color.org).
    let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    let env_theme = std::env::var("DELULU_THEME").ok();
    let config_path = theme_config_path();
    let (theme, warn) = delulu_diag::resolve_theme(
        theme_flag.as_deref(),
        env_theme.as_deref(),
        config_path.as_deref(),
    );

    let _ = SURFACE.set(Surface { color_flag, delulu_color, no_color, theme });

    let warnings = warn
        .into_iter()
        .map(|m| Diagnostic::warning("DL1790", m))
        .collect();
    (cleaned, warnings)
}

/// The theme file path: `$DELULU_THEME_FILE` if set (used by tests and power users), else
/// `~/.delulu/theme.toml` (HOME, then USERPROFILE on Windows — the same home resolution the broker
/// state dir uses).
fn theme_config_path() -> Option<std::path::PathBuf> {
    if let Some(explicit) = std::env::var_os("DELULU_THEME_FILE") {
        return Some(std::path::PathBuf::from(explicit));
    }
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(std::path::PathBuf::from(home).join(".delulu").join("theme.toml"))
}

/// Enable Windows console VT processing so ANSI SGR renders instead of printing literally. Uses the
/// `windows-sys` already in this crate's tree (addendum §2.5 — no new dependency). No-op elsewhere
/// and harmless when the handle is a pipe.
#[cfg(windows)]
fn enable_vt() {
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
        STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
    };
    for which in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
        unsafe {
            let handle = GetStdHandle(which);
            if handle.is_null() {
                continue;
            }
            let mut mode = 0u32;
            if GetConsoleMode(handle, &mut mode) != 0 {
                let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

#[cfg(not(windows))]
fn enable_vt() {}

/// Parsed common options.
struct Opts {
    json: bool,
    grants: Vec<String>,
    grant_manifest: bool,
    no_prompt: bool,
    /// `--locked`: refuse any resolution not already pinned in delulu.lock.
    locked: bool,
    /// `--accept-authority <pkg>`: names packages whose authority widening is explicitly accepted.
    accept_authority: Vec<String>,
    /// `--diff <old.lock>`: for `authority`, the previous lockfile to diff against (§8).
    diff: Option<String>,
    /// `--trace-effects`: emit one JSON trace record per effectful operation (spec §6.1).
    trace_effects: bool,
    /// `--trace-out <file>`: write the trace there instead of stderr.
    trace_out: Option<String>,
    /// `--assert-trace`: verify trace ⊆ the checker's row of main; violations are DL1101, exit 3.
    assert_trace: bool,
    /// `--actors-threads N` (Stage 7): scheduler worker count; default = available parallelism.
    actors_threads: Option<usize>,
    /// `--on-quiesce report` (Stage 7): print surviving actor count at quiescence.
    on_quiesce_report: bool,
    trace_memory: bool,
    /// `--on-actor-death abort` (Stage 7 §6.6): whole-program abort on a behavior fault
    /// (default keeps the system live; sends to the dead actor drop and count).
    on_actor_death_abort: bool,
    /// `--debug-rcaps` (Stage 7 §6.4): verify statically-proven iso moves are unaliased at
    /// each actor boundary (DL1610 on violation — a compiler-bug detector).
    debug_rcaps: bool,
    /// `--seed <u64>`: deterministic Cap[Rand] (spec §6.2).
    seed: Option<u64>,
    /// `--clock fixed:<ms>`: deterministic Cap[Clock] (spec §6.2).
    clock_ms: Option<i64>,
    /// `--engine wasm|interp`: which execution engine `run` uses (default interp).
    engine: Option<String>,
    /// `--target wasm`: for `build`, emit a `.dwx` WebAssembly artifact from a single source file.
    target: Option<String>,
    /// `-o <path>` / `--out <path>`: output path for `build --target wasm`.
    out: Option<String>,
    /// `--foreign-max-ret <bytes>`: ceiling on a returned foreign string (invariant 21; default
    /// 64 MiB). A return exceeding it is `ForeignErr::BadReturn`, never a truncated silent success.
    foreign_max_ret: Option<usize>,
    /// `--broker embedded|daemon` (Stage 5 phase 5f): where authority lives for this run. Default
    /// embedded (Stage 1–4 behavior, byte-identical — criterion 11); `daemon` routes every effectful
    /// op through the broker daemon (fail closed DL1401 when unreachable, invariant 27).
    broker: Option<String>,
    /// `--epoch-ms <N>` (spec §4.1): the epoch-class snapshot refresh interval in daemon mode.
    /// Clamped to default 50 / ceiling 250 by `broker_client::clamp_epoch_ms`.
    epoch_ms: Option<u64>,
    /// `--foreign-isolation inproc|process` (spec §5 phase 5h): where a granted C library executes.
    /// `inproc` (dev) loads it in the host process (Stage-4 behavior). `process` runs each library in
    /// an isolated worker subprocess (blast-radius containment; a worker crash is DL1409, host
    /// survives). Default: `process` in `--broker daemon` mode, `inproc` otherwise.
    foreign_isolation: Option<String>,
    /// `--isolation none|process|microvm` (Stage 5 phase 5i, spec §6): the run's isolation profile.
    /// `none` (default) = in-process, language + custody enforcement only. `process` = the code
    /// outside the proof (foreign libs) runs in minimum-privilege worker subprocesses; labeled.
    /// `microvm` = Firecracker-class guest — Linux+KVM only; anywhere else it is DL1408 with the
    /// documented, explicitly-weaker fallback (never a silent approximation — playbook trap 8).
    /// The honest capability matrix is spec §6.1.
    isolation: Option<String>,
    /// `--lease <token>` (Stage 5 phase 5j, spec §3.2/§3.3): redeem a delegated lease token and run
    /// under EXACTLY that node's authority — the orchestration payoff: whoever holds a grant
    /// delegates a slice, hands the token over, and this run can acquire nothing outside it.
    lease: Option<String>,
    /// `--sign <keyfile>` (Stage 6 phase 6h, spec §2.2/§6): for `plugin build`, sign the artifact
    /// with the raw 32-byte ed25519 seed in `keyfile`. Signatures authenticate ORIGIN, not behavior
    /// (spec §10) — a signed plugin is not a safe plugin.
    sign: Option<String>,
    /// `--broker-profile sim|hw:<adapter>` (Stage 10 phase 10f, spec §5.4): which device backend
    /// this run's actuators and sensors are bound to. Absent = the null adapter (10e behavior:
    /// envelopes still refuse, sensors still read `NoDevice`).
    broker_profile: Option<String>,
    /// `--sim-step <ms>` (build-order D20): under `--broker-profile sim`, advance the dead-man
    /// lease clock by this many SIMULATED milliseconds per device interaction (command, read, or
    /// beat) instead of wall-clock time. Makes lease timing — loss-of-signal, heartbeat — a
    /// deterministic function of the command sequence, identical across debug/release/load. Only
    /// meaningful with `sim`; the wall clock (its real-time dead-man) is the default.
    /// `--adapter-cmd <command>`: the hardware driver to run under a `hw:` profile (RFC 0001 dish
    /// 3). Its absence under `hw:` is a REFUSAL at the first command, never a silent no-op.
    adapter_cmd: Option<String>,
    /// `--require-signed-adapter`: refuse a hardware driver that carries no signature (D52).
    require_signed_adapter: bool,
    /// `--adapter-artifact <path>`: WHICH bytes carry the driver's provenance (D53). Without it
    /// the first token of `--adapter-cmd` is used, and that token is the INTERPRETER for a
    /// script-hosted driver (`powershell -File drive.ps1`) — so the flag exists to make
    /// `--require-signed-adapter` usable for the commonest driver shape instead of unusable.
    adapter_artifact: Option<String>,
    /// `--adapter-signer <hex>`: the ed25519 public key whose signature this run will accept on the
    /// driver (D53). Without it, a verifying signature proves only that SOMEBODY signed these
    /// bytes — and the `.sig` sits beside the driver, so anyone who can replace one can replace
    /// both. Naming the key is what turns the check into a control. Implies the signature is
    /// required.
    adapter_signer: Option<String>,
    /// Directory for the hash-chained record of the driver-provenance decision (C60).
    adapter_record: Option<String>,
    sim_step: Option<u64>,
    /// `--signoff <path>`: on a successful `sim` run, write the artifact's content hash as the
    /// approved-for-hardware record (invariant 48).
    signoff: Option<String>,
    /// `--approved <path>`: the sign-off record a `hw:` run is checked against. Its absence is a
    /// refusal, never a pass — DL1905.
    approved: Option<String>,
    /// `--deny-advisories` (Stage 10 phase 10k, spec §4): for `build`, turn every advisory match
    /// into an error (the CI gate) instead of a DL1903 warning — AND refuse to pass at all if the
    /// advisory feed cannot be read, so a gate never opens because it could not find its evidence.
    deny_advisories: bool,
    /// `--advisory-feed <path>`: the advisory feed to consult (default `delulu.advisories.json`
    /// next to the package). Synced out-of-band from the registry; read offline at build time.
    advisory_feed: Option<String>,
}

fn parse_opts(rest: &[String]) -> (Option<String>, Opts) {
    let mut file = None;
    let mut opts = Opts {
        json: false,
        grants: Vec::new(),
        grant_manifest: false,
        no_prompt: false,
        locked: false,
        accept_authority: Vec::new(),
        diff: None,
        trace_effects: false,
        trace_out: None,
        assert_trace: false,
        actors_threads: None,
        on_quiesce_report: false,
        trace_memory: false,
        on_actor_death_abort: false,
        debug_rcaps: false,
        seed: None,
        clock_ms: None,
        engine: None,
        target: None,
        out: None,
        foreign_max_ret: None,
        broker: None,
        epoch_ms: None,
        foreign_isolation: None,
        isolation: None,
        lease: None,
        sign: None,
        broker_profile: None,
        adapter_cmd: None,
        require_signed_adapter: false,
        adapter_artifact: None,
        adapter_signer: None,
        adapter_record: None,
        sim_step: None,
        signoff: None,
        approved: None,
        deny_advisories: false,
        advisory_feed: None,
    };
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => opts.json = true,
            "--grant-manifest" => opts.grant_manifest = true,
            "--no-prompt" => opts.no_prompt = true,
            "--locked" => opts.locked = true,
            "--accept-authority" => {
                if i + 1 < rest.len() {
                    opts.accept_authority.push(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--accept-authority=") => {
                opts.accept_authority.push(s["--accept-authority=".len()..].to_string())
            }
            "--diff" => {
                if i + 1 < rest.len() {
                    opts.diff = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--diff=") => opts.diff = Some(s["--diff=".len()..].to_string()),
            "--trace-effects" => opts.trace_effects = true,
            "--assert-trace" => opts.assert_trace = true,
            "--debug-rcaps" => opts.debug_rcaps = true,
            "--trace-out" => {
                if i + 1 < rest.len() {
                    opts.trace_out = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            "--seed" => {
                if i + 1 < rest.len() {
                    opts.seed = rest[i + 1].parse().ok();
                    i += 1;
                }
            }
            // Stage 10 (10f, spec §5.4): the sim-to-real workflow's three flags.
            "--broker-profile" => {
                if i + 1 < rest.len() {
                    opts.broker_profile = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            "--require-signed-adapter" => opts.require_signed_adapter = true,

            "--adapter-cmd" => {
                if i + 1 < rest.len() {
                    opts.adapter_cmd = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            "--adapter-artifact" => {
                if i + 1 < rest.len() {
                    opts.adapter_artifact = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            "--adapter-signer" => {
                if i + 1 < rest.len() {
                    opts.adapter_signer = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            "--adapter-record" => {
                if i + 1 < rest.len() {
                    opts.adapter_record = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            "--sim-step" => {
                if i + 1 < rest.len() {
                    opts.sim_step = rest[i + 1].parse().ok();
                    i += 1;
                }
            }
            "--signoff" => {
                if i + 1 < rest.len() {
                    opts.signoff = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            "--approved" => {
                if i + 1 < rest.len() {
                    opts.approved = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            // Stage 10 (10k, spec §4): the advisory-feed gate.
            "--deny-advisories" => opts.deny_advisories = true,
            "--advisory-feed" => {
                if i + 1 < rest.len() {
                    opts.advisory_feed = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--advisory-feed=") => {
                opts.advisory_feed = Some(s["--advisory-feed=".len()..].to_string())
            }
            "--clock" => {
                if i + 1 < rest.len() {
                    opts.clock_ms = rest[i + 1].strip_prefix("fixed:").and_then(|s| s.parse().ok());
                    i += 1;
                }
            }
            "--engine" => {
                if i + 1 < rest.len() {
                    opts.engine = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--engine=") => opts.engine = Some(s["--engine=".len()..].to_string()),
            // Stage 7 (spec §6.1): actor scheduler controls.
            "--actors-threads" => {
                if i + 1 < rest.len() {
                    opts.actors_threads = rest[i + 1].parse().ok();
                    i += 1;
                }
            }
            s if s.starts_with("--actors-threads=") => {
                opts.actors_threads = s["--actors-threads=".len()..].parse().ok();
            }
            "--on-quiesce" => {
                if i + 1 < rest.len() {
                    opts.on_quiesce_report = rest[i + 1] == "report";
                    i += 1;
                }
            }
            // Stage 10 (10c, B3): per-actor mailbox telemetry at quiescence — peaks, drops,
            // bounds. Heap bytes and collection counts arrive with the 10d collector.
            "--trace-memory" => opts.trace_memory = true,
            s if s.starts_with("--on-quiesce=") => {
                opts.on_quiesce_report = &s["--on-quiesce=".len()..] == "report";
            }
            "--on-actor-death" => {
                if i + 1 < rest.len() {
                    opts.on_actor_death_abort = rest[i + 1] == "abort";
                    i += 1;
                }
            }
            s if s.starts_with("--on-actor-death=") => {
                opts.on_actor_death_abort = &s["--on-actor-death=".len()..] == "abort";
            }
            "--target" => {
                if i + 1 < rest.len() {
                    opts.target = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--target=") => opts.target = Some(s["--target=".len()..].to_string()),
            "-o" | "--out" => {
                if i + 1 < rest.len() {
                    opts.out = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--out=") => opts.out = Some(s["--out=".len()..].to_string()),
            "--foreign-max-ret" => {
                if i + 1 < rest.len() {
                    opts.foreign_max_ret = rest[i + 1].parse().ok();
                    i += 1;
                }
            }
            s if s.starts_with("--foreign-max-ret=") => {
                opts.foreign_max_ret = s["--foreign-max-ret=".len()..].parse().ok();
            }
            "--broker" => {
                if i + 1 < rest.len() {
                    opts.broker = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--broker=") => opts.broker = Some(s["--broker=".len()..].to_string()),
            "--epoch-ms" => {
                if i + 1 < rest.len() {
                    opts.epoch_ms = rest[i + 1].parse().ok();
                    i += 1;
                }
            }
            s if s.starts_with("--epoch-ms=") => opts.epoch_ms = s["--epoch-ms=".len()..].parse().ok(),
            "--foreign-isolation" => {
                if i + 1 < rest.len() {
                    opts.foreign_isolation = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--foreign-isolation=") => {
                opts.foreign_isolation = Some(s["--foreign-isolation=".len()..].to_string())
            }
            "--isolation" => {
                if i + 1 < rest.len() {
                    opts.isolation = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--isolation=") => {
                opts.isolation = Some(s["--isolation=".len()..].to_string())
            }
            "--lease" => {
                if i + 1 < rest.len() {
                    opts.lease = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--lease=") => opts.lease = Some(s["--lease=".len()..].to_string()),
            "--sign" => {
                if i + 1 < rest.len() {
                    opts.sign = Some(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--sign=") => opts.sign = Some(s["--sign=".len()..].to_string()),
            "--grant" => {
                if i + 1 < rest.len() {
                    opts.grants.push(rest[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--grant=") => opts.grants.push(s["--grant=".len()..].to_string()),
            s if !s.starts_with('-') && file.is_none() => file = Some(s.to_string()),
            _ => {}
        }
        i += 1;
    }
    (file, opts)
}

/// The CLI entry point, and the one place that guarantees the `--json` contract.
///
/// `docs/for-agents.md` promises: *"Every `--json` command emits one object."* It did not. A usage or
/// I/O error — a missing subcommand argument, an unreadable path, a malformed flag — printed a human
/// sentence to stderr and exited 2 with **nothing at all on stdout**, across essentially every
/// subcommand (`HARDENING_CAMPAIGN.md` C2). Any programmatic caller, whether that is a shell script,
/// a CI job, or an agent, then has an exit code and no parseable answer.
///
/// The fix lives here rather than at the ~161 individual `return 2` sites for the reason that keeps
/// coming up in this campaign: a rule enforced at every site is a rule the next site will forget.
/// This wrapper cannot be bypassed by adding a subcommand.
///
/// Fires on any nonzero exit, gated on [`JSON_EMITTED`] so a path that already printed an envelope is
/// never double-reported. Exit 1 needs it as much as exit 2 does: `explain` on an unknown code and
/// `keygen` on a bad key path both failed with a bare stderr line. The invariant this enforces is
/// "exactly one object", not "at least one" — the sweep in
/// `crates/delulu/tests/json_contract.rs` parses stdout and counts, so a double-emit fails the build.
/// How many effect records `--trace-effects` retains before it stops recording and says so.
///
/// A diagnostic trace must not be able to exhaust memory on a long run: the buffer used to keep every
/// record until exit, so 100k effects took peak working set from 6.7 MB to 70.1 MB with no ceiling
/// (campaign C56, measured in `measurements/scale/RECORD.md`). 200,000 records is far past any run a
/// person reads by eye and still bounds the buffer at roughly 130 MB in the worst case.
///
/// `--assert-trace` is deliberately NOT capped — see `TraceSink`'s docs.
const TRACE_RECORD_CAP: usize = 200_000;

pub fn run(args: &[String]) -> i32 {
    let wants_json = args.iter().any(|a| a == "--json");
    let code = run_inner(args);
    if wants_json && code != 0 && !JSON_EMITTED.load(std::sync::atomic::Ordering::Relaxed) {
        // No DL code is invented here. The registry is a stable contract and a usage error is not a
        // language diagnostic, so `diagnostics` stays empty and the failure is reported in an
        // additive `error` object. `summary.errors = 1` keeps the documented pass test
        // (`summary.errors == 0`) correct for a caller that reads nothing else.
        let command = args.iter().find(|a| !a.starts_with('-')).cloned().unwrap_or_default();
        let envelope = json!({
            "command": command,
            "schema": 1,
            "delulu_version": env!("CARGO_PKG_VERSION"),
            "diagnostics": [],
            "summary": { "errors": 1, "warnings": 0 },
            "error": {
                "kind": "usage",
                "exit": 2,
                "message": "the command could not run; the human-readable reason is on stderr"
            }
        });
        note_json_emitted();
        println!("{}", serde_json::to_string_pretty(&envelope).expect("the fallback envelope serializes"));
    }
    code
}

fn run_inner(args: &[String]) -> i32 {
    // Resolve the global color/theme surface once, and strip `--color`/`--theme` so the per-command
    // parsers never mistake their values for a positional. A malformed theme is a DL1790 warning
    // (never a hard failure — addendum §2.5).
    let (args, surface_warnings) = init_surface(args);
    enable_vt();
    if !surface_warnings.is_empty() {
        let map = SourceMap::new();
        let palette = palette_stderr();
        for w in &surface_warnings {
            eprint!("{}", render_human_with(w, &map, &palette));
        }
    }
    // The locale surface (Stage 8, spec §6.2): strips `--locale`, runs the first-run
    // picker + welcome when — and only when — every machine channel is absent
    // (invariant 40), and pins the active catalog for human rendering.
    let (args, locale_warnings) = crate::locale::init_locale(&args);
    for w in &locale_warnings {
        eprintln!("{w}");
    }
    let args = &args[..];
    let Some(cmd) = args.first() else {
        eprintln!("{}", usage());
        return 2;
    };
    let rest = &args[1..];

    // `--help` is answered by the dispatch, before any subcommand runs (Stage 9c, build-order
    // D11). Subcommands treat unrecognized flags as arguments, so `delulu keygen --help` used to
    // GENERATE A KEY, `delulu test --help` ran the suite, and `delulu repl --help` opened the
    // REPL. Asking a tool what it does must never make it do the thing. Answered here rather than
    // in each subcommand so no future subcommand can forget.
    if rest.iter().any(|a| a == "--help" || a == "-h") {
        println!("{}", subcommand_help(cmd));
        return 0;
    }

    match cmd.as_str() {
        "new" => crate::new::cmd_new(rest),
        "check" => cmd_check(rest),
        "fix" => crate::fix::cmd_fix(rest),
        "fmt" => cmd_fmt(rest),
        "test" => cmd_test(rest),
        // Machine-only by construction: the first-run flow is suppressed for `lsp` in
        // locale.rs (a stray prompt on stdout would corrupt the JSON-RPC stream).
        "lsp" => crate::lsp::run_lsp(rest),
        "keygen" => crate::signing::cmd_keygen(rest),
        "sign" => crate::signing::cmd_sign(rest),
        "verify-sig" => crate::signing::cmd_verify_sig(rest),
        "publish" => crate::signing::cmd_publish(rest, package_authority_value),
        "deploy" => crate::deploy::cmd_deploy(rest, package_authority_value),
        "add" => crate::signing::cmd_add(rest),
        "login" => crate::signing::cmd_login(rest),
        "build" => cmd_build(rest),
        "lock" => cmd_lock(rest),
        "run" => cmd_run(rest),
        "plugin" => cmd_plugin(rest),
        "authority" => cmd_authority(rest),
        "why" => cmd_why(rest),
        "atlas" => cmd_atlas(rest),
        "repl" => repl_cmd(rest),
        "audit" => cmd_audit(rest),
        "grants" => cmd_grants(rest),
        "guard" => cmd_guard(rest),
        "broker" => crate::brokerd::cmd_broker(rest),
        // Hidden: the process-isolation foreign worker (spec §5 phase 5h), spawned by the host, not a
        // user-facing command. Loads one granted C library and serves marshalled calls over its pipe.
        s if s == crate::foreign_worker::WORKER_SUBCOMMAND => crate::foreign_worker::run_worker(rest),
        "fleet" => crate::fleet::cmd_fleet(rest),
        "secrets" => cmd_secrets(rest),
        "locale" => cmd_locale(rest),
        "morph" => cmd_morph(rest),
        "explain" => cmd_explain(rest),
        "doctor" => crate::doctor::cmd_doctor(rest),
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            0
        }
        "--version" | "-V" => {
            println!("delulu {}", env!("CARGO_PKG_VERSION"));
            0
        }
        other => {
            eprintln!("unknown command `{other}`\n\n{}", usage());
            2
        }
    }
}

/// Help for one subcommand: the usage lines that mention it, or the full usage when the name is
/// unknown. Derived from `usage()` rather than duplicated, so a subcommand's help cannot drift
/// from its documented invocation.
fn subcommand_help(cmd: &str) -> String {
    let needle = format!("delulu {cmd} ");
    let mut lines: Vec<&str> = Vec::new();
    let mut capturing = false;
    for line in usage().lines() {
        if line.contains(&needle) {
            capturing = true;
            lines.push(line);
        } else if capturing && line.trim_start().starts_with('[') {
            // A continuation line of the same invocation (wrapped option list).
            lines.push(line);
        } else {
            capturing = false;
        }
    }
    if lines.is_empty() {
        return usage().to_string();
    }
    format!("delulu {cmd}\n\nUSAGE:\n{}\n", lines.join("\n"))
}

fn usage() -> &'static str {
    "delulu — the DeluluLang compiler and runtime\n\
     \n\
     USAGE:\n\
     \x20 delulu new       <name> [--lib] [--json]   (a package that already checks, tests and runs;\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 its declared ceiling is exactly what its code does — one effect for a bin, none for a lib)\n\
     \x20 delulu check     <file.delulu | package-dir> [--json]\n\
     \x20 delulu build     <package-dir> [--locked] [--json]   (resolve deps + verify pins/authority)\n\
     \x20 delulu build     <file.delulu> --target wasm [-o out.dwx]  (emit an authority-carrying .dwx)\n\
     \x20 delulu lock      [package-dir] [--accept-authority <pkg>]... [--json]\n\
     \x20 delulu run       <file.delulu | package-dir | file.dwx> [--json] [--grant K[=V]]... [--grant-manifest] [--no-prompt]\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 (a package-dir runs a MULTI-PACKAGE program: the graph is checked, then flattened\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 for execution; two modules declaring the same top-level name are refused, not guessed — D61)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--trace-effects] [--trace-out F] [--assert-trace] [--seed N] [--clock fixed:MS]\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--engine wasm]  (run `main` on the WebAssembly backend instead of the interpreter)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--broker embedded|daemon] [--epoch-ms N]  (custody: daemon routes ops through the broker)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--lease TOKEN]  (run under a delegated lease — the authority is the delegated node's)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--isolation none|process|microvm]  (microvm is Linux+KVM; elsewhere DL1408, see spec §6.1)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--actors-threads N] [--on-quiesce report] [--on-actor-death abort] [--debug-rcaps]  (Stage 7 actors)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--broker-profile sim|hw:ADAPTER] [--sim-step MS] [--signoff F] [--approved F]  (devices: sim is deterministic under --seed;\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 hw needs the sign-off record of the artifact simulation approved — DL1905;\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 --sim-step MS makes sim lease timing deterministic per interaction, independent of build speed — D20;\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 --adapter-cmd CMD names the hw driver: a signature beside it that does NOT verify always refuses,\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 and --require-signed-adapter refuses an unsigned one — D52;\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 --adapter-artifact PATH names WHICH bytes were signed (an interpreter-hosted driver\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 names the interpreter in --adapter-cmd, not the driver), and --adapter-signer HEX pins\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 the key: unpinned, a `.sig` beside the driver proves only that SOMEBODY signed it — D53)\n\
     \x20 delulu authority <file.delulu | package-dir> [--json]\n\
     \x20 delulu authority --diff <old.lock> <new.lock-or-package-dir> [--json]\n\
     \x20 delulu why       <Effect> <file.delulu | package-dir> [--json]\n\
     \x20 delulu atlas     <file.delulu | package-dir> [--format tree|digest|json|dot|mermaid|html]
\n                        (mermaid is a MODULE-LEVEL overview — no functions/effects; dot|json|tree show the whole graph)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--out DIR] [--budget N] [--gods N] [--custody] [--json]\n\
     \x20 delulu atlas     node <name-or-id> | callers <fn> | calls <fn> | why <Effect|resource> [target] [--json] [--budget N]\n\
     \x20 delulu atlas     path <A> <B> [target] [--json]   (a typed, deterministic code + authority graph)\n\
     \x20 delulu plugin    build <package-dir> [-o out.dpx] [--sign keyfile] [--json]   (a `kind = \"plugin\"` package → .dpx)\n\
     \x20 delulu plugin    inspect <file.dpx> [--json]   (manifest, class, exports, section hashes, signature)\n\
     \x20 delulu plugin    verify  <file.dpx> [--json]   (load steps 1,2,5 — identical verdicts to a real load)\n\
     \x20 delulu repl      [--grant K[=V]]...\n\
     \x20 delulu audit     tail [N] | query [--node g_ID] [--action A] [--effect E] | verify\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--dir DIR] [--json]  (default DIR: ~/.delulu/audit)\n\
     \x20 delulu broker    start [--foreground] [--dangerously-bypass-guard] [--guard-policy F] | status | stop | rotate-key\n\
     \x20 delulu grants    list | tree | inspect <g_ID> | revoke <g_ID>\n\
     \x20 delulu grants    delegate [--parent g_ID] --effects E,.. [--fs-read P].. [--fs-write P]..\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--net H].. [--secret N].. [--declassify N].. [--device DEV:dim=lo..hi,..].. [--ttl 1h]\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--multi] [--owner CODE]  (prints a lease token)\n\
     \x20 delulu grants    certify --subject HEX --effects E,.. [--device D].. --ttl 20m [--key F] [--parent-cert F] [--out F]\n\
     \x20 delulu grants    adopt <cert>.. [--anchor HEX].. | pubkey [--key F]   (federation: mint offline, adopt locally)\n\
     \x20 delulu guard     status | policy [show | set <class:pattern> <tier> | unset <class:pattern>] | bypass on|off  [--owner CODE]\n\
     \x20 delulu guard     request <g_ID> --use <class:pattern>.. --why \"..\" | pending | permits [revoke <id> --owner CODE]\n\
     \x20 delulu guard     approve <req-id> --owner CODE [--ttl D] [--uses N] [--comment \"..\"] | deny <req-id> --owner CODE --comment \"..\"\n\
     \x20 delulu secrets   set NAME VALUE | list [--state-dir DIR]  (broker-resident secrets)\n\
     \x20 delulu fix       <file.delulu> [--dry-run] [--json] [--accept-widening <repair-id>]...\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 (applies the checker's own typed repairs; a repair that would WIDEN what the program\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 may do is never applied unless you name it, and there is no accept-all flag)\n\
     \x20 delulu fmt       <file-or-dir>... [--check] [--json] | --stdin | --migrate 0.7 <file-or-dir>...\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 (one canonical style, zero options; --check exits 1 on unformatted; unparseable files are refused)\n\
     \x20 delulu test      [paths|patterns]... [--json] [--seed N]   (authority-isolated tests; each holds only its declared, ceiling-bounded row)\n\
     \x20 delulu lsp       (LSP 3.17 over stdio — one server for every editor and agent IDE; analysis only)\n\
     \x20 delulu locale    add <file.dpx> [--yes] | remove <name> | list   (catalog plugins: verified-class, ZERO authority, prose only)\n\
     \x20 delulu morph     list | info <id> | check <file.toml> | render <file> (--to <id> | --to-canonical)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 (surface keyword skins: human languages or AI-compact profiles; the program is unchanged)\n\
     \x20 delulu keygen    [--name N]                          (mint an ed25519 signing key in ~/.delulu/keys)\n\
     \x20 delulu sign      <artifact> [--hybrid] [--unstable] | verify-sig <artifact> [--key HEX] [--require-hybrid] [--unstable]\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 (detached .sig over .dwx/.dpx/tarballs; --hybrid/--require-hybrid touch post-quantum\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 ML-DSA-65 and need --unstable — unaudited, pre-KAT, refused as DL1910 without it)\n\
     \x20 delulu publish   --dry-run <pkg-dir> [--index DIR]   (validate manifest + semver-authority + signature; no upload)\n\
     \x20 delulu add       <pkg> --index DIR                   (resolve + show authority from the index line, no download)\n\
     \x20 delulu login     --registry URL --token VALUE        (store a scoped publish token; never echoed)\n\
     \x20 delulu deploy    plan --service NAME=PKG_DIR --env ENVFILE.toml   (check each service against the environment's EFFECT ceiling; DL1909 when it exceeds.\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 Effects only: capability scopes and foreign holes are NOT compared — the verdict says so too)\n\
     \x20 delulu fleet     <verb>                              (fleet-level device/lease operations — see `delulu fleet` for the verb list)\n\
     \x20 delulu explain   <DLxxxx | E-REVOKE | E-GUARD | E-ATLAS | E-PALETTE | E-PLUGIN | E-ACTOR>\n\
     \x20 delulu doctor    [--check] [--json]  (is this machine healthy? inside the source tree, is\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 the repository map current and sound? regenerates it when behind; --check never writes)\n\
     \x20 global:          [--color never|always|auto] [--theme default|bright|mono]  (envs DELULU_COLOR, DELULU_THEME, NO_COLOR)\n\
     \x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20\x20 [--locale en-US|delulu-slang]  (env DELULU_LOCALE; human prose only — codes & JSON never change)\n\
     \n\
     `delulu authority` prints the compiler-computed answer to \"what can this program do?\"\n\
     `delulu authority --diff` compares two lockfile states and reports authority widening.\n\
     `delulu why <Effect>` explains, at function granularity, why a program can perform an\n\
     effect — the shortest chain of calls from `main` down to the function that performs it.\n\
     `delulu audit` reads the broker's hash-chained audit log (observability, not enforcement);\n\
     `verify` recomputes the whole chain and reports DL1405 at the first broken record.\n\
     `delulu build` resolves path dependencies and verifies each dependency's authority against\n\
     its pin (DL1001); `delulu lock` writes delulu.lock and enforces the semver-authority law\n\
     (DL1003 — authority never widens silently across versions)."
}

/// `delulu fmt --migrate 0.7 <file-or-dir>… [--json]` — invariant 37 (Stage 7): `consume` and
/// `recover` became keywords in v0.7; this renames every pre-0.7 identifier use to `consume_` /
/// `recover_` (the exact repair DL1608 carries), corpus-wide and in place. Token-stream based:
/// strings and comments are untouched because they are not identifier tokens. v0.7 ships ONLY
/// this migration form — a general formatter is out of scope (build-order deviation 6).
/// Expand directories to their `.delulu` files, recursively, deterministically.
fn collect_delulu_files(p: &std::path::Path, out: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    if p.is_dir() {
        let mut entries: Vec<_> =
            std::fs::read_dir(p)?.collect::<Result<Vec<_>, _>>()?.into_iter().map(|e| e.path()).collect();
        entries.sort();
        for e in entries {
            collect_delulu_files(&e, out)?;
        }
    } else if p.extension().and_then(|e| e.to_str()) == Some("delulu") {
        out.push(p.to_path_buf());
    }
    Ok(())
}

/// The two formatter laws, verified inline on EVERY run before any byte is written
/// (spec §4): identity (fingerprint + comment sequence) and idempotence. A violation is
/// DL1702 — compiler-bug class — and the file is NOT touched: fmt never corrupts code.
fn fmt_laws_ok(src: &str, out: &str) -> Result<(), &'static str> {
    use delulu_syntax::fmt::{ast_fingerprint, comment_sequence, format_source};
    let (m1, _) = delulu_syntax::parse_file(0, src);
    let (m2, d2) = delulu_syntax::parse_file(0, out);
    if d2.iter().any(|d| d.is_error()) {
        return Err("the formatted output no longer parses");
    }
    if ast_fingerprint(&m1) != ast_fingerprint(&m2) {
        return Err("the formatted output changed the program (identity law)");
    }
    if comment_sequence(src) != comment_sequence(out) {
        return Err("the formatted output moved or lost a comment");
    }
    match format_source(0, out) {
        Ok(again) if again == out => Ok(()),
        _ => Err("formatting is not idempotent on this file"),
    }
}

fn cmd_fmt(args: &[String]) -> i32 {
    let mut migrate: Option<String> = None;
    let mut json = false;
    let mut check = false;
    let mut stdin = false;
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--migrate" => {
                i += 1;
                migrate = args.get(i).cloned();
            }
            "--json" => json = true,
            "--check" => check = true,
            "--stdin" => stdin = true,
            other => paths.push(std::path::PathBuf::from(other)),
        }
        i += 1;
    }

    match migrate.as_deref() {
        Some("0.7") => {
            if paths.is_empty() {
                eprintln!("delulu fmt --migrate 0.7 needs at least one file or directory");
                return 2;
            }
            let mut files = Vec::new();
            for p in &paths {
                if !p.exists() {
                    eprintln!("no such file or directory: {}", p.display());
                    return 2;
                }
                if let Err(e) = collect_delulu_files(p, &mut files) {
                    eprintln!("cannot read {}: {e}", p.display());
                    return 2;
                }
            }
            return cmd_fmt_migrate(files, json);
        }
        Some(v) => {
            eprintln!("unknown migration `{v}` — the only migration is `0.7` (consume/recover keywords)");
            return 2;
        }
        None => {}
    }

    // `--stdin`: editor/agent integration — format stdin to stdout, diagnostics to stderr.
    if stdin {
        use std::io::Read as _;
        let mut src = String::new();
        if std::io::stdin().read_to_string(&mut src).is_err() {
            eprintln!("error: stdin is not valid UTF-8");
            return 2;
        }
        return match delulu_syntax::fmt::format_source(0, &src) {
            Err(diags) => {
                let mut map = SourceMap::new();
                map.add_file("<stdin>", src.clone());
                print_diagnostics("fmt", &diags, &map, None, json);
                1
            }
            Ok(out) => {
                if let Err(why) = fmt_laws_ok(&src, &out) {
                    // DL1702 is compiler-bug class; per spec §10 it carries no repair.
                    let map = SourceMap::new();
                    let d = Diagnostic::error("DL1702", format!("formatter law violation: {why}"));
                    print_diagnostics("fmt", &[d], &map, None, json);
                    return 2;
                }
                print!("{out}");
                0
            }
        };
    }

    if paths.is_empty() {
        eprintln!("delulu fmt needs files/directories, `--stdin`, or `--migrate 0.7` (see usage)");
        return 2;
    }
    let mut files = Vec::new();
    for p in &paths {
        if !p.exists() {
            eprintln!("no such file or directory: {}", p.display());
            return 2;
        }
        // A FILE named explicitly is a request about that file. `collect_delulu_files` keeps only
        // `*.delulu`, so `fmt notes.txt` and `fmt app.dwx` used to walk away with "reformatted 0
        // file(s)" and exit 0 — nothing done, success reported, on an argument the user chose
        // deliberately (campaign finding C66, the C26 class). A DIRECTORY is different: filtering is
        // the whole point of walking one, and a tree with no `.delulu` in it is not a mistake.
        if p.is_file() && p.extension().and_then(|e| e.to_str()) != Some("delulu") {
            eprintln!(
                "error: `{}` is not a `.delulu` source file, so `fmt` has nothing to do with it",
                p.display()
            );
            if looks_like_dwx(&p.display().to_string()) {
                eprintln!("note: that is a compiled `.dwx` artifact — format the source it was built from");
            }
            return 2;
        }
        if let Err(e) = collect_delulu_files(p, &mut files) {
            eprintln!("cannot read {}: {e}", p.display());
            return 2;
        }
    }

    let mut changed: Vec<String> = Vec::new();
    let mut refused: Vec<String> = Vec::new();
    let mut unchanged = 0usize;
    for f in &files {
        let src = match std::fs::read_to_string(f) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("cannot read {}: {e}", f.display());
                return 2;
            }
        };
        match delulu_syntax::fmt::format_source(0, &src) {
            Err(diags) => {
                // Unparseable input is never "formatted" — surface the real diagnostics.
                let mut map = SourceMap::new();
                map.add_file(&f.display().to_string(), src.clone());
                print_diagnostics("fmt", &diags, &map, None, false);
                refused.push(f.display().to_string());
            }
            Ok(out) => {
                if let Err(why) = fmt_laws_ok(&src, &out) {
                    let map = SourceMap::new();
                    let d = Diagnostic::error(
                        "DL1702",
                        format!("formatter law violation on {}: {why}", f.display()),
                    );
                    print_diagnostics("fmt", &[d], &map, None, json);
                    return 2; // compiler-bug class: stop, write nothing further
                }
                if out == src {
                    unchanged += 1;
                } else if check {
                    changed.push(f.display().to_string());
                } else {
                    if let Err(e) = std::fs::write(f, &out) {
                        eprintln!("cannot write {}: {e}", f.display());
                        return 2;
                    }
                    changed.push(f.display().to_string());
                }
            }
        }
    }

    if json {
        let report = serde_json::json!({
            "delulu_version": env!("CARGO_PKG_VERSION"),
            "schema": 1,
            "command": "fmt",
            "mode": if check { "check" } else { "write" },
            "changed": changed,
            "unchanged": unchanged,
            "refused": refused,
        });
        note_json_emitted();
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else if check {
        for f in &changed {
            println!("would reformat: {f}");
        }
        println!(
            "fmt --check: {} would change, {} clean, {} refused (parse errors)",
            changed.len(),
            unchanged,
            refused.len()
        );
    } else {
        println!(
            "fmt: reformatted {} file(s), {} already canonical, {} refused (parse errors)",
            changed.len(),
            unchanged,
            refused.len()
        );
    }
    if (check && !changed.is_empty()) || !refused.is_empty() {
        1
    } else {
        0
    }
}

/// `delulu fmt --migrate 0.7`: the Stage-7 keyword migration (invariant 37) — lexical
/// renames of pre-0.7 `consume`/`recover` identifiers, splice-based, never a reformat.
fn cmd_fmt_migrate(files: Vec<std::path::PathBuf>, json: bool) -> i32 {
    let mut total_renames = 0usize;
    let mut changed: Vec<String> = Vec::new();
    for f in &files {
        let src = match std::fs::read_to_string(f) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("cannot read {}: {e}", f.display());
                return 2;
            }
        };
        let (tokens, _lex_diags) = delulu_syntax::lexer::lex(0, &src);
        let mut spans: Vec<(u32, u32, &'static str)> = tokens
            .iter()
            .filter_map(|t| match t.kind {
                delulu_syntax::token::TokenKind::KwConsume => Some((t.span.start, t.span.end, "consume_")),
                delulu_syntax::token::TokenKind::KwRecover => Some((t.span.start, t.span.end, "recover_")),
                _ => None,
            })
            .collect();
        if spans.is_empty() {
            continue;
        }
        // Splice back-to-front so earlier byte offsets stay valid.
        spans.sort_by_key(|s| std::cmp::Reverse(s.0));
        let mut out = src.clone();
        for (start, end, insert) in &spans {
            out.replace_range(*start as usize..*end as usize, insert);
        }
        if let Err(e) = std::fs::write(f, &out) {
            eprintln!("cannot write {}: {e}", f.display());
            return 2;
        }
        total_renames += spans.len();
        changed.push(f.display().to_string());
    }

    if json {
        let report = serde_json::json!({
            "migration": "0.7",
            "files_scanned": files.len(),
            "files_changed": changed,
            "renames": total_renames,
        });
        note_json_emitted();
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else if total_renames == 0 {
        println!("migrate 0.7: nothing to do ({} file(s) scanned)", files.len());
    } else {
        println!(
            "migrate 0.7: renamed {} identifier(s) in {} file(s) (of {} scanned)",
            total_renames,
            changed.len(),
            files.len()
        );
    }
    0
}

/// The `[test-authority]` package ceiling (spec §5.1): effects + fs.read scopes every
/// test's declared row must fit inside (DL1703 otherwise). Absent table = PURE — the
/// couldn't-tell default grants nothing (invariant 41).
struct TestCeiling {
    effects: Vec<String>,
    fs_read: Vec<String>,
}

fn test_ceiling(dir: &std::path::Path) -> TestCeiling {
    let mut ceiling = TestCeiling { effects: Vec::new(), fs_read: Vec::new() };
    let Ok(text) = std::fs::read_to_string(dir.join("delulu.toml")) else { return ceiling };
    let mut in_section = false;
    for raw in text.lines() {
        let line = raw.trim();
        if let Some(name) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_section = name.trim() == "test-authority";
            continue;
        }
        if !in_section {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else { continue };
        let items: Vec<String> = v
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .filter_map(|s| {
                let s = s.trim().trim_matches('"').to_string();
                (!s.is_empty()).then_some(s)
            })
            .collect();
        match k.trim() {
            "effects" => ceiling.effects = items,
            "fs.read" => ceiling.fs_read = items,
            _ => {}
        }
    }
    ceiling
}

/// `delulu test [paths|patterns] [--json] [--seed N]` — the authority-isolated test
/// runner (Stage 8, phase 8g; spec §5). Deterministic by default (fixed clock, per-name
/// rand seed), `--trace-effects` semantics ALWAYS on: each test's actual effect list is
/// part of its report even when it passes. Under a reachable broker daemon, the run
/// executes inside a `test-session` broker node with per-file children, transitively
/// revoked at session end (build-order deviation 15 for the embedded fallback).
fn cmd_test(rest: &[String]) -> i32 {
    let mut json = false;
    let mut seed_flag: Option<u64> = None;
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    let mut patterns: Vec<String> = Vec::new();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--json" => json = true,
            "--seed" => {
                i += 1;
                seed_flag = rest.get(i).and_then(|v| v.parse().ok());
            }
            other if !other.starts_with("--") => {
                let p = std::path::PathBuf::from(other);
                if p.exists() {
                    paths.push(p);
                } else {
                    patterns.push(other.to_string());
                }
            }
            _ => {}
        }
        i += 1;
    }
    if paths.is_empty() {
        let default = std::path::PathBuf::from("tests");
        if default.is_dir() {
            paths.push(default);
        } else {
            eprintln!("error: `delulu test` needs test files/directories (no ./tests directory here)");
            return 2;
        }
    }
    let mut files = Vec::new();
    for p in &paths {
        if let Err(e) = collect_delulu_files(p, &mut files) {
            eprintln!("cannot read {}: {e}", p.display());
            return 2;
        }
    }
    let ceiling = test_ceiling(std::path::Path::new("."));

    // The broker session lane: reachable daemon ⇒ the whole run lives under a freshly
    // ISSUED `test-session` principal node (the broker tree starts empty; a test run is
    // its own principal); unreachable ⇒ embedded grants (the Stage-5 default posture).
    let state_dir = crate::brokerd::resolve_state_dir(None);
    let session = state_dir.as_ref().and_then(|sd| {
        let spec = crate::broker_ipc::AuthoritySpec {
            effects: ceiling.effects.clone(),
            fs_read: ceiling.fs_read.clone(),
            holder_kind: "process".into(),
            holder_desc: "test-session".into(),
            ..Default::default()
        };
        match crate::brokerd::request(sd, crate::broker_ipc::ReqBody::Issue(spec)) {
            Ok(crate::broker_ipc::Response::Issued { node }) => Some(node),
            _ => None,
        }
    });

    let seed_base = seed_flag.unwrap_or(0xDE1);
    let mut reports: Vec<Json> = Vec::new();
    let mut failed = 0usize;
    let mut passed = 0usize;
    let t_all = std::time::Instant::now();

    for f in &files {
        let (map, id, src) = match load(&f.display().to_string()) {
            Ok(x) => x,
            Err(c) => return c,
        };
        let checked = check_source(id, &src);
        if errors(&checked.diagnostics) > 0 {
            print_diagnostics("test", &checked.diagnostics, &map, None, false);
            reports.push(json!({
                "file": f.display().to_string(), "status": "check-failed",
            }));
            failed += 1;
            continue;
        }
        // A per-file broker child carrying exactly this file's needs (session lane only).
        let file_node = session.as_ref().zip(state_dir.as_ref()).and_then(|(sess, sd)| {
            let spec = crate::broker_ipc::AuthoritySpec {
                effects: ceiling.effects.clone(),
                fs_read: ceiling.fs_read.clone(),
                holder_kind: "process".into(),
                holder_desc: format!("test-file {}", f.display()),
                ..Default::default()
            };
            match crate::brokerd::request(
                sd,
                crate::broker_ipc::ReqBody::Attenuate {
                    parent: sess.clone(),
                    authority: spec,
                    owner: None,
                },
            ) {
                Ok(crate::broker_ipc::Response::Issued { node }) => Some(node),
                _ => None,
            }
        });
        let _ = &file_node;

        let interp_module = &checked.module;
        for item in &interp_module.items {
            let delulu_syntax::ast::Item::Test(t) = item else { continue };
            if !patterns.is_empty() && !patterns.iter().any(|p| t.name.contains(p.as_str())) {
                continue;
            }
            let declared: Vec<String> = t
                .row
                .iter()
                .flat_map(|r| r.effects.iter())
                .filter_map(|p| p.segs.last().map(|s| s.name.clone()))
                .collect();

            // DL1703: the test's declared row must fit the package ceiling.
            if let Some(excess) =
                declared.iter().find(|e| !ceiling.effects.iter().any(|c| c == *e))
            {
                let d = Diagnostic::error(
                    "DL1703",
                    format!(
                        "test \"{}\" declares effect `{excess}` but the package [test-authority] ceiling allows only {:?}",
                        t.name, ceiling.effects
                    ),
                )
                .with_span(delulu_diag::Span::new(id, t.name_span.start, t.name_span.end), "narrow the row, or widen delulu.toml's [test-authority] — a real review decision");
                print_diagnostics("test", &[d], &map, None, false);
                reports.push(json!({
                    "name": t.name, "file": f.display().to_string(),
                    "status": "fail", "failure": { "message": format!("DL1703: effect `{excess}` exceeds the test ceiling") },
                    "effects_traced": [],
                }));
                failed += 1;
                continue;
            }
            // Actor tests are post-v0.8 (build-order §5): refuse clearly, never half-run.
            if declared.iter().any(|e| e == "Async") {
                reports.push(json!({
                    "name": t.name, "file": f.display().to_string(), "status": "fail",
                    "failure": { "message": "actor (Async) tests are not supported by the v0.8 runner — build-order §5" },
                    "effects_traced": [],
                }));
                failed += 1;
                continue;
            }

            // Deterministic by default: fixed clock; rand seeded by (--seed ⊕ name hash).
            let name_hash: u64 =
                t.name.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3));
            delulu_runtime::set_fixed_clock_ms(Some(0));
            delulu_runtime::set_rand_seed(seed_base ^ name_hash);

            // Grants derive from the DECLARED row bounded by the ceiling — a pure test
            // holds nothing at all (invariant 41).
            let mut g = Grants {
                console: declared.iter().any(|e| e == "Write"),
                clock: declared.iter().any(|e| e == "Clock"),
                rand: declared.iter().any(|e| e == "Rand"),
                ..Grants::default()
            };
            if declared.iter().any(|e| e == "Read") {
                g.fs_read.extend(ceiling.fs_read.iter().cloned());
            }

            let sink = delulu_runtime::TraceSink::new();
            let interp = Interp::new(interp_module).with_trace(sink.clone());
            delulu_runtime::set_capture(true);
            let t0 = std::time::Instant::now();
            let outcome = interp.run_test(&t.name, Value::Root(std::rc::Rc::new(g.build_root())));
            let ms = t0.elapsed().as_millis() as u64;
            let output = delulu_runtime::take_capture().unwrap_or_default();

            let mut traced: Vec<String> =
                sink.records().iter().map(|r| r.effect.clone()).collect();
            traced.sort();
            traced.dedup();

            let (start_l, start_c) = map.position(id, t.name_span.start);
            let mut entry = json!({
                "name": t.name, "file": f.display().to_string(),
                "span": { "line": start_l, "col": start_c },
                "ms": ms, "effects_traced": traced,
            });
            match outcome {
                Ok(_) => {
                    entry["status"] = json!("pass");
                    passed += 1;
                    if !json {
                        ok_line!("ok: test \"{}\" ({} ms)", t.name, ms);
                    }
                }
                Err(fault) => {
                    let status = if fault.code == "DL1707" { "fail" } else { "panic" };
                    entry["status"] = json!(status);
                    // deviation 5: assert_eq's symmetric `a` != `b` maps to actual/expected.
                    let mut failure = json!({ "code": fault.code, "message": fault.message });
                    let ticks: Vec<&str> = fault
                        .message
                        .split('`')
                        .skip(1)
                        .step_by(2)
                        .collect();
                    if fault.code == "DL1707" && ticks.len() == 2 {
                        failure["actual"] = json!(ticks[0]);
                        failure["expected"] = json!(ticks[1]);
                    }
                    entry["failure"] = failure;
                    failed += 1;
                    if !json {
                        eprintln!("FAIL: test \"{}\" — {} ({} ms)", t.name, fault.message, ms);
                        if !output.is_empty() {
                            eprintln!("  captured output:\n{output}");
                        }
                    }
                }
            }
            reports.push(entry);
        }
    }

    // Session end: transitive revocation — nothing a test leaked survives the run.
    let custody = match (&session, &state_dir) {
        (Some(sess), Some(sd)) => {
            let revoked = matches!(
                crate::brokerd::request(
                    sd,
                    crate::broker_ipc::ReqBody::Revoke { caller: sess.clone(), target: sess.clone() },
                ),
                Ok(crate::broker_ipc::Response::Revoked { .. })
            );
            json!({ "mode": "daemon", "session": sess, "revoked_at_end": revoked })
        }
        _ => json!({ "mode": "embedded" }),
    };

    if json {
        note_json_emitted();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "delulu_version": env!("CARGO_PKG_VERSION"), "schema": 1, "command": "test",
                "tests": reports,
                "summary": { "passed": passed, "failed": failed, "ms": t_all.elapsed().as_millis() as u64 },
                "custody": custody,
            }))
            .unwrap()
        );
    } else {
        println!("test result: {} passed, {} failed", passed, failed);
    }
    if failed > 0 {
        1
    } else {
        0
    }
}

/// `delulu locale add <file.dpx> | remove <name> | list` (Stage 8, phase 8f — spec §6.1).
/// A catalog plugin is the plugin machinery's first zero-authority dogfood: verified
/// class, EMPTY ceiling, one export `catalog() -> Str`. Catalogs are prose — the add
/// confirmation states the bound: they can never alter codes, repairs, JSON, exit codes.
fn cmd_locale(rest: &[String]) -> i32 {
    let map = SourceMap::new();
    let refuse = |msg: String, json: bool| -> i32 {
        let d = Diagnostic::error("DL1704", msg);
        print_diagnostics("locale", &[d], &map, None, json);
        1
    };
    let json = rest.iter().any(|a| a == "--json");
    let yes = rest.iter().any(|a| a == "--yes");
    match rest.first().map(String::as_str) {
        Some("add") => {
            let Some(file) = rest.get(1).filter(|a| !a.starts_with("--")) else {
                eprintln!("error: `locale add` needs a `.dpx` catalog plugin");
                return 2;
            };
            let bytes = match std::fs::read(file) {
                Ok(b) => b,
                Err(e) => {
                    eprintln!("error: cannot read `{file}`: {e}");
                    return 2;
                }
            };
            use delulu_runtime::PluginEngine as _;
            let art = match delulu_wasm::WasmPluginEngine::new().read_artifact(&bytes) {
                Ok(a) => a,
                Err(e) => {
                    let d = Diagnostic::error(e.code, e.message);
                    print_diagnostics("locale", &[d], &map, None, json);
                    return 1;
                }
            };
            // Verified class, by rule — a Contained catalog has no typed export to trust.
            if art.class != "verified" {
                return refuse(
                    format!(
                        "catalog plugins are `verified`-class by rule — `{}` declares `{}`",
                        art.name(),
                        art.class
                    ),
                    json,
                );
            }
            // ZERO authority, by rule — the ceiling bounds the plugin to prose forever.
            let ceiling = art.ceiling();
            if !ceiling.effects.is_empty() {
                let names: Vec<String> =
                    ceiling.effects.iter().map(|e| format!("{e:?}")).collect();
                return refuse(
                    format!(
                        "a catalog plugin must declare ZERO authority — `{}` declares effects [{}]",
                        art.name(),
                        names.join(", ")
                    ),
                    json,
                );
            }
            // The full Verified replay: the checker itself re-proves the code.
            let verified = match delulu_runtime::step5_verified(&art) {
                Ok(v) => v,
                Err(e) => {
                    let d = Diagnostic::error(e.code, e.message.clone());
                    print_diagnostics("locale", &[d], &map, None, json);
                    return 1;
                }
            };
            match art.exports().get("catalog").map(String::as_str) {
                Some("fn() -> Str") => {}
                Some(other) => {
                    return refuse(
                        format!("the `catalog` export must be `fn() -> Str`, found `{other}`"),
                        json,
                    )
                }
                None => {
                    return refuse(
                        format!("`{}` exports no `catalog() -> Str`", art.name()),
                        json,
                    )
                }
            }
            // Evaluate the export: pure by type, zero-authority by ceiling — the module
            // holds no capability and can perform no effect while producing its TOML.
            let interp = Interp::new(&verified.dir.module);
            let toml = match interp.call_with("catalog", vec![]) {
                Ok(delulu_runtime::Value::Str(s)) => s.to_string(),
                Ok(other) => {
                    return refuse(format!("`catalog()` produced a non-string value: {other:?}"), json)
                }
                Err(e) => return refuse(format!("`catalog()` faulted: {}", e.message), json),
            };
            let (cat, cat_warnings) = delulu_diag::Catalog::parse(&toml);
            // Per-entry defects (unknown keys/placeholders, a welcome-override attempt)
            // warn-and-fall-back — shown HERE, at add time, where the human is looking.
            print_diagnostics("locale", &cat_warnings, &map, None, false);
            if cat.locale.is_empty() {
                return refuse("the catalog declares no `[meta] locale` name".to_string(), json);
            }
            if matches!(cat.locale.as_str(), "en-US" | "delulu-slang") {
                return refuse(
                    format!("locale `{}` is a built-in and cannot be replaced", cat.locale),
                    json,
                );
            }
            if cat.is_empty() {
                return refuse(
                    format!("the catalog for `{}` has no valid entries — nothing to install", cat.locale),
                    json,
                );
            }
            // The prose bound, stated where the spec says to state it.
            eprintln!(
                "note: a catalog changes HUMAN prose only — it can never alter codes, spans, \
                 repairs, JSON, or exit codes. A malicious catalog can mislead a reader; it \
                 cannot change what the compiler decides."
            );
            if crate::locale::interactive_tty() && !yes {
                use std::io::Write as _;
                eprint!("install locale `{}`? [y/N]: ", cat.locale);
                let _ = std::io::stderr().flush();
                let mut line = String::new();
                let _ = std::io::stdin().read_line(&mut line);
                if !line.trim().eq_ignore_ascii_case("y") {
                    eprintln!("not installed");
                    return 1;
                }
            }
            let Some(dir) = crate::locale::locales_dir() else {
                eprintln!("error: cannot resolve the locales directory (no HOME/USERPROFILE)");
                return 2;
            };
            if let Err(e) = std::fs::create_dir_all(&dir) {
                eprintln!("error: cannot create {}: {e}", dir.display());
                return 2;
            }
            let target = dir.join(format!("{}.toml", cat.locale));
            if let Err(e) = std::fs::write(&target, &toml) {
                eprintln!("error: cannot write {}: {e}", target.display());
                return 2;
            }
            if json {
                note_json_emitted();
                println!(
                    "{}",
                    json!({
                        "command": "locale", "subcommand": "add", "locale": cat.locale,
                        "entries": cat.len(), "coverage": cat.coverage,
                        "warnings": cat_warnings.len(),
                    })
                );
            } else {
                ok_line!(
                    "ok: installed locale `{}` ({} message(s){})",
                    cat.locale,
                    cat.len(),
                    cat.coverage.as_deref().map(|c| format!("; coverage: {c}")).unwrap_or_default()
                );
            }
            0
        }
        Some("remove") => {
            let Some(name) = rest.get(1).filter(|a| !a.starts_with("--")) else {
                eprintln!("error: `locale remove` needs a locale name");
                return 2;
            };
            if matches!(name.as_str(), "en-US" | "delulu-slang") {
                eprintln!("error: `{name}` is a built-in and cannot be removed");
                return 1;
            }
            let Some(dir) = crate::locale::locales_dir() else {
                eprintln!("error: cannot resolve the locales directory");
                return 2;
            };
            let target = dir.join(format!("{name}.toml"));
            if !target.exists() {
                eprintln!("error: locale `{name}` is not installed");
                return 1;
            }
            if let Err(e) = std::fs::remove_file(&target) {
                eprintln!("error: cannot remove {}: {e}", target.display());
                return 2;
            }
            ok_line!("ok: removed locale `{name}`");
            0
        }
        Some("list") | None => {
            let mut rows = vec![
                json!({ "locale": "en-US", "builtin": true, "coverage": "complete (the in-code prose)" }),
                json!({
                    "locale": "delulu-slang", "builtin": true,
                    "coverage": delulu_diag::Catalog::delulu_slang().coverage,
                }),
            ];
            if let Some(dir) = crate::locale::locales_dir() {
                let mut names: Vec<String> = std::fs::read_dir(&dir)
                    .map(|it| {
                        it.flatten()
                            .filter_map(|e| {
                                let p = e.path();
                                (p.extension().and_then(|x| x.to_str()) == Some("toml"))
                                    .then(|| p.file_stem().unwrap().to_string_lossy().to_string())
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                names.sort();
                for n in names {
                    if let Some(cat) = crate::locale::installed_catalog(&n) {
                        rows.push(json!({
                            "locale": cat.locale, "builtin": false,
                            "version": cat.version, "coverage": cat.coverage,
                            "entries": cat.len(),
                        }));
                    }
                }
            }
            if json {
                note_json_emitted();
                println!("{}", json!({ "command": "locale", "subcommand": "list", "locales": rows }));
            } else {
                for r in &rows {
                    let b = if r["builtin"].as_bool().unwrap_or(false) { " (built-in)" } else { "" };
                    println!(
                        "{}{b} — {}",
                        r["locale"].as_str().unwrap_or("?"),
                        r["coverage"].as_str().unwrap_or("(no coverage declared)")
                    );
                }
            }
            0
        }
        Some(other) => {
            eprintln!("error: unknown `locale` subcommand `{other}` (add | remove | list)");
            2
        }
    }
}

/// `delulu morph list | info <id> | check <file.toml> | render <file.delulu> --to <id>|--to-canonical`
///
/// Morphs are surfaces, not dialects (Stage 8 §6.5). `render` is the whole feature made visible: it
/// converts a file between two surfaces of the same program, and `--to-canonical` is how anything
/// morphed re-enters the part of the toolchain that only speaks canonical.
fn cmd_morph(rest: &[String]) -> i32 {
    let map = SourceMap::new();
    let json = rest.iter().any(|a| a == "--json");
    let refuse = |diags: Vec<Diagnostic>| -> i32 {
        print_diagnostics("morph", &diags, &map, None, json);
        1
    };
    match rest.first().map(String::as_str) {
        Some("list") => {
            let found = crate::morph_file::installed();
            if json {
                let arr: Vec<Json> = found
                    .iter()
                    .map(|(id, path)| {
                        serde_json::json!({ "morph": id, "path": path.display().to_string() })
                    })
                    .collect();
                note_json_emitted();
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({ "morphs": arr })).unwrap());
                return 0;
            }
            if found.is_empty() {
                println!("no morphs installed — canonical DeluluLang is the only surface");
                println!(
                    "  searched: {}",
                    crate::morph_file::search_dirs()
                        .iter()
                        .map(|d| d.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return 0;
            }
            println!("installed morphs:");
            for (id, path) in found {
                match crate::morph_file::load(&id) {
                    Ok(m) => println!("  {:<20} {:<8} {}  ({})", m.id, m.kind.name(), m.name, path.display()),
                    Err(e) => println!("  {id:<20} INVALID  {}", e.0.first().map(|d| d.message.clone()).unwrap_or_default()),
                }
            }
            0
        }
        Some("info") => {
            let Some(id) = rest.get(1).filter(|a| !a.starts_with("--")) else {
                eprintln!("error: `morph info` needs a morph id");
                return 2;
            };
            let m = match crate::morph_file::load(id) {
                Ok(m) => m,
                Err(e) => return refuse(e.0),
            };
            if json {
                let kw: Vec<Json> = m
                    .entries()
                    .map(|(c, a)| serde_json::json!({ "canonical": c, "alias": a }))
                    .collect();
                note_json_emitted();
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "morph": m.id, "name": m.name, "version": m.version,
                        "kind": m.kind.name(), "keywords": kw
                    }))
                    .unwrap()
                );
                return 0;
            }
            println!("morph {} ({}) — {}", m.id, m.kind.name(), m.name);
            println!("  version: {}", m.version);
            println!("  renames:");
            for (canon, alias) in m.entries() {
                println!("    {canon:<10} -> {alias}");
            }
            println!("  every other keyword keeps its canonical spelling; identifiers, strings, and");
            println!("  comments are never morphed, and every hash is computed on the canonical form");
            0
        }
        Some("check") => {
            let Some(file) = rest.get(1).filter(|a| !a.starts_with("--")) else {
                eprintln!("error: `morph check` needs a morph `.toml` file");
                return 2;
            };
            match crate::morph_file::load_path(std::path::Path::new(file)) {
                Ok(m) => {
                    ok_line!("ok: morph `{}` is valid ({} keyword(s) renamed)", m.id, m.entries().count());
                    0
                }
                Err(e) => refuse(e.0),
            }
        }
        Some("render") => {
            let Some(file) = rest.get(1).filter(|a| !a.starts_with("--")) else {
                eprintln!("error: `morph render` needs a `.delulu` file");
                return 2;
            };
            let to_canonical = rest.iter().any(|a| a == "--to-canonical");
            let to = rest.iter().position(|a| a == "--to").and_then(|i| rest.get(i + 1)).cloned();
            if to_canonical == to.is_some() {
                eprintln!("error: `morph render` needs exactly one of `--to <id>` or `--to-canonical`");
                return 2;
            }
            let Ok((smap, fid, src)) = load(file) else { return 2 };
            let out = if to_canonical {
                // The file's own pragma says which surface it is written in.
                let Some(id) = delulu_syntax::morph::pragma_of(&src) else {
                    eprintln!("error: `{file}` declares no `//! morph:` pragma — it is already canonical");
                    return 2;
                };
                let m = match crate::morph_file::load(id) {
                    Ok(m) => m,
                    Err(e) => return refuse(e.0),
                };
                match m.to_canonical(fid, &src) {
                    // Drop the pragma line: the output is canonical, and a stale pragma would make
                    // every later tool read it through a morph it is no longer written in.
                    Ok(text) => strip_morph_pragma(&text),
                    Err(diags) => return refuse(diags),
                }
            } else {
                let id = to.unwrap();
                let m = match crate::morph_file::load(&id) {
                    Ok(m) => m,
                    Err(e) => return refuse(e.0),
                };
                if delulu_syntax::morph::pragma_of(&src).is_some() {
                    eprintln!("error: `{file}` is already written in a morph — convert it to canonical first");
                    return 2;
                }
                match m.render(fid, &src) {
                    Ok(text) => format!("//! morph: {}\n{text}", m.id),
                    Err(diags) => return refuse(diags),
                }
            };
            let _ = smap;
            print!("{out}");
            0
        }
        _ => {
            eprintln!(
                "usage: delulu morph list | info <id> | check <file.toml> | render <file.delulu> (--to <id> | --to-canonical)"
            );
            2
        }
    }
}

/// Remove a leading `//! morph:` pragma line, for output that is canonical again.
fn strip_morph_pragma(src: &str) -> String {
    if delulu_syntax::morph::pragma_of(src).is_none() {
        return src.to_string();
    }
    match src.find('\n') {
        Some(i) => src[i + 1..].to_string(),
        None => String::new(),
    }
}

/// Read one `.delulu` source file named on the command line.
///
/// The directory case is handled explicitly (`HARDENING_CAMPAIGN.md` C27). Passing a package
/// directory to a file command is a natural mistake — `build` and `plugin build` take directories,
/// so `run`/`check`/`authority` look like they should — and the OS error for reading a directory as
/// a file is worse than useless: Windows reports `Access is denied. (os error 5)`, which reads as a
/// permissions problem and sends the user hunting for an ACL that was never involved. Linux says
/// `Is a directory`, so the misleading text was also platform-dependent. Name the real mistake and
/// the command that does what they meant.
/// Read a source file as the toolchain sees it: canonical text, its `SourceMap`, and its id.
///
/// `pub(crate)` for `delulu fix`, which must splice repair offsets into the same text the checker
/// computed them against — and which compares this result against the bytes on disk to prove the
/// two are identical before writing anything.
pub(crate) fn load(file: &str) -> Result<(SourceMap, u32, String), i32> {
    let path = std::path::Path::new(file);
    if path.is_dir() {
        eprintln!("error: `{file}` is a directory, and this command takes a single `.delulu` file");
        if path.join("delulu.toml").is_file() {
            eprintln!("note: it looks like a package — try `delulu build {file}`, or name a file such as `{}`", path.join("src").join("main.delulu").display());
        } else {
            eprintln!("note: name the file explicitly, e.g. `{}`", path.join("main.delulu").display());
        }
        return Err(2);
    }
    match std::fs::read_to_string(file) {
        Ok(src) => {
            // A `//! morph: <id>` pragma means this file is stored in a surface morph (Stage 8
            // §6.5). Convert it to canonical HERE, at the toolchain's edge, and hand the canonical
            // text to everything downstream — which is why no other command, checker, engine, or
            // hash needs to know morphs exist.
            //
            // The source map gets the CANONICAL text, so spans and the snippets rendered from them
            // agree with each other, and `--json` spans are canonical byte offsets exactly as the
            // spec requires. Morph rendering replaces keyword tokens in place and never adds or
            // removes a line, so line numbers are exact; a column inside a converted line can shift
            // by the difference in keyword length, and a diagnostic quotes the canonical spelling
            // rather than the alias the author typed. That is the documented trade, and it is the
            // one the spec asks for ("spans in canonical byte offsets").
            //
            // Deliberately NOT done in `delulu_syntax::parse_file`: resolving a pragma means reading
            // a morph file off disk, and a parser that acquires filesystem authority from a comment
            // in its input is ambient authority inside the compiler — the exact thing this language
            // exists to eliminate. The lookup belongs to the CLI, which already has that authority.
            let src = match delulu_syntax::morph::pragma_of(&src) {
                None => src,
                Some(id) => {
                    let m = match crate::morph_file::load(id) {
                        Ok(m) => m,
                        Err(e) => {
                            let map = SourceMap::new();
                            print_diagnostics("morph", &e.0, &map, None, false);
                            return Err(1);
                        }
                    };
                    // Lex the file in its own surface to find the keywords, then write canonical.
                    let mut probe = SourceMap::new();
                    let pid = probe.add_file(file, src.clone());
                    match m.to_canonical(pid, &src) {
                        Ok(canonical) => canonical,
                        Err(diags) => {
                            print_diagnostics("morph", &diags, &probe, None, false);
                            return Err(1);
                        }
                    }
                }
            };
            let mut map = SourceMap::new();
            let id = map.add_file(file, src.clone());
            Ok((map, id, src))
        }
        Err(e) => {
            // A compiled artifact handed to a source command produced a raw
            // `stream did not contain valid UTF-8`, which tells the reader nothing about what they
            // did or what to do instead — the C27 class of surfacing an OS-level error where a
            // diagnostic belongs (campaign finding C65). The `.dwx` is the DISTRIBUTION format, so
            // this is the mistake a reviewer makes first.
            if looks_like_dwx(file) {
                eprintln!(
                    "error: `{file}` is a compiled `.dwx` artifact, not DeluluLang source, and this \
                     command reads source"
                );
                eprintln!(
                    "note: the artifact carries its own authority — `delulu authority {file}` reports \
                     what it declares, and `delulu run {file}` verifies and executes it"
                );
                return Err(2);
            }
            eprintln!("error: cannot read `{file}`: {e}");
            Err(2)
        }
    }
}

/// `authority <file>.dwx`: report the artifact's own embedded `delulu:authority` manifest.
///
/// This goes through the SAME `read_and_verify` the runner uses, so the report cannot describe an
/// artifact the runner would reject: a tampered or missing section is DL1202 and a version mismatch
/// is DL1204, here exactly as at run time. A review surface that would vouch for bytes the runtime
/// refuses is worse than no review surface (finding C65, ruling D59).
///
/// What it reports is the artifact's DECLARATION, which is a different thing from a source report and
/// is labelled as such: there is no source in a `.dwx`, so there are no spans, no per-function purity
/// and no `why` chain — those need the code. What travels is the ceiling.
fn authority_artifact(file: &str, opts: &Opts) -> i32 {
    let bytes = match std::fs::read(file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read `{file}`: {e}");
            return 2;
        }
    };
    let artifact = match delulu_wasm::read_and_verify(&bytes) {
        Ok(a) => a,
        Err(e) => {
            let d = Diagnostic::error(e.code(), format!("`{file}`: {}", e.message()));
            print_diagnostics("authority", &d_slice(&d), &SourceMap::new(), None, opts.json);
            return 1;
        }
    };
    let effects = artifact.authority.get("effects").map(strs).unwrap_or_default();
    let secrets = artifact.authority.get("secrets").map(strs).unwrap_or_default();
    if opts.json {
        let report = json!({
            "artifact": file,
            "kind": "dwx",
            "verified": true,
            "authority": artifact.authority,
            "summary": { "effects": effects.len(), "secrets": secrets.len(), "errors": 0 },
        });
        println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
        note_json_emitted();
        return 0;
    }
    println!("Authority of `{file}` — a COMPILED ARTIFACT's own embedded manifest, re-verified:");
    println!(
        "  effects:      {}",
        if effects.is_empty() { "(none — provably pure)".to_string() } else { effects.join(", ") }
    );
    println!("  secrets:      {}", if secrets.is_empty() { "(none)".to_string() } else { secrets.join(", ") });
    for key in ["fs_read", "fs_write", "net", "declassify"] {
        let v = artifact.authority.get(key).map(strs).unwrap_or_default();
        if !v.is_empty() {
            println!("  {key:<13} {}", v.join(", "));
        }
    }
    // Said plainly, because the source report says more and a reader comparing the two must know
    // why: this is the declaration that ships, not an analysis of code that is not here.
    println!(
        "  note:         a `.dwx` carries no source, so there is no per-function purity and no \
         `why` chain — run those against the `.delulu` it was built from"
    );
    0
}

/// One diagnostic as a slice, for the `print_diagnostics` signature.
fn d_slice(d: &Diagnostic) -> [Diagnostic; 1] {
    [d.clone()]
}

/// Is `file` a compiled WebAssembly artifact rather than source?
///
/// Checked by CONTENT (the `\0asm` magic) and not only by extension: a `.dwx` renamed to `.delulu`
/// is the same mistake and deserves the same answer, and an extension is not evidence.
fn looks_like_dwx(file: &str) -> bool {
    if let Ok(bytes) = std::fs::read(file) {
        return bytes.starts_with(b"\0asm");
    }
    false
}

/// Whether a `--json` envelope has already reached stdout in this process.
///
/// Read by [`run`] to decide whether a nonzero exit needs a fallback envelope — see the comment
/// there for why an exit with no object at all is a contract break (`HARDENING_CAMPAIGN.md` C2).
static JSON_EMITTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn note_json_emitted() {
    JSON_EMITTED.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn print_diagnostics(command: &str, diags: &[Diagnostic], map: &SourceMap, authority: Option<Json>, json: bool) {
    if json {
        // Machine channel: never colored (addendum §2.5 / criterion 8), never localized
        // (invariant 39 — the envelope API cannot even see a catalog).
        note_json_emitted();
        println!("{}", envelope_to_string(command, diags, authority, map));
    } else {
        let palette = palette_stderr();
        let catalog = crate::locale::active_catalog();
        // Cap the HUMAN render (`HARDENING_CAMPAIGN.md` C32). One malformed expression can produce
        // thousands of diagnostics — `x.a.a.a…` 5000 deep is 5000 unknown-field errors — and past the
        // first few dozen the list stops informing anyone: a human scrolls, an agent spends its
        // context. The count is reported honestly rather than the tail being dropped in silence, and
        // the FIRST diagnostics are kept because the later ones are usually consequences of them.
        //
        // The machine channel above is deliberately uncapped: `--json` is a contract to report every
        // diagnostic, and a consumer that asked for all of them can page through them itself.
        const MAX_HUMAN_DIAGNOSTICS: usize = 50;
        for d in diags.iter().take(MAX_HUMAN_DIAGNOSTICS) {
            eprint!("{}", delulu_diag::render_human_localized(d, map, &palette, catalog));
            eprintln!();
        }
        if let Some(hidden) = diags.len().checked_sub(MAX_HUMAN_DIAGNOSTICS).filter(|n| *n > 0) {
            eprintln!(
                "note: {hidden} more diagnostic(s) not shown ({MAX_HUMAN_DIAGNOSTICS} of {} displayed) \
                 — fix these first, or re-run with `--json` for the complete list",
                diags.len()
            );
        }
    }
}

fn errors(diags: &[Diagnostic]) -> usize {
    diags.iter().filter(|d| d.is_error()).count()
}

// ----- check ---------------------------------------------------------------

fn cmd_check(rest: &[String]) -> i32 {
    let (file, opts) = parse_opts(rest);
    let Some(file) = file else {
        eprintln!("error: `check` needs a file or package directory");
        return 2;
    };
    if std::path::Path::new(&file).is_dir() {
        return build_workspace(&file, &opts, "check", opts.locked);
    }
    let (map, id, src) = match load(&file) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let checked = check_source(id, &src);
    let n = errors(&checked.diagnostics);
    print_diagnostics("check", &checked.diagnostics, &map, None, opts.json);
    if !opts.json {
        if n == 0 {
            ok_line!("ok: {} checked clean", file);
        } else {
            eprintln!("{n} error(s)");
        }
    }
    if n == 0 {
        0
    } else {
        1
    }
}

// ----- authority (the flagship) --------------------------------------------

fn cmd_authority(rest: &[String]) -> i32 {
    let (file, opts) = parse_opts(rest);
    if let Some(old_path) = opts.diff.clone() {
        let Some(new_arg) = file else {
            eprintln!("error: `authority --diff` needs <old.lock> and <new.lock-or-package-dir>");
            return 2;
        };
        return cmd_authority_diff(&old_path, &new_arg, &opts);
    }
    let Some(file) = file else {
        eprintln!("error: `authority` needs a file or package directory");
        return 2;
    };
    if std::path::Path::new(&file).is_dir() {
        return authority_package(&file, &opts);
    }
    // A `.dwx` is the format you SHIP, and the Book's claim for it is "authority that travels with
    // the code". The review command could not read it (campaign finding C65): it fell through to
    // the source loader and died on invalid UTF-8, while `run` happily verified the same embedded
    // manifest and announced the effects. The manifest was always there; nothing asked it.
    if looks_like_dwx(&file) {
        return authority_artifact(&file, &opts);
    }
    let (map, id, src) = match load(&file) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let checked = check_source(id, &src);
    if errors(&checked.diagnostics) > 0 {
        print_diagnostics("authority", &checked.diagnostics, &map, None, opts.json);
        return 1;
    }
    let mut scopes = manifest_scopes(&file);
    // Foreign blocks the program declares + its embedded-Python use → the `foreign_calls` array under
    // the "outside the proof" separator (spec §6). A program with no `foreign` blocks and no Python
    // use yields `[]`, keeping the report byte-identical to Stage 3 (criterion 7).
    let python_allowlist = manifest_python_allowlist(&file);
    scopes.foreign_calls = foreign_calls_json(&checked.module, &map, &python_allowlist);
    let program = checked.module.name.dotted();
    let mut report = authority_report(&program, &checked.result, &scopes);
    stamp_custody(&mut report, &opts);
    stamp_foreign_isolation(&mut report, &opts);
    stamp_isolation(&mut report, &opts);
    stamp_plugins(&mut report, &checked.module, &file, &map);
    stamp_native_emission(&mut report, &checked.module);
    if opts.json {
        note_json_emitted();
        println!("{}", envelope_to_string("authority", &[], Some(report), &map));
    } else {
        print!("{}", render_authority(&report));
    }
    0
}

/// Stamp the `plugins` array on an authority report (Stage 6 "Live", spec §6): the runtime plugins
/// this program **loads**, found by walking the checked AST for `load(host, "path", grant)` call
/// sites. Each entry carries the plugin's declared grant effects (from the `Grant { … }` literal),
/// its `loaded_at` source location, and — when the `.dpx` path resolves on disk — the artifact's
/// name, class, and signer identity (through the SAME `verify_signature` a load runs).
///
/// **Only added when the program actually loads a plugin** — a program with no `load` call gets no
/// `plugins` key, so every prior authority report is byte-identical (criterion 11). Grant/effects
/// extraction is best-effort over a literal `Grant { … }`; a computed grant reports what it can.
/// Whether any declaration in the module carries a `@jit` hint (Stage 10, invariant 46) — the
/// static form of "this program requests native-code emission".
fn module_requests_native(module: &delulu_syntax::ast::Module) -> bool {
    use delulu_syntax::ast::Item;
    let jit = |attrs: &[delulu_syntax::ast::Attribute]| attrs.iter().any(|a| a.name.name == "jit");
    jit(&module.attrs)
        || module.items.iter().any(|i| match i {
            Item::Fn(f) => jit(&f.attrs),
            Item::Actor(a) => jit(&a.attrs),
            // Every other `Item` has no `attrs` field at all, so none of them can carry a hint. That
            // is a fact about the AST, not a choice made here, and `jit_policy_cli.rs`'s
            // `every_ast_item_that_can_carry_an_attribute_is_one_the_native_hint_scan_looks_at`
            // reads `ast.rs` to keep it a fact — a `_ => false` that silently absorbs a new
            // attribute-carrying item is exactly how a hint would run unreported (C44).
            _ => false,
        })
}

/// Stamp `native_emission` on an authority report — ONLY when the program carries a `@jit` hint,
/// so every hint-free report stays byte-identical to 1.0 (the stamp_plugins pattern; house rule 4).
fn stamp_native_emission(report: &mut Json, module: &delulu_syntax::ast::Module) {
    if module_requests_native(module) {
        report["native_emission"] = serde_json::json!({ "requested": true, "via": "@jit" });
    }
}

fn stamp_plugins(report: &mut Json, module: &delulu_syntax::ast::Module, src_path: &str, map: &SourceMap) {
    let mut entries: Vec<Json> = Vec::new();
    let mut loads = Vec::new();
    for item in &module.items {
        if let Item::Fn(f) = item {
            collect_plugin_loads(&f.body, &mut loads);
        }
    }
    let base = std::path::Path::new(src_path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
    for lc in &loads {
        let (line, _col) = map.position(lc.span.file, lc.span.start);
        let file_name = map.name(lc.span.file).to_string();
        let mut entry = json!({
            "path": lc.path,
            "grant": { "effects": lc.grant_effects },
            "loaded_at": [{ "file": file_name, "line": line }],
            "name": Json::Null,
            "class": Json::Null,
            "signed_by": Json::Null,
        });
        // Enrich from the artifact on disk when the path resolves (best-effort — a missing file is
        // not an error here; `authority` is a static report, not a loader).
        if let Some(p) = &lc.path {
            let full = base.join(p);
            if let Ok(bytes) = std::fs::read(&full) {
                if let Ok(dpx) = delulu_wasm::read_dpx(&bytes) {
                    let obj = entry.as_object_mut().unwrap();
                    obj.insert("name".into(), dpx.manifest.get("name").cloned().unwrap_or(Json::Null));
                    obj.insert("class".into(), json!(dpx.class));
                    let payload = if dpx.class == "verified" { dpx.dir.as_deref() } else { dpx.wasm.as_deref() };
                    if let delulu_runtime::SignatureStatus::Valid { signer } =
                        delulu_runtime::verify_signature(&dpx.manifest, payload, dpx.sig.as_deref())
                    {
                        obj.insert("signed_by".into(), json!(signer));
                    }
                }
            }
        }
        entries.push(entry);
    }
    if !entries.is_empty() {
        if let Some(obj) = report.as_object_mut() {
            obj.insert("plugins".to_string(), Json::Array(entries));
        }
    }
}

/// One `load(...)` call site found in the AST: the string-literal path (if literal), the grant's
/// declared effects (if a `Grant { effects: [...] }` literal), and the call span.
struct PluginLoad {
    path: Option<String>,
    grant_effects: Vec<String>,
    span: delulu_diag::Span,
}

/// Recursively collect `load(host, path, grant)` call sites in a block. `load` is the Stage-6
/// builtin (checker §3.1); we recognize it by callee name and read its literal arguments. The grant
/// argument resolves either from an inline `Grant { … }` literal or from a same-function
/// `let g = Grant { … }` binding (a single-level, best-effort resolution — a computed grant reports
/// what it can, honestly).
fn collect_plugin_loads(block: &delulu_syntax::ast::Block, out: &mut Vec<PluginLoad>) {
    use delulu_syntax::ast::{Expr, Stmt};
    type Grants = std::collections::HashMap<String, Vec<String>>;

    fn resolve_grant(arg: &Expr, grants: &Grants) -> Vec<String> {
        match arg {
            Expr::Record { .. } => grant_effects_of(arg),
            Expr::Var { path, .. } if path.segs.len() == 1 => {
                grants.get(&path.segs[0].name).cloned().unwrap_or_default()
            }
            _ => Vec::new(),
        }
    }
    fn walk_expr(e: &Expr, out: &mut Vec<PluginLoad>, grants: &Grants) {
        match e {
            Expr::Call { callee, args, span, .. } => {
                if let Expr::Var { path, .. } = &**callee {
                    if path.segs.len() == 1 && path.segs[0].name == "load" {
                        out.push(PluginLoad {
                            path: args.get(1).and_then(literal_str),
                            grant_effects: args.get(2).map(|a| resolve_grant(a, grants)).unwrap_or_default(),
                            span: *span,
                        });
                    }
                }
                walk_expr(callee, out, grants);
                for a in args {
                    walk_expr(a, out, grants);
                }
            }
            Expr::Try { inner, .. } => walk_expr(inner, out, grants),
            Expr::Method { recv, args, .. } => {
                walk_expr(recv, out, grants);
                for a in args {
                    walk_expr(a, out, grants);
                }
            }
            Expr::List { items, .. } => items.iter().for_each(|i| walk_expr(i, out, grants)),
            Expr::Record { fields, .. } => fields.iter().for_each(|(_, fe)| walk_expr(fe, out, grants)),
            Expr::Field { recv, .. } => walk_expr(recv, out, grants),
            Expr::Index { recv, index, .. } => {
                walk_expr(recv, out, grants);
                walk_expr(index, out, grants);
            }
            Expr::Unary { operand, .. } => walk_expr(operand, out, grants),
            Expr::Binary { lhs, rhs, .. } => {
                walk_expr(lhs, out, grants);
                walk_expr(rhs, out, grants);
            }
            Expr::If { cond, then_, else_, .. } => {
                walk_expr(cond, out, grants);
                walk_block(then_, out, grants);
                if let Some(e) = else_ {
                    walk_expr(e, out, grants);
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                walk_expr(scrutinee, out, grants);
                arms.iter().for_each(|a| walk_expr(&a.body, out, grants));
            }
            Expr::Lambda { body, .. } => walk_block(body, out, grants),
            Expr::Block(b) => walk_block(b, out, grants),
            // Stage 7: a `load(…)` can sit inside spawn arguments or a recover block.
            Expr::Spawn { args, .. } => args.iter().for_each(|a| walk_expr(a, out, grants)),
            Expr::Recover { body, .. } => walk_block(body, out, grants),
            Expr::Lit { .. } | Expr::Var { .. } | Expr::Consume { .. } => {}
        }
    }
    fn walk_block(b: &delulu_syntax::ast::Block, out: &mut Vec<PluginLoad>, grants: &Grants) {
        // A block-local view of the grant bindings, so a `let g = Grant { … }` before a `load(…, g)`
        // resolves. Clone-on-extend keeps outer bindings visible without leaking inner ones out.
        let mut grants = grants.clone();
        for stmt in &b.stmts {
            match stmt {
                Stmt::Let { name, value, .. } => {
                    if let Expr::Record { path, .. } = value {
                        if path.segs.last().is_some_and(|s| s.name == "Grant") {
                            grants.insert(name.name.clone(), grant_effects_of(value));
                        }
                    }
                    walk_expr(value, out, &grants);
                }
                Stmt::Assign { value, .. } => walk_expr(value, out, &grants),
                Stmt::While { cond, body, .. } => {
                    walk_expr(cond, out, &grants);
                    walk_block(body, out, &grants);
                }
                Stmt::Return { value, .. } => {
                    if let Some(e) = value {
                        walk_expr(e, out, &grants);
                    }
                }
                Stmt::Expr(e) => walk_expr(e, out, &grants),
            }
        }
    }
    walk_block(block, out, &std::collections::HashMap::new());
}

/// The string value of a string-literal expression (else `None` — a computed path is not extracted).
fn literal_str(e: &delulu_syntax::ast::Expr) -> Option<String> {
    use delulu_syntax::ast::{Expr, LitKind};
    match e {
        Expr::Lit { kind: LitKind::Str(s), .. } => Some(s.clone()),
        _ => None,
    }
}

/// The declared effect names of a literal `Grant { effects: ["Read", …], … }` expression (else
/// empty — a computed grant reports no static effects here).
fn grant_effects_of(e: &delulu_syntax::ast::Expr) -> Vec<String> {
    use delulu_syntax::ast::Expr;
    if let Expr::Record { fields, .. } = e {
        for (name, fe) in fields {
            if name.name == "effects" {
                if let Expr::List { items, .. } = fe {
                    return items.iter().filter_map(literal_str).collect();
                }
            }
        }
    }
    Vec::new()
}

/// Stamp the custody label on an authority report (Stage 5, playbook 5j): `embedded` unless the
/// caller passed `--broker daemon`. Additive JSON key; the human render prints it as a line.
fn stamp_custody(report: &mut Json, opts: &Opts) {
    let custody = if opts.broker.as_deref() == Some("daemon") { "daemon" } else { "embedded" };
    if let Some(obj) = report.as_object_mut() {
        obj.insert("custody".to_string(), json!(custody));
    }
}

/// Stamp the foreign-isolation label on an authority report (Stage 5 phase 5h, spec §5): the mode
/// that WOULD apply when this program runs — `process` if `--foreign-isolation process` (or daemon
/// mode's default), else `inproc`. Additive JSON key; the human render prints it only inside a
/// non-empty foreign section, so a no-foreign report stays byte-identical (criterion 7).
fn stamp_foreign_isolation(report: &mut Json, opts: &Opts) {
    let daemon = opts.broker.as_deref() == Some("daemon");
    let mode = match opts.foreign_isolation.as_deref() {
        Some("process") => "process",
        Some("inproc") => "inproc",
        _ => {
            if daemon {
                "process"
            } else {
                "inproc"
            }
        }
    };
    if let Some(obj) = report.as_object_mut() {
        obj.insert("foreign_isolation".to_string(), json!(mode));
    }
}

/// Why `--isolation microvm` is unavailable here (`Err(detail)`), or `Ok(())` once the Linux+KVM
/// guest launch exists. v0.5: always `Err` — on Linux the probe names the first missing
/// prerequisite (or the pending launch work); everywhere else it is a platform refusal. Honest by
/// construction (trap 8): there is no code path that quietly substitutes weaker isolation.
#[cfg(target_os = "linux")]
fn microvm_unavailable() -> Result<(), String> {
    crate::microvm::probe()
}

#[cfg(not(target_os = "linux"))]
fn microvm_unavailable() -> Result<(), String> {
    Err(format!(
        "`--isolation microvm` requires Linux x86_64/aarch64 with KVM; this platform is `{}`",
        std::env::consts::OS
    ))
}

/// Stamp the requested isolation profile on an authority report (Stage 5 phase 5i, spec §6) —
/// only when `--isolation` was passed, so a default report stays byte-identical to prior stages
/// (criterion 11). An unattainable `microvm` request is labeled unavailable, never affirmed.
fn stamp_isolation(report: &mut Json, opts: &Opts) {
    let Some(iso) = &opts.isolation else { return };
    let label = if iso == "microvm" && microvm_unavailable().is_err() {
        "microvm (unavailable here — a run refuses with DL1408; see spec §6.1)".to_string()
    } else {
        iso.clone()
    };
    if let Some(obj) = report.as_object_mut() {
        obj.insert("isolation".to_string(), json!(label));
    }
}

fn strs(v: &Json) -> Vec<String> {
    v.as_array()
        .map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

fn render_authority(report: &Json) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    let name = report["program"].as_str().unwrap_or("?");
    let _ = writeln!(out, "Authority of `{name}` — what this program can do to your system:");
    let modules = strs(&report["modules"]);
    if !modules.is_empty() {
        let _ = writeln!(out, "  modules:      {}", modules.join(", "));
    }
    let effects = strs(&report["effects"]);
    let _ = writeln!(out, "  effects:      {}", if effects.is_empty() { "(none — provably pure)".to_string() } else { effects.join(", ") });
    let caps = report["capabilities"].as_array().cloned().unwrap_or_default();
    if caps.is_empty() {
        let _ = writeln!(out, "  capabilities: (none)");
    } else {
        let _ = writeln!(out, "  capabilities:");
        for c in &caps {
            let kind = c["kind"].as_str().unwrap_or("?");
            let scopes = strs(&c["scopes"]);
            let scope_str = if scopes.is_empty() { "(scope granted at runtime)".to_string() } else { scopes.join(", ") };
            let _ = writeln!(out, "    - {kind:<8} {scope_str}");
        }
    }
    let secrets = strs(&report["secrets"]);
    let _ = writeln!(out, "  secrets:      {}", if secrets.is_empty() { "(none)".to_string() } else { secrets.join(", ") });
    // The credential-exposure line (`HARDENING_CAMPAIGN.md` C25).
    //
    // Every fact needed to conclude "a credential can leave this program" was ALREADY in this
    // report — `Declassify` on the effects line, the names on the secrets line, the reach on the
    // foreign/effects lines — and the reader had to join three separate lines to see it. That is the
    // wrong division of labour: this report exists to be read by a human (or an agent) deciding
    // whether to type `--grant declassify`, and the decision turns precisely on the join. So the
    // report states the conclusion.
    //
    // It reports CAPABILITY, never behaviour: `Declassify` in the row means `expose` *can* be
    // called, not that it is. Nothing here changes what the language permits — no new refusal, no
    // change to any grant relation — it is strictly a sentence about facts already computed. The
    // machine channel is unchanged for the same reason: `--json` already carries `effects`,
    // `secrets`, and `foreign_calls`, so an agent could always derive this; only the human could not.
    if effects.iter().any(|e| e == "Declassify") && !secrets.is_empty() {
        let foreign_reach = report["foreign_calls"].as_array().is_some_and(|a| !a.is_empty());
        let mut reach: Vec<&str> = Vec::new();
        if foreign_reach {
            reach.push("foreign code (outside the proof)");
        }
        if effects.iter().any(|e| e == "Net") {
            reach.push("the network");
        }
        if effects.iter().any(|e| e == "Write") {
            reach.push("files/console");
        }
        if reach.is_empty() {
            let _ = writeln!(
                out,
                "  exposure:     {} declassifiable, but this program has no egress in its row",
                secrets.join(", ")
            );
        } else {
            let _ = writeln!(
                out,
                "  exposure:     {} declassifiable -> {}",
                secrets.join(", "),
                reach.join(", ")
            );
            let _ = writeln!(
                out,
                "                an exposed secret is an ordinary value; the language cannot follow it past `expose`"
            );
        }
    }
    let pure = strs(&report["pure_functions"]);
    // Bounded for the human, complete for the machine — D38's rule, which this line did not follow
    // (campaign finding C67). On a 24,600-line program the list is 2,536 names on ONE line of 28,242
    // characters, which buries the six lines a reviewer actually came for: the effects, the
    // capabilities, the secrets and the exposure verdict. `--json` carries every name in
    // `authority.pure_functions` and is deliberately uncapped, so nothing is lost — the count is what
    // carries the meaning here anyway ("2,536 of 2,539 functions cannot touch the world"), not the
    // roll-call.
    const MAX_HUMAN_PURE_FNS: usize = 40;
    let _ = writeln!(
        out,
        "  pure fns:     {}",
        if pure.is_empty() {
            "(none)".to_string()
        } else if pure.len() <= MAX_HUMAN_PURE_FNS {
            pure.join(", ")
        } else {
            format!(
                "{} total — showing {}: {}, … (`--json` lists every one under `authority.pure_functions`)",
                pure.len(),
                MAX_HUMAN_PURE_FNS,
                pure.iter().take(MAX_HUMAN_PURE_FNS).cloned().collect::<Vec<_>>().join(", ")
            )
        }
    );
    // Foreign code lives OUTSIDE the effect proof. With none declared, the report is byte-identical
    // to Stage 3 (criterion 7); with foreign blocks, they list under the mandated separator line.
    // The custody label (Stage 5): where authority lives when this program runs. The JSON report
    // always carries it (criterion 11: "custody label present in reports"); the HUMAN render shows
    // it only when custody is non-default (daemon), so the default embedded report stays
    // byte-identical to Stages 1–4 (criterion 11's no-regression half, matching the run path).
    if let Some(custody) = report["custody"].as_str() {
        if custody != "embedded" {
            let _ = writeln!(out, "  custody:      {custody}");
        }
    }
    // The isolation profile (Stage 5 phase 5i): present only when explicitly requested, so the
    // default report is byte-identical to prior stages (criterion 11).
    if let Some(iso) = report["isolation"].as_str() {
        let _ = writeln!(out, "  isolation:    {iso}");
    }
    // Native-code emission (Stage 10, invariant 46): the line appears ONLY when the program
    // actually carries a `@jit` hint, so every hint-free report stays byte-identical to 1.0.
    // `authority` is static (kind, not grants — §5.3): it reports the REQUEST; whether the run
    // was granted `exec.native` is answered at run time (DL1906 when it was not).
    if report["native_emission"]["requested"].as_bool() == Some(true) {
        let _ = writeln!(
            out,
            "  native-emission: requested (`@jit`) — off by default; granted only by explicit `--grant exec.native` (no native tier exists in v1.x)"
        );
    }
    let foreign = report["foreign_calls"].as_array().cloned().unwrap_or_default();
    if foreign.is_empty() {
        let _ = writeln!(out, "  foreign:      (none — no code outside the guarantee)");
    } else {
        let _ = writeln!(out, "  foreign:");
        let _ = writeln!(out, "    -- outside the proof (contained at process level) --");
        // Stage 5 phase 5h: which isolation profile bounds these foreign libraries' blast radius when
        // the program runs (spec §5). `process` = each library in an isolated worker subprocess (a
        // crash is DL1409, the host survives); `inproc` = Stage-4 in-process (a crash takes the host).
        let iso = report["foreign_isolation"].as_str().unwrap_or("inproc");
        let _ = writeln!(out, "    isolation: {iso}");
        for f in &foreign {
            let abi = f["abi"].as_str().unwrap_or("c");
            if abi == "python" {
                // The embedded-Python entry: the import allowlist (interface gate, §5.3) and the
                // statically-seen imports — no lib/symbols/binary.
                let allow = strs(&f["allowlist"]);
                let seen = strs(&f["imports_seen"]);
                let allow_str = if allow.is_empty() { "(none granted)".to_string() } else { allow.join(", ") };
                let seen_str = if seen.is_empty() { "(none seen statically)".to_string() } else { seen.join(", ") };
                let _ = writeln!(out, "    - python  allowlist: [{allow_str}]  imports seen: [{seen_str}]");
                continue;
            }
            if abi == "compute" {
                // `compute/<device> [kernels…]` — the shape spec §7.1 names. The kernels are the
                // names the program dispatches; which ARTIFACT each resolves to, and its hash, is
                // a grant-time human decision reported by the run, not by this static command.
                let devices = strs(&f["devices"]);
                let kernels = strs(&f["kernels"]);
                let ks = if kernels.is_empty() { "(none named statically)".to_string() } else { kernels.join(", ") };
                for d in &devices {
                    let _ = writeln!(out, "    - compute/{d} [{ks}]  (kernels are signed artifacts; what they compute is not proven)");
                }
                if devices.is_empty() {
                    let _ = writeln!(out, "    - compute/(device chosen at runtime) [{ks}]");
                }
                continue;
            }
            let lib = f["lib"].as_str().unwrap_or("?");
            let symbols = strs(&f["symbols"]);
            let binary = f["granted_path"].as_str().unwrap_or("(chosen by the human at grant time)");
            let _ = writeln!(out, "    - {abi} {lib} [{}]  binary: {binary}", symbols.join(", "));
        }
    }
    // The runtime plugins this program loads (Stage 6, spec §6). Present only when the program has a
    // `load(...)` call, so a plugin-free report is byte-identical to prior stages (criterion 11).
    if let Some(plugins) = report["plugins"].as_array() {
        if !plugins.is_empty() {
            let _ = writeln!(out, "  plugins:");
            for p in plugins {
                let name = p["name"].as_str().unwrap_or_else(|| p["path"].as_str().unwrap_or("?"));
                let class = p["class"].as_str().unwrap_or("?");
                let effects = strs(&p["grant"]["effects"]);
                let eff = if effects.is_empty() { "(none)".to_string() } else { effects.join(", ") };
                let signed = p["signed_by"].as_str().map(|s| format!("  signed-by {s}")).unwrap_or_default();
                let loc = p["loaded_at"].as_array().and_then(|a| a.first());
                let at = loc
                    .map(|l| format!("{}:{}", l["file"].as_str().unwrap_or("?"), l["line"].as_i64().unwrap_or(0)))
                    .unwrap_or_default();
                let _ = writeln!(out, "    - {name} [{class}]  grant.effects: [{eff}]  loaded at {at}{signed}");
            }
        }
    }
    out
}

fn manifest_scopes(file: &str) -> delulu_check::ScopeInfo {
    let dir = std::path::Path::new(file).parent().unwrap_or_else(|| std::path::Path::new("."));
    scopes_in_dir(dir)
}

fn scopes_in_dir(dir: &std::path::Path) -> delulu_check::ScopeInfo {
    match std::fs::read_to_string(dir.join("delulu.toml")) {
        Ok(src) => parse_manifest(&src).scope_info(),
        Err(_) => delulu_check::ScopeInfo::default(),
    }
}

/// The manifest's declared `foreign.python` import allowlist next to `file` (empty when no manifest).
fn manifest_python_allowlist(file: &str) -> Vec<String> {
    let dir = std::path::Path::new(file).parent().unwrap_or_else(|| std::path::Path::new("."));
    match std::fs::read_to_string(dir.join("delulu.toml")) {
        Ok(src) => parse_manifest(&src).foreign_python,
        Err(_) => Vec::new(),
    }
}

// ----- package mode (multi-module, Stage 2) --------------------------------

fn cmd_build(rest: &[String]) -> i32 {
    let (path, opts) = parse_opts(rest);
    let Some(path) = path else {
        eprintln!("error: `build` needs a source file (`--target wasm`) or a package directory");
        return 2;
    };
    // `build --target wasm <file.delulu>` emits a `.dwx` artifact (Stage 3 §5).
    if opts.target.as_deref() == Some("wasm") {
        return build_wasm_artifact(&path, &opts);
    }
    if let Some(t) = &opts.target {
        eprintln!("error: unknown --target `{t}` (only `wasm` is supported)");
        return 2;
    }
    if !std::path::Path::new(&path).is_dir() {
        eprintln!("error: `build` expects a package directory (with src/ and delulu.toml)");
        return 2;
    }
    build_workspace(&path, &opts, "build", opts.locked)
}

/// `build --target wasm <file.delulu>`: check the program, compile `main` to WebAssembly, and write
/// a `.dwx` artifact with the compiler-computed authority embedded as a `delulu:authority` custom
/// section (hash-bound to the code). This is the single, self-describing, authority-carrying
/// artifact: `delulu run <file>.dwx` re-verifies it before running.
fn build_wasm_artifact(file: &str, opts: &Opts) -> i32 {
    let (map, id, src) = match load(file) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let checked = check_source(id, &src);
    if errors(&checked.diagnostics) > 0 {
        print_diagnostics("build", &checked.diagnostics, &map, None, opts.json);
        return 1;
    }
    if !checked.result.main_present {
        let d = Diagnostic::error("DL1201", format!("`{file}` has no `fn main(root: Root)` — a `.dwx` artifact needs an entry point to run"));
        print_diagnostics("build", &[d], &map, None, opts.json);
        return 1;
    }
    let wasm = match delulu_wasm::compile_module_with(&checked.module, &checked.result.foreign_binds) {
        Ok(w) => w,
        Err(e) => {
            let d = Diagnostic::error(e.code(), format!("{} — this program can't be built to a `.dwx` yet (run it on the interpreter)", e.message()));
            print_diagnostics("build", &[d], &map, None, opts.json);
            return 1;
        }
    };
    // Embed the same authority answer `delulu authority` reports (incl. the `foreign_calls` entries),
    // so the artifact carries its own truthful manifest of what it can do — a foreign-using program's
    // manifest lists its foreign entries under the "outside the proof" separator (Stage 4 phase 4g).
    let mut scopes = manifest_scopes(file);
    let python_allowlist = manifest_python_allowlist(file);
    scopes.foreign_calls = foreign_calls_json(&checked.module, &map, &python_allowlist);
    let program = checked.module.name.dotted();
    let authority = authority_report(&program, &checked.result, &scopes);
    let dwx = delulu_wasm::embed_authority(&wasm, &authority);

    let out_path = opts.out.clone().unwrap_or_else(|| {
        std::path::Path::new(file).with_extension("dwx").to_string_lossy().to_string()
    });
    if let Err(e) = std::fs::write(&out_path, &dwx) {
        eprintln!("error: could not write `{out_path}`: {e}");
        return 2;
    }
    if opts.json {
        let report = json!({ "artifact": out_path, "bytes": dwx.len(), "authority": authority });
        note_json_emitted();
        println!("{}", envelope_to_string("build", &[], Some(report), &map));
    } else {
        ok_line!(
            "ok: wrote `{out_path}` ({} bytes) with authority embedded as `{}`",
            dwx.len(),
            delulu_wasm::AUTHORITY_SECTION
        );
    }
    0
}

// ----- plugin (Stage 6 "Live": build | inspect) --------------------------------------------------

fn cmd_plugin(rest: &[String]) -> i32 {
    let Some(sub) = rest.first() else {
        eprintln!("error: `plugin` needs a subcommand: build <package-dir> | inspect <file.dpx> | verify <file.dpx>");
        return 2;
    };
    match sub.as_str() {
        "build" => cmd_plugin_build(&rest[1..]),
        "inspect" => cmd_plugin_inspect(&rest[1..]),
        "verify" => cmd_plugin_verify(&rest[1..]),
        other => {
            eprintln!("error: unknown `plugin` subcommand `{other}` (expected build | inspect | verify)");
            2
        }
    }
}

/// `delulu plugin verify <file.dpx> [--json]` — run load-sequence steps 1, 2, 5 (and the signature
/// check) WITHOUT instantiating (spec §6). It reads the container through the SAME engine path a
/// real load uses and calls the SAME verification functions, so it gives **identical verdicts to a
/// real load** (criterion 9 — no verify/load divergence).
fn cmd_plugin_verify(rest: &[String]) -> i32 {
    let (file, opts) = parse_opts(rest);
    let Some(file) = file else {
        eprintln!("error: `plugin verify` needs a `.dpx` file");
        return 2;
    };
    let bytes = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read `{file}`: {e}");
            return 2;
        }
    };
    let map = SourceMap::new();
    // The container read is the SAME `read_artifact` the loader uses (spec §9.9), so a corrupt
    // artifact refuses here exactly as at load.
    use delulu_runtime::PluginEngine as _;
    let art = match delulu_wasm::WasmPluginEngine::new().read_artifact(&bytes) {
        Ok(a) => a,
        Err(e) => {
            let d = Diagnostic::error(e.code, e.message);
            print_diagnostics("plugin", &[d], &map, None, opts.json);
            return 1;
        }
    };
    match delulu_runtime::verify_plugin(&art) {
        Ok(report) => {
            let signed_by = match &report.signature {
                delulu_runtime::SignatureStatus::Valid { signer } => Some(signer.clone()),
                _ => None,
            };
            if opts.json {
                note_json_emitted();
                println!(
                    "{}",
                    json!({
                        "command": "plugin", "subcommand": "verify", "artifact": file,
                        "class": report.class.as_str(), "verdict": "ok",
                        "exports": report.exports, "signed_by": signed_by,
                    })
                );
            } else {
                let sig_note = signed_by.as_deref().map(|s| format!(", signed by {s}")).unwrap_or_default();
                ok_line!(
                    "ok: `{file}` verifies — {} plugin, {} export(s){sig_note}",
                    report.class.as_str(),
                    report.exports.len()
                );
            }
            0
        }
        Err(e) => {
            let d = Diagnostic::error(e.code, e.message);
            print_diagnostics("plugin", &[d], &map, None, opts.json);
            1
        }
    }
}

/// `delulu plugin build <package-dir> [-o out.dpx] [--json]` — build a `kind = "plugin"` package
/// into a `.dpx` (spec §2). Refusal honesty (house rule 4): every check runs BEFORE any byte is
/// written — a refused build leaves no partial artifact and never touches an existing output file.
fn cmd_plugin_build(rest: &[String]) -> i32 {
    let (dir, opts) = parse_opts(rest);
    let Some(dir) = dir else {
        eprintln!("error: `plugin build` needs a package directory (with delulu.toml and src/)");
        return 2;
    };
    let root = std::path::Path::new(&dir);
    let manifest_path = root.join("delulu.toml");
    let manifest_src = match std::fs::read_to_string(&manifest_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read `{}`: {e}", manifest_path.display());
            return 2;
        }
    };
    let mut map = SourceMap::new();
    let mfile = map.add_file(manifest_path.to_string_lossy(), manifest_src.clone());

    // [package] — must be kind = "plugin".
    let mut diags: Vec<Diagnostic> = Vec::new();
    let (manifest, mdiags) = delulu_check::Manifest::parse(&manifest_src, mfile);
    diags.extend(mdiags);
    let kind_ok = manifest.as_ref().map(|m| m.kind == delulu_check::PackageKind::Plugin).unwrap_or(false);
    if manifest.is_some() && !kind_ok {
        diags.push(
            Diagnostic::error(
                "DL1004",
                "`plugin build` needs a `kind = \"plugin\"` package (set `kind` under `[package]`)",
            )
            .with_bare_span(delulu_diag::Span::new(mfile, 0, 0)),
        );
    }

    // [plugin] / [plugin.authority] / [plugin.exports].
    let (pm, pdiags) = delulu_check::PluginManifest::parse(&manifest_src, mfile);
    diags.extend(pdiags);
    if errors(&diags) > 0 || pm.is_none() || manifest.is_none() {
        print_diagnostics("plugin", &diags, &map, None, opts.json);
        return 1;
    }
    let pm = pm.expect("checked above");

    // The plugin's source: exactly one module in v0.6 (build-order deviation 3 — refused cleanly,
    // never built partially).
    let mut sources = Vec::new();
    collect_delulu_sources(&root.join("src"), &mut sources);
    sources.sort();
    if sources.len() != 1 {
        diags.push(Diagnostic::error(
            "DL1004",
            format!(
                "plugin packages are single-module in v0.6 — found {} module file(s) under `src/`",
                sources.len()
            ),
        ));
        print_diagnostics("plugin", &diags, &map, None, opts.json);
        return 1;
    }
    let src_path = &sources[0];
    let src = match std::fs::read_to_string(src_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read `{}`: {e}", src_path.display());
            return 2;
        }
    };
    let sfile = map.add_file(src_path.to_string_lossy(), src.clone());

    // Check the code, then run the manifest-vs-code fence (DL1501 — the manifest never overrides
    // the code). The fence only runs on a clean check: its comparisons need trustworthy fn_types.
    let checked = check_source(sfile, &src);
    diags.extend(checked.diagnostics.iter().cloned());
    if errors(&diags) == 0 {
        diags.extend(delulu_check::check_plugin_module(&pm, &checked.table, &checked.result, &manifest_src, mfile));
    }
    if errors(&diags) > 0 {
        print_diagnostics("plugin", &diags, &map, None, opts.json);
        return 1;
    }

    // Class-specific payload (spec §2.2 table). DIR for Verified; the compiled module for Contained.
    let (dir_bytes, wasm_bytes): (Option<Vec<u8>>, Option<Vec<u8>>) = match pm.class {
        delulu_check::PluginClass::Verified => {
            (Some(delulu_check::dir_serialize(&checked.module, &checked.result)), None)
        }
        delulu_check::PluginClass::Contained => {
            match delulu_wasm::compile_module_with(&checked.module, &checked.result.foreign_binds) {
                Ok(w) => (None, Some(w)),
                Err(e) => {
                    let d = Diagnostic::error(
                        e.code(),
                        format!("{} — this program can't be built into a contained plugin", e.message()),
                    );
                    print_diagnostics("plugin", &[d], &map, None, opts.json);
                    return 1;
                }
            }
        }
    };
    let lock_bytes = std::fs::read(root.join("delulu.lock")).ok();

    let manifest_json = plugin_manifest_json(&pm);

    // Phase 6h: optionally sign the artifact. The signature covers the EXACT canonical manifest bytes
    // the container will hold (`augmented_plugin_manifest`) followed by the class payload — DIR for
    // Verified, the module for Contained (spec §2.2). Signatures authenticate origin, not behavior.
    let sig_bytes: Option<Vec<u8>> = match &opts.sign {
        None => None,
        Some(keyfile) => {
            let seed = match read_ed25519_seed(keyfile) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: --sign: {e}");
                    return 2;
                }
            };
            let augmented = delulu_wasm::augmented_plugin_manifest(
                &manifest_json,
                dir_bytes.as_deref(),
                wasm_bytes.as_deref(),
                lock_bytes.as_deref(),
            );
            // The signed payload is the class's own bytes (spec §2.2): DIR (Verified) or module (Contained).
            let payload = dir_bytes.as_deref().or(wasm_bytes.as_deref());
            Some(delulu_runtime::sign_plugin(&seed, &augmented, payload))
        }
    };

    let dpx = delulu_wasm::write_dpx(
        &manifest_json,
        dir_bytes.as_deref(),
        wasm_bytes.as_deref(),
        sig_bytes.as_deref(),
        lock_bytes.as_deref(),
    );

    let out_path = opts
        .out
        .clone()
        .unwrap_or_else(|| root.join(format!("{}.dpx", pm.name)).to_string_lossy().to_string());
    if let Err(e) = std::fs::write(&out_path, &dpx) {
        eprintln!("error: could not write `{out_path}`: {e}");
        return 2;
    }
    if opts.json {
        // Machine channel: never styled; deterministic (BTreeMap-ordered exports, sorted keys).
        note_json_emitted();
        println!(
            "{}",
            json!({
                "command": "plugin", "subcommand": "build",
                "artifact": out_path, "bytes": dpx.len(),
                "name": pm.name, "version": pm.version,
                "api": pm.api, "class": pm.class.as_str(),
                "exports": pm.exports.keys().cloned().collect::<Vec<_>>(),
                "signed": sig_bytes.is_some(),
            })
        );
    } else {
        let signed = if sig_bytes.is_some() { " (signed)" } else { "" };
        ok_line!(
            "ok: wrote `{out_path}` ({} bytes) — {} plugin `{}` v{}, {} export(s){signed}",
            dpx.len(),
            pm.class.as_str(),
            pm.name,
            pm.version,
            pm.exports.len()
        );
    }
    0
}

/// Read an ed25519 signing seed from a key file (phase 6h). Accepts a raw 32-byte seed or 64 hex
/// characters (whitespace ignored) — nothing is generated here; the human supplies the key.
fn read_ed25519_seed(path: &str) -> Result<[u8; 32], String> {
    let raw = std::fs::read(path).map_err(|e| format!("cannot read key file `{path}`: {e}"))?;
    if raw.len() == 32 {
        let mut s = [0u8; 32];
        s.copy_from_slice(&raw);
        return Ok(s);
    }
    let hex: String = String::from_utf8_lossy(&raw).chars().filter(|c| !c.is_whitespace()).collect();
    if hex.len() == 64 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        let mut s = [0u8; 32];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            s[i] = u8::from_str_radix(std::str::from_utf8(chunk).expect("ascii hex"), 16)
                .map_err(|_| "key file is not valid hex".to_string())?;
        }
        return Ok(s);
    }
    Err(format!(
        "key file must be a raw 32-byte ed25519 seed or 64 hex characters (found {} bytes)",
        raw.len()
    ))
}

/// Build the `delulu:plugin` section JSON from the parsed manifest (spec §2.2). Fixed shape,
/// every key always present, exports BTreeMap-ordered — so identical inputs give byte-identical
/// artifacts (house rule 8). `write_dpx` adds `container` + the section hash bindings.
fn plugin_manifest_json(pm: &delulu_check::PluginManifest) -> Json {
    json!({
        "name": pm.name,
        "version": pm.version,
        "api": pm.api,
        "class": pm.class.as_str(),
        "authority": {
            "effects": pm.authority.effects,
            "requires": pm.authority.requires,
            "fs_read": pm.authority.fs_read,
            "fs_write": pm.authority.fs_write,
            "net": pm.authority.net,
            "secrets": pm.authority.secrets,
            "declassify": pm.authority.declassify,
        },
        "exports": pm.exports,
    })
}

/// `delulu plugin inspect <file.dpx> [--json]` — describe an artifact: manifest, class, exports,
/// section hashes, signature identity. Runs the same container read path as `verify`/`load`
/// (spec §9.9 — one code path), so a corrupt artifact refuses here exactly as it would at load.
fn cmd_plugin_inspect(rest: &[String]) -> i32 {
    let (file, opts) = parse_opts(rest);
    let Some(file) = file else {
        eprintln!("error: `plugin inspect` needs a `.dpx` file");
        return 2;
    };
    let bytes = match std::fs::read(&file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read `{file}`: {e}");
            return 2;
        }
    };
    let map = SourceMap::new();
    let dpx = match delulu_wasm::read_dpx(&bytes) {
        Ok(d) => d,
        Err(e) => {
            let d = Diagnostic::error(e.code(), e.message());
            print_diagnostics("plugin", &[d], &map, None, opts.json);
            return 1;
        }
    };

    // The hashes shown are the manifest's content bindings — already verified against the actual
    // section bytes by `read_dpx` (class-aware; a Verified cache may be flagged invalid instead).
    // Phase 6h: the signature identity comes from the SAME `verify_signature` the loader runs (spec
    // §9.9), over the class payload (DIR for Verified, the module for Contained).
    let sig_payload = if dpx.class == "verified" { dpx.dir.as_deref() } else { dpx.wasm.as_deref() };
    let sig_status = delulu_runtime::verify_signature(&dpx.manifest, sig_payload, dpx.sig.as_deref());
    let (signed_by, sig_human): (Json, String) = match &sig_status {
        delulu_runtime::SignatureStatus::Unsigned => (Json::Null, "none".into()),
        delulu_runtime::SignatureStatus::Valid { signer } => (json!(signer), format!("valid — signed by {signer}")),
        delulu_runtime::SignatureStatus::Invalid { reason } => (Json::Null, format!("INVALID — {reason}")),
    };
    let null = Json::Null;
    let get = |k: &str| dpx.manifest.get(k).unwrap_or(&null).clone();
    if opts.json {
        note_json_emitted();
        println!(
            "{}",
            json!({
                "command": "plugin", "subcommand": "inspect",
                "artifact": file,
                "name": get("name"), "version": get("version"),
                "api": dpx.api, "class": dpx.class,
                "container": get("container"),
                "authority": get("authority"),
                "exports": get("exports"),
                "sections": {
                    "dir": get("dir_blake3"),
                    "wasm": get("wasm_blake3"),
                    "sig": dpx.sig.is_some(),
                    "lock": get("lock_blake3"),
                },
                "signed_by": signed_by,
                "wasm_cache_valid": dpx.wasm_cache_valid,
            })
        );
    } else {
        let name = get("name");
        let version = get("version");
        println!(
            "plugin `{}` v{} — class {}, api {}",
            name.as_str().unwrap_or("?"),
            version.as_str().unwrap_or("?"),
            dpx.class,
            dpx.api
        );
        if let Some(a) = dpx.manifest.get("authority") {
            let effects: Vec<&str> =
                a.get("effects").and_then(|v| v.as_array()).map(|xs| xs.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
            let requires: Vec<&str> =
                a.get("requires").and_then(|v| v.as_array()).map(|xs| xs.iter().filter_map(|x| x.as_str()).collect()).unwrap_or_default();
            println!("  authority ceiling: effects [{}]; requires [{}]", effects.join(", "), requires.join(", "));
        }
        if let Some(exports) = dpx.manifest.get("exports").and_then(|v| v.as_object()) {
            println!("  exports:");
            for (k, v) in exports {
                println!("    {k}: {}", v.as_str().unwrap_or("?"));
            }
        }
        let hash8 = |v: Json| v.as_str().map(|s| s[..s.len().min(12)].to_string());
        println!("  sections:");
        println!("    delulu:plugin  present");
        if let Some(h) = hash8(get("dir_blake3")) {
            println!("    delulu:dir     blake3 {h}…");
        }
        if let Some(h) = hash8(get("wasm_blake3")) {
            let note = if dpx.wasm_cache_valid { "" } else { "  (cache INVALID — would be recompiled from DIR)" };
            println!("    delulu:wasm    blake3 {h}…{note}");
        }
        if dpx.sig.is_some() {
            println!("    delulu:sig     present");
        }
        if let Some(h) = hash8(get("lock_blake3")) {
            println!("    delulu:lock    blake3 {h}…");
        }
        println!("  signature: {sig_human}");
    }
    0
}

/// Recursively collect `.delulu` files under `dir` (the plugin-build module walk).
fn collect_delulu_sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_delulu_sources(&p, out);
            } else if p.extension().and_then(|s| s.to_str()) == Some("delulu") {
                out.push(p);
            }
        }
    }
}

/// Resolve the workspace (root + path dependencies), check every package under one global type
/// registry, and verify the whole-graph authority story: DL1009 (package exceeds its own
/// manifest), DL1001 (a dependency exceeds its pin), and — under `--locked` — DL1010/DL1002/DL1011
/// against `delulu.lock`. This is the compile-time supply-chain gate: it runs BEFORE any code.
fn build_workspace(dir: &str, opts: &Opts, command: &str, locked: bool) -> i32 {
    let ws = resolve_workspace(dir);
    let program = check_workspace(&ws);

    let mut diags: Vec<Diagnostic> = Vec::new();
    diags.extend(ws.diagnostics.iter().cloned());
    diags.extend(program.diagnostics.iter().cloned());
    diags.extend(check_self_authority(&ws, &program));
    diags.extend(check_pins(&ws, &program));

    if locked {
        match std::fs::read_to_string(std::path::Path::new(dir).join("delulu.lock")) {
            Ok(text) => {
                let lock = Lockfile::parse(&text);
                diags.extend(verify_locked(&ws, &program, &lock));
            }
            Err(_) => {
                for pkg in &ws.packages {
                    diags.push(Diagnostic::error(
                        "DL1011",
                        format!("`--locked` build but delulu.lock is missing — package `{}` is unresolved; run `delulu lock`", pkg.name),
                    ));
                }
            }
        }
    }

    // Stage 10 phase 10k (Track C, spec §4): the registry advisory feed. Scoped to `build` — the
    // supply-chain gate — because `check`'s output is byte-stable and does not run CI gates. A
    // resolved dependency on an advised version is DL1903 (a warning by default, an error under
    // `--deny-advisories`); the `gate_note` carries the skip-branch refusals where the feed itself
    // cannot be trusted to have been evaluated (see `advisory_diagnostics`).
    let mut advisory_gate_note: Option<String> = None;
    if command == "build" {
        let (adv_diags, gate_note) = advisory_diagnostics(dir, &ws, opts);
        diags.extend(adv_diags);
        advisory_gate_note = gate_note;
    }

    let n = errors(&diags);
    print_diagnostics(command, &diags, &ws.source_map, None, opts.json);
    // Git dependencies are parsed and rev/tag-validated, but fetching is deferred to a later stage.
    // We refuse rather than pretend a git dependency's authority was verified.
    let git_blocked = !ws.git_deferred.is_empty();
    if git_blocked && !opts.json {
        eprintln!(
            "note: git dependency resolution is deferred to a later stage; cannot verify authority for: {}",
            ws.git_deferred.join(", ")
        );
    }
    // The advisory gate refusing (feed missing/unreadable under `--deny-advisories`) forces a
    // non-zero exit exactly as `git_blocked` does — same posture, same reason: a supply-chain
    // check that could not run must not report success.
    if let Some(note) = &advisory_gate_note {
        if !opts.json {
            eprintln!("note: {note}");
        }
    }
    // A package with no modules used to report `built clean (1 package(s), 0 module(s))` and exit 0.
    // The toolchain had looked in `<root>/src`, found nothing, checked nothing, and called it
    // success — so a flat layout (sources beside `delulu.toml` instead of under `src/`) produced a
    // green build of an empty program. Same posture as `git_blocked` above, for the same reason: a
    // check that could not run must never report success. See `HARDENING_CAMPAIGN.md` C26.
    let no_modules = ws.modules.is_empty();
    // The note needs the root package's directory, and there may not BE a root package: an
    // unreadable manifest fails resolution before one is recorded, leaving zero packages and zero
    // modules. That case already has its own diagnostic (and `failed` below is still true, so the
    // exit stays non-zero) — what it must not do is panic while composing a courtesy note (C49).
    if no_modules && !opts.json {
        if let Some(root) = ws.root_pkg() {
            eprintln!(
                "note: no `.delulu` modules found under `{}` — a DeluluLang package keeps its \
                 sources in `src/`, so nothing was checked",
                root.dir.join("src").display()
            );
        }
    }
    let failed = n > 0 || git_blocked || advisory_gate_note.is_some() || no_modules;
    // §5.5: `interface.json` is a build artifact, not a check artifact — only `build` writes it,
    // and only after a clean whole-graph check (never on a failed/diagnostic-bearing build).
    if command == "build" && !failed {
        write_interfaces(&ws, &program);
    }
    if !opts.json {
        if !failed {
            // `check` and `build` share this path but do different things — §5.5 is explicit that
            // only `build` writes `interface.json`. Saying "built clean" for a `check` claimed an
            // artifact step that never ran.
            // Reachable only when nothing failed, which implies a root package was resolved — but
            // the name is taken defensively all the same, because a success line is the last place
            // that should be able to abort a run (C49).
            ok_line!(
                "ok: `{}` {} clean ({} package(s), {} module(s); authority within manifest and pins)",
                ws.root_pkg().map_or("<unnamed>", |p| p.name.as_str()),
                if command == "build" { "built" } else { "checked" },
                ws.packages.len(),
                ws.modules.len()
            );
        } else if n > 0 {
            eprintln!("{n} error(s)");
        } else {
            // Refused with no diagnostic: a required gate could not RUN (git deps deferred, the
            // advisory feed unreadable, or no modules found). The old closing line here was
            // `0 error(s)`, which reads as success next to a nonzero exit — the note above carries
            // the reason, so say that instead of counting errors that were never the problem.
            eprintln!(
                "refused: the source produced no errors, but a required check could not run (see the note above)"
            );
        }
    }
    if failed {
        1
    } else {
        0
    }
}

/// The advisory-feed scan for a `build` (Stage 10 phase 10k, Track C, spec §4).
///
/// Returns the DL1903 diagnostics (warnings by default, errors under `--deny-advisories`) and an
/// optional *gate-blocked note*: the skip-branch conditions under `--deny-advisories` where the
/// feed cannot be trusted to have been fully evaluated (absent, unreadable, or carrying a record
/// this toolchain cannot parse). Those are not DL1903 — DL1903 means specifically "this dependency
/// is on an advised version," and a gate that cannot read its feed has not found such a dependency;
/// it has found that it *cannot answer*, which under a CI gate must fail. The note + forced
/// non-zero exit mirrors the existing `git_deferred` refusal exactly: a condition about the
/// toolchain's evidence, not the program, so it takes a note rather than a code.
///
/// Skip-branch discipline (the reason this function is written the way it is): WITHOUT the gate an
/// absent feed is silence — there is genuinely nothing known to warn about — but WITH the gate an
/// absent, unreadable, or partly-unparseable feed is a refusal, because "I could not find the
/// evidence" must never render as "the scan was clean." This is the same rule DL1905 draws for a
/// missing hardware sign-off record.
fn advisory_diagnostics(dir: &str, ws: &Workspace, opts: &Opts) -> (Vec<Diagnostic>, Option<String>) {
    use crate::advisories::{load_feed, scan, FeedStatus};

    let feed_path = match &opts.advisory_feed {
        Some(p) => std::path::PathBuf::from(p),
        None => std::path::Path::new(dir).join("delulu.advisories.json"),
    };
    let deny = opts.deny_advisories;
    let mut diags = Vec::new();

    let advisories = match load_feed(&feed_path) {
        FeedStatus::Absent => {
            if deny {
                return (
                    diags,
                    Some(format!(
                        "`--deny-advisories` was set, but no advisory feed was found (looked for \
                         `{p}`). A gate with nothing to check passes nothing — refusing rather than \
                         reporting a clean scan against a feed that is not there. Point at one with \
                         `--advisory-feed <path>`, or sync one from the registry \
                         (`delulu-registry advisory export --out {p}`).",
                        p = feed_path.display()
                    )),
                );
            }
            // No feed and no gate: genuine silence. Nothing known, so nothing said.
            return (diags, None);
        }
        FeedStatus::Unreadable(why) => {
            if deny {
                return (
                    diags,
                    Some(format!(
                        "`--deny-advisories` was set, but the advisory feed could not be read: \
                         {why}. Refusing to pass a gate it cannot evaluate."
                    )),
                );
            }
            // A feed that exists but cannot be read is a visible warning, never a silent skip:
            // the file is there, so advisory checks were expected, and they did not run.
            diags.push(Diagnostic::warning(
                "DL1903",
                format!("advisory feed could not be read: {why} — building without advisory checks"),
            ));
            return (diags, None);
        }
        FeedStatus::Loaded { advisories, malformed } => {
            if malformed > 0 {
                if deny {
                    return (
                        diags,
                        Some(format!(
                            "`--deny-advisories` was set, but the advisory feed carries {malformed} \
                             record(s) this toolchain cannot parse. Refusing to enforce a feed it \
                             cannot fully read — an unreadable record is exactly where a gate would \
                             silently miss the advisory that mattered."
                        )),
                    );
                }
                diags.push(Diagnostic::warning(
                    "DL1903",
                    format!(
                        "advisory feed carries {malformed} unparseable record(s), skipped — the \
                         readable advisories were still checked"
                    ),
                ));
            }
            advisories
        }
    };

    // The resolved set is every workspace package name@version. The feed names none of the packages
    // that have no advisory, so scanning all of them is safe and needs no graph bookkeeping the
    // workspace does not already carry.
    let deps: Vec<(String, String)> = ws
        .packages
        .iter()
        .map(|p| (p.name.clone(), p.manifest.version.clone()))
        .collect();

    for hit in scan(&advisories, &deps) {
        let fix = match &hit.patched {
            Some(v) => format!(" — upgrade to {v} (`delulu add {}@{v}`, then `delulu lock`)", hit.package),
            None => String::new(),
        };
        let summary = if hit.summary.is_empty() { String::new() } else { format!(": {}", hit.summary) };
        let msg = format!(
            "dependency `{}` {} is affected by advisory {} ({}){}{}",
            hit.package, hit.version, hit.advisory_id, hit.severity, summary, fix
        );
        diags.push(if deny {
            Diagnostic::error("DL1903", msg)
        } else {
            Diagnostic::warning("DL1903", msg)
        });
    }

    (diags, None)
}

/// §5.5: write `<pkgdir>/target/<pkg>/interface.json` for every package in the workspace (root +
/// path dependencies) — its `pub` surface (name, full type, effect row) plus the same
/// `api_row_hash` the lockfile carries, so an agent can introspect a dependency without reading
/// its source. The spec text frames this as "per lib package"; this writes it for every package
/// regardless of `kind`, since a `bin` package's `pub` fns are equally legitimate machine surface
/// and the extra file is harmless when there are none (`exports: []`).
fn write_interfaces(ws: &Workspace, program: &Program) {
    for (idx, pkg) in ws.packages.iter().enumerate() {
        let mut exports: Vec<Json> = Vec::new();
        for wm in ws.modules.iter().filter(|m| m.pkg == idx) {
            for item in &wm.unit.module.items {
                if let Item::Fn(f) = item {
                    if !f.public {
                        continue;
                    }
                    let key = format!("{}::{}", wm.unit.name, f.name.name);
                    let ty = program
                        .fn_types
                        .get(&key)
                        .map(|t| t.show(&program.type_names).to_string())
                        .unwrap_or_default();
                    let row = program
                        .facts
                        .get(&key)
                        .map(|f| Row::closed(f.effects.clone()).to_string())
                        .unwrap_or_else(|| Row::pure().to_string());
                    exports.push(json!({ "name": f.name.name, "type": ty, "row": row }));
                }
            }
        }
        exports.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
        let doc = json!({
            "package": pkg.name,
            "version": pkg.manifest.version,
            "exports": exports,
            "api_row_hash": api_row_hash(ws, idx),
        });
        let out_dir = pkg.dir.join("target").join(&pkg.name);
        if let Err(e) = std::fs::create_dir_all(&out_dir) {
            eprintln!("warning: cannot create `{}`: {e}", out_dir.display());
            continue;
        }
        match serde_json::to_string_pretty(&doc) {
            Ok(text) => {
                let out_file = out_dir.join("interface.json");
                if let Err(e) = std::fs::write(&out_file, text) {
                    eprintln!("warning: cannot write `{}`: {e}", out_file.display());
                }
            }
            Err(e) => eprintln!("warning: cannot serialize interface.json for `{}`: {e}", pkg.name),
        }
    }
}

/// `delulu lock`: (re)compute delulu.lock next to the root manifest and enforce the
/// semver-authority law (DL1003) against the previous lock.
fn cmd_lock(rest: &[String]) -> i32 {
    let (path, opts) = parse_opts(rest);
    let dir = path.unwrap_or_else(|| ".".to_string());
    if !std::path::Path::new(&dir).is_dir() {
        eprintln!("error: `lock` expects a package directory (with src/ and delulu.toml)");
        return 2;
    }

    let ws = resolve_workspace(&dir);
    let program = check_workspace(&ws);
    let mut diags: Vec<Diagnostic> = Vec::new();
    diags.extend(ws.diagnostics.iter().cloned());
    diags.extend(program.diagnostics.iter().cloned());
    diags.extend(check_self_authority(&ws, &program));
    diags.extend(check_pins(&ws, &program));
    if !ws.git_deferred.is_empty() {
        diags.push(Diagnostic::error(
            "DL1007",
            format!("cannot lock: git dependency resolution is deferred ({})", ws.git_deferred.join(", ")),
        ));
    }
    if errors(&diags) > 0 {
        print_diagnostics("lock", &diags, &ws.source_map, None, opts.json);
        if !opts.json {
            eprintln!("{} error(s) — refusing to write delulu.lock", errors(&diags));
        }
        return 1;
    }

    let mut newlock = compute_lockfile(&ws, &program);
    let lock_path = std::path::Path::new(&dir).join("delulu.lock");
    if let Ok(text) = std::fs::read_to_string(&lock_path) {
        let old = Lockfile::parse(&text);
        let law = enforce_semver_law(&old, &mut newlock, &opts.accept_authority);
        if !law.is_empty() {
            print_diagnostics("lock", &law, &ws.source_map, None, opts.json);
            if !opts.json {
                eprintln!("semver-authority law violated — refusing to write delulu.lock");
            }
            return 1;
        }
    }
    if let Err(e) = std::fs::write(&lock_path, newlock.render()) {
        eprintln!("error: cannot write {}: {e}", lock_path.display());
        return 2;
    }
    if opts.json {
        note_json_emitted();
        println!("{}", envelope_to_string("lock", &[], None, &ws.source_map));
    } else {
        ok_line!("ok: wrote {} ({} package(s))", lock_path.display(), newlock.packages.len());
    }
    0
}

/// The authority report Value for a package directory (Stage 8, phase 8h: `publish
/// --dry-run` embeds this summary in the index line). `None` if the package does not
/// check clean — an unpublishable package has no honest authority to advertise.
pub(crate) fn package_authority_value(dir: &str) -> Option<Json> {
    let pkg = load_package(dir);
    let program = check_program(&pkg);
    if errors(&program.diagnostics) > 0 {
        return None;
    }
    let name = program.entry_module.clone().unwrap_or_else(|| "package".to_string());
    let scopes = scopes_in_dir(std::path::Path::new(dir));
    Some(program_authority(&program, &name, &scopes))
}

/// `delulu authority <dir>`.
///
/// **A `delulu.toml` is what makes a directory a package, and that decides which loader runs.**
///
/// With a manifest, this resolves the whole dependency graph exactly as `build`, `check`, `lock` and
/// `authority --diff` do. It used to use the single-package loader instead, which cannot see a
/// dependency — so `authority` refused every package that had one with DL0303 ("unknown module")
/// while `build` on the same package succeeded (`HARDENING_CAMPAIGN.md` C51). In a monorepo that is
/// nearly every package, and "what does this dependency let my package do?" is the supply-chain
/// question this command exists to answer.
///
/// Without a manifest it stays on the single-package loader, deliberately: `resolve_workspace`
/// requires a manifest and reports DL1004 without one, so routing everything through it would have
/// refused a plain directory of modules — the case C26/D33 ruled legal and C50 explicitly preserved.
/// Trading C51 for that regression would have been no fix at all.
fn authority_package(dir: &str, opts: &Opts) -> i32 {
    if std::path::Path::new(dir).join("delulu.toml").is_file() {
        authority_workspace(dir, opts)
    } else {
        authority_loose_dir(dir, opts)
    }
}

/// The package path: resolve dependencies, check the whole graph, report the root's authority.
fn authority_workspace(dir: &str, opts: &Opts) -> i32 {
    let ws = resolve_workspace(dir);
    let program = check_workspace(&ws);
    // Both lists, for C50's reason: the workspace's own diagnostics carry an unreadable or
    // non-conforming manifest (DL1004/DL1001/…), and dropping them let the review surface assert
    // `summary.errors: 0` for a package `check` refuses.
    let mut diags: Vec<Diagnostic> = ws.diagnostics.clone();
    diags.extend(program.diagnostics.iter().cloned());
    if errors(&diags) > 0 {
        print_diagnostics("authority", &diags, &ws.source_map, None, opts.json);
        return 1;
    }
    // The entry module names the report when there is one. A library package has no `fn main`, and
    // the old fallback called it `package` — a placeholder that tells a reviewer nothing about what
    // they are reading. The root package's own name is right there once the graph is resolved.
    let name = program
        .entry_module
        .clone()
        .or_else(|| ws.root_pkg().map(|p| p.name.clone()))
        .unwrap_or_else(|| "package".to_string());
    emit_authority(&program, &name, dir, &ws.source_map, opts)
}

/// The loose-directory path: no manifest, so no dependency graph and nothing to resolve.
fn authority_loose_dir(dir: &str, opts: &Opts) -> i32 {
    let pkg = load_package(dir);
    let program = check_program(&pkg);
    let mut diags: Vec<Diagnostic> = pkg.diagnostics.clone();
    diags.extend(program.diagnostics.iter().cloned());
    if errors(&diags) > 0 {
        print_diagnostics("authority", &diags, &pkg.source_map, None, opts.json);
        return 1;
    }
    let name = program.entry_module.clone().unwrap_or_else(|| "package".to_string());
    emit_authority(&program, &name, dir, &pkg.source_map, opts)
}

/// The shared tail: compute the report, stamp the run-time facts, emit it on the caller's surface.
/// One function so the two loaders above cannot drift into reporting different things.
fn emit_authority(
    program: &delulu_check::Program,
    name: &str,
    dir: &str,
    map: &SourceMap,
    opts: &Opts,
) -> i32 {
    let scopes = scopes_in_dir(std::path::Path::new(dir));
    let mut report = program_authority(program, name, &scopes);
    stamp_custody(&mut report, opts);
    stamp_foreign_isolation(&mut report, opts);
    stamp_isolation(&mut report, opts);
    if opts.json {
        note_json_emitted();
        println!("{}", envelope_to_string("authority", &[], Some(report), map));
    } else {
        print!("{}", render_authority(&report));
    }
    0
}

// ----- authority --diff (§8: the supply-chain review surface) --------------

/// `delulu authority --diff <old.lock> <new.lock-or-dir>`: compare two authority *states*.
/// `old_path` is always a `delulu.lock`; `new_arg` is either another lockfile or a package
/// directory to resolve+check+lock fresh (so a maintainer can diff "what's locked" against
/// "what HEAD would lock" without writing a file first).
fn cmd_authority_diff(old_path: &str, new_arg: &str, opts: &Opts) -> i32 {
    let old_text = match std::fs::read_to_string(old_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read `{old_path}`: {e}");
            return 2;
        }
    };
    let old_lock = Lockfile::parse(&old_text);

    let new_lock = if std::path::Path::new(new_arg).is_dir() {
        let ws = resolve_workspace(new_arg);
        let program = check_workspace(&ws);
        let mut diags: Vec<Diagnostic> = ws.diagnostics.clone();
        diags.extend(program.diagnostics.iter().cloned());
        if errors(&diags) > 0 {
            print_diagnostics("authority", &diags, &ws.source_map, None, opts.json);
            return 1;
        }
        compute_lockfile(&ws, &program)
    } else {
        let text = match std::fs::read_to_string(new_arg) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: cannot read `{new_arg}`: {e}");
                return 2;
            }
        };
        Lockfile::parse(&text)
    };

    // Union of every package name mentioned on either side, so additions/removals surface too.
    let mut names: BTreeSet<String> = BTreeSet::new();
    names.extend(old_lock.packages.iter().map(|p| p.name.clone()));
    names.extend(new_lock.packages.iter().map(|p| p.name.clone()));

    let mut added_effects = serde_json::Map::new();
    let mut removed_effects = serde_json::Map::new();
    let mut added_scopes = serde_json::Map::new();
    let mut api_row_changes = serde_json::Map::new();
    let mut widened = false;

    for name in &names {
        // A package missing on one side diffs against an empty (`Default`) entry: a package
        // that disappeared loses authority (never a widening); a brand-new package's authority
        // is entirely "added".
        let old_e = old_lock.get(name).cloned().unwrap_or_default();
        let new_e = new_lock.get(name).cloned().unwrap_or_default();

        let old_fx: BTreeSet<&str> = old_e.effects.iter().map(String::as_str).collect();
        let new_fx: BTreeSet<&str> = new_e.effects.iter().map(String::as_str).collect();
        let added: Vec<&str> = new_fx.difference(&old_fx).copied().collect();
        let removed: Vec<&str> = old_fx.difference(&new_fx).copied().collect();
        if !added.is_empty() {
            added_effects.insert(name.clone(), json!(added));
        }
        if !removed.is_empty() {
            removed_effects.insert(name.clone(), json!(removed));
        }

        let mut scope_obj = serde_json::Map::new();
        added_scope_field(&mut scope_obj, "cap_kinds", &old_e.cap_kinds, &new_e.cap_kinds);
        added_scope_field(&mut scope_obj, "secrets", &old_e.secrets, &new_e.secrets);
        added_scope_field(&mut scope_obj, "net", &old_e.net, &new_e.net);
        added_scope_field(&mut scope_obj, "fs.read", &old_e.fs_read, &new_e.fs_read);
        added_scope_field(&mut scope_obj, "fs.write", &old_e.fs_write, &new_e.fs_write);
        if !scope_obj.is_empty() {
            added_scopes.insert(name.clone(), Json::Object(scope_obj));
        }

        if old_e.api_row_hash != new_e.api_row_hash {
            api_row_changes.insert(
                name.clone(),
                json!({ "old_hash": old_e.api_row_hash, "new_hash": new_e.api_row_hash }),
            );
        }

        // Reuses the same subset check the semver-authority law enforces (lockfile.rs) — a
        // package missing from one side compares against a `Default` (empty-authority) entry.
        if authority_widened(&old_e, &new_e) {
            widened = true;
        }
    }

    let verdict = if widened {
        "WIDENING — requires major version + --accept-authority"
    } else {
        "OK"
    };

    let report = json!({
        "added_effects": added_effects,
        "removed": removed_effects,
        "added_scopes": added_scopes,
        "api_row_changes": api_row_changes,
        "verdict": verdict,
    });

    if opts.json {
        note_json_emitted();
        println!("{}", serde_json::to_string_pretty(&report).expect("diff report serializes"));
    } else {
        println!("Authority diff: {old_path} -> {new_arg}");
        if added_effects.is_empty() && removed_effects.is_empty() && added_scopes.is_empty() && api_row_changes.is_empty() {
            println!("  (no authority or public-API changes)");
        } else {
            for (pkg, v) in &added_effects {
                println!("  {pkg}: + effects {}", strs(v).join(", "));
            }
            for (pkg, v) in &removed_effects {
                println!("  {pkg}: - effects {}", strs(v).join(", "));
            }
            for (pkg, v) in &added_scopes {
                println!("  {pkg}: + scopes {v}");
            }
            for (pkg, v) in &api_row_changes {
                println!(
                    "  {pkg}: api row changed ({} -> {})",
                    v["old_hash"].as_str().unwrap_or("?"),
                    v["new_hash"].as_str().unwrap_or("?")
                );
            }
        }
        println!("verdict: {verdict}");
    }

    0
}

fn added_scope_field(obj: &mut serde_json::Map<String, Json>, key: &str, old: &[String], new: &[String]) {
    let old_set: BTreeSet<&str> = old.iter().map(String::as_str).collect();
    let added: Vec<&str> = new.iter().map(String::as_str).filter(|s| !old_set.contains(s)).collect();
    if !added.is_empty() {
        obj.insert(key.to_string(), json!(added));
    }
}

// ----- why -------------------------------------------------------------------------------------

/// `delulu why <Effect> <file-or-dir>`: explain, at FUNCTION granularity, why a program can
/// perform an effect (§8). This is an honest approximation, not the spec's full "op with that
/// effect" story:
///
/// - It names the deepest *function* that originates the effect (the one where the effect is
///   introduced by a capability operation rather than by calling something else that has it),
///   not the specific call-site/operation within that function's body. `FnFacts` records a
///   function's row, not a per-statement trace, so finer granularity isn't available from these
///   facts without walking the body AST again.
/// - It reports exactly ONE origin path (the first one found, in `BTreeSet` — i.e. lexical —
///   callee order), not "all minimal paths, ≤ 10" as §8 literally asks for. A same-effect cycle
///   among callees is cut by the `visited` set so the walk always terminates.
///
/// Algorithm: resolve/check the program; if `main`'s declared row doesn't contain the effect,
/// report "cannot perform" (exit 0). Otherwise walk from `main`: at each step, look at the
/// current function's callees (resolved to their owning module via `call_owner`); descend into
/// the first one whose row still contains the effect; stop when none do — that function performs
/// it directly.
fn cmd_why(rest: &[String]) -> i32 {
    // `why` takes two positionals (`<Effect> <file-or-dir>`) where every other command takes one;
    // peel the effect name off the front and hand the rest to the shared `parse_opts`.
    let mut effect_name: Option<String> = None;
    let mut remainder: Vec<String> = Vec::new();
    for a in rest {
        if effect_name.is_none() && !a.starts_with('-') {
            effect_name = Some(a.clone());
        } else {
            remainder.push(a.clone());
        }
    }
    let Some(effect_name) = effect_name else {
        eprintln!("error: `why` needs an effect name and a file or package directory, e.g. `delulu why Net examples/greeter`");
        return 2;
    };
    let (path, opts) = parse_opts(&remainder);
    let Some(path) = path else {
        eprintln!("error: `why` needs a file or package directory");
        return 2;
    };

    // A `.dpx` plugin artifact: traverse the plugin's OWN authority chain (criterion 10, spec §6).
    // A Verified plugin's DIR carries real per-function facts, so `why` follows the chain to the
    // primitive op; a Contained plugin is an opaque boundary and is labeled as such.
    if std::path::Path::new(&path).extension().and_then(|e| e.to_str()) == Some("dpx") {
        return cmd_why_plugin(&effect_name, &path, &opts);
    }

    let (diags, program, locations, map): (Vec<Diagnostic>, Program, HashMap<String, (String, u32)>, SourceMap) =
        if std::path::Path::new(&path).is_dir() {
            let ws = resolve_workspace(&path);
            let program = check_workspace(&ws);
            let mut diags: Vec<Diagnostic> = ws.diagnostics.clone();
            diags.extend(program.diagnostics.iter().cloned());
            let locations = locations_workspace(&ws);
            (diags, program, locations, ws.source_map)
        } else {
            let (map, id, src) = match load(&path) {
                Ok(x) => x,
                Err(c) => return c,
            };
            let checked = check_source(id, &src);
            let locations = locations_single(&checked, &map);
            let diags = checked.diagnostics.clone();
            let program = synth_single_program(&checked);
            (diags, program, locations, map)
        };

    if errors(&diags) > 0 {
        print_diagnostics("why", &diags, &map, None, opts.json);
        return 1;
    }

    // A "known" effect is a core primitive, or a user-declared `effect` that appears anywhere in
    // this program's facts (reachable or not — a program can ask "why" about a dead branch too).
    let is_known = Effect::core_from_name(&effect_name).is_some()
        || program.facts.values().any(|f| f.effects.iter().any(|e| e.name() == effect_name));
    if !is_known {
        eprintln!(
            // Derived from CORE_EFFECT_NAMES, never restated. The hand-written list that stood
            // here named seven effects while the checker accepted ten, so a reader who mistyped
            // one was told a set omitting `Load`, `Async` and `Actuate` — and could not discover
            // them from the error that exists to teach them.
            "error: `{effect_name}` is not a known effect (core effects: {}; \
             or a user-declared `effect` visible in this program)",
            delulu_check::check::CORE_EFFECT_NAMES.join(", ")
        );
        return 2;
    }

    let Some(entry_mod) = program.entry_module.clone() else {
        eprintln!("error: `why` needs an executable entry point (`fn main`); none was found in `{path}`");
        return 2;
    };
    let main_key = format!("{entry_mod}::main");
    let Some(main_facts) = program.facts.get(&main_key) else {
        eprintln!("error: no `main` function found in `{path}`");
        return 2;
    };

    if !main_facts.effects.iter().any(|e| e.name() == effect_name) {
        if opts.json {
            note_json_emitted();
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "effect": effect_name, "performs": false, "path": [] }))
                    .expect("why report serializes")
            );
        } else {
            println!("program cannot perform `{effect_name}`");
        }
        return 0;
    }

    // ----- the origin walk ---------------------------------------------------------------------
    let mut path_nodes = vec![main_key.clone()];
    let mut visited: std::collections::HashSet<String> = std::iter::once(main_key.clone()).collect();
    let mut current = main_key;
    loop {
        let Some((cur_mod, _)) = current.split_once("::") else { break };
        let Some(facts) = program.facts.get(&current) else { break };
        let mut next = None;
        for callee in &facts.callees {
            let Some(owner) = program.call_owner.get(cur_mod).and_then(|o| o.get(callee)) else { continue };
            let key = format!("{owner}::{callee}");
            if visited.contains(&key) {
                continue;
            }
            if let Some(cf) = program.facts.get(&key) {
                if cf.effects.iter().any(|e| e.name() == effect_name) {
                    next = Some(key);
                    break;
                }
            }
        }
        match next {
            Some(k) => {
                visited.insert(k.clone());
                path_nodes.push(k.clone());
                current = k;
            }
            None => break,
        }
    }

    if opts.json {
        note_json_emitted();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({ "effect": effect_name, "performs": true, "path": path_nodes }))
                .expect("why report serializes")
        );
    } else {
        let mut out = String::new();
        for (i, key) in path_nodes.iter().enumerate() {
            let fname = key.split_once("::").map(|(_, f)| f).unwrap_or(key.as_str());
            if i > 0 {
                out.push_str(" -> ");
            }
            out.push_str(fname);
            if let Some((file, line)) = locations.get(key) {
                out.push_str(&format!(" ({file}:{line})"));
            }
        }
        out.push_str(&format!(" — {effect_name}"));
        println!("{out}");
    }
    0
}

/// `delulu why <Effect> <file.dpx>` (criterion 10, spec §6): traverse a plugin artifact's own
/// authority chain. A **Verified** plugin's DIR carries real per-function facts, so `why` follows
/// the chain from an export to the primitive op that performs the effect (a real chain). A
/// **Contained** plugin is opaque — `why` stops at the module boundary and labels the edge
/// `→ [contained plugin <name>] — <Effect>` (spec §6), because containment is module-granular and
/// the internals are not re-checkable.
fn cmd_why_plugin(effect_name: &str, path: &str, opts: &Opts) -> i32 {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read `{path}`: {e}");
            return 2;
        }
    };
    let map = SourceMap::new();
    let dpx = match delulu_wasm::read_dpx(&bytes) {
        Ok(d) => d,
        Err(e) => {
            print_diagnostics("why", &[Diagnostic::error(e.code(), e.message())], &map, None, opts.json);
            return 1;
        }
    };
    let name = dpx.manifest.get("name").and_then(|v| v.as_str()).unwrap_or("<unnamed>").to_string();

    if dpx.class == "contained" {
        // The opaque boundary. R-1: a Contained plugin's exports type at effects(grant); its ceiling
        // bounds what it may ever perform. We cannot see inside — we label the boundary honestly.
        let ceiling: Vec<String> = dpx
            .manifest
            .get("authority")
            .and_then(|a| a.get("effects"))
            .and_then(|v| v.as_array())
            .map(|xs| xs.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
            .unwrap_or_default();
        let declares = ceiling.iter().any(|e| e == effect_name);
        if opts.json {
            note_json_emitted();
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "effect": effect_name, "plugin": name, "class": "contained",
                    "boundary": true, "declares": declares,
                    "edge": if declares { format!("→ [contained plugin {name}] — {effect_name}") } else { Json::Null.to_string() },
                }))
                .expect("why report serializes")
            );
        } else if declares {
            // The spec §6 labeled edge, verbatim shape.
            println!("→ [contained plugin {name}] — {effect_name}");
            println!("  (opaque module: containment is module-granular; internals are not re-checkable)");
        } else {
            println!("contained plugin `{name}` does not declare `{effect_name}` (its authority ceiling omits it)");
        }
        return 0;
    }

    // Verified: replay the DIR's own facts and follow the real chain to the primitive op.
    let Some(dir_bytes) = dpx.dir.as_deref() else {
        eprintln!("error: verified plugin `{name}` carries no DIR");
        return 1;
    };
    let dir = match delulu_check::dir_deserialize(dir_bytes) {
        Ok(d) => d,
        Err(e) => {
            print_diagnostics("why", &[Diagnostic::error(e.code(), e.message())], &map, None, opts.json);
            return 1;
        }
    };

    let is_known = delulu_check::Effect::core_from_name(effect_name).is_some()
        || dir.facts.values().any(|f| f.effects.iter().any(|e| e.name() == effect_name));
    if !is_known {
        eprintln!("error: `{effect_name}` is not a known effect visible in plugin `{name}`");
        return 2;
    }

    // Start from an EXPORT that performs the effect (the plugin's real entry points).
    let exports: Vec<String> = dpx
        .manifest
        .get("exports")
        .and_then(|v| v.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    let start = exports
        .iter()
        .find(|ex| dir.facts.get(*ex).is_some_and(|f| f.effects.iter().any(|e| e.name() == effect_name)));
    let Some(start) = start else {
        if opts.json {
            note_json_emitted();
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({ "effect": effect_name, "plugin": name, "class": "verified", "performs": false, "path": [] }))
                    .expect("why report serializes")
            );
        } else {
            println!("verified plugin `{name}` has no export that performs `{effect_name}`");
        }
        return 0;
    };

    let chain = plugin_why_chain(&dir.facts, start, effect_name);
    if opts.json {
        note_json_emitted();
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({ "effect": effect_name, "plugin": name, "class": "verified", "performs": true, "path": chain }))
                .expect("why report serializes")
        );
    } else {
        println!("[verified plugin {name}] {} — {effect_name}", chain.join(" -> "));
    }
    0
}

/// Follow a Verified plugin's real call chain over its DIR facts from `start` to the primitive op:
/// at each step, descend into a callee that also performs the effect; when none does, the current
/// function is the origin (it performs the primitive itself). Cycle-safe via a visited set.
fn plugin_why_chain(
    facts: &std::collections::BTreeMap<String, delulu_check::FnFacts>,
    start: &str,
    effect: &str,
) -> Vec<String> {
    let mut chain = vec![start.to_string()];
    let mut visited: std::collections::HashSet<String> = std::iter::once(start.to_string()).collect();
    let mut current = start.to_string();
    while let Some(f) = facts.get(&current) {
        let mut next = None;
        for callee in &f.callees {
            if visited.contains(callee) {
                continue;
            }
            if facts.get(callee).is_some_and(|cf| cf.effects.iter().any(|e| e.name() == effect)) {
                next = Some(callee.clone());
                break;
            }
        }
        match next {
            Some(k) => {
                visited.insert(k.clone());
                chain.push(k.clone());
                current = k;
            }
            None => break,
        }
    }
    chain
}

/// Function-declaration locations (display file + 1-based line) across a whole workspace,
/// keyed `"module::fn"` — the same key shape as `Program::facts`.
fn locations_workspace(ws: &Workspace) -> HashMap<String, (String, u32)> {
    let mut out = HashMap::new();
    for wm in &ws.modules {
        for item in &wm.unit.module.items {
            if let Item::Fn(f) = item {
                let (line, _col) = ws.source_map.position(f.name.span.file, f.name.span.start);
                out.insert(format!("{}::{}", wm.unit.name, f.name.name), (ws.source_map.name(f.name.span.file).to_string(), line));
            }
        }
    }
    out
}

/// Same as [`locations_workspace`], for a single checked file (one implicit module).
fn locations_single(checked: &Checked, map: &SourceMap) -> HashMap<String, (String, u32)> {
    let mod_name = checked.module.name.dotted();
    let mut out = HashMap::new();
    for item in &checked.module.items {
        match item {
            Item::Fn(f) => {
                let (line, _col) = map.position(f.name.span.file, f.name.span.start);
                out.insert(format!("{mod_name}::{}", f.name.name), (map.name(f.name.span.file).to_string(), line));
            }
            // Stage 7: actor members locate for the causal why-chain (criterion 5).
            Item::Actor(a) => {
                let aname = &a.name.name;
                let (cl, _) = map.position(a.ctor.span.file, a.ctor.span.start);
                out.insert(format!("{mod_name}::{aname}.new"), (map.name(a.ctor.span.file).to_string(), cl));
                for b in &a.behaviors {
                    let (line, _col) = map.position(b.name.span.file, b.name.span.start);
                    out.insert(format!("{mod_name}::{aname}.{}", b.name.name), (map.name(b.name.span.file).to_string(), line));
                }
                for f in &a.fns {
                    let (line, _col) = map.position(f.name.span.file, f.name.span.start);
                    out.insert(format!("{mod_name}::{aname}.{}", f.name.name), (map.name(f.name.span.file).to_string(), line));
                }
            }
            _ => {}
        }
    }
    out
}

/// Build a `Program`-shaped view of one checked file so `why` can walk it exactly like a
/// multi-module package. There is no cross-module structure for a lone file — the whole file is
/// treated as its own module (named by its `module` header), and `call_owner` maps every
/// declared top-level function back to that one module. `Program`'s fields are all `pub`, so this
/// is a plain struct literal — no new API surface needed in `delulu-check`.
fn synth_single_program(checked: &Checked) -> Program {
    let mod_name = checked.module.name.dotted();
    let mut facts = HashMap::new();
    let mut fn_types = HashMap::new();
    for (name, f) in &checked.result.facts {
        facts.insert(format!("{mod_name}::{name}"), f.clone());
    }
    for (name, t) in &checked.result.fn_types {
        fn_types.insert(format!("{mod_name}::{name}"), t.clone());
    }
    let mut owner = HashMap::new();
    for name in checked.table.fns.keys() {
        owner.insert(name.clone(), mod_name.clone());
    }
    // Stage 7: spawn/send callee edges (`Counter.new`, `Counter.add`) resolve to this module
    // too, so the why-chain crosses the actor boundary (criterion 5).
    for (aname, adef) in &checked.table.actors {
        owner.insert(format!("{aname}.new"), mod_name.clone());
        for b in &adef.behaviors {
            owner.insert(format!("{aname}.{}", b.name), mod_name.clone());
        }
        for f in &adef.fns {
            owner.insert(format!("{aname}.{}", f.name), mod_name.clone());
        }
    }
    let mut call_owner = HashMap::new();
    call_owner.insert(mod_name.clone(), owner);
    let entry_module = if checked.result.main_present { Some(mod_name) } else { None };
    Program {
        diagnostics: checked.diagnostics.clone(),
        facts,
        fn_types,
        call_owner,
        entry_module,
        type_names: checked.table.type_names(),
    }
}

// ----- run -----------------------------------------------------------------

/// Map a WASM host refusal message to the diagnostic code it carries. The host tags refusals with
/// their code inline (DL0703 denied root slice, DL0903 out-of-bounds memory access, DL140x custody
/// denials from the Stage-5 broker seam); anything else is a capability-scope violation (DL0904).
fn wasm_fault_code(msg: &str) -> &'static str {
    // The deterministic arithmetic/recursion traps (DL0901/0902/0903/0905) are mapped from the
    // structured wasmtime trap by `delulu_wasm::clean_trap` and embedded here, so the WASM engine
    // reports the SAME code as the interpreter for the same fault (parity, invariant 15; C20). The
    // DL07xx/DL13xx/DL14xx entries are host-side capability/custody denials whose reason strings
    // already carry the code.
    for code in [
        "DL0703", "DL0901", "DL0902", "DL0903", "DL0905", "DL1306", "DL1401", "DL1402", "DL1403",
    ] {
        if msg.contains(code) {
            // The registry stores codes as &'static str; return the matching literal.
            return match code {
                "DL0703" => "DL0703",
                "DL0901" => "DL0901",
                "DL0902" => "DL0902",
                "DL0903" => "DL0903",
                "DL0905" => "DL0905",
                "DL1306" => "DL1306",
                "DL1401" => "DL1401",
                "DL1402" => "DL1402",
                _ => "DL1403",
            };
        }
    }
    "DL0904"
}

/// Map this run's grants to the root-node authority the daemon issues (Stage 5 phase 5f: the
/// `--grant` flags in daemon mode are sugar for issue-then-run at the root, spec §3.2). Paths are
/// the SAME absolute, lexically-normalized strings the embedded `RootVal` carries, so the broker's
/// path lattice sees exactly what the runtime resolves against.
/// Mint one grant node per actuator device, under this run's node, and return the e-stop probe
/// that watches them (10g, spec §5.2; addendum §2.4's "revoking one robot kills exactly its
/// subtree").
///
/// Each child carries `{Actuate}` and nothing else — strictly less than the parent, which is what
/// makes it an attenuation rather than a second grant. The device's envelope is NOT re-encoded
/// here: the envelope lives in the device broker, where it is enforced per command, and copying it
/// into the node would create a second authority of record that could drift from the first.
///
/// The probe asked once per watchdog tick, per device. **Every non-`live` answer is death,
/// including no answer at all.** A revoked node, an expired one, a daemon that stopped responding,
/// a reply in a shape we do not recognise — the run cannot distinguish "your authority was
/// withdrawn" from "you can no longer confirm you have any", and for a process holding a machine
/// those are the same situation. The consequence is deliberate and worth stating plainly: a broker
/// outage parks the arm. That is the direction to fail in.
fn mint_device_nodes(
    c: &mut crate::broker_client::BrokerClientCustody,
    actuators: &[delulu_runtime::value::ActuatorEnvelope],
) -> Result<(delulu_runtime::AuthorityProbe, Vec<String>), delulu_runtime::CustodyDenial> {
    use delulu_runtime::{AuthorityState, Custody};
    let dir = c.state_dir().to_path_buf();
    let mut nodes: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    for env in actuators {
        let mut authority = delulu_broker::Authority::default();
        authority.effects.insert(delulu_check::Effect::Actuate);
        let holder = delulu_broker::Holder::new("device", &env.device, "");
        let child = c.attenuate(authority, holder)?;
        nodes.insert(env.device.clone(), child.as_str().to_string());
    }
    let ids: Vec<String> = nodes.values().cloned().collect();
    Ok((Box::new(move |device: &str| {
        // A device with no node of its own is not a device this run may command. Unknown is dead,
        // like every other unanswerable case.
        let Some(node) = nodes.get(device) else {
            return AuthorityState::Dead(format!("`{device}` has no grant node in this run"));
        };
        let req = crate::broker_ipc::ReqBody::NodeState { node: node.clone() };
        match crate::brokerd::request(&dir, req) {
            Ok(crate::broker_ipc::Response::NodeState { state, by_seq, .. }) => {
                if state == "live" {
                    AuthorityState::Live
                } else {
                    AuthorityState::Dead(match by_seq {
                        Some(seq) => format!("grant node `{node}` is {state} (by audit seq {seq})"),
                        None => format!("grant node `{node}` is {state}"),
                    })
                }
            }
            Ok(other) => AuthorityState::Dead(format!(
                "the broker answered the liveness of `{node}` with {other:?}, which is not an answer"
            )),
            Err(e) => AuthorityState::Dead(format!("the broker did not answer for `{node}`: {e}")),
        }
    }), ids))
}

/// Revoke this run's device nodes as it exits (10g). Best-effort and silent: the run is over and
/// the devices are already released; failing to tidy the tree must not change the run's verdict.
///
/// Why this exists at all — it was found by a measurement harness, not by reasoning. A run's device
/// nodes were staying `[live]` after the process was gone, so `delulu grants list` showed arms
/// nobody held. An operator aiming an e-stop at one of those ghosts gets `ok: revoked 1 node(s)`
/// and a machine that keeps moving. **An e-stop that reports success without stopping anything is
/// worse than no e-stop**, because it ends the search for the real one.
///
/// Note the residual, which this does not fix and does not hide: the run's OWN node still outlives
/// it (Stage 5 behaviour, shared with every other run and relied on by `run --lease`). So
/// `grants list` after a run still shows a `(process)` node — but no longer a `(device)` one, and
/// the device is what an e-stop is aimed at.
fn revoke_device_nodes(state_dir: &std::path::Path, ids: &[String]) {
    for id in ids {
        let req = crate::broker_ipc::ReqBody::Revoke { caller: id.clone(), target: id.clone() };
        let _ = crate::brokerd::request(state_dir, req);
    }
}

/// Render a runtime [`ActuatorEnvelope`] into the canonical device-grant string the broker parses
/// (RFC 0001 F1).
///
/// This string is the contract between two crates that cannot share a type: `delulu-broker` depends
/// only on `delulu-diag`/`delulu-check` (crate ruling 1), so it cannot see `ActuatorEnvelope`, and
/// the runtime does not depend on the broker's `DeviceScope`. Two parsers for one grammar is a
/// drift risk, and the mitigation is mechanical rather than cultural:
/// `device_grant_strings_round_trip` in `tests/actuate_cli.rs` renders every envelope this function
/// can produce, parses it back with `delulu_broker::device_scope::parse`, and compares field by
/// field. If the grammars ever diverge, that test fails rather than a device silently losing a
/// bound.
///
/// Dimensions are emitted in sorted order so the string is byte-canonical regardless of the order
/// the human wrote them at the prompt.
fn device_grant_string(env: &delulu_runtime::value::ActuatorEnvelope) -> String {
    let mut dims: Vec<(&str, f64, f64)> =
        env.dims.iter().map(|(d, lo, hi)| (d.as_str(), *lo, *hi)).collect();
    dims.sort_by(|a, b| a.0.cmp(b.0));
    let mut parts: Vec<String> = dims.iter().map(|(d, lo, hi)| format!("{d}={lo}..{hi}")).collect();
    if let Some(hz) = env.rate_hz {
        parts.push(format!("rate_hz={hz}"));
    }
    parts.push(format!("heartbeat_ms={}", env.heartbeat_ms));
    parts.push(format!("ttl_ms={}", env.ttl_ms));
    parts.push(format!("fail={}", env.fail_state.name()));
    format!("{}:{}", env.device, parts.join(","))
}

fn authority_spec_from_grants(grants: &Grants, program: &str) -> crate::broker_ipc::AuthoritySpec {
    let root = grants.build_root();
    let mut effects: BTreeSet<&'static str> = BTreeSet::new();
    if root.console || !root.fs_write.is_empty() {
        effects.insert("Write");
    }
    if !root.fs_read.is_empty() {
        effects.insert("Read");
    }
    if !root.net.is_empty() {
        effects.insert("Net");
    }
    if root.clock {
        effects.insert("Clock");
    }
    if root.rand {
        effects.insert("Rand");
    }
    if root.declassify {
        effects.insert("Declassify");
    }
    if root.foreign_load {
        effects.insert("ForeignCall");
    }
    // 10g: an actuator grant puts `Actuate` in the NODE's authority, which is what lets the grant
    // tree carry it — attenuate it to a child, show it in `delulu authority`, and above all revoke
    // it. Sensor grants deliberately add nothing: a sensor read is `Read` with a sensor scope
    // (spec §5.1), and minting `Read` here would hand the node a *file-reading* effect it was
    // never granted — the fs scope would still be empty, but an effect nobody asked for is exactly
    // the widening this line exists to avoid.
    if !grants.actuators.is_empty() {
        effects.insert("Actuate");
    }
    // RFC 0001 F1 (D12e): the envelope now travels WITH the effect. Before this, the node carried
    // `Actuate` and nothing else, so the broker could answer only "may this node command anything?"
    // — and `validate.rs` accepted whatever device path arrived. The grant tree can now say which
    // device, and how far.
    let mut device: Vec<String> = grants.actuators.iter().map(device_grant_string).collect();
    device.sort();
    let mut secret_names: Vec<String> = grants.secrets.keys().cloned().collect();
    secret_names.sort();
    crate::broker_ipc::AuthoritySpec {
        effects: effects.into_iter().map(str::to_string).collect(),
        fs_read: root.fs_read.iter().map(|p| p.to_string_lossy().to_string()).collect(),
        fs_write: root.fs_write.iter().map(|p| p.to_string_lossy().to_string()).collect(),
        net: root.net.clone(),
        secrets: secret_names,
        declassify: Vec::new(),
        foreign_c: {
            let mut libs: Vec<String> = grants.foreign_c.keys().cloned().collect();
            libs.sort();
            libs
        },
        foreign_python: grants.foreign_python.clone(),
        device,
        holder_kind: "process".to_string(),
        holder_desc: program.to_string(),
        ttl_millis: None,
    }
}

/// The inverse of [`authority_spec_from_grants`], for `--lease` runs (phase 5j): derive the run's
/// local `Grants` from the delegated node's authority, so the in-process Stage 1–4 scope checks see
/// exactly the leased slice (the broker's per-§4 checks enforce it regardless — this keeps the two
/// layers agreeing instead of the local layer being wider). `foreign_c` binary PATHS are grant data
/// a human supplies via `--grant foreign.c=LIB:PATH`; the lease's `foreign.c` scope bounds which
/// lib NAMES the broker will actually bind.
fn grants_from_lease(info: &crate::broker_ipc::NodeInfo, foreign_c: HashMap<String, String>) -> Grants {
    let has = |e: &str| info.effects.iter().any(|x| x == e);
    Grants {
        // Console is Write-effect-gated at the broker (spec §4.1: Console requires Write).
        console: has("Write"),
        fs_read: info.fs_read.clone(),
        fs_write: info.fs_write.clone(),
        net: info.net.clone(),
        clock: has("Clock"),
        rand: has("Rand"),
        declassify: has("Declassify"),
        // Daemon mode strips local secret VALUES and keeps only broker-handle names (invariant 23);
        // the names come from the lease's `secrets` scope.
        secrets: info.secrets.iter().map(|n| (n.clone(), String::new())).collect(),
        foreign_c,
        foreign_python: info.foreign_python.clone(),
        // A lease can never confer native emission in v1.x: the broker has no `exec.native`
        // dimension yet (build-order D6 — the lattice dimension lands WITH the first native
        // tier in 10l, never after it), so the leased slice is always denied here. Fail closed.
        exec_native: false,
        // RFC 0001 F1 (D12e): a lease now DOES confer physical authority — but only the authority
        // the delegating side actually bounded. Each entry is a full envelope minted by whoever
        // delegated this node, `⊑`-checked at every hop of the tree on the way down, so a holder
        // receives a corridor rather than a licence. This is the mechanism the UAS lost-link pattern
        // needs (addendum §2.2): the mission grant and the lost-link grant are two delegations of
        // the same device, the second a strict attenuation of the first.
        //
        // Skip branch: an entry that does not parse is DROPPED, and dropping is the safe direction —
        // the actuator is then absent, so `root.actuator(…)` refuses at the mint (DL0703) before any
        // command is issued. A device this side cannot read the bounds of is a device it must not
        // drive.
        actuators: info
            .device
            .iter()
            .filter_map(|s| delulu_runtime::value::ActuatorEnvelope::parse(s).ok())
            .collect(),
        // Sensors are NOT covered by F1 and the distinction is real, not an oversight: a sensor read
        // is `Read` under a sensor scope (spec §5.1), and `Scopes` has no sensor dimension. A lease
        // still confers no sensor, and `--lease` still refuses a local `--grant sensor=` by name.
        sensors: Vec::new(),
        // And the same for accelerators (10h). The reason is sharper here than the note above:
        // 10g established that a grant node cannot express a device ENVELOPE at all (D12e), so a
        // lease could only ever confer "some compute, bounds unknown" — which is not a bound.
        // Fail closed, and `--lease` refuses a local compute grant by name rather than dropping it.
        computes: Vec::new(),
    }
}

/// `delulu secrets set NAME VALUE | list [--state-dir DIR]` (Stage 5 phase 5g, minimal v0.5 form —
/// full CLI polish is a later chunk). Writes the broker's secret store directly; a daemon started
/// AFTER the write sees the secret (the daemon loads the store at startup — restart to pick up new
/// names; flagged as a v0.5 limitation).
fn cmd_secrets(rest: &[String]) -> i32 {
    let Some(sub) = rest.first().map(String::as_str) else {
        eprintln!("error: `secrets` needs a subcommand: set NAME VALUE | list [--state-dir DIR]");
        return 2;
    };
    let state_flag = rest.iter().position(|a| a == "--state-dir").and_then(|i| rest.get(i + 1)).cloned();
    let Some(state_dir) = crate::brokerd::resolve_state_dir(state_flag.as_deref()) else {
        eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE) — pass --state-dir DIR");
        return 2;
    };
    let store = delulu_broker::SecretStore::load(state_dir.join("secrets.json"));
    match sub {
        "set" => {
            // Positionals = everything after `set` minus `--state-dir <DIR>` and other flags.
            let mut positionals: Vec<&String> = Vec::new();
            let mut i = 1;
            while i < rest.len() {
                match rest[i].as_str() {
                    "--state-dir" => i += 1, // skip its value too
                    s if s.starts_with("--") => {}
                    _ => positionals.push(&rest[i]),
                }
                i += 1;
            }
            let (Some(name), Some(value)) = (positionals.first(), positionals.get(1)) else {
                eprintln!("error: `secrets set` needs NAME and VALUE");
                return 2;
            };
            match store.set(name.as_str(), value.as_str()) {
                Ok(()) => {
                    ok_line!("ok: secret `{name}` stored (broker-resident; bytes enter a program only on `expose`)");
                    eprintln!("note: a running broker daemon loads the store at startup — restart it to pick up new names");
                    0
                }
                Err(e) => {
                    eprintln!("error: cannot write the secret store: {e}");
                    2
                }
            }
        }
        "list" => {
            for name in store.names() {
                println!("{name}");
            }
            0
        }
        other => {
            eprintln!("error: unknown secrets subcommand `{other}` (set | list)");
            2
        }
    }
}

// ----- grants (Stage 5 phase 5j: the grant-tree CLI surface, spec §3.2) --------------------------
//
// Every verb talks to the RUNNING daemon over `broker/1` — there is no local fallback: with the
// daemon down each verb fails DL1401 carrying the exact start command (invariant 27, playbook trap
// 4). The CLI is the human at the top of the tree (spec §3.1): `delegate` with no `--parent` first
// issues a root node holding exactly the requested authority — the typed command line IS the human
// action (the same footing as the `--grant` issue-then-run sugar) — then delegates under it.

/// Parse a TTL duration like `500ms`, `90s`, `30m`, `1h`, `2d` into a millisecond count.
fn parse_ttl_millis(s: &str) -> Option<i64> {
    let (num, mult) = if let Some(n) = s.strip_suffix("ms") {
        (n, 1)
    } else if let Some(n) = s.strip_suffix('s') {
        (n, 1_000)
    } else if let Some(n) = s.strip_suffix('m') {
        (n, 60_000)
    } else if let Some(n) = s.strip_suffix('h') {
        (n, 3_600_000)
    } else if let Some(n) = s.strip_suffix('d') {
        (n, 86_400_000)
    } else {
        return None;
    };
    let v: i64 = num.parse().ok()?;
    if v <= 0 {
        return None;
    }
    v.checked_mul(mult)
}

/// Wall-clock "now" in epoch millis — the CLI side of an absolute TTL deadline (the broker's own
/// clock does the enforcement; spec §4.3).
fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// One daemon round-trip for a `grants` verb. Fail closed (invariant 27): an unreachable daemon
/// prints DL1401 with the exact start command and yields `Err(1)`; a daemon `Error` reply prints
/// its diagnostic (e.g. a DL0802 whose message carries the computed intersection) and yields
/// `Err(1)`.
fn grants_rpc(
    state_dir: &std::path::Path,
    body: crate::broker_ipc::ReqBody,
    json: bool,
) -> Result<crate::broker_ipc::Response, i32> {
    let map = SourceMap::new();
    match crate::brokerd::request(state_dir, body) {
        Ok(crate::broker_ipc::Response::Error { code, message, .. }) => {
            let d = Diagnostic::error(crate::broker_client::static_code(&code), message);
            print_diagnostics("grants", &[d], &map, None, json);
            Err(1)
        }
        Ok(resp) => Ok(resp),
        Err(e) => {
            let d = Diagnostic::error(
                "DL1401",
                format!(
                    "broker unreachable: {e} — start it with `delulu broker start` \
                     (fail closed, invariant 27: `grants` verbs never fall back to local state)"
                ),
            );
            print_diagnostics("grants", &[d], &map, None, json);
            Err(1)
        }
    }
}

/// One wire node as a compact human line (`grants list`). Holder fields are display DATA — never a
/// decision input (criterion 9).
fn render_node_line(n: &crate::broker_ipc::NodeInfo) -> String {
    let authority = crate::brokerd::spec_to_authority(&n.authority_spec()).render_compact();
    let state = match (n.state.as_str(), n.by_seq) {
        ("revoked", Some(seq)) => format!("revoked@{seq}"),
        (s, _) => s.to_string(),
    };
    let parent = n.parent.as_deref().unwrap_or("-");
    format!("{}  [{}]  parent={}  {}  ({}) {}", n.id, state, parent, authority, n.holder_kind, n.holder_desc)
}

fn cmd_grants(rest: &[String]) -> i32 {
    let Some(sub) = rest.first().map(String::as_str) else {
        eprintln!(
            "error: `grants` needs a subcommand: list | tree | inspect <g_ID> | revoke <g_ID> | \
             certify | adopt | pubkey | \
             delegate [--parent g_ID] --effects E,.. [--fs-read P].. [--device DEV:..] [--ttl 1h] [--multi]"
        );
        return 2;
    };
    let args = &rest[1..];
    let json = args.iter().any(|a| a == "--json");
    let state_flag = args
        .iter()
        .position(|a| a == "--state-dir")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .or_else(|| args.iter().find_map(|a| a.strip_prefix("--state-dir=").map(str::to_string)));
    let Some(state_dir) = crate::brokerd::resolve_state_dir(state_flag.as_deref()) else {
        eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE) — pass --state-dir DIR");
        return 2;
    };
    // The first positional after the subcommand (`inspect`/`revoke` take a node id). `--state-dir`
    // is the only value-taking flag these verbs accept, so skip it and its value.
    let node_arg = {
        let mut found = None;
        let mut skip = false;
        for a in args {
            if skip {
                skip = false;
                continue;
            }
            if a == "--state-dir" {
                skip = true;
                continue;
            }
            if a.starts_with("--") {
                continue;
            }
            found = Some(a.clone());
            break;
        }
        found
    };

    match sub {
        "list" => {
            let resp = match grants_rpc(&state_dir, crate::broker_ipc::ReqBody::List, json) {
                Ok(r) => r,
                Err(c) => return c,
            };
            let crate::broker_ipc::Response::Listed { nodes } = resp else {
                eprintln!("error: unexpected list response: {resp:?}");
                return 2;
            };
            if json {
                let arr: Vec<Json> = nodes.iter().map(|n| serde_json::to_value(n).expect("node serializes")).collect();
                note_json_emitted();
                println!(
                    "{}",
                    json!({ "command": "grants", "subcommand": "list", "count": nodes.len(), "nodes": arr })
                );
            } else if nodes.is_empty() {
                println!("(no grants — the tree is empty)");
            } else {
                let palette = palette_stdout();
                for n in &nodes {
                    println!("{}", colorize_node_ids(&render_node_line(n), &palette));
                }
            }
            0
        }
        "tree" => {
            let resp = match grants_rpc(&state_dir, crate::broker_ipc::ReqBody::Tree, json) {
                Ok(r) => r,
                Err(c) => return c,
            };
            let crate::broker_ipc::Response::Tree { text } = resp else {
                eprintln!("error: unexpected tree response: {resp:?}");
                return 2;
            };
            if json {
                note_json_emitted();
                println!("{}", json!({ "command": "grants", "subcommand": "tree", "tree": text }));
            } else if text.is_empty() {
                println!("(no grants — the tree is empty)");
            } else {
                print!("{}", colorize_node_ids(&text, &palette_stdout()));
            }
            0
        }
        "inspect" => {
            let Some(id) = node_arg else {
                eprintln!("error: `grants inspect` needs a node id (g_…)");
                return 2;
            };
            let resp = match grants_rpc(&state_dir, crate::broker_ipc::ReqBody::Inspect { node: id }, json) {
                Ok(r) => r,
                Err(c) => return c,
            };
            let crate::broker_ipc::Response::Inspected { node: n } = resp else {
                eprintln!("error: unexpected inspect response: {resp:?}");
                return 2;
            };
            if json {
                note_json_emitted();
                println!(
                    "{}",
                    json!({ "command": "grants", "subcommand": "inspect", "node": serde_json::to_value(&n).expect("node serializes") })
                );
            } else {
                println!("id:        {}", n.id);
                println!("parent:    {}", n.parent.as_deref().unwrap_or("(root)"));
                // Display only — the holder is data, never a decision input (criterion 9).
                println!("holder:    {} — {} [{}]", n.holder_kind, n.holder_desc, n.holder_peer);
                match (n.state.as_str(), n.by_seq) {
                    ("revoked", Some(seq)) => println!("state:     revoked (by audit seq {seq})"),
                    (s, _) => println!("state:     {s}"),
                }
                match n.ttl_millis {
                    Some(ttl) => println!("ttl:       {}", delulu_broker::render_ts_utc(ttl)),
                    None => println!("ttl:       (none)"),
                }
                println!("created:   {}", delulu_broker::render_ts_utc(n.created_millis));
                println!("audit_seq: {}", n.audit_seq);
                println!("authority: {}", crate::brokerd::spec_to_authority(&n.authority_spec()).render_compact());
            }
            0
        }
        "revoke" => {
            let Some(id) = node_arg else {
                eprintln!("error: `grants revoke` needs a node id (g_…)");
                return 2;
            };
            // The CLI revokes AS the named node (self-or-descendant is always satisfied by
            // caller == target, spec §3.2); transitivity kills the whole subtree.
            let resp = match grants_rpc(
                &state_dir,
                crate::broker_ipc::ReqBody::Revoke { caller: id.clone(), target: id },
                json,
            ) {
                Ok(r) => r,
                Err(c) => return c,
            };
            let crate::broker_ipc::Response::Revoked { by_seq, epoch, newly_revoked } = resp else {
                eprintln!("error: unexpected revoke response: {resp:?}");
                return 2;
            };
            if json {
                note_json_emitted();
                println!(
                    "{}",
                    json!({
                        "command": "grants", "subcommand": "revoke",
                        "by_seq": by_seq, "epoch": epoch, "newly_revoked": newly_revoked,
                        "revocation_takes_effect": delulu_diag::REVOCATION_BOUND,
                    })
                );
            } else {
                if newly_revoked.is_empty() {
                    ok_line!("ok: already revoked (idempotent; audit seq {by_seq}, epoch {epoch})");
                } else {
                    ok_line!(
                        "ok: revoked {} node(s) (audit seq {by_seq}, epoch {epoch}): {}",
                        newly_revoked.len(),
                        newly_revoked.join(", ")
                    );
                }
                // The honest §4.2 bound, stated at the point of revocation (playbook trap 3).
                eprintln!("takes effect: {}", delulu_diag::REVOCATION_BOUND);
            }
            0
        }
        "delegate" => cmd_grants_delegate(args, &state_dir, json),
        // RFC 0001 F3 — federation. `certify` and `pubkey` are the GROUND side (offline, no
        // daemon); `adopt` is the VEHICLE side.
        "certify" => cmd_grants_certify(args, json),
        "pubkey" => cmd_grants_pubkey(args, json),
        "adopt" => cmd_grants_adopt(args, &state_dir, json),
        "receipt" => cmd_grants_receipt(args, json),
        "renew" => cmd_grants_renew(args, &state_dir, json),
        other => {
            eprintln!(
                "error: unknown grants subcommand `{other}` \
                 (list | tree | inspect | revoke | delegate | certify | adopt | renew | receipt | pubkey)"
            );
            2
        }
    }
}

// ===================================================================================================
// RFC 0001 phase F3 — the federation verbs.
//
// The broker NEVER opens a network socket (RFC §1). These three commands read and write
// self-contained signed documents; carrying them across a link is the operator's existing
// business — a pass, a file drop, a store-and-forward queue. DeluluLang does not own the radio.
// ===================================================================================================

/// `--name value` or `--name=value` (both accepted, like the shared `parse_opts`). Shared by every
/// `grants` subcommand that parses its own flags.
fn flag_value(args: &[String], i: &mut usize, name: &str) -> Option<String> {
    let a = &args[*i];
    if let Some(v) = a.strip_prefix(name) {
        if let Some(v) = v.strip_prefix('=') {
            return Some(v.to_string());
        }
    }
    if a == name && *i + 1 < args.len() {
        *i += 1;
        return Some(args[*i].clone());
    }
    None
}

/// Resolve the grant signing-key path. The library never chooses a path (broker ruling 2); this is
/// the CLI's decision, alongside `~/.delulu/broker.key`.
fn grant_key_path(explicit: Option<&str>) -> Option<std::path::PathBuf> {
    match explicit {
        Some(p) => Some(std::path::PathBuf::from(p)),
        None => crate::brokerd::resolve_state_dir(None).map(|d| d.join("grant.key")),
    }
}

/// `delulu grants pubkey [--key FILE]` — print the public identity of a signing key.
///
/// This is the value that goes in a peer's trust-anchor list and in a certificate's `subject`. It
/// is deliberately its own command: an operator should be able to answer "what am I trusting?"
/// without minting anything.
fn cmd_grants_pubkey(args: &[String], json: bool) -> i32 {
    let mut key: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if let Some(v) = flag_value(args, &mut i, "--key") {
            key = Some(v);
        }
        i += 1;
    }
    let Some(path) = grant_key_path(key.as_deref()) else {
        eprintln!("error: cannot resolve the key path (no HOME/USERPROFILE) — pass --key FILE");
        return 2;
    };
    let seed = match delulu_broker::load_or_create_key(&path) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("error: cannot load or create the grant key at {}: {e}", path.display());
            return 2;
        }
    };
    let hex = crate::cert_crypto::public_key_hex(&seed);
    if json {
        let report = serde_json::json!({ "command": "grants pubkey", "schema": 1, "pubkey": hex });
        note_json_emitted();
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else {
        println!("{hex}");
    }
    0
}

/// `delulu grants certify --subject HEX --effects E,.. [--device D].. --ttl 1h [--key F]
/// [--parent-cert F] [--out F]`
///
/// Mint one grant certificate. **Offline by design** — no daemon, no tree, no network. The ground
/// segment can run this on an air-gapped machine and hand the result to a spacecraft over whatever
/// link exists.
fn cmd_grants_certify(args: &[String], json: bool) -> i32 {
    let mut effects: Vec<String> = Vec::new();
    let (mut fs_read, mut fs_write, mut net) = (Vec::new(), Vec::new(), Vec::new());
    let mut device: Vec<String> = Vec::new();
    let (mut subject, mut ttl, mut key, mut parent_cert, mut out) = (None, None, None, None, None);
    // RFC 0001 F4: how long the holder may run WITHOUT a contact receipt. Absent = the
    // certificate's own window is the only bound.
    let mut uplink_ttl: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--json" {
            // handled by the caller
        } else if let Some(v) = flag_value(args, &mut i, "--effects") {
            effects.extend(v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
        } else if let Some(v) = flag_value(args, &mut i, "--device") {
            device.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--fs-read") {
            fs_read.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--fs-write") {
            fs_write.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--net") {
            net.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--subject") {
            subject = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--ttl") {
            ttl = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--uplink-ttl") {
            uplink_ttl = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--key") {
            key = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--parent-cert") {
            parent_cert = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--out") {
            out = Some(v);
        } else {
            eprintln!("error: unknown `grants certify` argument `{a}`");
            return 2;
        }
        i += 1;
    }
    let Some(subject) = subject else {
        eprintln!(
            "error: `grants certify` needs --subject HEX (the holder's public key; get it with \
             `delulu grants pubkey` on that machine)"
        );
        return 2;
    };
    let Some(ttl_str) = ttl else {
        eprintln!(
            "error: `grants certify` needs --ttl (e.g. 20m for a contact window). A certificate \
             with no expiry cannot be revoked across a partition, and expiry is the ONLY bound that \
             survives one"
        );
        return 2;
    };
    let Some(ttl_ms) = parse_ttl_millis(&ttl_str) else {
        eprintln!("error: bad --ttl `{ttl_str}` (use e.g. 500ms, 90s, 30m, 1h, 2d)");
        return 2;
    };
    for e in &effects {
        if Effect::core_from_name(e).is_none() {
            eprintln!("error: unknown effect `{e}`");
            return 2;
        }
    }
    // Canonicalize device envelopes through the broker's parser, reporting its message verbatim —
    // the same discipline `grants delegate --device` uses.
    let mut device_canon = Vec::with_capacity(device.len());
    for d in &device {
        match delulu_broker::device_scope::parse(d) {
            Ok(p) => device_canon.push(p),
            Err(e) => {
                eprintln!("error: bad --device `{d}`: {e}");
                return 2;
            }
        }
    }
    let Some(path) = grant_key_path(key.as_deref()) else {
        eprintln!("error: cannot resolve the key path (no HOME/USERPROFILE) — pass --key FILE");
        return 2;
    };
    let seed = match delulu_broker::load_or_create_key(&path) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("error: cannot load or create the grant key at {}: {e}", path.display());
            return 2;
        }
    };
    // Chain onto a parent certificate, or anchor. When chaining, the parent is READ AND VERIFIED
    // structurally here so an operator finds out now — not on the vehicle — that they signed with
    // the wrong key or asked for more than they hold.
    let (parent_link, parent_authority) = match &parent_cert {
        None => (delulu_broker::cert::ANCHOR.to_string(), None),
        Some(f) => {
            let text = match std::fs::read_to_string(f) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("error: cannot read --parent-cert {f}: {e}");
                    return 2;
                }
            };
            match delulu_broker::cert::parse(&text) {
                Ok(p) => {
                    let me = crate::cert_crypto::public_key_hex(&seed);
                    if p.subject != me {
                        eprintln!(
                            "error: --parent-cert was delegated to `{}`, but this key is `{me}`. \
                             Only the holder may delegate onward, so the vehicle would refuse this \
                             chain (DL1415). Sign with the key the parent names as its subject.",
                            p.subject
                        );
                        return 2;
                    }
                    (p.fingerprint(), Some(p.authority))
                }
                Err(d) => {
                    eprintln!("error: --parent-cert does not parse: {}", d.to_diagnostic().message);
                    return 2;
                }
            }
        }
    };
    let uplink_ttl_ms = match &uplink_ttl {
        None => None,
        Some(s) => match parse_ttl_millis(s) {
            Some(d) => Some(d as u64),
            None => {
                eprintln!("error: bad --uplink-ttl `{s}` (use e.g. 500ms, 90s, 30m, 1h, 2d)");
                return 2;
            }
        },
    };
    let now = now_millis();
    let authority = delulu_broker::Authority::new(
        effects.iter().filter_map(|e| Effect::core_from_name(e)),
        delulu_broker::Scopes {
            fs_read: fs_read.into_iter().collect(),
            fs_write: fs_write.into_iter().collect(),
            net: net.into_iter().collect(),
            device: device_canon.into_iter().map(|d| (d.device.clone(), d)).collect(),
            ..Default::default()
        },
    );
    // Catch a widening at MINT time rather than letting the holder discover it at use time. The
    // vehicle would refuse it anyway (DL1416) — but by then it is out of contact.
    if let Some(pa) = &parent_authority {
        if let Err(intersection) = delulu_broker::attenuation_check(&authority, pa) {
            eprintln!(
                "error: this certificate asks for more than --parent-cert holds, so the vehicle \
                 would refuse the chain (DL1416).\n  requested: {}\n  most it can carry: {}",
                authority.render_compact(),
                intersection.render_compact()
            );
            return 2;
        }
    }
    let mut c = delulu_broker::cert::Certificate {
        alg: crate::cert_crypto::ALG_ED25519.to_string(),
        issuer: crate::cert_crypto::public_key_hex(&seed),
        subject,
        parent: parent_link,
        not_before: now,
        not_after: now + ttl_ms,
        nonce: crate::cert_crypto::fresh_nonce(),
        uplink_ttl_ms,
        authority,
        sig: Vec::new(),
    };
    c.sig = crate::cert_crypto::sign(&seed, &c.signing_bytes());
    let wire = c.to_wire();
    if let Some(f) = &out {
        if let Err(e) = std::fs::write(f, &wire) {
            eprintln!("error: cannot write {f}: {e}");
            return 2;
        }
    }
    if json {
        let report = serde_json::json!({
            "command": "grants certify",
            "schema": 1,
            "issuer": c.issuer,
            "subject": c.subject,
            "fingerprint": c.fingerprint(),
            "not_after": c.not_after,
            "certificate": wire,
        });
        note_json_emitted();
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else if out.is_some() {
        eprintln!("wrote {} ({} bytes)", out.as_deref().unwrap_or(""), wire.len());
        eprintln!("fingerprint: {}", c.fingerprint());
        eprintln!("expires:     {}", delulu_broker::render_ts_utc(c.not_after));
    } else {
        print!("{wire}");
    }
    0
}

/// `delulu grants receipt --for FINGERPRINT --ttl 1h [--key F] [--out F]`
///
/// RFC 0001 F4 — the ground mints a **contact receipt**: proof it was in contact, and nothing more.
/// It grants no authority; it only lets an already-adopted credential keep running for another
/// uplink term. Offline, like `certify`.
fn cmd_grants_receipt(args: &[String], json: bool) -> i32 {
    let (mut for_fp, mut ttl, mut key, mut out) = (None, None, None, None);
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--json" {
        } else if let Some(v) = flag_value(args, &mut i, "--for") {
            for_fp = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--ttl") {
            ttl = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--key") {
            key = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--out") {
            out = Some(v);
        } else {
            eprintln!("error: unknown `grants receipt` argument `{a}`");
            return 2;
        }
        i += 1;
    }
    let Some(for_fp) = for_fp else {
        eprintln!(
            "error: `grants receipt` needs --for FINGERPRINT (the certificate being renewed; \
             `grants certify --json` prints it, and `grants adopt` echoes it)"
        );
        return 2;
    };
    let Some(ttl_str) = ttl else {
        eprintln!("error: `grants receipt` needs --ttl (how long until the next contact is due)");
        return 2;
    };
    let Some(ttl_ms) = parse_ttl_millis(&ttl_str) else {
        eprintln!("error: bad --ttl `{ttl_str}` (use e.g. 500ms, 90s, 30m, 1h, 2d)");
        return 2;
    };
    let Some(path) = grant_key_path(key.as_deref()) else {
        eprintln!("error: cannot resolve the key path (no HOME/USERPROFILE) — pass --key FILE");
        return 2;
    };
    let seed = match delulu_broker::load_or_create_key(&path) {
        Ok(k) => k,
        Err(e) => {
            eprintln!("error: cannot load or create the grant key at {}: {e}", path.display());
            return 2;
        }
    };
    let mut r = delulu_broker::cert::Receipt {
        alg: crate::cert_crypto::ALG_ED25519.to_string(),
        issuer: crate::cert_crypto::public_key_hex(&seed),
        certificate: for_fp,
        not_after: now_millis() + ttl_ms,
        nonce: crate::cert_crypto::fresh_nonce(),
        sig: Vec::new(),
    };
    r.sig = crate::cert_crypto::sign(&seed, &r.signing_bytes());
    let wire = r.to_wire();
    if let Some(f) = &out {
        if let Err(e) = std::fs::write(f, &wire) {
            eprintln!("error: cannot write {f}: {e}");
            return 2;
        }
    }
    if json {
        let report = serde_json::json!({
            "command": "grants receipt", "schema": 1,
            "issuer": r.issuer, "certificate": r.certificate,
            "not_after": r.not_after, "receipt": wire,
        });
        note_json_emitted();
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    } else if out.is_some() {
        eprintln!("wrote {} — next contact due by {}", out.as_deref().unwrap_or(""), delulu_broker::render_ts_utc(r.not_after));
    } else {
        print!("{wire}");
    }
    0
}

/// `delulu grants renew <receipt-file> [--anchor HEX]... [--anchors-file F]`
///
/// The vehicle applies a contact receipt. Extension is **monotone**: a receipt can push the uplink
/// lease forward and can never pull it back, so replaying a stale one is a harmless no-op rather
/// than a way to strip a vehicle of authority. Shortening is `grants revoke`'s job.
fn cmd_grants_renew(args: &[String], state_dir: &std::path::Path, json: bool) -> i32 {
    let mut file: Option<String> = None;
    let mut anchors: Vec<String> = Vec::new();
    let mut anchors_file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--json" {
        } else if let Some(_v) = flag_value(args, &mut i, "--state-dir") {
        } else if let Some(v) = flag_value(args, &mut i, "--anchor") {
            anchors.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--anchors-file") {
            anchors_file = Some(v);
        } else if a.starts_with("--") {
            eprintln!("error: unknown `grants renew` argument `{a}`");
            return 2;
        } else {
            file = Some(a.to_string());
        }
        i += 1;
    }
    let Some(file) = file else {
        eprintln!("error: `grants renew` needs a receipt file");
        return 2;
    };
    let anchors_path =
        anchors_file.map(std::path::PathBuf::from).unwrap_or_else(|| state_dir.join("anchors"));
    if let Ok(text) = std::fs::read_to_string(&anchors_path) {
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if !line.is_empty() {
                anchors.push(line.to_string());
            }
        }
    }
    if anchors.is_empty() {
        eprintln!(
            "error: no trust anchors — a receipt must be signed by a key you configured.\n  \
             Pass --anchor <hex>, or list one key per line in {}",
            anchors_path.display()
        );
        return 2;
    }
    let receipt = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: cannot read {file}: {e}");
            return 2;
        }
    };
    match grants_rpc(state_dir, crate::broker_ipc::ReqBody::Renew { receipt, anchors }, json) {
        Ok(crate::broker_ipc::Response::Renewed { node, ttl_millis }) => {
            if json {
                let report = serde_json::json!({
                    "command": "grants renew", "schema": 1, "node": node, "ttl_millis": ttl_millis,
                });
                note_json_emitted();
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("{node}");
                eprintln!("uplink lease now runs to {}", delulu_broker::render_ts_utc(ttl_millis));
            }
            0
        }
        Ok(other) => {
            eprintln!("error: unexpected renew response: {other:?}");
            2
        }
        Err(code) => code,
    }
}

/// `delulu grants adopt <cert-file>... [--anchor HEX]... [--anchors-file F]`
///
/// The vehicle side: verify a chain and adopt it as a LOCAL ROOT. After this the on-vehicle broker
/// is a real broker — `Actuate` round-trips to it locally at full speed, `grants revoke` reaches
/// it, and no link sits in the command path.
fn cmd_grants_adopt(args: &[String], state_dir: &std::path::Path, json: bool) -> i32 {
    let mut files: Vec<String> = Vec::new();
    let mut anchors: Vec<String> = Vec::new();
    let mut anchors_file: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--json" {
        } else if let Some(_v) = flag_value(args, &mut i, "--state-dir") {
        } else if let Some(v) = flag_value(args, &mut i, "--anchor") {
            anchors.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--anchors-file") {
            anchors_file = Some(v);
        } else if a.starts_with("--") {
            eprintln!("error: unknown `grants adopt` argument `{a}`");
            return 2;
        } else {
            files.push(a.to_string());
        }
        i += 1;
    }
    if files.is_empty() {
        eprintln!(
            "error: `grants adopt` needs at least one certificate file, root-most FIRST \
             (delulu grants adopt ground.dlcert vehicle.dlcert --anchor <hex>)"
        );
        return 2;
    }
    // Anchors may also live in a file, one hex key per line, `#` comments allowed. The default is
    // `<state-dir>/anchors`; an absent file is NOT an error but an EMPTY anchor set, which trusts
    // nothing — the fail-closed reading.
    let anchors_path =
        anchors_file.map(std::path::PathBuf::from).unwrap_or_else(|| state_dir.join("anchors"));
    if let Ok(text) = std::fs::read_to_string(&anchors_path) {
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if !line.is_empty() {
                anchors.push(line.to_string());
            }
        }
    }
    if anchors.is_empty() {
        eprintln!(
            "error: no trust anchors. A chain must be rooted in a key you configured — an unknown \
             issuer is refused, never assumed trustworthy for being well-formed.\n  \
             Pass --anchor <hex>, or list one key per line in {}",
            anchors_path.display()
        );
        return 2;
    }
    let mut chain = Vec::with_capacity(files.len());
    for f in &files {
        match std::fs::read_to_string(f) {
            Ok(t) => chain.push(t),
            Err(e) => {
                eprintln!("error: cannot read {f}: {e}");
                return 2;
            }
        }
    }
    match grants_rpc(state_dir, crate::broker_ipc::ReqBody::Adopt { chain, anchors }, json) {
        Ok(crate::broker_ipc::Response::Adopted { node, fingerprint, ttl_millis }) => {
            if json {
                let report = serde_json::json!({
                    "command": "grants adopt",
                    "schema": 1,
                    "node": node,
                    "fingerprint": fingerprint,
                    "ttl_millis": ttl_millis,
                });
                note_json_emitted();
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("{node}");
                eprintln!("adopted:  {fingerprint}");
                match ttl_millis {
                    Some(t) => eprintln!(
                        "expires:  {} (the shortest hop in the chain — and, across a partition, \
                         the ONLY bound left)",
                        delulu_broker::render_ts_utc(t)
                    ),
                    None => eprintln!("expires:  never"),
                }
            }
            0
        }
        Ok(other) => {
            eprintln!("error: unexpected adopt response: {other:?}");
            2
        }
        Err(code) => code,
    }
}

/// `delulu grants delegate` (spec §3.2): attenuate + mint a portable lease token, printed to
/// STDOUT so a script/orchestrator can capture it (`--json` for the structured form). With no
/// `--parent`, a root node holding exactly the requested authority is issued first — the typed
/// command line is the human action at the top of the tree (spec §3.1).
fn cmd_grants_delegate(args: &[String], state_dir: &std::path::Path, json: bool) -> i32 {
    use crate::broker_ipc::{AuthoritySpec, ReqBody, Response};

    let mut effects: Vec<String> = Vec::new();
    let (mut fs_read, mut fs_write, mut net) = (Vec::new(), Vec::new(), Vec::new());
    let (mut secrets, mut declassify) = (Vec::new(), Vec::new());
    let (mut foreign_c, mut foreign_python) = (Vec::new(), Vec::new());
    // RFC 0001 F1 (D12e): the dimension that lets a delegation say "you may fly this corridor only"
    // rather than merely "you may actuate".
    let mut device: Vec<String> = Vec::new();
    let mut ttl: Option<String> = None;
    let mut parent: Option<String> = None;
    let mut multi = false;
    let mut holder_desc = "delegated lease".to_string();
    // The guard owner code (Stage 5 chunk 6): required to delegate GUARDED authority (declassify /
    // native code by default) — the principal signing off on handing a guarded slice to an agent.
    let mut owner: Option<String> = std::env::var("DELULU_GUARD_OWNER").ok();

    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--multi" {
            multi = true;
        } else if a == "--json" {
            // handled by the caller
        } else if let Some(_v) = flag_value(args, &mut i, "--state-dir") {
            // handled by the caller
        } else if let Some(v) = flag_value(args, &mut i, "--effects") {
            effects.extend(v.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));
        } else if let Some(v) = flag_value(args, &mut i, "--fs-read") {
            fs_read.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--fs-write") {
            fs_write.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--net") {
            net.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--secret") {
            secrets.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--declassify") {
            declassify.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--foreign-c") {
            foreign_c.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--foreign-python") {
            foreign_python.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--device") {
            device.push(v);
        } else if let Some(v) = flag_value(args, &mut i, "--ttl") {
            ttl = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--parent") {
            parent = Some(v);
        } else if let Some(v) = flag_value(args, &mut i, "--holder-desc") {
            holder_desc = v;
        } else if let Some(v) = flag_value(args, &mut i, "--owner") {
            owner = Some(v);
        } else {
            eprintln!("error: unknown `grants delegate` argument `{a}`");
            return 2;
        }
        i += 1;
    }

    // A typo'd effect name must not silently vanish (it would delegate LESS than asked — safe but
    // baffling). Validate up front.
    for e in &effects {
        if Effect::core_from_name(e).is_none() {
            eprintln!(
                "error: unknown effect `{e}` (core effects: {})",
                delulu_check::check::CORE_EFFECT_NAMES.join(", ")
            );
            return 2;
        }
    }
    // A malformed device envelope must not silently vanish either, and for a device the stakes are
    // higher than a typo'd effect: a dropped envelope is a bound nobody enforces. Parse each one
    // here, report the parser's own message, and re-render from the parsed value so what reaches
    // the broker is byte-canonical regardless of how the human ordered the dimensions.
    let device = {
        let mut out = Vec::with_capacity(device.len());
        for d in &device {
            match delulu_broker::device_scope::parse(d) {
                Ok(parsed) => out.push(parsed.to_grant_string()),
                Err(e) => {
                    eprintln!(
                        "error: bad --device `{d}`: {e}\n  \
                         form: DEVICE:dim=lo..hi[,...][,rate_hz=N],heartbeat_ms=N,ttl_ms=N,fail=hold|coast|safe-park"
                    );
                    return 2;
                }
            }
        }
        out.sort();
        out
    };
    // fs scope paths are absolutized + lexically normalized against THIS command's cwd — the same
    // frame `--grant fs.*` uses (see `authority_spec_from_grants`: "the SAME absolute, lexically-
    // normalized strings the embedded RootVal carries") — so the broker's path lattice sees exactly
    // what an agent's runtime will resolve its file arguments against. Consequence for holders:
    // mint the delegation from the directory the paths are relative to.
    let norm = |v: Vec<String>| -> Vec<String> {
        v.into_iter()
            .map(|p| delulu_runtime::prim::granted_root(&p).to_string_lossy().to_string())
            .collect()
    };
    let fs_read = norm(fs_read);
    let fs_write = norm(fs_write);
    let ttl_millis = match &ttl {
        None => None,
        Some(s) => match parse_ttl_millis(s) {
            Some(d) => Some(now_millis() + d),
            None => {
                eprintln!("error: bad --ttl `{s}` (use e.g. 500ms, 90s, 30m, 1h, 2d)");
                return 2;
            }
        },
    };

    let parent = match parent {
        Some(p) => p,
        None => {
            // No parent named: issue a root holding EXACTLY the requested authority (nothing
            // wider), then delegate under it. Root issuance stays a human action — this typed
            // command line — never a programmatic path (Constitution §5.16 law 4).
            let root_spec = AuthoritySpec {
                effects: effects.clone(),
                fs_read: fs_read.clone(),
                fs_write: fs_write.clone(),
                net: net.clone(),
                secrets: secrets.clone(),
                declassify: declassify.clone(),
                foreign_c: foreign_c.clone(),
                foreign_python: foreign_python.clone(),
                device: device.clone(),
                holder_kind: "human".to_string(),
                holder_desc: "delulu grants delegate (root)".to_string(),
                ttl_millis: None,
            };
            match grants_rpc(state_dir, ReqBody::Issue(root_spec), json) {
                Ok(Response::Issued { node }) => node,
                Ok(other) => {
                    eprintln!("error: unexpected issue response: {other:?}");
                    return 2;
                }
                Err(code) => return code,
            }
        }
    };

    let child_spec = AuthoritySpec {
        effects,
        fs_read,
        fs_write,
        net,
        secrets,
        declassify,
        foreign_c,
        foreign_python,
        device,
        holder_kind: "delegate".to_string(),
        holder_desc,
        ttl_millis,
    };
    match grants_rpc(state_dir, ReqBody::Delegate { parent: parent.clone(), authority: child_spec, multi, owner }, json) {
        Ok(Response::Delegated { node, token }) => {
            if json {
                note_json_emitted();
                println!(
                    "{}",
                    json!({
                        "command": "grants", "subcommand": "delegate",
                        "parent": parent, "node": node, "token": token,
                        "multi": multi, "ttl_millis": ttl_millis,
                    })
                );
            } else {
                ok_line!(
                    "ok: delegated `{node}` ⊑ `{parent}`{}{}",
                    match ttl_millis {
                        Some(t) => format!(", expires {}", delulu_broker::render_ts_utc(t)),
                        None => String::new(),
                    },
                    if multi { ", multi-redemption" } else { ", single-redemption" }
                );
                eprintln!("hand the token to the agent; it runs with: delulu run <file> --lease <token>");
                println!("{token}");
            }
            0
        }
        Ok(other) => {
            eprintln!("error: unexpected delegate response: {other:?}");
            2
        }
        Err(code) => code,
    }
}

// ----- guard (Stage 5 chunk 6: the Guard CLI surface, addendum §3.1) -----------------------------
//
// Read verbs (`status`, `policy show`) need no owner code — awareness must be free (addendum §2.2).
// Admin verbs (`policy set|unset`) carry the owner code via `--owner <code>` or DELULU_GUARD_OWNER;
// a missing/wrong code is DL1414. Every verb fails closed (invariant 27): a daemon-down verb is
// DL1401 with the exact start command.

/// Resolve the guard owner code: `--owner <code>` (or `--owner=code`), else DELULU_GUARD_OWNER.
fn guard_owner(args: &[String]) -> Option<String> {
    args.iter()
        .position(|a| a == "--owner")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .or_else(|| args.iter().find_map(|a| a.strip_prefix("--owner=").map(str::to_string)))
        .or_else(|| std::env::var("DELULU_GUARD_OWNER").ok())
}

/// One daemon round-trip for a `guard` verb (fail closed, invariant 27; command label `guard`).
fn guard_rpc(
    state_dir: &std::path::Path,
    body: crate::broker_ipc::ReqBody,
    json: bool,
) -> Result<crate::broker_ipc::Response, i32> {
    let map = SourceMap::new();
    match crate::brokerd::request(state_dir, body) {
        Ok(crate::broker_ipc::Response::Error { code, message, .. }) => {
            let d = Diagnostic::error(crate::broker_client::static_code(&code), message);
            print_diagnostics("guard", &[d], &map, None, json);
            Err(1)
        }
        Ok(resp) => Ok(resp),
        Err(e) => {
            let d = Diagnostic::error(
                "DL1401",
                format!(
                    "broker unreachable: {e} — start it with `delulu broker start` \
                     (fail closed, invariant 27: `guard` verbs never fall back to local state)"
                ),
            );
            print_diagnostics("guard", &[d], &map, None, json);
            Err(1)
        }
    }
}

fn cmd_guard(rest: &[String]) -> i32 {
    let Some(sub) = rest.first().map(String::as_str) else {
        eprintln!(
            "error: `guard` needs a subcommand: status | policy [show | set <class:pattern> <tier> | \
             unset <class:pattern>]  [--owner CODE] [--json]"
        );
        return 2;
    };
    let args = &rest[1..];
    let json = args.iter().any(|a| a == "--json");
    let state_flag = args
        .iter()
        .position(|a| a == "--state-dir")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .or_else(|| args.iter().find_map(|a| a.strip_prefix("--state-dir=").map(str::to_string)));
    let Some(state_dir) = crate::brokerd::resolve_state_dir(state_flag.as_deref()) else {
        eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE) — pass --state-dir DIR");
        return 2;
    };

    match sub {
        "status" => {
            let resp = match guard_rpc(&state_dir, crate::broker_ipc::ReqBody::GuardStatus, json) {
                Ok(r) => r,
                Err(c) => return c,
            };
            print_guard_status(&resp, "status", json)
        }
        "policy" => cmd_guard_policy(args, &state_dir, json),
        "bypass" => cmd_guard_bypass(args, &state_dir, json),
        "request" => cmd_guard_request(args, &state_dir, json),
        "pending" => cmd_guard_pending(&state_dir, json),
        "approve" => cmd_guard_approve(args, &state_dir, json),
        "deny" => cmd_guard_deny(args, &state_dir, json),
        "permits" => cmd_guard_permits(args, &state_dir, json),
        other => {
            eprintln!(
                "error: unknown guard subcommand `{other}` (status | policy | bypass | request | \
                 pending | approve | deny | permits)"
            );
            2
        }
    }
}

/// `guard bypass on|off --owner <code>` (addendum §2.6): the runtime toggle. Enabling ALWAYS prints
/// the banner (spec-fixed exact text) — the principal must see what they just turned off.
fn cmd_guard_bypass(args: &[String], state_dir: &std::path::Path, json: bool) -> i32 {
    use crate::broker_ipc::ReqBody;
    let (positional, _uses, _vals) = guard_parse(args);
    let on = match positional.as_deref() {
        Some("on") => true,
        Some("off") => false,
        _ => {
            eprintln!("error: `guard bypass` needs `on` or `off` (plus --owner CODE)");
            return 2;
        }
    };
    let owner = guard_owner(args);
    match guard_rpc(state_dir, ReqBody::GuardBypass { owner, on }, json) {
        Ok(_) => {
            if on {
                // The banner is mandatory on every enable (addendum §2.6) — stderr, so `--json`
                // stdout stays machine-clean (the dcg robot-mode convention, §2.7). Painted in the
                // strongest error style (Palette `guard_banner` role) when color is on.
                eprintln!(
                    "{}",
                    palette_stderr().paint(Role::GuardBanner, delulu_diag::GUARD_BYPASS_BANNER)
                );
            }
            if json {
                note_json_emitted();
                println!("{}", json!({ "command": "guard", "subcommand": "bypass", "bypass": on }));
            } else if on {
                ok_line!("ok: guard bypass ENABLED");
            } else {
                ok_line!("ok: guard bypass disabled — guarded rules enforce again");
            }
            0
        }
        Err(c) => c,
    }
}

/// The one-line guard status a `run --lease` prints before user output (addendum §2.7, criterion 8).
/// `BYPASSED` shows prominently; the `on` form lists the guarded/sealed rules and names the request
/// command, so an agent knows the escalation path BEFORE it hits DL1410.
fn lease_guard_status_line(bypass: bool, poisoned: bool, rules: &[crate::broker_ipc::GuardRuleWire]) -> String {
    if bypass {
        return "guard: BYPASSED by the principal — guarded uses will proceed and be audited".to_string();
    }
    let guarded: Vec<String> = rules
        .iter()
        .filter(|r| r.tier != "warn")
        .map(|r| format!("{}:{}", r.class, r.pattern))
        .collect();
    let poisoned = if poisoned { " [policy store unreadable — fail closed]" } else { "" };
    if guarded.is_empty() {
        format!("guard: on — no guarded/sealed classes{poisoned}")
    } else {
        format!(
            "guard: on — guarded: {}{poisoned} — to request access: delulu guard request",
            guarded.join(", ")
        )
    }
}

/// A repeatable `--use class:pattern` collector + the first bare positional (node / request / permit
/// id). `--owner`/`--use`/`--why`/`--ttl`/`--uses`/`--comment`/`--state-dir` take a value.
fn guard_parse(args: &[String]) -> (Option<String>, Vec<String>, std::collections::HashMap<String, String>) {
    let value_flags = ["--owner", "--why", "--ttl", "--uses", "--comment", "--state-dir"];
    let mut positional = None;
    let mut uses: Vec<String> = Vec::new();
    let mut vals = std::collections::HashMap::new();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == "--use" {
            if i + 1 < args.len() {
                uses.push(args[i + 1].clone());
                i += 1;
            }
        } else if let Some(rest) = a.strip_prefix("--use=") {
            uses.push(rest.to_string());
        } else if let Some(flag) = value_flags.iter().find(|f| a == **f) {
            if i + 1 < args.len() {
                vals.insert((*flag).to_string(), args[i + 1].clone());
                i += 1;
            }
        } else if let Some((flag, v)) = value_flags.iter().find_map(|f| a.strip_prefix(&format!("{f}=")).map(|v| (*f, v))) {
            vals.insert(flag.to_string(), v.to_string());
        } else if a == "--json" {
            // handled by the caller
        } else if !a.starts_with("--") && positional.is_none() {
            positional = Some(a.to_string());
        }
        i += 1;
    }
    (positional, uses, vals)
}

/// `guard request <node> --use <class:pattern>… --why "<text>"` (addendum §2.5). `--why` is REQUIRED
/// — the agent must explain why it needs the authority. No owner (awareness/escalation is free).
fn cmd_guard_request(args: &[String], state_dir: &std::path::Path, json: bool) -> i32 {
    use crate::broker_ipc::{ReqBody, Response};
    let (node, uses, vals) = guard_parse(args);
    let Some(node) = node else {
        eprintln!("error: `guard request` needs a node id (g_…)");
        return 2;
    };
    if uses.is_empty() {
        eprintln!("error: `guard request` needs at least one `--use <class:pattern>` (e.g. --use declassify:*)");
        return 2;
    }
    let Some(why) = vals.get("--why").cloned() else {
        eprintln!("error: `guard request` requires `--why \"<justification>\"` — explain why you need the authority");
        return 2;
    };
    match guard_rpc(state_dir, ReqBody::GuardRequest { node, uses, why }, json) {
        Ok(Response::GuardRequested { id, deduped }) => {
            if json {
                note_json_emitted();
                println!("{}", json!({ "command": "guard", "subcommand": "request", "id": id, "deduped": deduped }));
            } else {
                if deduped {
                    ok_line!("ok: a matching request was already pending — id {id}");
                } else {
                    ok_line!("ok: guard request recorded — id {id}");
                }
                eprintln!("the principal decides with `delulu guard approve {id}` / `deny {id}`; retry your op once approved");
                println!("{id}");
            }
            0
        }
        Ok(other) => {
            eprintln!("error: unexpected guard request response: {other:?}");
            2
        }
        Err(c) => c,
    }
}

/// `guard pending` — list pending/decided requests WITH their justifications (addendum §2.5).
fn cmd_guard_pending(state_dir: &std::path::Path, json: bool) -> i32 {
    use crate::broker_ipc::{ReqBody, Response};
    let resp = match guard_rpc(state_dir, ReqBody::GuardPending, json) {
        Ok(r) => r,
        Err(c) => return c,
    };
    let Response::GuardPendingList { requests } = resp else {
        eprintln!("error: unexpected guard pending response: {resp:?}");
        return 2;
    };
    if json {
        let arr: Vec<Json> = requests
            .iter()
            .map(|r| json!({ "id": r.id, "node": r.node, "uses": r.uses, "why": r.why, "created_millis": r.created_millis, "status": r.status }))
            .collect();
        note_json_emitted();
        println!("{}", json!({ "command": "guard", "subcommand": "pending", "count": requests.len(), "requests": arr }));
    } else if requests.is_empty() {
        println!("(no guard requests)");
    } else {
        for r in &requests {
            println!("{}  [{}]  {}  use=[{}]  why: {}", r.id, r.status, r.node, r.uses.join(","), r.why);
        }
    }
    0
}

/// `guard approve <req-id> --owner <code> [--ttl <dur>] [--uses <n>] [--comment "<text>"]` (§2.5).
fn cmd_guard_approve(args: &[String], state_dir: &std::path::Path, json: bool) -> i32 {
    use crate::broker_ipc::{ReqBody, Response};
    let (id, _uses, vals) = guard_parse(args);
    let Some(id) = id else {
        eprintln!("error: `guard approve` needs a request id (gr_…)");
        return 2;
    };
    let ttl_millis = match vals.get("--ttl") {
        None => None,
        Some(s) => match parse_ttl_millis(s) {
            Some(d) => Some(d), // a permit TTL is a DURATION; the broker adds it to its own clock
            None => {
                eprintln!("error: bad --ttl `{s}` (use e.g. 5m, 15m, 1h)");
                return 2;
            }
        },
    };
    let uses_n: Option<u64> = match vals.get("--uses") {
        None => None,
        Some(s) => match s.parse() {
            Ok(n) => Some(n),
            Err(_) => {
                eprintln!("error: bad --uses `{s}` (a positive integer)");
                return 2;
            }
        },
    };
    let comment = vals.get("--comment").cloned();
    let owner = guard_owner(args);
    match guard_rpc(state_dir, ReqBody::GuardApprove { owner, id: id.clone(), ttl_millis, uses: uses_n, comment }, json) {
        Ok(Response::GuardApproved { permit_id }) => {
            if json {
                note_json_emitted();
                println!("{}", json!({ "command": "guard", "subcommand": "approve", "request": id, "permit": permit_id }));
            } else {
                ok_line!("ok: approved request {id} — minted permit {permit_id}");
                println!("{permit_id}");
            }
            0
        }
        Ok(other) => {
            eprintln!("error: unexpected guard approve response: {other:?}");
            2
        }
        Err(c) => c,
    }
}

/// `guard deny <req-id> --owner <code> --comment "<text>"` (addendum §2.5). `--comment` is REQUIRED —
/// the agent must learn WHY (the comment rides DL1412 verbatim on the retried use).
fn cmd_guard_deny(args: &[String], state_dir: &std::path::Path, json: bool) -> i32 {
    use crate::broker_ipc::ReqBody;
    let (id, _uses, vals) = guard_parse(args);
    let Some(id) = id else {
        eprintln!("error: `guard deny` needs a request id (gr_…)");
        return 2;
    };
    let Some(comment) = vals.get("--comment").cloned() else {
        eprintln!("error: `guard deny` requires `--comment \"<text>\"` — the agent must learn why it was denied");
        return 2;
    };
    let owner = guard_owner(args);
    match guard_rpc(state_dir, ReqBody::GuardDeny { owner, id: id.clone(), comment }, json) {
        Ok(_) => {
            if json {
                note_json_emitted();
                println!("{}", json!({ "command": "guard", "subcommand": "deny", "request": id }));
            } else {
                ok_line!("ok: denied request {id} (the agent's retry will carry your comment)");
            }
            0
        }
        Err(c) => c,
    }
}

/// `guard permits [list | revoke <id> --owner <code>]` (addendum §2.5).
fn cmd_guard_permits(args: &[String], state_dir: &std::path::Path, json: bool) -> i32 {
    use crate::broker_ipc::{ReqBody, Response};
    let (positional, _uses, _vals) = guard_parse(args);
    // `revoke <id>` when the first positional is `revoke`; else (`list`/empty) list the permits.
    if positional.as_deref() == Some("revoke") {
        // The permit id is the SECOND positional; guard_parse only keeps the first, so re-scan.
        let Some(id) = args.iter().filter(|a| !a.starts_with("--")).nth(1).cloned() else {
            eprintln!("error: `guard permits revoke` needs a permit id (gp_…)");
            return 2;
        };
        let owner = guard_owner(args);
        return match guard_rpc(state_dir, ReqBody::GuardPermitRevoke { owner, id: id.clone() }, json) {
            Ok(_) => {
                if json {
                    note_json_emitted();
                    println!("{}", json!({ "command": "guard", "subcommand": "permits revoke", "permit": id }));
                } else {
                    ok_line!("ok: revoked permit {id}");
                }
                0
            }
            Err(c) => c,
        };
    }
    let resp = match guard_rpc(state_dir, ReqBody::GuardPermits, json) {
        Ok(r) => r,
        Err(c) => return c,
    };
    let Response::GuardPermitList { permits } = resp else {
        eprintln!("error: unexpected guard permits response: {resp:?}");
        return 2;
    };
    if json {
        let arr: Vec<Json> = permits
            .iter()
            .map(|p| json!({ "id": p.id, "node": p.node, "uses": p.uses, "remaining_uses": p.remaining_uses, "expires_millis": p.expires_millis }))
            .collect();
        note_json_emitted();
        println!("{}", json!({ "command": "guard", "subcommand": "permits", "count": permits.len(), "permits": arr }));
    } else if permits.is_empty() {
        println!("(no permits)");
    } else {
        for p in &permits {
            let uses = p.remaining_uses.map(|n| n.to_string()).unwrap_or_else(|| "unlimited".to_string());
            println!("{}  {}  use=[{}]  remaining={}", p.id, p.node, p.uses.join(","), uses);
        }
    }
    0
}

/// Render a `Response::GuardStatus` for `guard status` / `guard policy show`.
fn print_guard_status(resp: &crate::broker_ipc::Response, subcommand: &str, json: bool) -> i32 {
    let crate::broker_ipc::Response::GuardStatus { bypass, poisoned, rules, pending, permits } = resp else {
        eprintln!("error: unexpected guard status response: {resp:?}");
        return 2;
    };
    if json {
        let rules_json: Vec<Json> = rules
            .iter()
            .map(|r| json!({ "class": r.class, "pattern": r.pattern, "tier": r.tier }))
            .collect();
        note_json_emitted();
        println!(
            "{}",
            json!({
                "command": "guard", "subcommand": subcommand,
                "mode": if *bypass { "bypassed" } else { "on" },
                "bypass": bypass, "poisoned": poisoned,
                "rules": rules_json, "pending": pending, "permits": permits,
            })
        );
    } else {
        let mode = if *bypass {
            "BYPASSED (--dangerously-bypass-guard) — guarded uses proceed and are audited".to_string()
        } else if *poisoned {
            "on [policy store unreadable — fail closed]".to_string()
        } else {
            "on".to_string()
        };
        println!("guard: {mode}");
        if rules.is_empty() {
            println!("rules: (none)");
        } else {
            println!("rules:");
            for r in rules {
                println!("  {}:{} \u{2192} {}", r.class, r.pattern, r.tier);
            }
        }
        println!("pending requests: {pending}   permits: {permits}");
    }
    0
}

fn cmd_guard_policy(args: &[String], state_dir: &std::path::Path, json: bool) -> i32 {
    use crate::broker_ipc::ReqBody;
    // Positionals after `policy`: the action, then its operands. `--owner`/`--state-dir` values skip.
    // `args` here is everything after `guard policy` — args[0] is the action (`show`/`set`/`unset`),
    // then its operands. Collect positionals (skipping flags + their values).
    let mut positionals: Vec<&String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--owner" | "--state-dir" => i += 1, // skip the flag value too
            s if s.starts_with("--") => {}
            _ => positionals.push(&args[i]),
        }
        i += 1;
    }
    let action = positionals.first().map(|s| s.as_str()).unwrap_or("show");
    match action {
        "show" => {
            let resp = match guard_rpc(state_dir, ReqBody::GuardPolicyShow, json) {
                Ok(r) => r,
                Err(c) => return c,
            };
            print_guard_status(&resp, "policy", json)
        }
        "set" => {
            let (Some(rule), Some(tier)) = (positionals.get(1), positionals.get(2)) else {
                eprintln!("error: `guard policy set` needs <class:pattern> <tier> (e.g. `guard policy set net:* guarded`)");
                return 2;
            };
            let Some((class, pattern)) = rule.split_once(':') else {
                eprintln!("error: a rule is `class:pattern` (e.g. `declassify:*`, `fs_write:./out`)");
                return 2;
            };
            let owner = guard_owner(args);
            match guard_rpc(
                state_dir,
                ReqBody::GuardPolicySet {
                    owner,
                    class: class.to_string(),
                    pattern: pattern.to_string(),
                    tier: tier.to_string(),
                },
                json,
            ) {
                Ok(_) => {
                    if json {
                        note_json_emitted();
                        println!("{}", json!({ "command": "guard", "subcommand": "policy set", "rule": format!("{class}:{pattern}"), "tier": tier, "takes_effect": delulu_diag::GUARD_POLICY_BOUND }));
                    } else {
                        ok_line!("ok: guard policy set `{class}:{pattern}` \u{2192} {tier}");
                        // The honest bound, stated at the point of a policy edit (criterion 11).
                        eprintln!("{}", delulu_diag::GUARD_POLICY_BOUND);
                    }
                    0
                }
                Err(c) => c,
            }
        }
        "unset" => {
            let Some(rule) = positionals.get(1) else {
                eprintln!("error: `guard policy unset` needs <class:pattern>");
                return 2;
            };
            let Some((class, pattern)) = rule.split_once(':') else {
                eprintln!("error: a rule is `class:pattern`");
                return 2;
            };
            let owner = guard_owner(args);
            match guard_rpc(
                state_dir,
                ReqBody::GuardPolicyUnset { owner, class: class.to_string(), pattern: pattern.to_string() },
                json,
            ) {
                Ok(_) => {
                    if json {
                        note_json_emitted();
                        println!("{}", json!({ "command": "guard", "subcommand": "policy unset", "rule": format!("{class}:{pattern}"), "takes_effect": delulu_diag::GUARD_POLICY_BOUND }));
                    } else {
                        ok_line!("ok: guard policy unset `{class}:{pattern}`");
                        eprintln!("{}", delulu_diag::GUARD_POLICY_BOUND);
                    }
                    0
                }
                Err(c) => c,
            }
        }
        other => {
            eprintln!("error: unknown `guard policy` action `{other}` (show | set | unset)");
            2
        }
    }
}

/// `run <package-dir>`: resolve the dependency graph, check it, and flatten it for execution.
///
/// Closes campaign finding **C59** (ruling D61): `delulu run` took a single `.delulu` file or a
/// `.dwx`, so a multi-package program — and even the shipped two-module `examples/greeter/` — could
/// only ever be *checked*. `kind = "bin"` was a manifest field the toolchain could not honour.
///
/// **The order here is the ruling.** `check_workspace` is authoritative: per-module visibility,
/// package authority ceilings, dependency pins, `pub`/`pub import` export rules. Nothing runs unless
/// it passes. Only then is the program flattened into one module for the interpreter, and the
/// flattened form's own check exists solely to *discover a name collision between modules* — never
/// to admit a program the workspace rejected.
///
/// A collision is refused with its own message rather than silently resolved by merge order. That is
/// the same bargain the WASM backend makes with DL1201: a bounded capability with a fail-closed edge
/// beats an unbounded one that sometimes runs the wrong function. Lifting it needs per-module
/// resolution inside `Interp`, for which the checker already computes `Program::call_owner`.
fn load_package_for_run(dir: &str, opts: &Opts) -> Result<(SourceMap, delulu_check::Checked), i32> {
    // A directory that is not a package at all. Before D61 this said "is a directory — try `delulu
    // build`", which was right then and is wrong now: `run` DOES take a directory. What it must never
    // do is leak the raw OS error (`Access is denied. (os error 5)` on Windows, `Is a directory` on
    // Linux) — two different misleading texts for one mistake, which is what C27 was.
    if !std::path::Path::new(dir).join("delulu.toml").is_file() {
        eprintln!("error: `{dir}` is a directory and not a DeluluLang package — there is no `delulu.toml` in it");
        eprintln!(
            "note: name the file to run, e.g. `{}`, or add a `delulu.toml` to run the directory as a package",
            std::path::Path::new(dir).join("main.delulu").display()
        );
        return Err(2);
    }

    let ws = delulu_check::deps::resolve_workspace(dir);
    // An empty package: a manifest with no sources. `build` already refuses this with its own note
    // (C26/D33 — "nothing checked, success claimed" was the defect); `run` says the same thing rather
    // than inventing a second vocabulary for one condition.
    if ws.modules.is_empty() {
        let mut diags: Vec<Diagnostic> = ws.diagnostics.clone();
        if errors(&diags) > 0 {
            print_diagnostics("run", &diags, &ws.source_map, None, opts.json);
        } else {
            diags.clear();
            eprintln!(
                "error: no `.delulu` modules found under `{}` — a DeluluLang package keeps its \
                 sources in `src/`, so there is nothing to run",
                std::path::Path::new(dir).join("src").display()
            );
        }
        return Err(2);
    }
    let program = delulu_check::deps::check_workspace(&ws);

    // The authoritative gate.
    let mut diags: Vec<Diagnostic> = ws.diagnostics.clone();
    diags.extend(program.diagnostics.iter().cloned());
    if errors(&diags) > 0 {
        print_diagnostics("run", &diags, &ws.source_map, None, opts.json);
        return Err(1);
    }

    let Some(entry_name) = program.entry_module.clone().or_else(|| ws.root_entry_module()) else {
        eprintln!("error: `{dir}` declares no entry module — a runnable package needs one");
        return Err(2);
    };
    if !ws.modules.iter().any(|m| m.unit.name == entry_name) {
        eprintln!("error: `{dir}`'s entry module `{entry_name}` was not loaded");
        return Err(2);
    }

    // Flatten the SOURCE, then parse once. Merging the module ASTs instead looks equivalent and is
    // not: each was parsed separately so their `NodeId`s overlap, and every checker side table keyed
    // by node id then reads one module's entry for another module's expression. The first attempt did
    // that and produced a nonsense reference-capability complaint about a correct program.
    let module_count = ws.modules.len();
    let texts: Vec<&str> =
        ws.modules.iter().map(|m| ws.source_map.file(m.unit.file).src.as_str()).collect();
    let flat_src = delulu_check::flatten_sources(&entry_name, &texts);

    // The flattened program gets its own file in the map, so a diagnostic from it can still be
    // rendered with a real snippet rather than pointing into a file whose offsets no longer apply.
    let mut map = ws.source_map;
    let fid = map.add_file(format!("{dir} (flattened for execution)"), flat_src.clone());
    let flat = check_source(fid, &flat_src);

    if errors(&flat.diagnostics) > 0 {
        // The workspace accepted this program, so these are not the author's errors: flattening
        // collided. Say which name, and say that the program is CORRECT — the runner is what is
        // limited. Anything less would send someone hunting a bug in code the checker just approved.
        let dup: Vec<String> = flat
            .diagnostics
            .iter()
            .filter(|d| d.is_error())
            .map(|d| d.message.clone())
            .take(3)
            .collect();
        eprintln!(
            "error: `{dir}` checks clean as a {module_count}-module program, but two of its modules \
             declare the same top-level name, and running it flattens them into one scope"
        );
        for m in &dup {
            eprintln!("  collision: {m}");
        }
        eprintln!(
            "note: the program is not wrong — `delulu check`/`build`/`authority` all handle it. The \
             RUNNER cannot yet distinguish two same-named declarations from different modules \
             (campaign finding C59, ruling D61). Rename one, or run the entry module as a single \
             file if it does not import."
        );
        return Err(1);
    }

    Ok((map, flat))
}

/// `run <file>.dwx`: re-verify a pre-built artifact's embedded `delulu:authority` manifest against
/// its code (DL1202 on a missing/tampered section, DL1204 on an incompatible version), announce
/// what it declares it can do, then run `main` under the deny-by-default Wasmtime host.
fn run_dwx_artifact(file: &str, opts: &Opts) -> i32 {
    let bytes = match std::fs::read(file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read `{file}`: {e}");
            return 2;
        }
    };
    let map = SourceMap::new(); // a .dwx carries no source text; diagnostics have no span
    let artifact = match delulu_wasm::read_and_verify(&bytes) {
        Ok(a) => a,
        Err(e) => {
            let d = Diagnostic::error(e.code(), format!("`{file}`: {}", e.message()));
            print_diagnostics("run", &[d], &map, None, opts.json);
            return 1;
        }
    };

    let mut grants = Grants::default();
    for g in &opts.grants {
        if let Err(e) = grants.add(g) {
            eprintln!("error: bad --grant: {e}");
            return 2;
        }
    }

    // The thesis made concrete: the artifact travels with its authority. Announce it before running
    // (human mode only; JSON mode keeps stdout clean for the program's own output).
    if !opts.json {
        let effects = artifact.authority.get("effects").map(strs).unwrap_or_default();
        let effects_str = if effects.is_empty() { "(none — provably pure)".to_string() } else { effects.join(", ") };
        eprintln!("running `{file}` — authority verified; declared effects: {effects_str}");
    }

    // A `.dwx` carries no source AST, so it has no foreign signatures to bind: a foreign-using `.dwx`
    // is not executable in phase 4g (its authority manifest DOES record the foreign entries — the
    // manifest requirement — but running foreign code needs the source's marshalling signatures).
    let cfg = delulu_wasm::HostConfig {
        console: grants.console,
        clock: grants.clock,
        rand: grants.rand,
        fs_read_roots: grants.build_root().fs_read,
        fixed_clock_ms: opts.clock_ms,
        rand_seed: opts.seed,
        ..delulu_wasm::HostConfig::default()
    };
    match delulu_wasm::run_main(&artifact.wasm, &cfg) {
        Ok(output) => {
            print!("{output}");
            0
        }
        Err(e) => {
            let msg = e.message();
            let d = Diagnostic::error(wasm_fault_code(&msg), format!("`{file}`: {msg}"));
            print_diagnostics("run", &[d], &map, None, opts.json);
            1
        }
    }
}

fn cmd_run(rest: &[String]) -> i32 {
    let (file, mut opts) = parse_opts(rest);
    let Some(file) = file else {
        eprintln!("error: `run` needs a file");
        return 2;
    };

    // ----- isolation profile gate (Stage 5 phase 5i, spec §6) -----------------------------------
    // Decided BEFORE anything runs. `microvm` is Linux+KVM-first and refused honestly everywhere
    // else — DL1408 with the documented fallback, NEVER a silent substitution of weaker isolation
    // (playbook trap 8). The capability matrix is spec §6.1.
    match opts.isolation.as_deref() {
        None | Some("none") | Some("process") => {}
        Some("microvm") => {
            match microvm_unavailable() {
                Err(detail) => {
                    let d = Diagnostic::error(
                        "DL1408",
                        format!(
                            "isolation profile `microvm` is unavailable: {detail} — fall back to \
                             `--isolation process` (worker-style OS containment of the code \
                             outside the proof; explicitly weaker: no guest boundary, no \
                             virtio-fs scope mounts, no default-deny egress) [a human must choose \
                             the weaker profile; see `delulu explain DL1408` and spec §6.1]"
                        ),
                    );
                    print_diagnostics("run", &[d], &SourceMap::new(), None, opts.json);
                    return 1;
                }
                Ok(()) => {
                    // The Linux+KVM guest-launch path (spec §6) lands with criterion-8 CI work;
                    // v0.5's probe never returns Ok. Refuse loudly rather than pretend.
                    eprintln!("error: the microVM guest launch is not wired in this build (v0.5)");
                    return 2;
                }
            }
        }
        Some(other) => {
            eprintln!("error: unknown --isolation `{other}` (none | process | microvm)");
            return 2;
        }
    }
    if opts.isolation.as_deref() == Some("process") && opts.foreign_isolation.is_none() {
        // The process profile's v0.5 mechanism: force the code OUTSIDE the proof (foreign libs)
        // into minimum-privilege worker subprocesses (phase 5h). The verified program itself stays
        // in-process, bounded by the effect system + custody — stated honestly in the label below
        // and in the spec §6.1 matrix (never sold as microVM-equivalent).
        opts.foreign_isolation = Some("process".to_string());
    }
    if let Some(iso) = &opts.isolation {
        if !opts.json {
            let label = match iso.as_str() {
                "process" => {
                    "process — foreign code in minimum-privilege worker subprocesses; the \
                     verified program remains in-process (weaker than microvm; spec §6.1)"
                }
                _ => "none — in-process (language + custody enforcement only)",
            };
            eprintln!("isolation: {label}");
        }
    }

    // ----- device profile + the sim-to-hardware gate (Stage 10 phase 10f, spec §5.4) ------------
    // Decided BEFORE anything runs, for the same reason the isolation profile is: a hardware
    // grant for an artifact nobody approved must never reach the point of moving something.
    let device_profile = match resolve_device_profile(&file, &opts) {
        Ok(p) => p,
        Err(code) => return code,
    };
    // D20: the deterministic simulator clock. Only meaningful under `sim`, where it replaces the
    // wall-clock dead-man with a step-per-interaction clock so a demonstration's timing is a
    // function of the command sequence, not of interpreter speed. Refused loudly elsewhere: a
    // hardware run's dead-man is a real-time promise and must never be quietly stepped.
    let device_clock = match opts.sim_step {
        None => delulu_runtime::ClockMode::Wall,
        Some(0) => {
            eprintln!("error: --sim-step needs a positive number of simulated milliseconds");
            return 2;
        }
        Some(ms) => {
            if !matches!(device_profile, delulu_runtime::Profile::Sim { .. }) {
                eprintln!(
                    "error: --sim-step is only meaningful with --broker-profile sim; the wall-clock \
                     dead-man is a real-time guarantee and is not stepped"
                );
                return 2;
            }
            delulu_runtime::ClockMode::Stepped { step_us: ms.saturating_mul(1000) }
        }
    };


    // A `.dwx` is a pre-built, authority-carrying artifact — re-verify and run it directly.
    if file.ends_with(".dwx") {
        return run_dwx_artifact(&file, &opts);
    }
    // A package DIRECTORY: resolve the dependency graph, check it authoritatively, then flatten it
    // for execution (ruling D61, closing C59 — `kind = "bin"` was declarable and unexecutable).
    let is_package = std::path::Path::new(&file).is_dir();
    let (map, checked) = if is_package {
        match load_package_for_run(&file, &opts) {
            Ok(x) => x,
            Err(c) => return c,
        }
    } else {
        let (map, id, src) = match load(&file) {
            Ok(x) => x,
            Err(c) => return c,
        };
        let checked = check_source(id, &src);
        (map, checked)
    };
    if errors(&checked.diagnostics) > 0 {
        print_diagnostics("run", &checked.diagnostics, &map, None, opts.json);
        return 1;
    }
    if !checked.result.main_present {
        eprintln!("error: `{file}` has no `fn main(root: Root)` to run");
        return 2;
    }

    // Grant flow (§7.2): manifest DL0701 check, then reconcile grants. For a package the manifest
    // is the package's own; for a single file it is whatever sits beside it.
    let file_dir = std::path::Path::new(&file).parent().unwrap_or_else(|| std::path::Path::new("."));
    let dir = if is_package { std::path::Path::new(&file) } else { file_dir };
    let manifest = std::fs::read_to_string(dir.join("delulu.toml")).ok().map(|s| parse_manifest(&s));
    let empty = std::collections::BTreeSet::new();
    let main_row = checked.result.main_row.as_ref().unwrap_or(&empty);

    let mut grants = Grants::default();
    if let Some(m) = &manifest {
        let d = m.check_main_row(main_row);
        if !d.is_empty() {
            print_diagnostics("run", &d, &map, None, opts.json);
            return 1;
        }
        if opts.grant_manifest {
            grants.accept_manifest(m);
        }
    }
    for g in &opts.grants {
        if let Err(e) = grants.add(g) {
            eprintln!("error: bad --grant: {e}");
            return 2;
        }
    }

    // ----- the compute attestation gate (Stage 10 phase 10h, spec §7.1 — DL1911) -----------------
    // Decided here, before a line of the program runs, for a sharper reason than convenience: the
    // claim this gate protects is that a device envelope is enforced in TWO places. If the adapter
    // cannot attest a layer below itself, the claim is single — and the run must either say so out
    // loud, via an explicit human waiver in the grant, or not happen. *Silently* single is the one
    // outcome this code exists to prevent.
    for env in &grants.computes {
        if let Err(refusal) = delulu_runtime::compute::check_grant(env) {
            let d = Diagnostic::error(
                "DL1911",
                format!(
                    "`{}`: {} [a human must accept single enforcement for this device; see \
                     `delulu explain DL1911` and spec §7.1]",
                    env.device,
                    refusal.message()
                ),
            );
            print_diagnostics("run", &[d], &map, None, opts.json);
            return 1;
        }
    }

    // Stage 10 (invariant 46): a `@jit` hint without the `exec.native` grant is IGNORED — the
    // program runs interpreted, sandbox intact — and DL1906 says so on the human channel
    // (stderr, the DL1790 surface-warning pattern; never inside `--json`, whose bytes for
    // untouched programs are law). A hint may not change whether a program runs (invariant 45),
    // so this is a warning by construction, never a refusal.
    if module_requests_native(&checked.module) && !grants.exec_native {
        let mut w = Diagnostic::warning(
            "DL1906",
            "`@jit` hint ignored: native-code emission was not granted (`--grant exec.native`)",
        );
        if manifest.as_ref().is_some_and(|m| !m.exec_native) {
            w.message.push_str(
                " — and the manifest does not declare the request (`[authority] exec.native = true`)",
            );
        }
        if !opts.json {
            let wmap = SourceMap::new();
            let palette = palette_stderr();
            eprint!("{}", render_human_with(&w, &wmap, &palette));
        }
    }

    // ----- lease redemption (Stage 5 phase 5j, spec §3.2/§3.3) ----------------------------------
    // `--lease <token>` redeems a delegated lease and runs under EXACTLY that node: the local
    // grants are DERIVED from the node's authority (never widened locally — the broker enforces per
    // §4 regardless), and custody binds to the redeemed node via `for_node`. This is the
    // orchestration payoff: whoever holds a grant delegates a slice, hands the token to an agent,
    // and the agent runs under exactly that attenuated authority. Fail closed: broker down ⇒
    // DL1401; a bad/expired/already-redeemed token ⇒ DL1407/DL1402 — before `main` ever runs.
    let lease_mode = opts.lease.is_some();
    let mut lease_custody: Option<crate::broker_client::BrokerClientCustody> = None;
    if let Some(token) = &opts.lease {
        if opts.broker.as_deref() == Some("embedded") {
            eprintln!("error: `--lease` runs under the broker daemon; it cannot be combined with `--broker embedded`");
            return 2;
        }
        // The lease IS the authority. Local `--grant` flags may only supply `foreign.c` binary
        // PATHS (the path is grant data — a human decision; the lease's `foreign.c` scope still
        // bounds WHICH libs, enforced by the broker's ForeignBind check). Anything else would be a
        // confusing local widening the broker would deny anyway — refuse it up front.
        // 10g: device grants are named here explicitly, and the reason is worth recording. This
        // list enumerates what a lease run may NOT be given locally, so every grant kind added
        // after it was written fell through to `else` and was silently DISCARDED by
        // `grants_from_lease` below. That is exactly what happened to `actuator=`/`sensor=`: the
        // operator typed a device grant, was told nothing, and the program then died at the mint
        // with `DL0703: actuator was not granted` — a diagnostic that blames the program for the
        // CLI having thrown the grant away. A refusal list is a skip branch wearing a disguise.
        if grants.console
            || grants.clock
            || grants.rand
            || grants.declassify
            || !grants.fs_read.is_empty()
            || !grants.fs_write.is_empty()
            || !grants.net.is_empty()
            || !grants.secrets.is_empty()
            || !grants.foreign_python.is_empty()
            || opts.grant_manifest
        {
            eprintln!(
                "error: a `--lease` run derives its authority from the delegated node — only \
                 `--grant foreign.c=LIB:PATH` (the binary path, which is grant data) may accompany it"
            );
            return 2;
        }
        // Devices get their own refusal, because the honest answer is not "you may not" but "this
        // cannot be delegated yet". A grant tree node carries the authority to actuate; it cannot
        // yet carry an ENVELOPE (`Scopes` has no actuator dimension), so there is no way for the
        // delegating side to say *how far* the holder may move a machine. Rather than run with the
        // device silently absent, say which grant was refused and why.
        // Two different refusals now, because the two cases stopped having the same reason.
        //
        // An actuator IS expressible in a grant node since RFC 0001 F1, so the refusal is no longer
        // "this cannot be bounded" — it is "you do not get to bound it yourself." A holder that
        // could hand itself a local envelope would be choosing its own corridor, which is precisely
        // the authority the delegating side is supposed to hold.
        if !grants.actuators.is_empty() {
            eprintln!(
                "error: a `--lease` run cannot take a local `--grant actuator=`: its device \
                 authority comes FROM the delegation, bounded by whoever delegated it. Put the \
                 envelope on the delegation instead:\n  \
                 delulu grants delegate --effects Actuate --device \
                 'arm0/elbow:angle_deg=-30..95,heartbeat_ms=200,ttl_ms=60000,fail=hold'"
            );
            return 2;
        }
        // A sensor still has no scope dimension at all, so this half of the old refusal stands
        // unchanged, with its original reason (build-order D12e, narrowed to sensors).
        if !grants.sensors.is_empty() {
            eprintln!(
                "error: a `--lease` run cannot take a local `--grant sensor=`: a sensor read is \
                 `Read` under a sensor scope, and `Scopes` has no sensor dimension yet, so the \
                 delegating side could not bound it. Run the program under `--broker daemon` with \
                 its own sensor grants instead (Stage 10 build order, ruling D12e)."
            );
            return 2;
        }
        let Some(state_dir) = crate::brokerd::resolve_state_dir(None) else {
            eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE)");
            return 2;
        };
        let fail = |code: &str, message: String| -> i32 {
            let d = Diagnostic::error(crate::broker_client::static_code(code), message);
            print_diagnostics("run", &[d], &map, None, opts.json);
            1
        };
        let unreachable_msg = |e: &dyn std::fmt::Display| {
            format!(
                "broker unreachable: {e} — start it with `delulu broker start` \
                 (fail closed, invariant 27: a lease run never falls back to embedded custody)"
            )
        };
        let peer = format!("pid:{}", std::process::id());
        let node = match crate::brokerd::request(
            &state_dir,
            crate::broker_ipc::ReqBody::Redeem { token: token.clone(), peer },
        ) {
            Ok(crate::broker_ipc::Response::Redeemed { node }) => node,
            Ok(crate::broker_ipc::Response::Error { code, message, .. }) => return fail(&code, message),
            Ok(other) => return fail("DL1401", format!("unexpected redeem response: {other:?}")),
            Err(e) => return fail("DL1401", unreachable_msg(&e)),
        };
        // Learn this run's OWN node's authority (its slice — never a parent's or a sibling's;
        // invariant 25 is upheld by what the ops expose, and this asks only about itself).
        let info = match crate::brokerd::request(
            &state_dir,
            crate::broker_ipc::ReqBody::Inspect { node: node.clone() },
        ) {
            Ok(crate::broker_ipc::Response::Inspected { node }) => node,
            Ok(crate::broker_ipc::Response::Error { code, message, .. }) => return fail(&code, message),
            Ok(other) => return fail("DL1401", format!("unexpected inspect response: {other:?}")),
            Err(e) => return fail("DL1401", unreachable_msg(&e)),
        };
        let authority = crate::brokerd::spec_to_authority(&info.authority_spec());
        grants = grants_from_lease(&info, std::mem::take(&mut grants.foreign_c));
        // The guard awareness line (addendum §2.7, criterion 8): EVERY `run --lease` prints one
        // guard status line before user code output — stderr always (in `--json` mode too: stdout
        // stays the machine surface, decorations ride stderr — the dcg robot-mode convention).
        // Sourced from a GuardStatus wire call post-redeem; an unreachable broker here would already
        // have failed the redeem above, but fail closed anyway.
        match crate::brokerd::request(&state_dir, crate::broker_ipc::ReqBody::GuardStatus) {
            Ok(crate::broker_ipc::Response::GuardStatus { bypass, poisoned, rules, .. }) => {
                eprintln!("{}", lease_guard_status_line(bypass, poisoned, &rules));
            }
            Ok(other) => return fail("DL1401", format!("unexpected guard status response: {other:?}")),
            Err(e) => return fail("DL1401", unreachable_msg(&e)),
        }
        let custody = crate::broker_client::BrokerClientCustody::for_node(
            state_dir,
            delulu_broker::GrantId::from_trusted(node),
            authority,
            opts.epoch_ms,
        );
        if !opts.json {
            eprintln!("lease: running under delegated node `{}`", custody.node());
        }
        lease_custody = Some(custody);
    }

    // Foreign grant flow (Stage 4, spec §4.1 / criterion 4): every `foreign` lib the program binds
    // must be permitted by the manifest (if any) and granted a binary at startup. An ungranted lib
    // is DL1303 HERE — before `main` runs — never mid-run. The path is grant data: the human/broker
    // decides which binary satisfies the logical name, prompted with the full symbol list.
    if let Err(code) = foreign_grant_preflight(&checked, &manifest, &mut grants, &opts, &map) {
        return code;
    }

    // ----- custody selection (Stage 5 phase 5f) ------------------------------------------------
    // Default: embedded — byte-identical Stage 1–4 behavior (criterion 11). `--broker daemon`
    // routes every effectful op through the broker daemon: the `--grant` flags become sugar for
    // issue-then-run at the root (spec §3.2), and an unreachable broker is DL1401 BEFORE `main`
    // runs (fail closed, invariant 27 — never a silent fallback to embedded).
    let daemon_mode = match opts.broker.as_deref() {
        // A `--lease` run's custody is the daemon by definition (the node lives there).
        None | Some("embedded") => lease_mode,
        Some("daemon") => true,
        Some(other) => {
            eprintln!("error: unknown --broker mode `{other}` (embedded | daemon)");
            return 2;
        }
    };
    if (opts.broker.is_some() || lease_mode) && !opts.json {
        // The custody label (playbook 5j; printed when custody was explicitly chosen so default
        // embedded output stays byte-identical to prior stages).
        eprintln!("custody: {}", if daemon_mode { "daemon" } else { "embedded" });
    }
    // A lease run already bound custody to the redeemed node above; a plain `--broker daemon` run
    // issues its root here (the `--grant` flags as issue-then-run sugar, spec §3.2).
    let mut daemon_custody: Option<crate::broker_client::BrokerClientCustody> = lease_custody;
    if daemon_mode && daemon_custody.is_none() {
        let Some(state_dir) = crate::brokerd::resolve_state_dir(None) else {
            eprintln!("error: cannot resolve the broker state directory (no HOME/USERPROFILE)");
            return 2;
        };
        let spec = authority_spec_from_grants(&grants, &file);
        match crate::broker_client::BrokerClientCustody::issue_root(state_dir, spec, opts.epoch_ms) {
            Ok(c) => daemon_custody = Some(c),
            Err(d) => {
                let diag = Diagnostic::error(d.code, d.message);
                print_diagnostics("run", &[diag], &map, None, opts.json);
                return 1;
            }
        }
    }

    // ----- foreign isolation selection (Stage 5 phase 5h) --------------------------------------
    // `process` runs each granted C library in an isolated worker subprocess (a worker crash is
    // DL1409, the host survives — spec §5, criterion 7); `inproc` is the Stage-4 in-process path.
    // Default: `process` in daemon mode, `inproc` otherwise. Labeled below.
    let foreign_process = match opts.foreign_isolation.as_deref() {
        Some("process") => true,
        Some("inproc") => false,
        Some(other) => {
            eprintln!("error: unknown --foreign-isolation `{other}` (inproc | process)");
            return 2;
        }
        None => daemon_mode,
    };
    let program_binds_foreign = !checked.result.foreign_binds.is_empty();
    if program_binds_foreign && (opts.foreign_isolation.is_some() || daemon_mode) && !opts.json {
        eprintln!("foreign-isolation: {}", if foreign_process { "process" } else { "inproc" });
    }

    // WASM engine (spec §5.11 / Stage 3; foreign C FFI in Stage 4 phase 4g): compile `main` to
    // WebAssembly and run it under the deny-by-default Wasmtime host, with capabilities minted and
    // checked host-side and all foreign FFI performed host-side (§4.3). Constructs the backend can't
    // compile are DL1201 — omit `--engine wasm` to use the interpreter.
    if opts.engine.as_deref() == Some("wasm") {
        // Phase-5h scope note: worker isolation is wired on the INTERPRETER engine (the criterion-7
        // reference). The WASM host's foreign path stays in-process for v0.5 — flagged in spec §11.
        if foreign_process && program_binds_foreign && !opts.json {
            eprintln!("note: --foreign-isolation process is not yet applied on the WASM engine (foreign runs in-process); use the interpreter engine for worker isolation");
        }
        let wasm = match delulu_wasm::compile_module_with(&checked.module, &checked.result.foreign_binds) {
            Ok(w) => w,
            Err(e) => {
                let d = Diagnostic::error(e.code(), format!("{} — omit `--engine wasm` to run it on the interpreter", e.message()));
                print_diagnostics("run", &[d], &map, None, opts.json);
                return 1;
            }
        };
        // Effect tracing on the WASM engine (spec §6.1; criterion 6): the host records the same
        // `TraceRecord`s the interpreter would, so a foreign program's trace is byte-identical.
        let sink = if opts.assert_trace {
            Some(delulu_runtime::TraceSink::new())
        } else if opts.trace_effects {
            Some(delulu_runtime::TraceSink::bounded(TRACE_RECORD_CAP))
        } else {
            None
        };
        let max_ret = opts.foreign_max_ret.unwrap_or(delulu_runtime::foreign::DEFAULT_MAX_RET);
        let root = grants.build_root();
        // Stage 5 phase 5f: the WASM host calls the SAME custody seam as the interpreter. A broker
        // denial inside a host callback is recorded in HostState and surfaced after the call —
        // never an Err across the wasm frame (playbook trap 5).
        let wasm_custody: Option<delulu_wasm::CustodyHandle> = daemon_custody
            .take()
            .map(|c| Rc::new(std::cell::RefCell::new(Box::new(c) as Box<dyn delulu_runtime::Custody>)));
        let cfg = delulu_wasm::HostConfig {
            console: grants.console,
            clock: grants.clock,
            rand: grants.rand,
            fs_read_roots: root.fs_read.clone(),
            fixed_clock_ms: opts.clock_ms,
            rand_seed: opts.seed,
            foreign_load: root.foreign_load,
            foreign_grants: grants.foreign_c.clone(),
            foreign_sigs: delulu_wasm::foreign_sigs(&checked.module),
            foreign_max_ret: max_ret,
            trace: sink.clone(),
            custody: wasm_custody.clone(), // Stage 5 phase 5f: Some(...) in daemon mode, None embedded
        };
        // Stage 7 phase 7h: a module with actors runs under the WASM engine's COOPERATIVE
        // single-threaded scheduler (spec §6.5 — identical semantics, no parallelism,
        // labeled in output). A module without actors takes the byte-identical v0.6 path.
        let wasm_has_actors =
            checked.module.items.iter().any(|it| matches!(it, delulu_syntax::ast::Item::Actor(_)));
        let run_result = if wasm_has_actors {
            eprintln!("engine: wasm (actors: cooperative single-threaded — semantics identical, parallelism absent)");
            let table = delulu_wasm::actor_table(&checked.module);
            delulu_wasm::run_main_actors(&wasm, &cfg, &table).map(|(output, report)| {
                if report.dead_actors > 0 || report.dropped_sends > 0 {
                    eprintln!(
                        "actors: {} died; {} message(s) to dead actors dropped",
                        report.dead_actors, report.dropped_sends
                    );
                }
                if opts.on_quiesce_report {
                    eprintln!(
                        "quiesce: {} surviving actor(s), {} turn(s) run",
                        report.surviving_actors, report.total_turns
                    );
                }
                output
            })
        } else {
            delulu_wasm::run_main(&wasm, &cfg)
        };

        // Emit the trace before verdicts (the witness is available even on a fault).
        if let Some(s) = &sink {
            if opts.trace_effects {
                let lines = s.to_json_lines();
                match &opts.trace_out {
                    Some(path) => {
                        if let Err(e) = std::fs::write(path, lines + "\n") {
                            eprintln!("error: cannot write trace to `{path}`: {e}");
                        }
                    }
                    None => eprintln!("{lines}"),
                }
            }
        }

        let code = match run_result {
            Ok(output) => {
                print!("{output}");
                0
            }
            Err(e) => {
                let msg = e.message();
                let d = Diagnostic::error(wasm_fault_code(&msg), format!("WASM engine: {msg}"));
                print_diagnostics("run", &[d], &map, None, opts.json);
                1
            }
        };

        // The trace ⊆ row law (spec §6.2, invariant 12): a violation is compiler-bug class (DL1101).
        if opts.assert_trace {
            if let Some(s) = &sink {
                let allowed: std::collections::BTreeSet<String> =
                    main_row.iter().map(|e| e.name().to_string()).collect();
                let violations = delulu_runtime::assert_trace(&allowed, &s.records());
                if !violations.is_empty() {
                    let diags: Vec<Diagnostic> = violations
                        .iter()
                        .map(|v| Diagnostic::error("DL1101", format!("effect-trace assertion violation: {v} — this is a compiler-bug class failure; please report it")))
                        .collect();
                    print_diagnostics("run", &diags, &map, None, opts.json);
                    return 3;
                }
            }
        }
        return code;
    }

    // Determinism knobs (spec §6.2).
    if let Some(seed) = opts.seed {
        delulu_runtime::set_rand_seed(seed);
    }
    if let Some(ms) = opts.clock_ms {
        delulu_runtime::set_fixed_clock_ms(Some(ms));
    }

    // Effect tracing (spec §6.1) — also attached when --assert-trace needs the witness.
    let sink = if opts.assert_trace {
        // `--assert-trace` proves no effect outside the declared set occurred, so it needs EVERY
        // record: a dropped one could hide the violation (C56/D49).
        Some(delulu_runtime::TraceSink::new())
    } else if opts.trace_effects {
        Some(delulu_runtime::TraceSink::bounded(TRACE_RECORD_CAP))
    } else {
        None
    };

    let mut root_val = build_root(&grants);
    if daemon_mode {
        // Phase 5g: in daemon mode the program gets opaque broker-secret HANDLES — the byte values
        // live in the broker's store, never in this process pre-`expose` (invariant 23). The grant
        // names select which broker secrets are reachable; any locally-supplied values are DROPPED.
        let mut names: Vec<String> = root_val.secrets.keys().cloned().collect();
        names.sort();
        root_val.broker_secrets = names;
        root_val.secrets.clear();
    }
    let root = Value::Root(Rc::new(root_val));
    let max_ret = opts.foreign_max_ret.unwrap_or(delulu_runtime::foreign::DEFAULT_MAX_RET);
    let mut interp = Interp::new(&checked.module).with_foreign(
        checked.result.foreign_binds.clone(),
        grants.foreign_c.clone(),
        max_ret,
    );
    if foreign_process {
        // Stage 5 phase 5h: run each granted C library in an isolated worker subprocess. Additive —
        // a program with no foreign binds never spawns a worker (the binder is only consulted by
        // `root.foreign(load)`).
        match crate::foreign_worker::WorkerBinder::new() {
            Ok(b) => interp = interp.with_foreign_binder(Rc::new(b)),
            Err(e) => {
                eprintln!("error: cannot set up foreign process isolation (locating the worker executable): {e}");
                return 2;
            }
        }
    }
    // 10g: give each actuator its own child node under this run's, then build the e-stop probe
    // from them BEFORE custody moves into the interpreter. Two reasons for the child nodes rather
    // than watching the run's own node: `grants revoke` on a device stops that device and leaves
    // the program its console to report the loss with (spec §5.2's *subtree*), and revoking the
    // parent still reaches every device transitively — the tree already does that.
    // The device nodes' ids travel separately from the probe so the run can revoke them on the way
    // out; the probe itself has moved into the watchdog thread by then.
    let mut device_nodes: Vec<String> = Vec::new();
    let device_state_dir: Option<std::path::PathBuf> =
        daemon_custody.as_ref().map(|c| c.state_dir().to_path_buf());
    let authority_probe: Option<delulu_runtime::AuthorityProbe> = if grants.actuators.is_empty() {
        None
    } else {
        match daemon_custody.as_mut().map(|c| mint_device_nodes(c, &grants.actuators)) {
            None => None,
            Some(Ok((probe, ids))) => {
                device_nodes = ids;
                Some(probe)
            }
            Some(Err(d)) => {
                // Fail closed: if a device's own node cannot be minted, the e-stop has nothing to
                // aim at, and a run holding a machine with no way to stop it must not start.
                let diag = Diagnostic::error(d.code, d.message);
                print_diagnostics("run", &[diag], &map, None, opts.json);
                return 1;
            }
        }
    };
    if let Some(c) = daemon_custody.take() {
        // Stage 5 phase 5f: route every effectful op through the broker daemon.
        interp = interp.with_custody(Box::new(c));
    }
    if let Some(s) = &sink {
        interp = interp.with_trace(s.clone());
    }
    // Stage 10 (10f): the device broker starts its dead-man the moment the leases exist, which is
    // BEFORE `main` runs. A program that never reaches its first command still holds a lease it
    // is not beating, and the watchdog treats that exactly like any other silence.
    let devices = if grants.actuators.is_empty() && grants.sensors.is_empty() {
        None
    } else {
        // RFC 0001 dish 3: under `hw:` the driver is a separate process. It is spawned HERE,
        // after the DL1905 approval gate above has already refused an unapproved artifact — so a
        // hardware driver is never started for bytes a human did not sign off on.
        let hw_adapter = match (&device_profile, &opts.adapter_cmd) {
            (delulu_runtime::Profile::Hw { adapter }, Some(cmd)) => {
                let mut parts = cmd.split_whitespace();
                let Some(prog) = parts.next() else {
                    eprintln!("error: --adapter-cmd is empty");
                    return 2;
                };
                let args: Vec<String> = parts.map(str::to_string).collect();
                // Provenance, BEFORE the driver is spawned (D52, closing the gap D23 named).
                let (prov, gate) = check_adapter_signature(
                    prog,
                    opts.adapter_artifact.as_deref(),
                    opts.require_signed_adapter,
                    opts.adapter_signer.as_deref(),
                );
                // Recorded BEFORE the refusal is acted on, so a run stopped because the driver was
                // signed by the wrong key leaves the evidence that it happened (C60).
                if let Err(code) = record_adapter_provenance(&prov, opts.adapter_record.as_deref()) {
                    return code;
                }
                if let Some(code) = gate {
                    return code;
                }
                match delulu_runtime::adapter::ProcessAdapter::spawn(adapter, prog, &args) {
                    Ok(a) => Some(a),
                    Err(e) => {
                        // Fail closed and BEFORE `main`: a run that could not start its driver must
                        // not begin, or the program would discover the machine is unreachable
                        // partway through a motion.
                        eprintln!("error: {e}");
                        return 1;
                    }
                }
            }
            _ => None,
        };
        let b = std::sync::Arc::new(delulu_runtime::DeviceBroker::with_adapter(
            device_profile.clone(),
            &grants.actuators,
            &grants.sensors,
            authority_probe,
            device_clock,
            hw_adapter,
        ));
        interp = interp.with_devices(b.clone());
        Some(b)
    };
    // Stage 10 (10h): the compute broker, on the same gate — no compute grants, no adapter bound,
    // and a dispatch would answer `NoAdapter` rather than a number nobody computed.
    let computes = if grants.computes.is_empty() {
        None
    } else {
        // Kernels are DATA: every artifact is read, its detached signature verified, and its
        // content hashed BEFORE the program starts. A dispatch later resolves against these
        // verified artifacts, never against the grant string — so an artifact that failed
        // verification is not merely reported, it is absent, and nothing can run it.
        let mut broker = delulu_runtime::compute::ComputeBroker::new(&grants.computes);
        for env in &grants.computes {
            let mut arts = Vec::new();
            for (name, path) in &env.kernels {
                match delulu_runtime::compute::load_kernel(name, path, &env.formats) {
                    Ok(a) => arts.push(a),
                    Err(refusal) => {
                        let d = Diagnostic::error(
                            refusal.code(),
                            format!("`{}`: {}", env.device, refusal.message()),
                        );
                        print_diagnostics("run", &[d], &map, None, opts.json);
                        return 1;
                    }
                }
            }
            broker = broker.with_artifacts(&env.device, arts);
        }
        let b = std::sync::Arc::new(broker);
        interp = interp.with_computes(b.clone());
        Some(b)
    };
    // Stage 7 (phase 7g): a module with actors gets the actor system. Gated on declaration —
    // a program with no actors takes the identical path to v0.6, byte for byte.
    let has_actors =
        checked.module.items.iter().any(|it| matches!(it, delulu_syntax::ast::Item::Actor(_)));
    let mut actor_trace: Option<std::sync::Arc<std::sync::Mutex<Vec<delulu_runtime::TraceRecord>>>> = None;
    let actor_system = if has_actors {
        let threads = opts
            .actors_threads
            .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1));
        if sink.is_some() {
            actor_trace = Some(std::sync::Arc::new(std::sync::Mutex::new(Vec::new())));
        }
        let debug_set = opts
            .debug_rcaps
            .then(|| std::sync::Arc::new(checked.result.iso_moves.clone()));
        // Stage 10 (10c): the manifest's `[actors]` defaults. A decl-level `(mailbox = N)`
        // wins per actor; anything but the two known overflow values is warned about rather
        // than silently meaning `block` (the skip branch, made audible).
        let mb_default = manifest.as_ref().and_then(|m| m.actors_mailbox).map(|n| n as usize);
        let mb_overflow = manifest.as_ref().and_then(|m| m.actors_overflow.clone());
        if let Some(o) = &mb_overflow {
            if o != "block" && o != "drop-new" {
                eprintln!(
                    "warning: unknown `[actors] overflow = \"{o}\"` — valid values are \"block\" and \"drop-new\"; using \"block\""
                );
            }
        }
        let system = delulu_runtime::actors::ActorSystem::start_with(
            &checked.module,
            threads,
            opts.on_actor_death_abort,
            actor_trace.clone(),
            debug_set.clone(),
            mb_default,
            mb_overflow.as_deref() == Some("drop-new"),
        );
        interp = interp.with_actors(system.host());
        if let Some(d) = debug_set {
            interp = interp.with_debug_rcaps(d);
        }
        Some(system)
    } else {
        None
    };
    let run_result = interp.run_main(root);
    // Quiescence exit (spec §6.1): `main` returned AND all mailboxes empty AND no turn
    // running — then report per flags.
    let mut actor_abort = false;
    if let Some(system) = actor_system {
        let report = system.finish();
        // Merge worker-collected causal records into the main sink (seq renumbered to stay
        // strictly increasing; cross-actor order is by globally monotonic turn id).
        if let (Some(collector), Some(s)) = (&actor_trace, &sink) {
            let mut recs = collector.lock().unwrap().clone();
            recs.sort_by_key(|r| r.turn);
            let mut seq = s.len() as u64;
            for mut r in recs {
                r.seq = seq;
                seq += 1;
                s.append(r);
            }
        }
        actor_abort = report.aborted;
        if report.dead_actors > 0 || report.dropped_sends > 0 {
            eprintln!(
                "actors: {} died; {} message(s) to dead actors dropped",
                report.dead_actors, report.dropped_sends
            );
        }
        // Stage 10 (10c): overflow drops are never silent — counted and reported whenever they
        // happened, and itemized per actor under `--trace-memory` (spec §3 B3, mailbox half).
        if report.overflow_drops > 0 {
            eprintln!("actors: {} message(s) dropped by mailbox overflow (`drop-new`)", report.overflow_drops);
        }
        if opts.trace_memory {
            eprintln!("trace-memory: {} bounded mailbox(es)", report.mailbox.len());
            for m in &report.mailbox {
                eprintln!(
                    "  {}: bound {}, peak depth {}, overflow drops {}",
                    m.actor, m.bound, m.peak, m.drops
                );
            }
            // 10d: collector accounting — cell COUNTS, deliberately not bytes (build-order D9:
            // a byte figure without a real size walk would be an invented number).
            eprintln!(
                "  cycle collector: {} sweep(s), {} garbage-cycle cell(s) collected",
                report.cycle_runs, report.cycle_collected
            );
        }
        if opts.on_quiesce_report {
            eprintln!(
                "quiesce: {} surviving actor(s), {} turn(s) run",
                report.surviving_actors, report.total_turns
            );
        }
    }
    if actor_abort {
        eprintln!("aborting: an actor died and --on-actor-death abort is set");
        return 1;
    }

    // Stage 10 (10f): stop the watchdog before any verdict is reported, so a run never leaves a
    // thread deciding things about physical devices after the program is over. Then merge its
    // journal into the trace — the watchdog runs on its own thread and `TraceSink` is an `Rc`, so
    // device events arrive here the same way Stage 7's worker records do. They carry `at_ms`
    // because appending them last would otherwise misrepresent when they happened.
    if let Some(b) = &devices {
        b.shutdown();
        let events = b.events();
        if let Some(s) = &sink {
            let base = s.len() as u64;
            for (i, e) in events.iter().enumerate() {
                s.append(delulu_runtime::TraceRecord {
                    seq: base + i as u64,
                    effect: "Actuate".to_string(),
                    op: e.op.clone(),
                    cap_kind: "Actuator".to_string(),
                    detail: Some(format!("[+{} ms] {}", e.at_ms, e.detail)),
                    span: None,
                    ..Default::default()
                });
            }
        }
        // A lost device is never silent, trace or no trace: losing an actuator mid-run is the
        // single most consequential thing that can happen to a program in this language.
        for e in events.iter().filter(|e| e.op == "lease.revoked") {
            eprintln!("devices: {}", e.detail);
        }
        // 10g: and the device's grant node dies with the run that minted it, so `grants list`
        // never offers an operator an arm that nobody holds (see `revoke_device_nodes`).
        if let Some(dir) = &device_state_dir {
            revoke_device_nodes(dir, &device_nodes);
        }
    }

    // Stage 10 (10h): compute telemetry. Every dispatch that reached the adapter is summarised on
    // stderr — how many elements crossed, how long the kernel took — and every refusal is named.
    // A dispatch is foreign code doing work on a device the human paid to bound; the run says what
    // actually happened rather than leaving it to be inferred from a return value.
    if let Some(cb) = &computes {
        let records = cb.records();
        if !records.is_empty() && !opts.json {
            let landed = records.iter().filter(|r| r.refused.is_none()).count();
            let refused = records.len() - landed;
            eprintln!("compute: {landed} dispatch(es) ran, {refused} refused");
            for r in records.iter() {
                if let Some((code, why)) = &r.refused {
                    eprintln!("compute: {code} {}/{}: {why}", r.device, r.kernel);
                }
            }
        }
    }

    // Emit the trace before verdicts, so the witness is available even on a fault.
    if let Some(s) = &sink {
        if opts.trace_effects {
            let lines = s.to_json_lines();
            match &opts.trace_out {
                Some(path) => {
                    if let Err(e) = std::fs::write(path, lines + "\n") {
                        eprintln!("error: cannot write trace to `{path}`: {e}");
                    }
                }
                None => eprintln!("{lines}"),
            }
            // A truncated trace says so, with the number withheld and how to get the rest — the
            // same contract D38 gave the diagnostic flood. A trace that silently stopped recording
            // would be worse than one that grew: the reader would conclude the effects stopped.
            if s.dropped() > 0 {
                eprintln!(
                    "note: {} further effect record(s) not traced — `--trace-effects` retains the \
                     first {TRACE_RECORD_CAP} so a long run cannot exhaust memory (campaign C56). \
                     `--assert-trace` is never capped, because it must see every effect to be sound.",
                    s.dropped()
                );
            }
        }
    }

    let code = match run_result {
        Ok(_) => 0,
        Err(fault) => {
            let mut d = Diagnostic::error(fault.code, fault.message.clone());
            if let Some(span) = fault.span {
                d = d.with_span(span, "runtime fault here");
            }
            print_diagnostics("run", &[d], &map, None, opts.json);
            1
        }
    };

    // The trace ⊆ row law (spec §6.2, invariant 12): a violation is compiler-bug class (DL1101)
    // and gets the dedicated exit code 3.
    if opts.assert_trace {
        if let Some(s) = &sink {
            let allowed: std::collections::BTreeSet<String> =
                main_row.iter().map(|e| e.name().to_string()).collect();
            // Stage 7 (invariant 35 executable): actor-attributed records check against the
            // EXECUTING member's row AND the send site's row via the cause chain; main-line
            // records keep the original law byte-for-byte.
            let member_rows: std::collections::HashMap<String, std::collections::BTreeSet<String>> = checked
                .result
                .facts
                .iter()
                .filter(|(k, _)| k.contains('.'))
                .map(|(k, f)| (k.clone(), f.effects.iter().map(|e| e.name().to_string()).collect()))
                .collect();
            let violations = if has_actors {
                delulu_runtime::assert_trace_causal(&allowed, &member_rows, &s.records())
            } else {
                delulu_runtime::assert_trace(&allowed, &s.records())
            };
            if !violations.is_empty() {
                let diags: Vec<Diagnostic> = violations
                    .iter()
                    .map(|v| Diagnostic::error("DL1101", format!("effect-trace assertion violation: {v} — this is a compiler-bug class failure; please report it")))
                    .collect();
                print_diagnostics("run", &diags, &map, None, opts.json);
                return 3;
            }
        }
    }

    // Stage 10 (10f, invariant 48): the sign-off is written only for a run that actually finished
    // clean under the simulator. Approving an artifact whose sim run FAULTED would be the gate
    // certifying the thing it exists to catch.
    if let Some(path) = &opts.signoff {
        if code == 0 {
            if let Err(e) = write_signoff(&file, path, &device_profile) {
                eprintln!("error: {e}");
                return 2;
            }
        } else {
            eprintln!("sign-off withheld: the run did not complete cleanly, so `{path}` was not written");
        }
    }

    code
}

/// The default simulator seed. Named rather than inlined so a run without `--seed` is still a
/// reproducible run — an unseeded simulator that quietly picked a fresh seed each time would make
/// "deterministic under `--seed`" true and useless.
const DEFAULT_SIM_SEED: u64 = 0xDE1;

/// Resolve `--broker-profile` and, for a hardware profile, run the artifact-hash gate
/// (spec §5.4, invariant 48, DL1905) before anything executes.
///
/// The gate has three outcomes and the middle one is the whole reason it exists:
///   - no sign-off record at all → REFUSED. "I could not tell" is the skip branch, and the skip
///     branch says no. A gate that opens when it cannot find its evidence is not a gate.
///   - a sign-off for different bytes → DL1905, `requires_human: true`.
///   - a matching sign-off → the gate PASSES, and the run proceeds only if a driver was actually
///     named with `--adapter-cmd` (RFC 0001 dish 3, build-order D23). Without one it stops, because
///     a `hw:` run that commanded nothing while reporting success would be a lie.
///
/// Note the distinction D23 introduced and every surface must keep: this tree ships the **adapter
/// mechanism** — a subprocess line protocol — and **no driver for any real device**. So a `hw:` run
/// can now reach code outside DeluluLang, and still nothing physical moves unless an operator
/// supplies a driver that touches hardware.
fn resolve_device_profile(file: &str, opts: &Opts) -> Result<delulu_runtime::Profile, i32> {
    let Some(spec) = opts.broker_profile.as_deref() else {
        return Ok(delulu_runtime::Profile::Null);
    };
    if spec == "sim" {
        return Ok(delulu_runtime::Profile::Sim { seed: opts.seed.unwrap_or(DEFAULT_SIM_SEED) });
    }
    let Some(adapter) = spec.strip_prefix("hw:") else {
        eprintln!("error: unknown --broker-profile `{spec}` (sim | hw:<adapter>)");
        return Err(2);
    };
    if adapter.is_empty() {
        eprintln!("error: --broker-profile hw: needs an adapter name");
        return Err(2);
    }
    let bytes = match std::fs::read(file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: cannot read `{file}` to check it against its sign-off: {e}");
            return Err(2);
        }
    };
    let actual = delulu_broker::content_hash(&bytes);
    let Some(approval_path) = opts.approved.as_deref() else {
        let d = Diagnostic::error(
            "DL1905",
            format!(
                "`--broker-profile hw:{adapter}` was requested for `{file}` with no sign-off \
                 record — pass `--approved <record>` naming the simulation that approved these \
                 exact bytes (`--broker-profile sim --signoff <record>`). This artifact hashes to \
                 {actual} [a human must approve hardware actuation for this artifact; see \
                 `delulu explain DL1905` and spec §5.4]"
            ),
        );
        print_diagnostics("run", &[d], &SourceMap::new(), None, opts.json);
        return Err(1);
    };
    let src = match std::fs::read_to_string(approval_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read the sign-off record `{approval_path}`: {e}");
            return Err(2);
        }
    };
    let approval = match delulu_runtime::Approval::parse(&src) {
        Ok(a) => a,
        Err(e) => {
            // An unreadable approval is not an absent approval — it is a damaged one, and the
            // answer to damage at a physical boundary is no.
            eprintln!("error: the sign-off record `{approval_path}` is not readable as an approval: {e}");
            return Err(2);
        }
    };
    if approval.hash != actual {
        let d = Diagnostic::error(
            "DL1905",
            format!(
                "`{file}` does not match its sign-off: the simulator approved {} but these bytes \
                 hash to {actual} — re-run the simulation and re-approve, or restore the approved \
                 artifact. The code that moved something in simulation is the only code this \
                 grant covers [a human must re-approve; see `delulu explain DL1905` and spec §5.4]",
                approval.hash
            ),
        );
        print_diagnostics("run", &[d], &SourceMap::new(), None, opts.json);
        return Err(1);
    }
    eprintln!(
        "device sign-off: OK — `{file}` matches the artifact approved under `{}` ({})",
        approval.profile, approval.hash
    );
    // Gate passed. What may run now depends on whether a driver was actually supplied.
    //
    // RFC 0001 dish 3 attached the FIRST real adapter: an operator-supplied subprocess speaking a
    // line protocol (`delulu_runtime::adapter`). That is deliberately NOT the mechanism spec §5.4
    // anticipated — §5.4 describes Verified-class Stage-6 plugins with `require_signed: true`, and
    // none of those ships in-tree either. The difference is worth stating rather than blurring,
    // because the two buy different things:
    //
    //   * A signed Verified plugin would give SUPPLY-CHAIN assurance — you would know who wrote the
    //     driver, and the loader would refuse an unsigned one.
    //   * `--adapter-cmd` gives ISOLATION and reach — the driver is a separate process, so it
    //     cannot corrupt the runtime, and it can be any program on the machine (a serial bridge, a
    //     CAN gateway, a vendor SDK shim). It carries NO signature check whatever.
    //
    // So: DL1905 approves the ARTIFACT — the DeluluLang program's exact bytes — and says nothing
    // about the driver. The operator chooses the driver by typing this flag, and the operator is
    // inside the trust boundary (spec §10). What still holds regardless of the driver is the part
    // that matters for custody: the envelope is enforced host-side BEFORE any byte reaches it, so
    // a driver can refuse more and can never permit more. What the driver then does with a
    // permitted command is below the boundary, exactly as invariant 52 says the hardware safety
    // chain must be — it has to work with DeluluLang absent.
    if opts.adapter_cmd.is_none() {
        eprintln!(
            "error: `--broker-profile hw:{adapter}` needs a driver — pass `--adapter-cmd <command>` \
             naming the process that speaks the device protocol (CMD/READ over stdio; see \
             `delulu_runtime::adapter`). No driver ships in-tree for any hardware, and none is \
             invented here: a `hw:` run that commanded nothing while reporting success would be a \
             lie. To exercise the program without hardware, use `--broker-profile sim`."
        );
        return Err(2);
    }
    Ok(delulu_runtime::Profile::Hw { adapter: adapter.to_string() })
}

/// Write the sim sign-off record (spec §5.4).
fn write_signoff(file: &str, path: &str, profile: &delulu_runtime::Profile) -> Result<(), String> {
    // Only a simulator run can approve an artifact for hardware. A null-adapter run commanded
    // nothing and observed nothing; letting it sign would make the gate a formality.
    let profile_name = match profile {
        delulu_runtime::Profile::Sim { seed } => format!("sim:seed={seed}"),
        _ => {
            return Err(format!(
                "--signoff needs `--broker-profile sim`: an artifact is approved for hardware by \
                 having been exercised in simulation, and this run bound no simulator, so \
                 `{path}` was not written"
            ))
        }
    };
    let bytes = std::fs::read(file).map_err(|e| format!("cannot read `{file}` to sign it off: {e}"))?;
    let approval = delulu_runtime::Approval {
        artifact: file.to_string(),
        hash: delulu_broker::content_hash(&bytes),
        profile: profile_name,
    };
    std::fs::write(path, approval.to_json())
        .map_err(|e| format!("cannot write the sign-off record `{path}`: {e}"))?;
    eprintln!("device sign-off: wrote `{path}` for {} ({})", approval.artifact, approval.hash);
    Ok(())
}

fn build_root(grants: &Grants) -> RootVal {
    grants.build_root()
}

// ----- foreign grant flow + authority (Stage 4, spec §4.1/§6) --------------

/// One `foreign` block, extracted from the AST for the grant prompt and the authority report.
struct ForeignBlock {
    abi: String,
    lib: String,
    symbols: Vec<String>,
    span: delulu_diag::Span,
}

fn foreign_blocks_of(module: &delulu_syntax::ast::Module) -> Vec<ForeignBlock> {
    module
        .items
        .iter()
        .filter_map(|it| match it {
            Item::Foreign(fd) => Some(ForeignBlock {
                abi: fd.abi.clone(),
                lib: fd.name.name.clone(),
                symbols: fd.fns.iter().map(|f| f.name.name.clone()).collect(),
                span: fd.span,
            }),
            _ => None,
        })
        .collect()
}

/// Deny-by-default grant flow for the foreign libs a program binds (spec §4.1 / criterion 4).
/// Returns `Err(exit_code)` when a bound lib exceeds the manifest ceiling or has no grant; on success
/// `grants.foreign_c` holds a binary path for every bound lib. An interactive human is prompted with
/// the full symbol list; agents, `--json`, `--no-prompt`, and non-terminals are never prompted (they
/// get DL1303 and refuse). Everything happens BEFORE `main` runs — never mid-run.
fn foreign_grant_preflight(
    checked: &Checked,
    manifest: &Option<delulu_runtime::Manifest>,
    grants: &mut Grants,
    opts: &Opts,
    map: &SourceMap,
) -> Result<(), i32> {
    use std::io::IsTerminal;

    let mut diags: Vec<Diagnostic> = Vec::new();

    // ----- C libs (spec §4.1) ----------------------------------------------
    let bound: BTreeSet<&String> = checked.result.foreign_binds.values().collect();
    let blocks = foreign_blocks_of(&checked.module);
    let symbols_of = |lib: &str| -> Vec<String> {
        blocks.iter().find(|b| b.lib == lib).map(|b| b.symbols.clone()).unwrap_or_default()
    };
    for lib in bound {
        // Manifest ceiling: a bound lib must be listed in `[authority] foreign.c` when a manifest
        // exists. Exceeding it is the program reaching for authority it never declared (DL1303).
        if let Some(m) = manifest {
            if !m.foreign_c.iter().any(|l| l == lib) {
                diags.push(Diagnostic::error(
                    "DL1303",
                    format!("program uses foreign lib `{lib}` not permitted by the authority manifest (`[authority] foreign.c`)"),
                ));
                continue;
            }
        }
        if grants.foreign_c.contains_key(lib) {
            continue;
        }
        let interactive = !opts.json && !opts.no_prompt && std::io::stdin().is_terminal();
        if interactive {
            if let Some(path) = prompt_foreign_grant(lib, &symbols_of(lib)) {
                grants.foreign_c.insert(lib.clone(), path);
                continue;
            }
        }
        diags.push(Diagnostic::error(
            "DL1303",
            format!("foreign lib `{lib}` was not granted — pass `--grant foreign.c={lib}:PATH` (the human chooses which binary)"),
        ));
    }

    // ----- embedded Python (spec §5.1) -------------------------------------
    // A program that reaches `root.python` needs a granted `foreign.python` allowlist. Deny-by-
    // default at the grant flow (DL1303), before `main` runs — never mid-run. The runtime allowlist
    // check at `py.import` (DL1305) is a second, per-import gate on top of this.
    if python_usage(&checked.module).is_some() {
        if let Some(m) = manifest {
            if m.foreign_python.is_empty() {
                diags.push(Diagnostic::error(
                    "DL1303",
                    "program uses embedded Python not permitted by the authority manifest (`[authority] foreign.python`)".to_string(),
                ));
            } else {
                // Granted patterns must not exceed the manifest's declared allowlist (attenuation).
                for p in &grants.foreign_python {
                    if !m.foreign_python.iter().any(|d| d == p) {
                        diags.push(Diagnostic::error(
                            "DL1303",
                            format!("python import pattern `{p}` exceeds the authority manifest (`[authority] foreign.python`)"),
                        ));
                    }
                }
            }
        }
        if grants.foreign_python.is_empty() {
            diags.push(Diagnostic::error(
                "DL1303",
                "embedded Python was not granted — pass `--grant foreign.python=numpy` (an import allowlist pattern; the human chooses which modules)".to_string(),
            ));
        }
    }

    if diags.is_empty() {
        Ok(())
    } else {
        print_diagnostics("run", &diags, map, None, opts.json);
        Err(1)
    }
}

/// Prompt an interactive human for a foreign lib's binary path, showing the FULL symbol list first
/// (spec §4.1). Returns `None` on an empty line (deny) or EOF. Uses stderr so `--json`/piped stdout
/// stays clean — though this is only ever reached for a real terminal.
fn prompt_foreign_grant(lib: &str, symbols: &[String]) -> Option<String> {
    use std::io::Write as _;
    eprintln!("foreign lib `{lib}` (abi \"c\") requests these symbols:");
    if symbols.is_empty() {
        eprintln!("    (no symbols declared)");
    } else {
        for s in symbols {
            eprintln!("    - {s}");
        }
    }
    eprint!("grant a binary path for `{lib}` (empty line to deny): ");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).ok()? == 0 {
        return None;
    }
    let p = line.trim().to_string();
    if p.is_empty() {
        None
    } else {
        Some(p)
    }
}

/// Build the `foreign_calls` JSON array (spec §6) from a program's `foreign` blocks AND its embedded
/// Python use. `granted_path` is null: `delulu authority` is a static command with no grants, and the
/// path is a runtime human decision, not program data. `used_at` reports the declaration/use site at
/// declaration granularity, the same altitude `delulu why` reports at. The `python_allowlist` is the
/// manifest's declared `foreign.python` (empty when there is no manifest).
fn foreign_calls_json(module: &delulu_syntax::ast::Module, map: &SourceMap, python_allowlist: &[String]) -> Vec<Json> {
    let mut out: Vec<Json> = foreign_blocks_of(module)
        .into_iter()
        .map(|b| {
            let (line, _col) = map.position(b.span.file, b.span.start);
            json!({
                "abi": b.abi,
                "lib": b.lib,
                "symbols": b.symbols,
                "granted_path": Json::Null,
                "used_at": [ { "file": map.name(b.span.file), "line": line } ],
            })
        })
        .collect();
    // The embedded-Python entry (spec §6): `{abi:"python", allowlist, imports_seen, used_at}`. Present
    // only when the program actually reaches for Python (`root.python`), so a non-Python program's
    // report is unchanged. `imports_seen` is the statically-known set of `py.import("literal")` names.
    // Stage 10 (10h, spec §7.1): compute devices ride the SAME separator as C and Python, because
    // they are the same kind of thing — code outside the proof. DeluluLang bounds a kernel's
    // reachability, resources and provenance; it says nothing about what the kernel computes, and
    // the report puts that fact where a reader meets it rather than in a footnote.
    if let Some(usage) = compute_usage(module) {
        let (line, _col) = map.position(usage.used_at.file, usage.used_at.start);
        out.push(json!({
            "abi": "compute",
            "devices": usage.devices,
            "kernels": usage.kernels,
            "used_at": [ { "file": map.name(usage.used_at.file), "line": line } ],
        }));
    }
    if let Some(usage) = python_usage(module) {
        let (line, _col) = map.position(usage.used_at.file, usage.used_at.start);
        out.push(json!({
            "abi": "python",
            "allowlist": python_allowlist,
            "imports_seen": usage.imports_seen,
            "used_at": [ { "file": map.name(usage.used_at.file), "line": line } ],
        }));
    }
    out
}

/// A program's embedded-Python use, discovered by an AST walk: where `root.python` is first reached
/// (`used_at`) and the statically-known `py.import("literal")` module names (`imports_seen`).
struct PythonUsage {
    used_at: delulu_diag::Span,
    imports_seen: Vec<String>,
}

fn python_usage(module: &delulu_syntax::ast::Module) -> Option<PythonUsage> {
    let mut w = PyWalk {
        python_call: None,
        imports: Vec::new(),
        compute_call: None,
        devices: Vec::new(),
        kernels: Vec::new(),
    };
    for item in &module.items {
        match item {
            Item::Fn(f) => w.walk_block(&f.body),
            Item::Const(c) => w.walk_expr(&c.value),
            _ => {}
        }
    }
    w.python_call.map(|used_at| PythonUsage { used_at, imports_seen: w.imports })
}

/// A program's compute use (Stage 10 phase 10h): where `root.compute` is first reached, the device
/// names it mints, and the kernel names it dispatches — all statically, from string literals.
struct ComputeUsage {
    used_at: delulu_diag::Span,
    devices: Vec<String>,
    kernels: Vec<String>,
}

fn compute_usage(module: &delulu_syntax::ast::Module) -> Option<ComputeUsage> {
    let mut w = PyWalk {
        python_call: None,
        imports: Vec::new(),
        compute_call: None,
        devices: Vec::new(),
        kernels: Vec::new(),
    };
    for item in &module.items {
        match item {
            Item::Fn(f) => w.walk_block(&f.body),
            Item::Const(c) => w.walk_expr(&c.value),
            _ => {}
        }
    }
    w.compute_call.map(|used_at| ComputeUsage { used_at, devices: w.devices, kernels: w.kernels })
}

/// A read-only AST walk collecting embedded-Python use: the first `root.python(...)` method call
/// (which is what mints a `Cap[Python]`) and every `py.import("literal")` module name.
struct PyWalk {
    python_call: Option<delulu_diag::Span>,
    imports: Vec<String>,
    /// Stage 10 (10h): `root.compute("dev")` sites and the kernel names dispatched, collected by
    /// the same walk. Compute belongs in the same report section as Python and C for the same
    /// reason — it is code outside the proof.
    compute_call: Option<delulu_diag::Span>,
    devices: Vec<String>,
    kernels: Vec<String>,
}

impl PyWalk {
    fn walk_block(&mut self, b: &delulu_syntax::ast::Block) {
        for s in &b.stmts {
            self.walk_stmt(s);
        }
    }

    fn walk_stmt(&mut self, s: &delulu_syntax::ast::Stmt) {
        use delulu_syntax::ast::Stmt::*;
        match s {
            Let { value, .. } => self.walk_expr(value),
            Assign { value, .. } => self.walk_expr(value),
            While { cond, body, .. } => {
                self.walk_expr(cond);
                self.walk_block(body);
            }
            Return { value, .. } => {
                if let Some(e) = value {
                    self.walk_expr(e);
                }
            }
            Expr(e) => self.walk_expr(e),
        }
    }

    fn walk_expr(&mut self, e: &delulu_syntax::ast::Expr) {
        use delulu_syntax::ast::{Expr::*, LitKind};
        match e {
            Method { recv, name, args, span, .. } => {
                self.walk_expr(recv);
                for a in args {
                    self.walk_expr(a);
                }
                if name.name == "python" && self.python_call.is_none() {
                    self.python_call = Some(*span);
                }
                if name.name == "compute" {
                    if self.compute_call.is_none() {
                        self.compute_call = Some(*span);
                    }
                    if let Some(Lit { kind: LitKind::Str(d), .. }) = args.first() {
                        if !self.devices.contains(d) {
                            self.devices.push(d.clone());
                        }
                    }
                }
                if name.name == "dispatch" {
                    if let Some(Lit { kind: LitKind::Str(k), .. }) = args.first() {
                        if !self.kernels.contains(k) {
                            self.kernels.push(k.clone());
                        }
                    }
                }
                if name.name == "import" {
                    if let Some(Lit { kind: LitKind::Str(s), .. }) = args.first() {
                        if !self.imports.contains(s) {
                            self.imports.push(s.clone());
                        }
                    }
                }
            }
            List { items, .. } => items.iter().for_each(|it| self.walk_expr(it)),
            Record { fields, .. } => fields.iter().for_each(|(_, v)| self.walk_expr(v)),
            Call { callee, args, .. } => {
                self.walk_expr(callee);
                args.iter().for_each(|a| self.walk_expr(a));
            }
            Field { recv, .. } => self.walk_expr(recv),
            Index { recv, index, .. } => {
                self.walk_expr(recv);
                self.walk_expr(index);
            }
            Unary { operand, .. } => self.walk_expr(operand),
            Binary { lhs, rhs, .. } => {
                self.walk_expr(lhs);
                self.walk_expr(rhs);
            }
            If { cond, then_, else_, .. } => {
                self.walk_expr(cond);
                self.walk_block(then_);
                if let Some(e) = else_ {
                    self.walk_expr(e);
                }
            }
            Match { scrutinee, arms, .. } => {
                self.walk_expr(scrutinee);
                arms.iter().for_each(|a| self.walk_expr(&a.body));
            }
            Lambda { body, .. } => self.walk_block(body),
            Try { inner, .. } => self.walk_expr(inner),
            Block(b) => self.walk_block(b),
            // Stage 7: python reachability inside spawn arguments / recover blocks.
            Spawn { args, .. } => args.iter().for_each(|a| self.walk_expr(a)),
            Recover { body, .. } => self.walk_block(body),
            Lit { .. } | Var { .. } | Consume { .. } => {}
        }
    }
}

// ----- audit (Stage 5 phase 5d: the hash-chained log's read surface) --------
//
// `delulu audit tail|query|verify` reads the broker's log files DIRECTLY — no daemon involved (the
// daemon is chunk 3). The log is observability, not enforcement (spec §7): nothing here feeds any
// authority decision. Revocation latency honesty (spec §4.2): records observe that revocations take
// effect synchronously before the next use / within one epoch interval — never "immediately".

/// Default log dir: `~/.delulu/audit` (HOME, else USERPROFILE on Windows). Only the CLI knows this
/// path — the `delulu-broker` library takes injected paths only.
fn default_audit_dir() -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(std::path::PathBuf::from(home).join(".delulu").join("audit"))
}

/// One human line per record: seq, UTC time (display-only ISO render of the stored epoch millis),
/// action, decision, then whichever of actor/target the record carries.
fn render_audit_record(r: &delulu_broker::AuditRecord) -> String {
    let mut s = format!(
        "seq {:>5}  {}  {:<10} {:<5}",
        r.seq,
        delulu_broker::render_ts_utc(r.ts),
        r.action,
        r.decision
    );
    if let Some(a) = &r.actor_node {
        s.push_str(&format!("  actor={a}"));
    }
    if let Some(t) = &r.target {
        s.push_str(&format!("  target={t}"));
    }
    s
}

fn audit_record_json(r: &delulu_broker::AuditRecord) -> Json {
    let mut m = serde_json::Map::new();
    m.insert("seq".into(), json!(r.seq));
    m.insert("ts".into(), json!(r.ts));
    m.insert("prev_hash".into(), json!(r.prev_hash));
    m.insert("hash".into(), json!(r.hash));
    if let Some(a) = &r.actor_node {
        m.insert("actor_node".into(), json!(a));
    }
    m.insert("action".into(), json!(r.action));
    if let Some(t) = &r.target {
        m.insert("target".into(), json!(t));
    }
    if let Some(a) = &r.authority {
        m.insert("authority".into(), a.clone());
    }
    if let Some(s) = &r.span {
        m.insert("span".into(), json!(s));
    }
    m.insert("decision".into(), json!(r.decision));
    Json::Object(m)
}

fn cmd_audit(rest: &[String]) -> i32 {
    let Some(sub) = rest.first().map(String::as_str) else {
        eprintln!("error: `audit` needs a subcommand: tail [N] | query [--node g_ID] [--action A] [--effect E] | verify | bundle [--out F] | reconcile <FILE> [--expect-start HASH]");
        return 2;
    };
    let args = &rest[1..];

    // The audit flag set (own small parser, like `why` peels its own positional).
    let mut json = false;
    let mut dir_flag: Option<String> = None;
    let mut filter = delulu_broker::QueryFilter::default();
    let mut tail_n: usize = 10;
    // RFC 0001 F5 — reconciliation.
    let mut bundle_out: Option<String> = None;
    let mut bundle_in: Option<String> = None;
    let mut expect_start: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json = true,
            "--dir" => {
                if i + 1 < args.len() {
                    dir_flag = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--dir=") => dir_flag = Some(s["--dir=".len()..].to_string()),
            "--out" => {
                if i + 1 < args.len() {
                    bundle_out = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--out=") => bundle_out = Some(s["--out=".len()..].to_string()),
            // The head the incoming segment must chain onto. Omitting it accepts any starting
            // point, which is only right for a FIRST contact: without it a dropped middle segment
            // would go unnoticed, and that is the property a sequence of bundles exists to have.
            "--expect-start" => {
                if i + 1 < args.len() {
                    expect_start = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--expect-start=") => {
                expect_start = Some(s["--expect-start=".len()..].to_string())
            }
            "--node" => {
                if i + 1 < args.len() {
                    filter.node = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--node=") => filter.node = Some(s["--node=".len()..].to_string()),
            "--action" => {
                if i + 1 < args.len() {
                    filter.action = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--action=") => filter.action = Some(s["--action=".len()..].to_string()),
            "--effect" => {
                if i + 1 < args.len() {
                    filter.effect = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--effect=") => filter.effect = Some(s["--effect=".len()..].to_string()),
            s if !s.starts_with('-') && s.chars().all(|c| c.is_ascii_digit()) => {
                tail_n = s.parse().unwrap_or(10);
            }
            // A non-numeric positional is `reconcile`'s bundle file. Only the FIRST is taken:
            // reconciling two bundles in one command would need two cross-link records, and
            // silently using the last one would drop a segment.
            s if !s.starts_with('-') && bundle_in.is_none() => bundle_in = Some(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    let dir = match dir_flag.map(std::path::PathBuf::from).or_else(default_audit_dir) {
        Some(d) => d,
        None => {
            eprintln!("error: cannot resolve the audit directory (no HOME/USERPROFILE) — pass --dir DIR");
            return 2;
        }
    };
    let map = SourceMap::new(); // audit records carry no source text; diagnostics have no span

    match sub {
        "tail" | "query" => {
            let result = if sub == "tail" {
                delulu_broker::tail(&dir, tail_n)
            } else {
                delulu_broker::query(&dir, &filter)
            };
            let records = match result {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("error: cannot read audit log at `{}`: {e}", dir.display());
                    return 2;
                }
            };
            // Verify the chain before showing anything (`HARDENING_CAMPAIGN.md` C30).
            //
            // `tail` and `query` read records without checking a single hash, and this is the surface
            // an operator uses after an incident. Two demonstrated consequences: a record whose
            // `decision` was flipped from "allow" to "deny" was displayed with the forged value and
            // no warning, and a record corrupted into non-JSON simply VANISHED from the output —
            // records 1, 2, 4 shown, 3 gone, no gap marker, no error, no hint that the list was
            // shorter than the log.
            //
            // The chain verifier already caught both cases. Nothing called it here. So the fix is not
            // new machinery, it is using what exists: verify, then still SHOW the records — because
            // an operator investigating a tampered log is exactly the person who most needs to read
            // it — with a warning they cannot miss. The audit log remains "observability, not
            // enforcement"; the point is that the observation must be truthful about its own
            // integrity. Exit is nonzero so a script cannot treat a corrupt read as a clean one.
            let integrity = delulu_broker::verify(&dir).err().map(|e| e.to_string());
            if json {
                let arr: Vec<Json> = records.iter().map(audit_record_json).collect();
                let mut report = json!({ "command": "audit", "subcommand": sub, "records": arr, "count": records.len() });
                // Always present, so a machine consumer never has to infer integrity from absence.
                let obj = report.as_object_mut().expect("json! built an object");
                obj.insert("chain_verified".into(), json!(integrity.is_none()));
                if let Some(detail) = &integrity {
                    obj.insert("chain_error".into(), json!(detail));
                }
                note_json_emitted();
                println!("{}", serde_json::to_string_pretty(&report).expect("audit report serializes"));
            } else {
                if let Some(detail) = &integrity {
                    eprintln!(
                        "WARNING: the audit chain does NOT verify — {detail}\n\
                         \x20        the records below are shown as found on disk and MUST NOT be trusted;\n\
                         \x20        entries may have been altered, and any entry that was corrupted\n\
                         \x20        beyond parsing is MISSING from this listing entirely"
                    );
                }
                if records.is_empty() {
                    eprintln!("(no matching audit records under `{}`)", dir.display());
                }
                for r in &records {
                    println!("{}", render_audit_record(r));
                }
            }
            i32::from(integrity.is_some())
        }
        // ----- RFC 0001 F5: audit reconciliation ------------------------------------------------
        // `bundle` is the VEHICLE side (export a transcript); `reconcile` is the GROUND side
        // (verify it, and record the cross-link in its OWN chain). The two chains are never merged
        // — see `delulu_broker::audit::Bundle` for why that is impossible rather than merely
        // undesirable.
        "bundle" => match delulu_broker::audit::bundle(&dir) {
            Ok(b) => {
                let summary = match b.verify(Some(delulu_broker::GENESIS_HASH)) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("error: this broker's own chain does not verify: {e}");
                        return 1;
                    }
                };
                let wire = b.to_wire();
                if let Some(f) = &bundle_out {
                    if let Err(e) = std::fs::write(f, &wire) {
                        eprintln!("error: cannot write {f}: {e}");
                        return 2;
                    }
                }
                if json {
                    let report = json!({
                        "command": "audit", "subcommand": "bundle",
                        "digest": summary.digest, "records": summary.records,
                        "first_seq": summary.first_seq, "last_seq": summary.last_seq,
                        "head": summary.head,
                    });
                    note_json_emitted();
                    println!("{}", serde_json::to_string_pretty(&report).expect("serializes"));
                } else if bundle_out.is_some() {
                    eprintln!(
                        "wrote {} — {} record(s), seq {}..{}, digest {}",
                        bundle_out.as_deref().unwrap_or(""),
                        summary.records,
                        summary.first_seq,
                        summary.last_seq,
                        &summary.digest[..16]
                    );
                } else {
                    print!("{wire}");
                }
                0
            }
            Err(e) => {
                eprintln!("error: cannot read audit log at `{}`: {e}", dir.display());
                2
            }
        },
        "reconcile" => {
            let Some(path) = bundle_in else {
                eprintln!("error: `audit reconcile` needs a bundle file (from `audit bundle --out`)");
                return 2;
            };
            let text = match std::fs::read_to_string(&path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("error: cannot read {path}: {e}");
                    return 2;
                }
            };
            // A bundle that does not verify is an INCIDENT, not a denial. The log is observability,
            // not enforcement (spec §7, playbook trap 6): this reports and records, and nothing
            // here may ever gate a vehicle's ability to operate. Exit 1 says "look at this", not
            // "authority withdrawn".
            let (summary, problem) = match delulu_broker::audit::parse_bundle(&text)
                .and_then(|b| b.verify(expect_start.as_deref()))
            {
                Ok(s) => (Some(s), None),
                Err(e) => (None, Some(e.to_string())),
            };
            let mut log = match delulu_broker::AuditLog::open(&dir) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("error: cannot open this broker's audit log at `{}`: {e}", dir.display());
                    return 2;
                }
            };
            // The cross-link goes in THIS chain: one `reconcile` record naming the other chain's
            // digest and range. Exactly the mechanism a new day file already uses to link to the
            // previous day's head — generalized across brokers instead of days.
            let target = match &summary {
                Some(s) => format!("{} seq {}..{} head {}", s.digest, s.first_seq, s.last_seq, s.head),
                None => format!("UNVERIFIED bundle from {path}"),
            };
            // Continue THIS chain's numbering. `verify` checks hashes and prev-links rather than
            // seq monotonicity, so a fixed 0 would still verify — and two reconciliations would
            // then both claim seq 0, which is exactly the kind of quietly-wrong record an audit log
            // exists not to contain.
            let next_seq = delulu_broker::tail(&dir, 1).ok().and_then(|v| v.last().map(|r| r.seq + 1)).unwrap_or(1);
            let entry = delulu_broker::AuditEntry {
                seq: next_seq,
                ts: now_millis(),
                actor_node: None,
                action: "reconcile".to_string(),
                target: Some(target),
                authority: None,
                span: None,
                decision: if problem.is_some() { "deny" } else { "allow" }.to_string(),
            };
            use delulu_broker::AuditSink as _;
            if let Err(e) = log.append(entry) {
                eprintln!("error: cannot append the reconcile record: {e}");
                return 2;
            }
            match (&summary, &problem) {
                (Some(s), _) => {
                    if json {
                        let report = json!({
                            "command": "audit", "subcommand": "reconcile", "ok": true,
                            "digest": s.digest, "records": s.records,
                            "first_seq": s.first_seq, "last_seq": s.last_seq, "head": s.head,
                        });
                        note_json_emitted();
                        println!("{}", serde_json::to_string_pretty(&report).expect("serializes"));
                    } else {
                        ok_line!(
                            "ok: reconciled {} record(s), seq {}..{} — cross-linked in this chain \
                             (the two chains are NOT merged; `seq` is per-broker, so a combined \
                             timeline is not a global order)",
                            s.records,
                            s.first_seq,
                            s.last_seq
                        );
                    }
                    0
                }
                (None, Some(p)) => {
                    eprintln!(
                        "INCIDENT: the audit bundle did not verify: {p}\n  \
                         Recorded in this chain as a failed reconciliation. This does NOT withdraw \
                         any authority — the audit log is observability, not enforcement."
                    );
                    1
                }
                (None, None) => unreachable!("verify returns Ok or Err"),
            }
        }
        "verify" => match delulu_broker::verify(&dir) {
            Ok(stats) => {
                if json {
                    let report = json!({
                        "command": "audit",
                        "subcommand": "verify",
                        "ok": true,
                        "files": stats.files,
                        "records": stats.records,
                        "head": stats.head,
                    });
                    note_json_emitted();
                    println!("{}", serde_json::to_string_pretty(&report).expect("audit report serializes"));
                } else {
                    ok_line!(
                        "ok: audit chain verified — {} record(s) across {} file(s), head {}",
                        stats.records,
                        stats.files,
                        &stats.head[..16.min(stats.head.len())]
                    );
                }
                0
            }
            Err(e) => {
                // A broken chain is DL1405 at the failing seq (requires_human, spec §8); an I/O
                // problem is a plain CLI error (exit 2), not a tamper verdict.
                match e.denial() {
                    Some(d) => {
                        print_diagnostics("audit", &[d.to_diagnostic()], &map, None, json);
                        1
                    }
                    None => {
                        eprintln!("error: cannot verify audit log at `{}`: {e}", dir.display());
                        2
                    }
                }
            }
        },
        other => {
            eprintln!("error: unknown audit subcommand `{other}` (tail | query | verify | bundle | reconcile)");
            2
        }
    }
}

// ----- explain / repl ------------------------------------------------------

// ----- atlas (Surface addendum §2) ---------------------------------------------------------------

/// `delulu atlas` — the typed, deterministic code + authority graph. Either a graph command
/// (`delulu atlas <file|dir> [--format …]`) or a query verb (`node`/`path`/`callers`/`calls`/`why`).
fn cmd_atlas(rest: &[String]) -> i32 {
    match rest.first().map(String::as_str) {
        Some(v @ ("node" | "callers" | "calls" | "why" | "path")) => atlas_query_cmd(v, &rest[1..]),
        _ => atlas_graph_cmd(rest),
    }
}

/// Build the atlas for a target: a `.delulu` file, a package directory, or a persisted `atlas.json`.
/// On check errors it prints the diagnostics + DL1780 and refuses (no partial graph); returns the
/// exit code to propagate. `gods` is the god-node cap.
fn atlas_from_target(target: &str, gods: usize, json: bool) -> Result<Atlas, i32> {
    // A persisted graph: reuse it directly (agents can `atlas <pkg> --out .` then query atlas.json).
    if target.ends_with(".json") {
        return match std::fs::read_to_string(target) {
            Ok(s) => serde_json::from_str::<Atlas>(&s).map_err(|e| {
                eprintln!("error: `{target}` is not a valid atlas/1 file: {e}");
                2
            }),
            Err(e) => {
                eprintln!("error: cannot read `{target}`: {e}");
                Err(2)
            }
        };
    }

    if std::path::Path::new(target).is_dir() {
        let ws = resolve_workspace(target);
        let program = check_workspace(&ws);
        let mut diags: Vec<Diagnostic> = ws.diagnostics.clone();
        diags.extend(program.diagnostics.iter().cloned());
        let n = errors(&diags);
        if n > 0 {
            diags.push(atlas_refusal(n));
            print_diagnostics("atlas", &diags, &ws.source_map, None, json);
            return Err(1);
        }
        // Past the error gate above, so a root package exists; `?`-style handling rather than an
        // index keeps `atlas` from being the next command to abort on an unreadable manifest (C49).
        let Some(root) = ws.root_pkg().map(|p| p.name.clone()) else {
            diags.push(atlas_refusal(1));
            print_diagnostics("atlas", &diags, &ws.source_map, None, json);
            return Err(1);
        };
        let scopes = scopes_in_dir(std::path::Path::new(target));
        let authority = program_authority(&program, &root, &scopes);
        let modules: Vec<ModuleView> = ws
            .modules
            .iter()
            .map(|wm| ModuleView {
                package: ws.packages[wm.pkg].name.clone(),
                name: wm.unit.name.clone(),
                module: &wm.unit.module,
            })
            .collect();
        let packages: Vec<PackageView> = ws
            .packages
            .iter()
            .map(|p| PackageView {
                name: p.name.clone(),
                deps: p.dep_idxs.iter().map(|&i| ws.packages[i].name.clone()).collect(),
                is_root: p.is_root,
            })
            .collect();
        return Ok(Atlas::build(BuildInput {
            root,
            packages,
            modules,
            program: &program,
            source_map: &ws.source_map,
            authority,
            god_n: gods,
            custody: None,
        }));
    }

    // Single file.
    let (map, id, src) = load(target)?;
    let checked = check_source(id, &src);
    let n = errors(&checked.diagnostics);
    if n > 0 {
        let mut diags = checked.diagnostics.clone();
        diags.push(atlas_refusal(n));
        print_diagnostics("atlas", &diags, &map, None, json);
        return Err(1);
    }
    let program = synth_single_program(&checked);
    let root = checked.module.name.dotted();
    let mut scopes = manifest_scopes(target);
    let python_allowlist = manifest_python_allowlist(target);
    scopes.foreign_calls = foreign_calls_json(&checked.module, &map, &python_allowlist);
    let authority = authority_report(&root, &checked.result, &scopes);
    let modules = vec![ModuleView { package: root.clone(), name: root.clone(), module: &checked.module }];
    let packages = vec![PackageView { name: root.clone(), deps: vec![], is_root: true }];
    Ok(Atlas::build(BuildInput {
        root,
        packages,
        modules,
        program: &program,
        source_map: &map,
        authority,
        god_n: gods,
        custody: None,
    }))
}

/// DL1780 — the atlas refuses to build from a program with check errors (no partial graph).
fn atlas_refusal(n: usize) -> Diagnostic {
    Diagnostic::error(
        "DL1780",
        format!("the atlas is built from checked facts — fix the {n} error(s) above first"),
    )
}

/// Collect atlas flags shared by the graph + query commands.
struct AtlasFlags {
    positionals: Vec<String>,
    format: Option<String>,
    out: Option<String>,
    budget: Option<usize>,
    gods: usize,
    json: bool,
    custody: bool,
}

fn parse_atlas_flags(args: &[String]) -> AtlasFlags {
    let mut f = AtlasFlags {
        positionals: Vec::new(),
        format: None,
        out: None,
        budget: None,
        gods: 10,
        json: false,
        custody: false,
    };
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        match a {
            "--json" => f.json = true,
            "--custody" => f.custody = true,
            "--format" => {
                if i + 1 < args.len() {
                    f.format = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--format=") => f.format = Some(s["--format=".len()..].to_string()),
            "--out" | "-o" => {
                if i + 1 < args.len() {
                    f.out = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            s if s.starts_with("--out=") => f.out = Some(s["--out=".len()..].to_string()),
            "--budget" => {
                if i + 1 < args.len() {
                    f.budget = args[i + 1].parse().ok();
                    i += 1;
                }
            }
            s if s.starts_with("--budget=") => f.budget = s["--budget=".len()..].parse().ok(),
            "--gods" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse() {
                        f.gods = n;
                    }
                    i += 1;
                }
            }
            s if s.starts_with("--gods=") => {
                if let Ok(n) = s["--gods=".len()..].parse() {
                    f.gods = n;
                }
            }
            s if !s.starts_with('-') => f.positionals.push(s.to_string()),
            _ => {}
        }
        i += 1;
    }
    f
}

/// Fetch the custody overlay from the broker (read-only `List` verb via the existing wire — no new
/// verbs, no owner code). Broker down ⇒ `Err(DL1781 note)`: the atlas is still emitted without the
/// overlay (graceful, never blocking — addendum §2.1 / criterion 6).
fn atlas_custody_overlay() -> Result<Json, Diagnostic> {
    let Some(state_dir) = crate::brokerd::resolve_state_dir(None) else {
        return Err(Diagnostic::warning(
            "DL1781",
            "custody overlay unavailable — cannot resolve the broker state directory; atlas emitted without it",
        ));
    };
    match crate::brokerd::request(&state_dir, crate::broker_ipc::ReqBody::List) {
        Ok(crate::broker_ipc::Response::Listed { nodes }) => {
            let grants: Vec<Json> = nodes
                .iter()
                .map(|n| {
                    json!({
                        "id": n.id,
                        "parent": n.parent,
                        "state": n.state,
                        "authority": crate::brokerd::spec_to_authority(&n.authority_spec()).render_compact(),
                    })
                })
                .collect();
            Ok(json!({ "grants": grants }))
        }
        Ok(other) => Err(Diagnostic::warning(
            "DL1781",
            format!("custody overlay unavailable — unexpected broker response {other:?}; atlas emitted without it"),
        )),
        Err(e) => Err(Diagnostic::warning(
            "DL1781",
            format!(
                "custody overlay unavailable — broker daemon not reachable ({e}); atlas emitted \
                 without it (start it with `delulu broker start`)"
            ),
        )),
    }
}

/// `delulu atlas <file|dir> [--format tree|digest|json|dot|mermaid|html] [--out DIR] [--budget N]
/// [--gods N] [--custody] [--json]`.
fn atlas_graph_cmd(args: &[String]) -> i32 {
    let f = parse_atlas_flags(args);
    let Some(target) = f.positionals.first().cloned() else {
        eprintln!("error: `atlas` needs a file or package directory (e.g. `delulu atlas examples/demo.delulu`)");
        return 2;
    };
    let fmt = if f.json { "json".to_string() } else { f.format.clone().unwrap_or_else(|| "tree".to_string()) };
    let mut atlas = match atlas_from_target(&target, f.gods, f.json) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let budget = f.budget.unwrap_or(delulu_atlas::DEFAULT_BUDGET);

    // `--custody`: overlay broker grant state (read-only awareness). Broker down degrades to a
    // DL1781 note on stderr — the atlas is still emitted without the overlay, never blocked.
    if f.custody {
        match atlas_custody_overlay() {
            Ok(overlay) => atlas.attach_custody(overlay, f.gods),
            Err(note) => {
                let map = SourceMap::new();
                // The note rides stderr even under --json: stdout stays the machine surface.
                eprint!("{}", render_human_with(&note, &map, &palette_stderr()));
            }
        }
    }

    // `--out DIR` writes the agent bundle: ATLAS.md (digest) + atlas.json (+ atlas.html for
    // `--format html`).
    if let Some(dir) = &f.out {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("error: cannot create `{dir}`: {e}");
            return 2;
        }
        let md = std::path::Path::new(dir).join("ATLAS.md");
        let js = std::path::Path::new(dir).join("atlas.json");
        if let Err(e) = std::fs::write(&md, atlas.render_digest(budget)) {
            eprintln!("error: cannot write `{}`: {e}", md.display());
            return 2;
        }
        if let Err(e) = std::fs::write(&js, atlas.to_json_string()) {
            eprintln!("error: cannot write `{}`: {e}", js.display());
            return 2;
        }
        let mut wrote = format!("ok: wrote {} and {}", md.display(), js.display());
        if fmt == "html" {
            let ht = std::path::Path::new(dir).join("atlas.html");
            if let Err(e) = std::fs::write(&ht, atlas.render_html()) {
                eprintln!("error: cannot write `{}`: {e}", ht.display());
                return 2;
            }
            wrote = format!("{wrote} and {}", ht.display());
        }
        eprintln!("{}", ok(wrote));
        return 0;
    }

    match fmt.as_str() {
        "tree" => {
            // Colored via the Palette when the stream is a TTY (plain otherwise). The graph body is
            // a stdout human surface.
            print!("{}", colorize_atlas_tree(&atlas.render_tree(), &palette_stdout()));
        }
        "digest" => print!("{}", atlas.render_digest(budget)),
        "json" => println!("{}", atlas.to_json_string()),
        "dot" => print!("{}", atlas.render_dot()),
        "mermaid" => print!("{}", atlas.render_mermaid()),
        "html" => print!("{}", atlas.render_html()),
        other => {
            eprintln!("error: unknown --format `{other}` (tree | digest | json | dot | mermaid | html)");
            return 2;
        }
    }
    0
}


/// Query verbs: `node <name>`, `callers <fn>`, `calls <fn>`, `why <x>`, `path <A> <B>`. Each takes
/// an optional trailing target (file/dir/atlas.json); default is the current directory.
fn atlas_query_cmd(verb: &str, args: &[String]) -> i32 {
    let f = parse_atlas_flags(args);
    let n_query = if verb == "path" { 2 } else { 1 };
    if f.positionals.len() < n_query {
        eprintln!(
            "error: `atlas {verb}` needs {} argument(s){}",
            n_query,
            if verb == "path" { " (<A> <B>)" } else { "" }
        );
        return 2;
    }
    let target = f.positionals.get(n_query).cloned().unwrap_or_else(|| ".".to_string());
    let atlas = match atlas_from_target(&target, f.gods, f.json) {
        Ok(a) => a,
        Err(c) => return c,
    };
    let a = &f.positionals[0];
    let b = if verb == "path" { Some(f.positionals[1].as_str()) } else { None };
    if f.json {
        note_json_emitted();
        println!("{}", serde_json::to_string_pretty(&atlas.query_json(verb, a, b)).expect("query json"));
        return 0;
    }
    let text = match verb {
        "node" => atlas.query_node(a, f.budget),
        "callers" => atlas.query_callers(a, f.budget),
        "calls" => atlas.query_calls(a, f.budget),
        "why" => atlas.query_why(a, f.budget),
        "path" => atlas.query_path(a, b.unwrap_or(""), f.budget),
        _ => unreachable!(),
    };
    print!("{text}");
    0
}

/// Colorize the plain atlas tree via the Palette (A2: node ids + effect rows). A disabled palette
/// returns the text unchanged, keeping non-TTY output byte-identical (determinism, criterion 1).
fn colorize_atlas_tree(text: &str, palette: &Palette) -> String {
    if !palette.is_enabled() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("package ") || trimmed.starts_with("module ") || trimmed.starts_with("fn ") {
            out.push_str(&palette.paint(Role::Heading, line));
        } else if trimmed.starts_with("- ") {
            out.push_str(&palette.paint(Role::Note, line));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

fn cmd_explain(rest: &[String]) -> i32 {
    let code = rest.iter().find(|a| !a.starts_with('-')).map(|s| s.trim_start_matches("E-").to_string());
    let Some(code) = code else {
        eprintln!("error: `explain` needs a code, e.g. `delulu explain DL0501`");
        return 2;
    };
    // Named topics (Stage 5 phase 5j): `delulu explain E-REVOKE` states the spec §4.2 revocation
    // latency bound VERBATIM (playbook trap 3 — never "immediate").
    if let Some((title, body)) = delulu_diag::topic_explain(&code) {
        println!("E-{code}: {title}");
        println!("\n{body}");
        return 0;
    }
    match delulu_diag::code_title(&code) {
        Some(title) => {
            println!("{code}: {title}");
            // A longer explanation, where one exists (DL13xx foreign codes carry the spec §10
            // honesty caveats verbatim — reachability-not-behavior, link forward to Stage 5).
            if let Some(body) = delulu_diag::code_explain(&code) {
                println!("\n{body}");
            }
            0
        }
        None => {
            eprintln!("unknown code `{code}`");
            1
        }
    }
}

fn repl_cmd(rest: &[String]) -> i32 {
    let (_file, opts) = parse_opts(rest);
    let mut grants = Grants::default();
    for g in &opts.grants {
        let _ = grants.add(g);
    }
    crate::repl::run(grants)
}

// Silence an unused import in configurations where json! is not exercised.
#[allow(dead_code)]
fn _json_marker() -> Json {
    json!({})
}

/// What a run decided about its driver's provenance.
///
/// D53 made the decision correct and printed it to stderr; C60 was that it went nowhere else. A run
/// that commanded machinery left no durable, queryable evidence of WHICH KEY signed the driver that
/// moved it — the one question an incident actually asks.
struct AdapterProvenance {
    artifact: String,
    /// `verified-pinned` / `verified` / `unsigned` / `unverifiable` / `refused-*`.
    decision: &'static str,
    signer: Option<String>,
    pinned: Option<String>,
}

/// Record a provenance decision in the broker's hash-chained audit log.
///
/// **Deliberately not a new artifact.** The chain, its tamper-evidence, its day-file rotation and
/// its reader (`delulu audit verify|tail|query`) already exist and are already the place an operator
/// looks; C60's real gap was that nothing wrote the adapter decision INTO it. A second format would
/// have been a second thing to verify, and two records of one machine is how they disagree.
///
/// **Refusals are recorded too, and that is the point.** A run that was stopped because the driver
/// was signed by the wrong key is exactly the event worth keeping.
///
/// **There is deliberately NO default sink, and the reason is a property of hash chains.** A chain
/// has exactly one writer: `AuditLog::open` reads the head, then appends against it. The broker is a
/// single long-lived process and satisfies that. `delulu run` is short-lived and many can run at
/// once, so defaulting to the shared `~/.delulu/audit` made every concurrent run a second writer.
/// That is not theoretical — the first version of this function did default there, and the test
/// suite (which runs in parallel) produced a chain that failed `delulu audit verify` with a
/// `prev_hash` break plus two physically interleaved half-lines. An operator naming a sink is
/// choosing one they control; guessing one for them corrupts the log this feature exists to create.
///
/// Returns `Err` when the sink could not be written: asking for a record and silently not getting
/// one is the failure this project refuses everywhere else.
fn record_adapter_provenance(prov: &AdapterProvenance, explicit_dir: Option<&str>) -> Result<(), i32> {
    let Some(dir) = explicit_dir.map(std::path::PathBuf::from) else {
        eprintln!(
            "warning: this driver's provenance decision ({}) was printed but NOT recorded. Pass `--adapter-record <dir>` to keep durable evidence of which key signed the driver that moved the machine.",
            prov.decision
        );
        return Ok(());
    };
    let mut authority = serde_json::Map::new();
    authority.insert("decision".into(), serde_json::json!(prov.decision));
    if let Some(sig) = &prov.signer {
        authority.insert("signer".into(), serde_json::json!(sig));
    }
    if let Some(pin) = &prov.pinned {
        authority.insert("pinned_signer".into(), serde_json::json!(pin));
    }
    let entry = delulu_broker::AuditEntry {
        seq: 0,
        ts: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0),
        actor_node: None,
        action: "adapter.provenance".to_string(),
        target: Some(prov.artifact.clone()),
        authority: Some(serde_json::Value::Object(authority)),
        span: None,
        decision: prov.decision.to_string(),
    };
    let written = delulu_broker::AuditLog::open(&dir)
        .and_then(|mut log| delulu_broker::AuditSink::append(&mut log, entry));
    match written {
        Ok(rec) => {
            eprintln!("adapter: provenance recorded in {} (seq {}, {})", dir.display(), rec.seq, rec.hash);
            Ok(())
        }
        Err(e) => {
            let d = Diagnostic::error(
                "DL1511",
                format!(
                    "this run was told to record its driver's provenance in `{}` and could not ({e}) — a record you asked for and did not get is worse than none, because you would believe you had it",
                    dir.display()
                ),
            );
            eprint!("{}", render_human_with(&d, &SourceMap::new(), &palette_stderr()));
            Err(1)
        }
    }
}

/// Verify a hardware adapter's provenance before it is spawned (ruling D52, extended by D53).
///
/// D52 closed the gap D23 named — "an operator-supplied SUBPROCESS with NO signature check" — by
/// applying Stage 6's plugin rule to drivers. D53 closed the hole D52 left, which was that the rule
/// as written answered a weaker question than it appeared to.
///
/// **The rule.** The asymmetry between these is the point:
///
/// - **A signature that is PRESENT but does not verify refuses the run, unconditionally** — whatever
///   the policy flag says. This is the branch a "not required, so don't check" reading would skip,
///   and it is the branch that matters: a signature that fails to verify means these are not the
///   bytes someone signed, whether that is tampering, the wrong key, or a truncated file.
/// - **A signature that is ABSENT is a policy question**, because requiring one everywhere would
///   refuse every driver an operator builds locally. Absent is allowed by default and *disclosed
///   loudly*; `--require-signed-adapter` turns it into a refusal.
/// - **A signature that verifies under a key the operator did not name proves very little.** The
///   `.sig` sits beside the driver and carries its own public key, so anyone who can replace
///   `drive.exe` can replace `drive.exe.sig` with one they signed themselves, and an unpinned check
///   passes. `--adapter-signer <hex>` names the acceptable key; any other signer is DL1510, which
///   is the "wrong-key" case that code has always claimed to cover.
///
/// **What this does NOT do**, stated here so no surface reads more into it:
///
/// - This is still an operator-supplied **subprocess**, not spec §5.4's Verified-class signed
///   plugin loaded into the host.
/// - Signing buys **provenance**, not behaviour: "these are the bytes this key vouched for" says
///   nothing about what the driver does once running. The envelope bounds that, enforced host-side
///   before one byte reaches the driver, and it is unchanged.
/// - Without `--adapter-signer` there is **no trust policy** — the same limit spec §10 states for
///   plugins. Unpinned, `--require-signed-adapter` stops accidents and unsigned drivers; it does
///   not stop an adversary who can write files next to the driver.
/// - Without `--adapter-record <dir>` the verdict is printed and **not kept**, and the run says so.
///   With it, the decision (refusals included) is appended to the broker's hash-chained audit log —
///   ruling D66, closing campaign finding **C60**. There is no default sink: a hash chain has one
///   writer and a short-lived `delulu run` is not one (**C69**).
fn check_adapter_signature(
    prog: &str,
    artifact: Option<&str>,
    require_signed: bool,
    expect_signer: Option<&str>,
) -> (AdapterProvenance, Option<i32>) {
    // Pinning a key is itself a demand for a signature: an operator who names the acceptable signer
    // has not said "unsigned is fine".
    let require_signed = require_signed || expect_signer.is_some();

    // `--adapter-cmd` is a command line, and its first token is not always the driver: an
    // interpreter-hosted driver (`powershell -File drive.ps1`, `python drive.py`) names the
    // INTERPRETER, and verifying that would vouch for the wrong bytes entirely. `--adapter-artifact`
    // is how an operator says which bytes are actually the driver; without it, the first token is
    // the only candidate there is.
    //
    // When nothing resolves to a readable file this does not quietly pass — "the checker could not
    // tell" must never read as "yes".
    let prog = artifact.unwrap_or(prog);
    // Every exit below returns one of these, so no decision path can forget to be recordable.
    let rec = |decision: &'static str, signer: Option<String>| AdapterProvenance {
        artifact: prog.to_string(),
        decision,
        signer,
        pinned: expect_signer.map(str::to_string),
    };
    if !std::path::Path::new(prog).is_file() {
        if require_signed {
            let d = Diagnostic::error(
                "DL1511",
                format!(
                    "the driver's provenance was required, but `{prog}` is not a file this run can \
                     read, so there is nothing to verify — an interpreter-hosted driver names the \
                     interpreter in `--adapter-cmd`, not the driver. Pass `--adapter-artifact \
                     <path>` naming the bytes that were signed"
                ),
            );
            eprint!("{}", render_human_with(&d, &SourceMap::new(), &palette_stderr()));
            return (rec("refused-unverifiable", None), Some(1));
        }
        eprintln!(
            "warning: hardware adapter `{prog}` is not a readable file (an interpreter-hosted \
             driver names the interpreter, not the driver), so its provenance was NOT checked. \
             Pass `--adapter-artifact <path>` to name the driver, or `--require-signed-adapter` to \
             refuse this."
        );
        return (rec("unverifiable", None), None);
    }
    let sig_path = format!("{prog}.sig");
    let sig = match std::fs::read(&sig_path) {
        Ok(s) => s,
        Err(_) => {
            if require_signed {
                let d = Diagnostic::error(
                    "DL1511",
                    format!(
                        "hardware adapter `{prog}` has no signature at `{sig_path}`, and this run \
                         requires one — a driver commands physical machinery, so its provenance is \
                         not something this run will assume"
                    ),
                );
                eprint!("{}", render_human_with(&d, &SourceMap::new(), &palette_stderr()));
                return (rec("refused-unsigned", None), Some(1));
            }
            eprintln!(
                "warning: hardware adapter `{prog}` is UNSIGNED (no `{sig_path}`) — its provenance is \
                 unknown. Pass `--require-signed-adapter` to refuse this."
            );
            return (rec("unsigned", None), None);
        }
    };
    let data = match std::fs::read(prog) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: cannot read hardware adapter `{prog}` to verify its signature: {e}");
            return (rec("refused-unreadable", None), Some(2));
        }
    };
    match delulu_runtime::pqc::verify(&data, &sig, delulu_runtime::pqc::Policy::AcceptClassical, false) {
        delulu_runtime::pqc::Verdict::Valid { signer, .. } => match expect_signer {
            // DL1510 covers "tampered, wrong-key, or malformed" (Stage 6 deviation 8). A signature
            // that verifies under a key the operator did not name IS the wrong-key case: these
            // bytes are vouched for by somebody, and not by anybody this run accepts.
            Some(want) if !signer.eq_ignore_ascii_case(want) => {
                let d = Diagnostic::error(
                    "DL1510",
                    format!(
                        "hardware adapter `{prog}` is signed by `{signer}`, and this run accepts \
                         only `{want}` — the signature verifies, which means somebody vouched for \
                         these bytes; it does not mean anybody you named did"
                    ),
                );
                eprint!("{}", render_human_with(&d, &SourceMap::new(), &palette_stderr()));
                (rec("refused-wrong-signer", Some(signer.clone())), Some(1))
            }
            Some(want) => {
                eprintln!("adapter: `{prog}` signature verifies under the pinned key {want}");
                (rec("verified-pinned", Some(signer.clone())), None)
            }
            // Deliberately says what it does NOT mean. An unpinned "verifies" reads as an
            // assurance, and on a `.sig` that travels beside the file it is not one.
            None => {
                eprintln!(
                    "adapter: `{prog}` signature verifies (signer {signer}) — this proves these \
                     bytes were signed by that key, NOT that the key is trusted. Pass \
                     `--adapter-signer {signer}` to make this run refuse any other signer."
                );
                (rec("verified", Some(signer.clone())), None)
            }
        },
        // Present and bad: refused REGARDLESS of `require_signed`. Stage-6 deviation 8's rule.
        delulu_runtime::pqc::Verdict::Invalid { reason } => {
            let d = Diagnostic::error(
                "DL1510",
                format!(
                    "hardware adapter `{prog}` carries a signature that does not verify: {reason} — \
                     these are not the bytes that were signed, and the driver is not started"
                ),
            );
            eprint!("{}", render_human_with(&d, &SourceMap::new(), &palette_stderr()));
            (rec("refused-bad-signature", None), Some(1))
        }
        delulu_runtime::pqc::Verdict::Refused { code, reason } => {
            let d = Diagnostic::error(code, format!("hardware adapter `{prog}`: {reason}"));
            eprint!("{}", render_human_with(&d, &SourceMap::new(), &palette_stderr()));
            (rec("refused-policy", None), Some(1))
        }
    }
}

#[cfg(test)]
mod plugin_why_tests {
    use super::*;

    #[test]
    fn criterion10_verified_why_traverses_the_real_chain_to_the_primitive_op() {
        // CRITERION 10 (Verified half): `why <Effect> <verified.dpx>` follows the plugin's OWN facts
        // from an export through its callees to the function that performs the primitive op.
        let src = "module reader\n\
            fn slurp(fs: Cap[FsRead], p: Str) -> Result[Str, IoErr] ! {Read} { fs.read_text(p) }\n\
            pub fn scan(fs: Cap[FsRead], p: Str) -> Result[Str, IoErr] ! {Read} { slurp(fs, p) }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        let facts: std::collections::BTreeMap<String, delulu_check::FnFacts> =
            checked.result.facts.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        let chain = plugin_why_chain(&facts, "scan", "Read");
        assert_eq!(chain, vec!["scan".to_string(), "slurp".to_string()], "export → helper → primitive");
        // The origin (last) performs Read but no callee of it performs Read — it IS the primitive op.
        let origin = chain.last().unwrap();
        assert!(facts[origin].effects.iter().any(|e| e.name() == "Read"), "the origin performs the effect");
        assert!(
            !facts[origin].callees.iter().any(|c| facts.get(c).is_some_and(|f| f.effects.iter().any(|e| e.name() == "Read"))),
            "the origin calls nothing that performs Read — it is the primitive site"
        );
    }

    #[test]
    fn plugin_why_chain_is_cycle_safe() {
        // The visited set makes a mutually-recursive chain terminate rather than loop forever.
        let src = "module m\n\
            fn a(fs: Cap[FsRead]) -> Result[Str, IoErr] ! {Read} { let _r = b(fs)\n fs.read_text(\"x\") }\n\
            fn b(fs: Cap[FsRead]) -> Result[Str, IoErr] ! {Read} { a(fs) }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        let facts: std::collections::BTreeMap<String, delulu_check::FnFacts> =
            checked.result.facts.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let chain = plugin_why_chain(&facts, "a", "Read"); // must terminate
        assert!(chain.first().map(String::as_str) == Some("a"));
        assert!(chain.len() <= facts.len() + 1, "a cycle never revisits a node");
    }

    #[test]
    fn authority_plugins_array_extracts_load_call_sites() {
        // The `plugins` array (spec §6) is built by walking the AST for `load(host, "path", grant)`
        // call sites. Parsing is enough (no type-check needed for the walk), so we exercise the
        // extraction directly: the literal path and the literal grant effects are pulled out.
        let src = "module host\n\
            fn main(root: Root) ! {Load, Read} { \
             let h = root.plugin_host()\n \
             let p = load(h, \"reader.dpx\", Grant { effects: [\"Read\", \"Net\"] })? }\n";
        let (module, _diags) = delulu_syntax::parse_file(0, src);
        let mut loads = Vec::new();
        for item in &module.items {
            if let delulu_syntax::ast::Item::Fn(f) = item {
                collect_plugin_loads(&f.body, &mut loads);
            }
        }
        assert_eq!(loads.len(), 1, "one load call site found");
        assert_eq!(loads[0].path.as_deref(), Some("reader.dpx"), "the literal path is extracted");
        assert_eq!(loads[0].grant_effects, vec!["Read".to_string(), "Net".to_string()], "the grant effects are extracted");
    }
}
