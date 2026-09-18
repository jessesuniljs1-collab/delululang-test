//! `delulu doctor` — one command that says whether this machine, and this checkout, are healthy.
//!
//! Two sections, and the split matters. **Environment** checks the things that decide whether any
//! `delulu` command will work at all, and runs everywhere. **Repository** checks the DeluluLang
//! source tree against its own map, and runs *only* when doctor is standing in that tree — a
//! contributor's check, not something to pretend at a user who installed the compiler.
//!
//! The repository section regenerates the Survey when it is behind and then verifies the map's
//! integrity, so the whole "is the map current, and is it sound?" loop is one command.
//!
//! # Why it does not write unless something is wrong
//!
//! Regeneration only touches files that actually differ, and each write goes through a temporary
//! file and a rename ([`delulu_survey::sync_outputs`]). In a healthy checkout `delulu doctor`
//! writes nothing at all, so running it is not a change to the repository — which is the property
//! that makes it safe to run from a script, a hook, or several test processes at once. Campaign
//! finding C69 was a short-lived process writing a shared artifact concurrently; the shape is the
//! same here and is designed out rather than hoped away.

use std::path::{Path, PathBuf};

/// What a single check concluded.
#[derive(PartialEq, Clone, Copy, Debug)]
enum Status {
    /// Healthy.
    Ok,
    /// Was wrong, and doctor corrected it.
    Fixed,
    /// True and worth a human's eye. Never fails the run.
    Note,
    /// Wrong, and doctor could not or would not fix it. Fails the run.
    Problem,
}

impl Status {
    fn word(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Fixed => "fixed",
            Status::Note => "note",
            Status::Problem => "problem",
        }
    }
}

struct Check {
    section: &'static str,
    name: &'static str,
    status: Status,
    detail: String,
}

#[derive(Default)]
struct Report {
    checks: Vec<Check>,
    regenerated: Vec<&'static str>,
    survey: Option<(usize, usize, [usize; 3])>,
}

impl Report {
    fn push(&mut self, section: &'static str, name: &'static str, status: Status, detail: impl Into<String>) {
        self.checks.push(Check { section, name, status, detail: detail.into() });
    }
    fn problems(&self) -> usize {
        self.checks.iter().filter(|c| c.status == Status::Problem).count()
    }
}

pub fn cmd_doctor(args: &[String]) -> i32 {
    let mut json = false;
    let mut check_only = false;
    for a in args {
        match a.as_str() {
            "--json" => json = true,
            // The read-only mode. Used by the CLI contract suite, which runs subcommands from the
            // workspace root in parallel: a doctor that regenerated there would have several
            // processes writing `docs/survey/` while another test reads it.
            "--check" => check_only = true,
            "--help" | "-h" => {
                println!("{}", help());
                return 0;
            }
            other => {
                // Human reason to stderr, exit 2, and nothing on stdout. The single JSON envelope
                // for a usage error is emitted by the outer layer (`cli.rs`), which is why this
                // must not emit one too — doing so produced *two* objects on stdout, the exact
                // shape campaign C2's fix was gated against, caught by `json_contract`'s rule that
                // stdout must parse as exactly one JSON value.
                eprintln!("error: unknown option `{other}`\n\n{}", help());
                return 2;
            }
        }
    }

    // **One command, two audiences, and the scope decided by where you are standing.**
    //
    // The environment section answers "is my install healthy?" and is for anyone who uses Delulu.
    // The repository section answers "is DeluluLang's own map current and sound?" and is only for
    // someone working ON the language — so it runs only inside that source tree.
    //
    // These are not two commands. It is one question whose answer has more to say in one place
    // than the other, the way `git status` says more inside a repository. Splitting them would
    // either duplicate the environment checks or oblige a contributor to remember two commands,
    // and the one they would forget is the repository one — which is the one that rots.
    //
    // The note below names *whose* repository, so a user with a Delulu project of their own does
    // not read it as a remark about theirs.
    let mut r = Report::default();
    environment(&mut r);
    // Before the repository map: this is about the machine the operator is deploying ON, which is
    // true whether or not they are standing in DeluluLang's own source tree.
    security_posture(&mut r);
    sandbox_section(&mut r);
    match delulu_survey::find_source_tree() {
        Some(root) => repository(&mut r, &root, check_only),
        None => r.push(
            "repository",
            "delulu source tree",
            Status::Note,
            "not inside DeluluLang's OWN source tree, so its repository-map checks do not apply \
             here — nothing about your project is being skipped",
        ),
    }

    if json {
        // Record the emission BEFORE the exit code is decided. `doctor` exits 1 when a check reports
        // a problem, and `cli::run` adds a fallback envelope on any nonzero exit that has not already
        // put one on stdout (campaign finding C2). Without this call the caller got TWO objects for
        // exactly the run they care about most: the one that found something wrong.
        crate::cli::note_json_emitted();
        println!("{}", envelope(&r));
    } else {
        render(&r);
    }
    if r.problems() > 0 {
        1
    } else {
        0
    }
}

fn help() -> String {
    "delulu doctor — check this machine and this checkout\n\n\
     USAGE:\n  \
       delulu doctor            check, and regenerate the repository map if it is behind\n  \
       delulu doctor --check    report only; never write\n  \
       delulu doctor --json     one JSON envelope\n\n\
     Exit 0 when healthy, 1 when a problem remains, 2 on a bad invocation."
        .to_string()
}

// --- environment ---------------------------------------------------------------------------------

fn environment(r: &mut Report) {
    r.push("environment", "delulu", Status::Ok, format!("version {}", env!("CARGO_PKG_VERSION")));

    // Whether embedded Python was compiled in. This is a build-time fact, so it is knowable
    // exactly — unlike whether an interpreter will initialize, which is only knowable by trying.
    #[cfg(feature = "python")]
    r.push("environment", "embedded Python", Status::Ok, "compiled in; `root.python(...)` is available with a grant");
    #[cfg(not(feature = "python"))]
    r.push(
        "environment",
        "embedded Python",
        Status::Note,
        "not compiled in — `root.python(...)` returns Unavailable (DL1307). Build without `--no-default-features` to include it",
    );

    match state_dir() {
        Some(dir) => {
            let shown = dir.display().to_string();
            if !dir.exists() {
                // Absent is normal before first use, and says nothing is wrong.
                r.push("environment", "state directory", Status::Ok, format!("{shown} (not created yet)"));
            } else if writable(&dir) {
                r.push("environment", "state directory", Status::Ok, shown);
            } else {
                r.push(
                    "environment",
                    "state directory",
                    Status::Problem,
                    format!("{shown} is not writable — keys, credentials and locales cannot be stored"),
                );
            }
        }
        None => r.push(
            "environment",
            "state directory",
            Status::Problem,
            "neither DELULU_HOME nor HOME/USERPROFILE is set, so there is nowhere to keep state",
        ),
    }

    let mode = std::env::var("DELULU_BROKER").unwrap_or_else(|_| "embedded".into());
    match mode.as_str() {
        "embedded" | "daemon" => r.push("environment", "broker mode", Status::Ok, mode),
        other => r.push(
            "environment",
            "broker mode",
            Status::Problem,
            format!("DELULU_BROKER is `{other}`; only `embedded` and `daemon` exist"),
        ),
    }

    audit_chain(r);
}

/// The audit chain is the one piece of durable state a broken machine can silently damage, so it is
/// worth verifying rather than assuming.
/// The **deployment** half of the security model, made checkable.
///
/// # Why this section exists
///
/// DeluluLang's honest position is that the same-OS-user boundary is *outside* the proof boundary
/// (category 7): a process running as this user is indistinguishable from the broker to the kernel,
/// and no amount of code changes that. The published answer has always been "run untrusted agents as
/// a separate OS account, and keep the anchor private key off the box" — but that answer lived only
/// in prose, so nothing ever told an operator whether they had actually done it.
///
/// A deployment property that cannot be checked is a deployment property nobody checks. These are
/// the three facts that decide whether the boundary is real on THIS machine, so `delulu doctor`
/// reports them. They are `Note`, not `Problem`, where they are a legitimate choice: legacy mode is
/// a supported configuration, and doctor's job here is to make the choice visible, not to fail a
/// checkout for it.
/// PS-0-05: what confines a program on THIS host, built on the `sandbox probe` attempts — only lines
/// that change a decision, and none of them fails the run (a missing boundary is a fact, not a fault).
fn sandbox_section(r: &mut Report) {
    let levels = crate::sandbox::probe();
    let top = levels.iter().filter(|l| l.available()).map(|l| l.level).max().unwrap_or(0);
    let s = "sandbox";
    r.push(s, "backend", Status::Note, "inproc — the language and custody in one process; no OS boundary around the program");
    let next = levels.iter().find(|l| !l.available()).and_then(|l| l.first_missing().map(|a| (l.level, a)));
    r.push(
        s,
        "level available",
        Status::Note,
        match next {
            Some((lvl, a)) => format!("L{top} — L{lvl} is absent: {}: {}", a.what, a.detail),
            None => format!("L{top}"),
        },
    );
    let kvm = levels[2].attempts.iter().find(|a| a.what == "KVM");
    r.push(
        s,
        "KVM",
        Status::Note,
        match kvm {
            Some(a) if a.ok => format!("available — {}", a.detail),
            Some(a) => format!("absent — {}", a.detail),
            None => "not attempted".to_string(),
        },
    );
    let prims: Vec<String> = levels[1]
        .attempts
        .iter()
        .filter(|a| a.what != "the L1 guest launcher")
        .map(|a| format!("{}: {}", a.what, if a.ok { "works" } else { "refused" }))
        .collect();
    r.push(s, "OS primitives", Status::Note, format!("{} (attempted now; the L1 jail that would use them is not built — PS-A)", prims.join(", ")));
    r.push(
        s,
        "network enforcement",
        Status::Note,
        "grant allowlist, special-use addresses refused unless `net.special=`; there is no network client (RW 4.12)",
    );
    r.push(
        s,
        "filesystem enforcement",
        Status::Note,
        "primitive-table containment on resolved paths; hostile spellings refused (D-NE-29); check-then-open (RW 4.6)",
    );
    r.push(s, "identity separation", Status::Note, "none — the program runs as this OS user (RW 4.4, category 7)");
    r.push(
        s,
        "resource controls",
        Status::Note,
        "none on the main program (RW 4.15); a foreign call is killed after its deadline (NE-21)",
    );
    r.push(s, "profile", Status::Note, "none — strict mode; sandbox profiles arrive with PS-A");
    r.push(s, "break-glass", Status::Note, "off — no break-glass mechanism exists in this build");
    let shortened = std::env::var("DELULU_FOREIGN_CALL_DEADLINE_MS").ok();
    r.push(
        s,
        "relaxed restrictions",
        Status::Note,
        match shortened {
            Some(v) => format!("none relaxed (DELULU_FOREIGN_CALL_DEADLINE_MS={v} can only tighten a bound)"),
            None => "none".to_string(),
        },
    );
}

fn security_posture(r: &mut Report) {
    let Some(dir) = state_dir() else { return };

    // 1. Root issuance (DISC-1). The single most consequential deployment fact: in legacy mode any
    //    process running as this user can mint root authority and command a guard-SEALED resource.
    let (anchor, poisoned) = crate::brokerd::load_root_policy(&dir);
    match (anchor, poisoned) {
        (Some(a), _) => r.push(
            "security posture",
            "root issuance",
            Status::Ok,
            format!("STRICT — a root may enter only by adopting a certificate that verifies against `{a}`; unsigned issuance is refused DL1421"),
        ),
        (None, true) => r.push(
            "security posture",
            "root issuance",
            Status::Problem,
            format!(
                "root policy `{}` EXISTS but is UNREADABLE — the broker refuses to create root authority by any path until it is repaired or removed (ROOTPOLICY-1)",
                crate::brokerd::root_policy_path(&dir).display()
            ),
        ),
        (None, false) => r.push(
            "security posture",
            "root issuance",
            Status::Note,
            "LEGACY — any process running as this OS user can mint root authority, including over a guard-sealed resource (DISC-1). Turn the gate on with `delulu broker start --require-anchored-roots <anchor-pubkey>`",
        ),
    }

    // 2. Anchor key custody. Strict mode's whole value is that the private half signs OFFLINE — the
    //    broker never needs it. A signing key sitting beside the state it protects collapses that
    //    back to file permissions, which is exactly the boundary category 7 says is not one.
    let key = dir.join("grant.key");
    if key.exists() {
        r.push(
            "security posture",
            "anchor key custody",
            Status::Note,
            format!(
                "a signing key is present at `{}`. If this is the anchor for strict mode, the boundary reduces to file permissions against a same-user process — keep the anchor PRIVATE key on another machine and copy only certificates here",
                key.display()
            ),
        );
    } else {
        r.push(
            "security posture",
            "anchor key custody",
            Status::Ok,
            "no signing key in the state directory — an offline anchor is what strict mode's guarantee rests on",
        );
    }

    // 3. Whether the filesystem can keep a secret at all (P21). On 9p/DrvFs/NFS a `chmod` is a
    //    silent no-op, so "owner-only" is a lie and a separate OS account buys nothing.
    if dir.exists() {
        match crate::signing::perms_unenforced_refusal(&dir, "the broker state directory", false, "") {
            Some(msg) => r.push("security posture", "state dir permissions", Status::Problem, msg),
            None => r.push(
                "security posture",
                "state dir permissions",
                Status::Ok,
                "on a filesystem that enforces owner-only permissions, so a separate OS account is a real boundary here",
            ),
        }
    }

    // 4. Does the RUNNING broker agree with the policy on disk? See `running_mode_agrees`.
    running_mode_agrees(r, &dir);

    // 5. Can THIS process reach the broker's state? The one fact that decides whether Tier 2 is real.
    reachability(r, &dir);
}

/// Does the broker that is actually running agree with the policy file?
///
/// `load_root_policy` reads what the policy says *now*; a daemon started before that file was written
/// is still serving the mode it booted with. Everything else in this section would then report STRICT
/// while the live broker happily mints unsigned roots — a posture check that reads configuration and
/// calls it behaviour.
///
/// The `root-policy-mode` record each start writes into the hash-chained log is what makes the
/// difference observable: it is the mode the running daemon *booted with*, not the mode someone
/// intended. A disagreement is reported as a `problem`, because "I set the policy" and "the gate is
/// on" are exactly the two things an operator must not confuse.
fn running_mode_agrees(r: &mut Report, dir: &Path) {
    let audit = dir.join("audit");
    if !audit.exists() {
        return; // nothing has ever run here; §1 already reported the configured mode
    }
    let Ok(records) = delulu_broker::tail(&audit, 500) else { return };
    let Some(last) = records.iter().rev().find(|x| x.action == "root-policy-mode") else {
        r.push(
            "security posture",
            "running broker mode",
            Status::Note,
            "this audit log predates mode recording, so what mode the running broker booted with cannot be read from it — restart the broker to record it",
        );
        return;
    };
    let booted = last.target.clone().unwrap_or_default();
    let configured = configured_mode(dir);
    let (status, detail) = mode_agreement(&booted, configured);
    r.push("security posture", "running broker mode", status, detail);
}

/// The mode the policy file asks for, as the word the audit log records.
fn configured_mode(dir: &Path) -> &'static str {
    match crate::brokerd::load_root_policy(dir) {
        (Some(_), _) => "strict",
        (None, true) => "unreadable-policy",
        (None, false) => "legacy",
    }
}

/// The decision, separated from the I/O so it can be tested exhaustively.
///
/// Kept pure deliberately: the interesting cases are combinations of two words, and a test that has
/// to start a daemon to reach them would exercise the process plumbing instead of the judgement —
/// and this repository allows exactly one integration test to spawn a real process.
fn mode_agreement(booted: &str, configured: &str) -> (Status, String) {
    if booted == configured {
        return (
            Status::Ok,
            format!("the last broker start recorded `{booted}`, which matches the policy on disk"),
        );
    }
    (
        Status::Problem,
        format!(
            "the policy on disk says `{configured}` but the last broker start recorded `{booted}` — a daemon serves the mode it BOOTED with, so restart the broker or the change has not taken effect"
        ),
    )
}

/// Can this process reach the broker's state directory?
///
/// This is the one fact that decides whether the Tier-2 boundary is real, and it is the one thing
/// prose could never tell an operator: `DEPLOYMENT.md` can say "run untrusted agents as a separate OS
/// account", but only running this check **as that account** answers whether they did.
///
/// It is deliberately a capability test rather than an identity comparison. A user name read from the
/// environment is a claim; attempting the write is the fact — and it is the same fact the attacker
/// would establish. Reported as a `note` either way, because doctor cannot know which account it is
/// being run as and must not pretend to: it states what this process can do and leaves the operator
/// to say whether that is the agent.
fn reachability(r: &mut Report, dir: &Path) {
    if !dir.exists() {
        return;
    }
    let who = std::env::var("USER").or_else(|_| std::env::var("USERNAME")).unwrap_or_else(|_| "<unknown>".into());
    if writable(dir) {
        r.push(
            "security posture",
            "state dir reachability",
            Status::Note,
            format!(
                "this process (user `{who}`) CAN write `{}`. If this is the account your untrusted programs run as, the separate-OS-account boundary is NOT in force here — it can read the broker key and edit the root policy",
                dir.display()
            ),
        );
    } else {
        r.push(
            "security posture",
            "state dir reachability",
            Status::Ok,
            format!("this process (user `{who}`) cannot write `{}` — run this as the account your agents use; being refused here is what Tier 2 looks like", dir.display()),
        );
    }
}

fn audit_chain(r: &mut Report) {
    let Some(dir) = state_dir().map(|h| h.join("audit")) else { return };
    if !dir.exists() {
        r.push("environment", "audit chain", Status::Ok, "no audit log yet");
        return;
    }
    match delulu_broker::verify(&dir) {
        Ok(stats) => r.push(
            "environment",
            "audit chain",
            Status::Ok,
            format!("verified — {} record(s) across {} file(s)", stats.records, stats.files),
        ),
        Err(e) => r.push(
            "environment",
            "audit chain",
            Status::Problem,
            format!("{} failed verification: {e}. Do not discard it — `delulu audit verify` reports where", dir.display()),
        ),
    }
}

/// The state directory doctor reports on — **the one the broker actually uses**.
///
/// # Campaign finding DOCTOR-STATEDIR-1
///
/// This resolved `DELULU_HOME`, else `$HOME/.delulu`, and never consulted `DELULU_STATE_DIR` — the
/// documented override the broker itself honors (`brokerd::resolve_state_dir`, and the one
/// `delulu audit --dir`'s help names). The two agree by default and diverge exactly when an operator
/// runs an isolated broker, which is when it matters: doctor then reported the audit chain, the
/// state directory and — once this file grew a security-posture section — the *root-issuance mode* of
/// a completely different store, with a confident `ok`.
///
/// That is the "reads the wrong store" class this project has already named twice (finding C75, and
/// F-CUSTODY-2, where `delulu audit` defaulted to the global log and could verify a chain unrelated
/// to the incident). F-CUSTODY-2 fixed `audit`; nobody fixed `doctor`, whose entire job is to answer
/// "is this deployment sound?".
///
/// `DELULU_STATE_DIR` therefore wins. `DELULU_HOME` stays as the next fallback: it is what
/// `signing.rs` uses and what the CLI tests set for isolation, so dropping it would silently point
/// the suite at the developer's real state.
fn state_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("DELULU_STATE_DIR") {
        return Some(PathBuf::from(d));
    }
    std::env::var_os("DELULU_HOME").map(PathBuf::from).or_else(|| {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(|h| PathBuf::from(h).join(".delulu"))
    })
}

fn writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".delulu-doctor-{}.tmp", std::process::id()));
    match std::fs::write(&probe, b"") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

// --- repository ----------------------------------------------------------------------------------

// Finding the source tree is knowledge about the repository, so it lives with the map:
// `delulu_survey::find_source_tree`. The CLI asking "where am I?" should not also be the thing
// that decides what a DeluluLang checkout looks like.

/// Orchestration only.
///
/// Every judgement below is the Survey's: whether the map is behind, whether to rewrite it, which
/// invariants exist and whether they hold, how the discrepancies tally. This function decides
/// nothing about the map — it asks once, and turns the answer into lines a person can read. That
/// separation is the point: an invariant added to `delulu_survey::health::integrity` appears here
/// with no change to this file, and is enforced by that crate's suite on the same commit.
fn repository(r: &mut Report, root: &Path, check_only: bool) {
    use delulu_survey::Repair;

    r.push("repository", "delulu source tree", Status::Ok, root.display().to_string());

    let mode = if check_only { Repair::ReportOnly } else { Repair::Regenerate };
    let h = delulu_survey::inspect(root, mode);

    if let Some(e) = &h.write_error {
        r.push("repository", "survey freshness", Status::Problem, format!("cannot write docs/survey: {e}"));
    } else if h.stale.is_empty() {
        r.push(
            "repository",
            "survey freshness",
            Status::Ok,
            format!("the map matches the tree — {} nodes, {} edges", h.nodes, h.edges),
        );
    } else if h.regenerated.is_empty() {
        r.push(
            "repository",
            "survey freshness",
            Status::Problem,
            format!("{} behind the tree: {}. Run `delulu doctor` without --check", h.stale.len(), h.stale.join(", ")),
        );
    } else {
        r.push(
            "repository",
            "survey freshness",
            Status::Fixed,
            format!(
                "was behind the tree; regenerated {} — commit it with the change that caused it",
                h.regenerated.join(", ")
            ),
        );
    }

    for check in &h.integrity {
        r.push(
            "repository",
            check.name,
            if check.ok { Status::Ok } else { Status::Problem },
            check.detail.clone(),
        );
    }

    let t = h.tally;
    r.regenerated = h.regenerated.clone();
    r.survey = Some((h.nodes, h.edges, [t.errors, t.warnings, t.notes]));
    // A discrepancy is a report, not a verdict: errors fail the run, the rest are for a reader.
    let status =
        if !t.is_healthy() { Status::Problem } else if t.warnings > 0 { Status::Note } else { Status::Ok };
    r.push(
        "repository",
        "discrepancies",
        status,
        format!("{} error, {} warning, {} note — docs/survey/DISCREPANCIES.md", t.errors, t.warnings, t.notes),
    );
}

// --- output ---------------------------------------------------------------------------------------

fn render(r: &Report) {
    let mut section = "";
    for c in &r.checks {
        if c.section != section {
            println!("\n{}", c.section);
            section = c.section;
        }
        println!("  {:<8} {:<28} {}", c.status.word(), c.name, c.detail);
    }

    let problems = r.problems();
    let fixed = r.checks.iter().filter(|c| c.status == Status::Fixed).count();
    println!();
    if problems == 0 {
        let tail = if fixed > 0 { format!(", {fixed} fixed") } else { String::new() };
        println!("ok: {} check(s) passed{tail}", r.checks.len());
    } else {
        println!("{problems} problem(s) remain — see the lines marked `problem` above");
    }
}

/// Exactly one JSON object (`docs/for-agents.md`). No DL code is invented for a doctor report — the
/// code registry is a stable contract and a health check is not a language diagnostic — so
/// `diagnostics` stays empty and the findings ride in an additive `doctor` object.
fn envelope(r: &Report) -> String {
    let mut checks = String::new();
    for (i, c) in r.checks.iter().enumerate() {
        if i > 0 {
            checks.push(',');
        }
        checks.push_str(&serde_json::json!({
            "section": c.section,
            "name": c.name,
            "status": c.status.word(),
            "detail": c.detail,
        }).to_string());
    }

    let mut root = serde_json::json!({
        "command": "doctor",
        "schema": 1,
        "delulu_version": env!("CARGO_PKG_VERSION"),
        "diagnostics": [],
        "summary": { "errors": r.problems(), "warnings": 0 },
    });

    let mut doctor = serde_json::json!({
        "checks": serde_json::from_str::<serde_json::Value>(&format!("[{checks}]")).unwrap_or(serde_json::json!([])),
        "regenerated": r.regenerated,
    });
    if let Some((nodes, edges, [e, w, n])) = r.survey {
        doctor["survey"] = serde_json::json!({
            "nodes": nodes,
            "edges": edges,
            "discrepancies": { "error": e, "warning": w, "note": n },
        });
    }
    root["doctor"] = doctor;
    root.to_string()
}

#[cfg(test)]
mod posture_tests {
    use super::{mode_agreement, Status};

    /// **The trap this check exists for.** `root issuance` reads the policy FILE; a daemon serves the
    /// mode it BOOTED with. An operator who writes a strict policy and does not restart would
    /// otherwise be told `STRICT` by one line while the live broker went on minting unsigned roots —
    /// a posture check reporting configuration as though it were behaviour. Verified end to end
    /// against a real daemon before this was extracted: doctor exits 1 on the disagreement, 0 once
    /// the broker is restarted.
    #[test]
    fn a_policy_the_running_broker_never_booted_is_a_problem() {
        let (s, msg) = mode_agreement("legacy", "strict");
        assert_eq!(s, Status::Problem, "configuration that is not yet behaviour must fail the run");
        assert!(msg.contains("restart"), "and must say what to do about it: {msg}");

        // The reverse is just as wrong: the file was removed but the daemon is still enforcing.
        assert_eq!(mode_agreement("strict", "legacy").0, Status::Problem);
        // A poisoned policy the daemon never saw is a disagreement like any other.
        assert_eq!(mode_agreement("legacy", "unreadable-policy").0, Status::Problem);
    }

    /// Agreement in every mode is `ok` — a gate that flagged a correctly-configured host would be
    /// noise, and noise is how a real warning gets ignored.
    #[test]
    fn agreement_in_any_mode_is_healthy() {
        for m in ["legacy", "strict", "unreadable-policy"] {
            let (s, msg) = mode_agreement(m, m);
            assert_eq!(s, Status::Ok, "`{m}` agreeing with itself is healthy");
            assert!(msg.contains(m), "and names the mode: {msg}");
        }
    }
}
