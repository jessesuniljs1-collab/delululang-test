//! PS-B-06: BREAK-GLASS — the one way past an operator's restriction, and it is not the program's.
//!
//! D-V2-09 is the owner's principle: **restrictions can be relaxed by an authorized external
//! principal, but a program cannot relax its own authority.** Break-glass is that external
//! principal's instrument, and it is built so it cannot be anything else:
//!
//! - **It breaks exactly one thing.** An operator may require the sandbox on a host
//!   (`delulu sandbox require`), after which every command that executes a program — `run` without
//!   `--sandbox`, `test`, `repl` — is refused. A break-glass ticket lets ONE named program (by the
//!   hash of its bytes) run once without the sandbox. It relaxes nothing else: the run is an ordinary
//!   strict run with the grants on its own command line, under the same Authority, custody, Guard and
//!   budget as any other. Language semantics are never what break-glass changes (§5 of
//!   `V2_SECURITY_MODEL.md` asks the design to keep the three apart, and this is how).
//! - **The credential is not on the host.** The operator holds an Ed25519 key; only its public half
//!   is pinned in the host policy. A ticket is a signed statement — which program, until when (at most
//!   a day), and why — and it is spent the first time it is used.
//! - **It cannot be reached from code.** There is no language surface for it: the ticket is a flag on
//!   the command line an operator types, and a sandboxed guest cannot read the state directory where
//!   the policy and the spent tickets live (T14).
//! - **It is loud.** A banner on standard error, `break_glass` in the run report, a record in the
//!   audit chain for every use and every refusal — and a use that cannot be recorded does not happen.
//!
//! **What it is not** (category 7, RW 4.4): a process that runs as the operator can delete the policy
//! file outright. The policy binds what runs through `delulu`, and a sandboxed guest, which cannot
//! reach the file; it does not bind a same-user shell, which is what a separate OS account is for.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The ticket format, named inside the signed bytes so a ticket can never be read as another format.
pub const TICKET_KIND: &str = "delulu-break-glass/1";
/// A ticket longer-lived than this is refused whatever it says: break-glass is an emergency, not a
/// standing permission.
pub const MAX_LIFETIME_MS: i64 = 24 * 60 * 60 * 1000;
/// What a ticket may relax. Two things, and only these.
pub const RELAX_SANDBOX: &str = "sandbox";
pub const RELAX_POLICY_OFF: &str = "policy-off";

/// The operator's host policy (`<state>/sandbox_policy.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostPolicy {
    pub schema: u32,
    pub sandbox_required: bool,
    /// The public halves of the keys whose tickets this host accepts, lowercase hex.
    pub break_glass_keys: Vec<String>,
}

/// What the policy file says, including when it cannot say anything.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Policy {
    /// No policy: the sandbox is the operator's choice per run (D-V2-26), and there is nothing to break.
    Absent,
    Required(HostPolicy),
    /// A policy file exists and cannot be read. Treated as REQUIRED WITH NO KEY — nothing runs
    /// unsandboxed and no ticket can open it — because a restriction that vanishes when its file is
    /// damaged is a restriction anyone can remove by damaging it.
    Unreadable(String),
}

impl Policy {
    pub fn requires_sandbox(&self) -> bool {
        !matches!(self, Policy::Absent)
    }
}

pub fn policy_path(state: &Path) -> PathBuf {
    state.join("sandbox_policy.json")
}

fn spent_dir(state: &Path) -> PathBuf {
    state.join("break-glass").join("spent")
}

pub fn load(state: &Path) -> Policy {
    let path = policy_path(state);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Policy::Absent,
        Err(e) => return Policy::Unreadable(format!("`{}` cannot be read: {e}", path.display())),
    };
    match serde_json::from_str::<HostPolicy>(&text) {
        Ok(p) if p.schema == 1 && p.sandbox_required => Policy::Required(p),
        Ok(p) if p.schema != 1 => Policy::Unreadable(format!("`{}` is schema {}, not 1", path.display(), p.schema)),
        // A file that says "not required" is not a policy this build writes; it is not taken as
        // permission either.
        Ok(_) => Policy::Unreadable(format!("`{}` does not say the sandbox is required", path.display())),
        Err(e) => Policy::Unreadable(format!("`{}` is not a host policy: {e}", path.display())),
    }
}

/// Write a policy that requires the sandbox. Refused when one exists: tightening is always allowed,
/// but REPLACING a policy could add a key, which widens who may break glass.
pub fn require(state: &Path, keys: &[String]) -> Result<HostPolicy, String> {
    if keys.is_empty() {
        return Err("name at least one `--break-glass-key` (the PUBLIC key; the private half stays off this host). \
                    A policy with no key can never be opened, which is allowed — say `--no-break-glass` to mean it"
            .to_string());
    }
    write_policy(state, keys)
}

pub fn require_without_break_glass(state: &Path) -> Result<HostPolicy, String> {
    write_policy(state, &[])
}

fn write_policy(state: &Path, keys: &[String]) -> Result<HostPolicy, String> {
    let path = policy_path(state);
    if path.exists() {
        return Err(format!(
            "`{}` already requires the sandbox. To change it, turn it off with a `policy-off` ticket first",
            path.display()
        ));
    }
    let mut pinned = Vec::new();
    for k in keys {
        let k = k.trim().to_ascii_lowercase();
        if k.len() != 64 || decode_hex(&k).is_none() {
            return Err(format!("`{k}` is not an ed25519 public key (64 hex characters)"));
        }
        if !pinned.contains(&k) {
            pinned.push(k);
        }
    }
    let policy = HostPolicy { schema: 1, sandbox_required: true, break_glass_keys: pinned };
    std::fs::create_dir_all(state).map_err(|e| format!("cannot create `{}`: {e}", state.display()))?;
    let text = serde_json::to_string_pretty(&policy).expect("a policy serializes");
    // Written whole or not at all: a half-written policy would read as Unreadable, which is safe, but
    // a crash should not be what decides that.
    let tmp = state.join(format!("sandbox_policy.json.tmp-{}", std::process::id()));
    std::fs::write(&tmp, format!("{text}\n")).map_err(|e| format!("cannot write `{}`: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("cannot put `{}` in place: {e}", path.display()))?;
    Ok(policy)
}

/// The signed part of a ticket. The field order IS the canonical byte order: the signature covers
/// `serde_json::to_vec(&body)`, and a struct serializes its fields in declaration order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TicketBody {
    pub kind: String,
    pub relax: String,
    /// blake3 of the program's bytes, for `sandbox`; `None` for `policy-off`.
    pub program_blake3: Option<String>,
    pub issued_ms: i64,
    pub not_after_ms: i64,
    pub reason: String,
    pub nonce: String,
    /// The signer's public key, lowercase hex; checked against the key that actually signed.
    pub key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ticket {
    pub body: TicketBody,
    /// The detached ed25519 signature over the body's bytes (public key ‖ signature), hex.
    pub signature: String,
}

impl Ticket {
    /// What a spent ticket is remembered by: the hash of exactly what was signed and the signature.
    pub fn fingerprint(&self) -> String {
        let mut h = blake3::Hasher::new();
        h.update(&serde_json::to_vec(&self.body).expect("a ticket body serializes"));
        h.update(self.signature.as_bytes());
        h.finalize().to_hex().to_string()
    }
}

/// Mint a ticket (the operator's side, run wherever the private key is).
pub fn mint(
    seed: &[u8; 32],
    relax: &str,
    program: Option<&[u8]>,
    now_ms: i64,
    ttl_ms: i64,
    reason: &str,
) -> Result<Ticket, String> {
    if relax != RELAX_SANDBOX && relax != RELAX_POLICY_OFF {
        return Err(format!("`{relax}` is not something a ticket can relax (sandbox, policy-off)"));
    }
    if relax == RELAX_SANDBOX && program.is_none() {
        return Err("a `sandbox` ticket names the program it lets run: pass `--program FILE`".into());
    }
    if ttl_ms <= 0 || ttl_ms > MAX_LIFETIME_MS {
        return Err(format!("a ticket lives between a millisecond and a day; {ttl_ms} ms is refused"));
    }
    if reason.trim().is_empty() {
        return Err("a ticket says WHY (`--reason`): the audit record is the point of it".into());
    }
    let body = TicketBody {
        kind: TICKET_KIND.to_string(),
        relax: relax.to_string(),
        program_blake3: if relax == RELAX_SANDBOX { program.map(|p| blake3::hash(p).to_hex().to_string()) } else { None },
        issued_ms: now_ms,
        not_after_ms: now_ms + ttl_ms,
        reason: reason.trim().to_string(),
        nonce: crate::cert_crypto::fresh_nonce(),
        key: delulu_runtime::plugin::public_key_hex(seed),
    };
    let msg = serde_json::to_vec(&body).expect("a ticket body serializes");
    let signature = encode_hex(&delulu_runtime::plugin::sign_detached(seed, &msg));
    Ok(Ticket { body, signature })
}

/// Why a ticket was refused, in the words the operator and the audit record both get.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    NothingToBreak,
    PolicyUnreadable(String),
    NotATicket(String),
    WrongKind(String),
    WrongRelax { wanted: String, got: String },
    KeyNotPinned(String),
    BadSignature(String),
    SignerMismatch,
    NotYetValid,
    Expired,
    TooLong,
    WrongProgram,
    Spent,
    CannotRecord(String),
}

impl Refusal {
    pub fn code(&self) -> &'static str {
        match self {
            Refusal::NothingToBreak => "nothing-to-break",
            Refusal::PolicyUnreadable(_) => "policy-unreadable",
            Refusal::NotATicket(_) => "not-a-ticket",
            Refusal::WrongKind(_) => "wrong-kind",
            Refusal::WrongRelax { .. } => "wrong-relax",
            Refusal::KeyNotPinned(_) => "key-not-pinned",
            Refusal::BadSignature(_) => "bad-signature",
            Refusal::SignerMismatch => "signer-mismatch",
            Refusal::NotYetValid => "not-yet-valid",
            Refusal::Expired => "expired",
            Refusal::TooLong => "too-long",
            Refusal::WrongProgram => "wrong-program",
            Refusal::Spent => "spent",
            Refusal::CannotRecord(_) => "cannot-record",
        }
    }

    pub fn explain(&self) -> String {
        match self {
            Refusal::NothingToBreak => "this host does not require the sandbox, so there is nothing for a ticket to open; \
                                        run the program as usual"
                .to_string(),
            Refusal::PolicyUnreadable(why) => format!(
                "the host policy cannot be read ({why}), so it is treated as requiring the sandbox with no key \
                 — no ticket opens it until the operator repairs the file"
            ),
            Refusal::NotATicket(why) => format!("this is not a break-glass ticket ({why})"),
            Refusal::WrongKind(k) => format!("`{k}` is not a ticket format this build accepts ({TICKET_KIND})"),
            Refusal::WrongRelax { wanted, got } => {
                format!("this ticket relaxes `{got}`, and what was asked for is `{wanted}`")
            }
            Refusal::KeyNotPinned(k) => format!("key `{k}` is not one this host's policy accepts tickets from"),
            Refusal::BadSignature(why) => format!("the signature does not verify ({why})"),
            Refusal::SignerMismatch => "the ticket names one key and was signed by another".to_string(),
            Refusal::NotYetValid => "the ticket was issued in the future by this host's clock".to_string(),
            Refusal::Expired => "the ticket has expired".to_string(),
            Refusal::TooLong => "the ticket claims a lifetime longer than a day, which no ticket may have".to_string(),
            Refusal::WrongProgram => "the ticket names a different program: its bytes do not hash to what was signed".to_string(),
            Refusal::Spent => "this ticket was already used; every ticket opens the glass once".to_string(),
            Refusal::CannotRecord(why) => {
                format!("the use could not be recorded ({why}), and a break-glass use that is not recorded does not happen")
            }
        }
    }
}

/// An accepted ticket, already spent.
#[derive(Clone, Debug)]
pub struct Accepted {
    pub fingerprint: String,
    pub body: TicketBody,
}

/// Check a ticket against the host policy and, if every check passes, SPEND it — before anything it
/// permits happens, so a crash after this point leaves a spent ticket, never a reusable one.
pub fn accept(state: &Path, ticket_text: &str, relax: &str, program: Option<&[u8]>, now_ms: i64) -> Result<Accepted, Refusal> {
    let policy = match load(state) {
        Policy::Absent => return Err(Refusal::NothingToBreak),
        Policy::Unreadable(why) => return Err(Refusal::PolicyUnreadable(why)),
        Policy::Required(p) => p,
    };
    let ticket: Ticket = serde_json::from_str(ticket_text.trim()).map_err(|e| Refusal::NotATicket(e.to_string()))?;
    let b = &ticket.body;
    if b.kind != TICKET_KIND {
        return Err(Refusal::WrongKind(b.kind.clone()));
    }
    if b.relax != relax {
        return Err(Refusal::WrongRelax { wanted: relax.to_string(), got: b.relax.clone() });
    }
    if !policy.break_glass_keys.iter().any(|k| *k == b.key.to_ascii_lowercase()) {
        return Err(Refusal::KeyNotPinned(b.key.clone()));
    }
    let sig = decode_hex(&ticket.signature).ok_or_else(|| Refusal::BadSignature("not hex".into()))?;
    let msg = serde_json::to_vec(b).expect("a ticket body serializes");
    match delulu_runtime::plugin::verify_detached(&msg, &sig) {
        delulu_runtime::plugin::SignatureStatus::Valid { signer } => {
            if !signer.eq_ignore_ascii_case(&b.key) {
                return Err(Refusal::SignerMismatch);
            }
        }
        delulu_runtime::plugin::SignatureStatus::Invalid { reason } => return Err(Refusal::BadSignature(reason)),
        other => return Err(Refusal::BadSignature(format!("{other:?}"))),
    }
    if b.not_after_ms - b.issued_ms > MAX_LIFETIME_MS || b.not_after_ms < b.issued_ms {
        return Err(Refusal::TooLong);
    }
    if now_ms < b.issued_ms {
        return Err(Refusal::NotYetValid);
    }
    if now_ms > b.not_after_ms {
        return Err(Refusal::Expired);
    }
    if relax == RELAX_SANDBOX {
        let got = program.map(|p| blake3::hash(p).to_hex().to_string());
        if got.is_none() || got != b.program_blake3 {
            return Err(Refusal::WrongProgram);
        }
    }
    let fingerprint = ticket.fingerprint();
    // Spent by an atomic create: of two uses racing, exactly one makes the file.
    let dir = spent_dir(state);
    std::fs::create_dir_all(&dir).map_err(|e| Refusal::CannotRecord(e.to_string()))?;
    match std::fs::OpenOptions::new().write(true).create_new(true).open(dir.join(&fingerprint)) {
        Ok(mut f) => {
            use std::io::Write as _;
            let _ = writeln!(f, "{now_ms}");
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => return Err(Refusal::Spent),
        Err(e) => return Err(Refusal::CannotRecord(e.to_string())),
    }
    Ok(Accepted { fingerprint, body: ticket.body })
}

/// Remove the policy (the `policy-off` ticket's one effect). Spent tickets are kept: they are the
/// record that the glass was broken, and a later policy must not accept one of them again.
pub fn remove_policy(state: &Path) -> Result<(), String> {
    std::fs::remove_file(policy_path(state)).map_err(|e| format!("cannot remove the host policy: {e}"))
}

/// How many tickets this host has spent, and when the last one was.
pub fn spent_summary(state: &Path) -> (usize, Option<std::time::SystemTime>) {
    let Ok(entries) = std::fs::read_dir(spent_dir(state)) else { return (0, None) };
    let mut n = 0;
    let mut last = None;
    for e in entries.flatten() {
        n += 1;
        if let Ok(t) = e.metadata().and_then(|m| m.modified()) {
            last = Some(last.map_or(t, |l: std::time::SystemTime| l.max(t)));
        }
    }
    (n, last)
}

// ----- the gate --------------------------------------------------------------------------------

/// The check every command that executes a program passes through, before it reads a line of it.
///
/// `sandboxed` is whether this run is going to a sandboxed guest; `ticket` is the path given with
/// `--break-glass`. `Ok(None)` means proceed as usual; `Ok(Some)` means proceed WITHOUT the sandbox
/// under an accepted, spent, recorded ticket; `Err(code)` means refuse, with the reason already
/// printed.
pub fn gate(command: &str, sandboxed: bool, ticket: Option<&str>, program: Option<&[u8]>) -> Result<Option<Accepted>, i32> {
    let Some(state) = crate::brokerd::resolve_state_dir(None) else {
        return match ticket {
            None => Ok(None),
            Some(_) => {
                eprintln!("error: `--break-glass`: {}", Refusal::NothingToBreak.explain());
                Err(2)
            }
        };
    };
    let policy = load(&state);
    match (policy.requires_sandbox(), sandboxed, ticket) {
        (false, _, None) => Ok(None),
        (false, _, Some(_)) => {
            eprintln!("error: `--break-glass`: {}", Refusal::NothingToBreak.explain());
            Err(2)
        }
        (true, true, None) => Ok(None),
        (true, true, Some(_)) => {
            eprintln!(
                "error: a break-glass ticket runs a program WITHOUT the sandbox, and this run asked for it; \
                 drop one of `--sandbox` and `--break-glass`"
            );
            Err(2)
        }
        (true, false, None) => {
            let why = match &policy {
                Policy::Unreadable(w) => format!(" (its policy file cannot be read — {w} — which counts as requiring it)"),
                _ => String::new(),
            };
            eprintln!(
                "error: this host requires the sandbox{why}, so `{command}` does not run a program outside it. \
                 Nothing ran.\n  run it with `--sandbox`, or present a ticket the operator signed: \
                 `delulu run FILE --break-glass TICKET` (`delulu sandbox status` shows the policy)"
            );
            Err(2)
        }
        (true, false, Some(path)) => {
            if command != "run" {
                eprintln!("error: a break-glass ticket opens one `delulu run`; `{command}` does not take one");
                return Err(2);
            }
            let text = match std::fs::read_to_string(path) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("error: cannot read the ticket `{path}`: {e}");
                    return Err(2);
                }
            };
            match accept(&state, &text, RELAX_SANDBOX, program, now_ms()) {
                Err(r) => {
                    // A refused ticket is evidence too: someone tried the glass.
                    let _ = crate::guest::audit_required(
                        "break-glass",
                        "deny",
                        Some(r.code().to_string()),
                        Some(serde_json::json!({ "relax": RELAX_SANDBOX, "why": r.explain() })),
                    );
                    eprintln!("error: break-glass refused: {}. Nothing ran.", r.explain());
                    Err(2)
                }
                Ok(a) => {
                    if let Err(e) = crate::guest::audit_required(
                        "break-glass",
                        "allow",
                        Some(a.fingerprint.clone()),
                        Some(record(&a)),
                    ) {
                        eprintln!("error: break-glass refused: {}. Nothing ran.", Refusal::CannotRecord(e).explain());
                        return Err(2);
                    }
                    banner(&a);
                    Ok(Some(a))
                }
            }
        }
    }
}

/// What the audit record and the run report say about an accepted ticket.
pub fn record(a: &Accepted) -> serde_json::Value {
    serde_json::json!({
        "ticket": a.fingerprint,
        "relax": a.body.relax,
        "program_blake3": a.body.program_blake3,
        "key": a.body.key,
        "reason": a.body.reason,
        "not_after": delulu_broker::render_ts_utc(a.body.not_after_ms),
    })
}

fn banner(a: &Accepted) {
    eprintln!("BREAK-GLASS: this program runs WITHOUT the sandbox this host requires.");
    eprintln!("  ticket   {} (key {}…), valid until {}", &a.fingerprint[..16], &a.body.key[..16], delulu_broker::render_ts_utc(a.body.not_after_ms));
    eprintln!("  reason   {}", a.body.reason);
    eprintln!("  Authority, custody, the Guard and the budget are unchanged; only the sandbox is not applied.");
    eprintln!("  The ticket is spent, and the use is in the audit chain as `break-glass`.");
}

// ----- the operator's commands (`delulu sandbox require | release | ticket`) ------------------------

fn value<'a>(args: &'a [String], flag: &str) -> Result<Vec<&'a str>, String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        if args[i] == flag {
            let v = args.get(i + 1).filter(|v| !v.starts_with("--")).ok_or(format!("`{flag}` needs a value"))?;
            out.push(v.as_str());
            i += 2;
            continue;
        }
        if let Some(v) = args[i].strip_prefix(&format!("{flag}=")) {
            out.push(v);
        }
        i += 1;
    }
    Ok(out)
}

/// Every flag present must be one this verb knows; anything else is refused, never ignored.
fn only(args: &[String], verb: &str, known: &[&str]) -> Result<(), String> {
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        if a == verb || a == "--json" {
            i += 1;
            continue;
        }
        let name = a.split('=').next().unwrap_or(a);
        if !a.starts_with("--") || !known.contains(&name) {
            return Err(format!("`sandbox {verb}` does not take `{a}`"));
        }
        // A value-taking flag written as two words consumes its value here.
        i += if a.contains('=') || matches!(name, "--no-break-glass" | "--release") { 1 } else { 2 };
    }
    Ok(())
}

fn state_or_refuse() -> Result<PathBuf, i32> {
    crate::brokerd::resolve_state_dir(None).ok_or_else(|| {
        eprintln!("error: cannot resolve the state directory (no HOME/USERPROFILE)");
        2
    })
}

/// `delulu sandbox require (--break-glass-key HEX)... | --no-break-glass`
pub fn cmd_require(args: &[String], json: bool) -> i32 {
    if let Err(e) = only(args, "require", &["--break-glass-key", "--no-break-glass"]) {
        eprintln!("error: {e}");
        return 2;
    }
    let keys: Vec<String> = match value(args, "--break-glass-key") {
        Ok(v) => v.into_iter().map(str::to_string).collect(),
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    let none = args.iter().any(|a| a == "--no-break-glass");
    if none && !keys.is_empty() {
        eprintln!("error: `--no-break-glass` and `--break-glass-key` contradict each other; say one");
        return 2;
    }
    let state = match state_or_refuse() {
        Ok(s) => s,
        Err(c) => return c,
    };
    let written = if none { require_without_break_glass(&state) } else { require(&state, &keys) };
    let policy = match written {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    if let Err(e) = crate::guest::audit_required(
        "sandbox-policy",
        "allow",
        Some("require".to_string()),
        Some(serde_json::json!({ "break_glass_keys": policy.break_glass_keys })),
    ) {
        // The policy is in force either way (it only restricts); say that its record is missing.
        eprintln!("warning: the policy is in force, but its audit record could not be written: {e}");
    }
    if json {
        crate::cli::print_success_envelope("sandbox", serde_json::json!({ "subcommand": "require", "policy": policy }));
    } else {
        println!("ok: this host now requires the sandbox for every program `delulu` runs");
        if policy.break_glass_keys.is_empty() {
            println!("  no break-glass key is pinned, so nothing opens it; `sandbox release` needs one, so this cannot be undone through `delulu`");
        } else {
            for k in &policy.break_glass_keys {
                println!("  break-glass tickets are accepted from key {k}");
            }
            println!("  keep the private half(s) OFF this host: a ticket signed here by a same-user process would defeat it");
        }
    }
    0
}

/// `delulu sandbox release --break-glass TICKET` — the policy comes off only by a ticket.
pub fn cmd_release(args: &[String], json: bool) -> i32 {
    if let Err(e) = only(args, "release", &["--break-glass"]) {
        eprintln!("error: {e}");
        return 2;
    }
    let path = match value(args, "--break-glass") {
        Ok(v) if v.len() == 1 => v[0],
        Ok(_) => {
            eprintln!("error: `sandbox release` needs exactly one `--break-glass TICKET` (a `policy-off` ticket the operator signed)");
            return 2;
        }
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    let state = match state_or_refuse() {
        Ok(s) => s,
        Err(c) => return c,
    };
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: cannot read the ticket `{path}`: {e}");
            return 2;
        }
    };
    let accepted = match accept(&state, &text, RELAX_POLICY_OFF, None, now_ms()) {
        Ok(a) => a,
        Err(r) => {
            let _ = crate::guest::audit_required(
                "break-glass",
                "deny",
                Some(r.code().to_string()),
                Some(serde_json::json!({ "relax": RELAX_POLICY_OFF, "why": r.explain() })),
            );
            eprintln!("error: release refused: {}. The policy stands.", r.explain());
            return 2;
        }
    };
    if let Err(e) = crate::guest::audit_required("break-glass", "allow", Some(accepted.fingerprint.clone()), Some(record(&accepted))) {
        eprintln!("error: release refused: {}. The policy stands.", Refusal::CannotRecord(e).explain());
        return 2;
    }
    if let Err(e) = remove_policy(&state) {
        eprintln!("error: {e}");
        return 2;
    }
    if json {
        crate::cli::print_success_envelope("sandbox", serde_json::json!({ "subcommand": "release", "ticket": record(&accepted) }));
    } else {
        println!("ok: this host no longer requires the sandbox (released by ticket {}…: {})", &accepted.fingerprint[..16], accepted.body.reason);
    }
    0
}

/// Parse `90s`, `15m`, `2h`: a ticket's lifetime.
fn ttl_ms(s: &str) -> Option<i64> {
    let (n, unit) = s.split_at(s.find(|c: char| !c.is_ascii_digit())?);
    let n: i64 = n.parse().ok()?;
    let ms = match unit {
        "s" => n * 1000,
        "m" => n * 60_000,
        "h" => n * 3_600_000,
        _ => return None,
    };
    (ms > 0).then_some(ms)
}

/// `delulu sandbox ticket --key SEED (--program FILE | --release) --ttl 15m --reason TEXT [--out FILE]`
///
/// Run where the private key is, which should not be the host the ticket is for.
pub fn cmd_ticket(args: &[String], json: bool) -> i32 {
    if let Err(e) = only(args, "ticket", &["--key", "--program", "--release", "--ttl", "--reason", "--out"]) {
        eprintln!("error: {e}");
        return 2;
    }
    let one = |flag: &str| -> Result<Option<String>, String> {
        match value(args, flag)?.as_slice() {
            [] => Ok(None),
            [v] => Ok(Some(v.to_string())),
            _ => Err(format!("`{flag}` is given more than once")),
        }
    };
    let parsed = (|| -> Result<_, String> {
        let key = one("--key")?.ok_or("`--key SEEDFILE` names the private key that signs (see `delulu keygen`)")?;
        let ttl = one("--ttl")?.ok_or("`--ttl` is required: 90s, 15m or 2h, at most a day")?;
        let reason = one("--reason")?.ok_or("`--reason` is required: it is what the audit record says")?;
        Ok((key, ttl, reason, one("--program")?, one("--out")?))
    })();
    let (key, ttl, reason, program, out) = match parsed {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    let release = args.iter().any(|a| a == "--release");
    if release == program.is_some() {
        eprintln!("error: a ticket either lets one program run (`--program FILE`) or releases the policy (`--release`) — exactly one");
        return 2;
    }
    let Some(ttl) = ttl_ms(&ttl) else {
        eprintln!("error: `--ttl {ttl}` is not a lifetime (90s, 15m, 2h)");
        return 2;
    };
    let seed: [u8; 32] = match std::fs::read(&key) {
        Ok(b) if b.len() == 32 => b.try_into().expect("32 bytes"),
        Ok(b) => {
            eprintln!("error: `{key}` is {} bytes, not a 32-byte ed25519 seed", b.len());
            return 2;
        }
        Err(e) => {
            eprintln!("error: cannot read `{key}`: {e}");
            return 2;
        }
    };
    let bytes = match &program {
        Some(p) => match std::fs::read(p) {
            Ok(b) => Some(b),
            Err(e) => {
                eprintln!("error: cannot read `{p}`: {e}");
                return 2;
            }
        },
        None => None,
    };
    let relax = if release { RELAX_POLICY_OFF } else { RELAX_SANDBOX };
    let ticket = match mint(&seed, relax, bytes.as_deref(), now_ms(), ttl, &reason) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            return 2;
        }
    };
    let text = serde_json::to_string_pretty(&ticket).expect("a ticket serializes");
    if let Some(path) = &out {
        if let Err(e) = std::fs::write(path, format!("{text}\n")) {
            eprintln!("error: cannot write `{path}`: {e}");
            return 2;
        }
    }
    if json {
        crate::cli::print_success_envelope("sandbox", serde_json::json!({ "subcommand": "ticket", "ticket": ticket, "fingerprint": ticket.fingerprint() }));
    } else if out.is_none() {
        println!("{text}");
    } else {
        println!("ok: wrote a `{relax}` ticket, valid until {} ({}…)", delulu_broker::render_ts_utc(ticket.body.not_after_ms), &ticket.fingerprint()[..16]);
    }
    0
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

pub fn encode_hex(b: &[u8]) -> String {
    b.iter().fold(String::with_capacity(b.len() * 2), |mut s, x| {
        use std::fmt::Write as _;
        let _ = write!(s, "{x:02x}");
        s
    })
}

pub fn decode_hex(s: &str) -> Option<Vec<u8>> {
    let s = s.trim();
    if !s.len().is_multiple_of(2) || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("delulu-bg-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    const SEED: [u8; 32] = [7u8; 32];
    const OTHER: [u8; 32] = [9u8; 32];
    const PROGRAM: &[u8] = b"module m\n\nfn main(root: Root) {\n}\n";

    fn pinned(tag: &str) -> PathBuf {
        let s = state(tag);
        require(&s, &[delulu_runtime::plugin::public_key_hex(&SEED)]).unwrap();
        s
    }

    fn ticket(seed: &[u8; 32], relax: &str, now: i64, ttl: i64) -> String {
        serde_json::to_string(&mint(seed, relax, Some(PROGRAM), now, ttl, "the pump controller is down").unwrap()).unwrap()
    }

    #[test]
    fn a_valid_ticket_opens_the_glass_exactly_once() {
        let s = pinned("once");
        let t = ticket(&SEED, RELAX_SANDBOX, 1_000, 60_000);
        let a = accept(&s, &t, RELAX_SANDBOX, Some(PROGRAM), 2_000).expect("a valid ticket is accepted");
        assert_eq!(a.body.reason, "the pump controller is down");
        assert_eq!(accept(&s, &t, RELAX_SANDBOX, Some(PROGRAM), 3_000).unwrap_err(), Refusal::Spent, "spent on first use");
        assert_eq!(spent_summary(&s).0, 1);
    }

    #[test]
    fn every_way_a_ticket_can_be_wrong_is_refused_and_named() {
        let s = pinned("wrong");
        let good = mint(&SEED, RELAX_SANDBOX, Some(PROGRAM), 1_000, 60_000, "why").unwrap();

        let unpinned = ticket(&OTHER, RELAX_SANDBOX, 1_000, 60_000);
        assert!(matches!(accept(&s, &unpinned, RELAX_SANDBOX, Some(PROGRAM), 2_000), Err(Refusal::KeyNotPinned(_))));

        let mut tampered = good.clone();
        tampered.body.reason = "a different reason".into();
        let tampered = serde_json::to_string(&tampered).unwrap();
        assert!(matches!(accept(&s, &tampered, RELAX_SANDBOX, Some(PROGRAM), 2_000), Err(Refusal::BadSignature(_))));

        // A body that names the pinned key but was signed by another: the signer is checked, not trusted.
        let mut forged = mint(&OTHER, RELAX_SANDBOX, Some(PROGRAM), 1_000, 60_000, "why").unwrap();
        forged.body.key = good.body.key.clone();
        let msg = serde_json::to_vec(&forged.body).unwrap();
        forged.signature = encode_hex(&delulu_runtime::plugin::sign_detached(&OTHER, &msg));
        let forged = serde_json::to_string(&forged).unwrap();
        assert_eq!(accept(&s, &forged, RELAX_SANDBOX, Some(PROGRAM), 2_000).unwrap_err(), Refusal::SignerMismatch);

        let good_text = serde_json::to_string(&good).unwrap();
        assert_eq!(accept(&s, &good_text, RELAX_SANDBOX, Some(PROGRAM), 70_000).unwrap_err(), Refusal::Expired);
        assert_eq!(accept(&s, &good_text, RELAX_SANDBOX, Some(PROGRAM), 500).unwrap_err(), Refusal::NotYetValid);
        assert_eq!(accept(&s, &good_text, RELAX_SANDBOX, Some(b"another program"), 2_000).unwrap_err(), Refusal::WrongProgram);
        assert!(matches!(accept(&s, &good_text, RELAX_POLICY_OFF, None, 2_000), Err(Refusal::WrongRelax { .. })));
        assert!(matches!(accept(&s, "{\"body\":1}", RELAX_SANDBOX, Some(PROGRAM), 2_000), Err(Refusal::NotATicket(_))));
        // None of the refusals spent it: a wrong use must not burn a good ticket.
        assert!(accept(&s, &good_text, RELAX_SANDBOX, Some(PROGRAM), 2_000).is_ok());
    }

    #[test]
    fn a_ticket_cannot_outlive_a_day_even_when_it_is_signed_to() {
        assert!(mint(&SEED, RELAX_SANDBOX, Some(PROGRAM), 0, MAX_LIFETIME_MS + 1, "why").is_err(), "not minted");
        let s = pinned("long");
        let mut long = mint(&SEED, RELAX_SANDBOX, Some(PROGRAM), 0, 60_000, "why").unwrap();
        long.body.not_after_ms = MAX_LIFETIME_MS * 30;
        let msg = serde_json::to_vec(&long.body).unwrap();
        long.signature = encode_hex(&delulu_runtime::plugin::sign_detached(&SEED, &msg));
        let long = serde_json::to_string(&long).unwrap();
        assert_eq!(accept(&s, &long, RELAX_SANDBOX, Some(PROGRAM), 1_000).unwrap_err(), Refusal::TooLong, "and not accepted");
    }

    #[test]
    fn no_policy_means_nothing_to_break_and_a_damaged_policy_opens_for_no_one() {
        let s = state("none");
        let t = ticket(&SEED, RELAX_SANDBOX, 1_000, 60_000);
        assert_eq!(accept(&s, &t, RELAX_SANDBOX, Some(PROGRAM), 2_000).unwrap_err(), Refusal::NothingToBreak);
        assert_eq!(load(&s), Policy::Absent);

        std::fs::write(policy_path(&s), "{ not json").unwrap();
        assert!(load(&s).requires_sandbox(), "a damaged policy still requires the sandbox");
        assert!(matches!(accept(&s, &t, RELAX_SANDBOX, Some(PROGRAM), 2_000), Err(Refusal::PolicyUnreadable(_))));
        // A file saying "not required" is not a way to switch it off either.
        std::fs::write(policy_path(&s), r#"{"schema":1,"sandbox_required":false,"break_glass_keys":[]}"#).unwrap();
        assert!(load(&s).requires_sandbox());
    }

    #[test]
    fn a_policy_is_never_replaced_in_place_and_its_keys_must_be_keys() {
        let s = state("replace");
        assert!(require(&s, &[]).is_err(), "no key means --no-break-glass, said out loud");
        assert!(require(&s, &["not-a-key".into()]).is_err());
        require(&s, &[delulu_runtime::plugin::public_key_hex(&SEED)]).unwrap();
        let err = require(&s, &[delulu_runtime::plugin::public_key_hex(&OTHER)]).unwrap_err();
        assert!(err.contains("already requires"), "adding a key would widen who may break glass: {err}");
    }

    #[test]
    fn hex_round_trips_and_refuses_what_is_not_hex() {
        assert_eq!(decode_hex(&encode_hex(&[0, 1, 254, 255])), Some(vec![0, 1, 254, 255]));
        for bad in ["0", "zz", "0g", "abc"] {
            assert_eq!(decode_hex(bad), None, "`{bad}`");
        }
    }
}
