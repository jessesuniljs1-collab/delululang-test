//! The runtime primitive table (spec §7.3): the execution half of the effect truth. Every
//! capability operation validates its scope host-side on every use (invariant 7 / §5.15.6),
//! independent of the compile-time check. Closure-taking methods (`map`) are handled in the
//! interpreter, which can call back into evaluation; everything else lives here.

use std::path::{Component, Path, PathBuf};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use delulu_check::ResourceKind;
use delulu_diag::Span;

use crate::value::{CapScope, CapVal, Fault, RootVal, SecretVal, Value};

/// A granted filesystem root, resolved to an absolute normalized path (used by the broker).
pub fn granted_root(rel: &str) -> PathBuf {
    normalize(&std::env::current_dir().unwrap_or_default().join(rel))
}

/// Resolve `rel` inside `root` with the SAME lexical `.`/`..` normalization the interpreter and the
/// broker use — no filesystem access (Stage 5 phase 5f). Public so the custody gate can present the
/// broker the exact resolved-path string its node's fs scope was granted against.
pub fn resolve_norm(root: &Path, rel: &str) -> PathBuf {
    normalize(&root.join(rel))
}

/// Lexically normalize a path, resolving `.` and `..` without touching the filesystem.
fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Canonicalize `p`, or — when it does not exist yet — its nearest existing ancestor with the
/// remaining components re-appended.
///
/// A write creates its file, and a granted directory may legitimately not exist until something
/// makes it, so refusing every path that is not already on disk would refuse ordinary programs.
/// Walking up to the deepest ancestor that *does* exist still resolves every link along the way,
/// which is the part that matters.
///
/// # Campaign finding SYMLINK-DANGLE-1 — why the walk checks `symlink_metadata`
///
/// The original walk re-appended the remaining components on the stated grounds that they "are
/// plain names the OS has not yet been asked to interpret". **That claim is false for exactly one
/// case, and it reopened C84.** A *dangling* symlink — one whose target does not exist — makes
/// `canonicalize` fail with `NotFound`, indistinguishable here from a name that is simply absent.
/// The walk therefore re-appended the link's own name as if it were a plain name, the prefix check
/// passed, and the caller then opened the path — at which point the OS *did* interpret it, followed
/// the link, and created the file wherever it pointed. Verified end-to-end: with a dangling
/// `<grant>/log.txt -> <outside>/pwned.txt`, `write_text("log.txt", …)` returned `Ok` and wrote
/// outside the grant, while the identical program against a link whose target *existed* was refused
/// `DL0904` — the same link, the same grant, opposite verdicts, decided only by whether the target
/// happened to exist.
///
/// Unlike the hardlink boundary below, this one **is workspace-deliverable**: git stores a symlink
/// as a path string (mode `120000`), so a cloned repository — or a project an agent was pointed at —
/// can carry the aimed link, which is what made C84 the urgent class.
///
/// The rule is therefore: a component that **exists as a symlink** but did not canonicalize cannot
/// be re-appended, because where it lands is precisely what could not be verified. Fail closed
/// (`None` → the caller refuses). Deliberately narrow: a component that exists and is *not* a
/// symlink is still re-appended (it is genuinely at that path — the hardlink disposition below is
/// unchanged), and a link whose target exists still resolves and is checked exactly as C84 fixed it,
/// so a link that stays inside the grant keeps working. A dangling link pointing *inside* the grant
/// is refused too: distinguishing it means resolving the link by hand, and narrowing is the safe
/// direction when the alternative is guessing.
fn canonical_existing(p: &Path) -> Option<PathBuf> {
    if let Ok(c) = std::fs::canonicalize(p) {
        return Some(c);
    }
    let mut tail: Vec<std::ffi::OsString> = Vec::new();
    let mut cur = p;
    loop {
        // SYMLINK-DANGLE-1. `cur` did not canonicalize. If it nonetheless EXISTS as a link, the OS
        // will interpret it at open time and we cannot say where it points — refuse rather than
        // hand back a path whose destination was never checked.
        if is_symlink(cur) {
            return None;
        }
        let name = cur.file_name()?;
        let parent = cur.parent()?;
        tail.push(name.to_owned());
        if let Ok(mut out) = std::fs::canonicalize(parent) {
            for n in tail.iter().rev() {
                out.push(n);
            }
            return Some(out);
        }
        cur = parent;
    }
}

/// The resolved form of `p` for a containment DECISION made outside the primitive table (the run
/// report and the trace file, PS-0-02): [`canonical_existing`], so both sides of a comparison go
/// through the one resolution the containment code trusts. `None` = cannot tell (refuse).
pub fn resolve_for_decision(p: &Path) -> Option<PathBuf> {
    canonical_existing(p)
}

/// Does `p` itself exist as a symbolic link (without following it)? `false` when `p` is absent or
/// cannot be stat'd — the callers treat "cannot tell" as "do not admit".
fn is_symlink(p: &Path) -> bool {
    std::fs::symlink_metadata(p).map(|m| m.file_type().is_symlink()).unwrap_or(false)
}

/// Is some component of `p` a symlink that does not resolve? Used only to explain a `DL0904`
/// refusal in the operator's terms — the containment decision itself is [`contains_on_disk`]'s.
fn unresolvable_link_on(p: &Path) -> bool {
    if std::fs::canonicalize(p).is_ok() {
        return false;
    }
    let mut cur = Some(p);
    while let Some(c) = cur {
        if is_symlink(c) {
            // The DEEPEST link on the path, and the only one that needs asking: `canonicalize`
            // resolves a whole chain, so if this one resolves, nothing above it dangles either.
            //
            // A link that resolves is NOT the dangling case — it was refused for the ordinary
            // reason that it lands outside the grant (C84), and saying "its target does not exist"
            // would be a false statement in a diagnostic. Caught by testing the fix against a
            // Windows junction aimed at a directory that did exist.
            return std::fs::canonicalize(c).is_err();
        }
        cur = c.parent().filter(|par| !par.as_os_str().is_empty());
    }
    false
}

/// Whether `candidate` really lives under `root` **on the filesystem**, not merely lexically.
///
/// Campaign finding C84. The lexical test pops `.` and `..` without touching the disk, so it cannot
/// see a symbolic link or a Windows directory junction *inside* the granted root. With a junction at
/// `<grant>/link` pointing at a sibling, `read_text("link/crown.txt")` returned a file outside the
/// grant and exited 0, while the lexically identical `../secret/crown.txt` was correctly refused —
/// same file, same grant, opposite verdicts, decided by a link the check never resolved.
///
/// This was never an undocumented extra: `STAGE3_SPECIFICATION.md` §4.3 has always stated, as
/// normative host-side law, "path canonicalization then prefix check (symlinks resolved host-side
/// **before** the check)". The rule was written; only the code was missing.
///
/// Both sides are canonicalized so the comparison is between two resolved paths — necessary on
/// Windows, where `canonicalize` returns a `\\?\` verbatim path that would never prefix-match a
/// path that had not been through it. **Fails closed:** if either side cannot be resolved at all,
/// the answer is `false` and the caller refuses.
pub fn contains_on_disk(root: &Path, candidate: &Path) -> bool {
    match (canonical_existing(root), canonical_existing(candidate)) {
        (Some(r), Some(c)) => c.starts_with(&r),
        _ => false,
    }
}

/// Why a path SPELLING is refused before any containment decision (D-NE-29, NE-19/NE-20), or `None`.
///
/// Containment decides on a spelling; the OS then acts on what the spelling MEANS. On Windows those
/// differ for a whole family of names, and each difference walked past a check: `NUL` wrote to the
/// null device, `CON` became a real file through the `\\?\` path the containment walk produced,
/// `trail.txt.` created `trail.txt` while the trace and audit recorded the other name, `C:foo` means
/// "the current directory of drive C:", a `:` names an alternate data stream. They are refused, not
/// normalized: a refusal cannot be walked past by a spelling nobody anticipated. On every platform an
/// embedded NUL is refused (the OS would truncate the name at it).
///
/// `allow_unc`: an operator's `--grant` may name a `\\server\share`; a program's path may not.
pub fn hostile_path(p: &str, allow_unc: bool) -> Option<String> {
    if p.contains('\0') {
        return Some("it contains an embedded NUL, at which the operating system would cut the name".into());
    }
    #[cfg(not(windows))]
    {
        let _ = allow_unc;
        None
    }
    #[cfg(windows)]
    {
        windows_hostile_path(p, allow_unc)
    }
}

#[cfg(any(windows, test))]
fn windows_hostile_path(p: &str, allow_unc: bool) -> Option<String> {
    let b = p.as_bytes();
    let sep = |c: u8| c == b'\\' || c == b'/';
    if b.len() >= 4 && sep(b[0]) && sep(b[1]) && (b[2] == b'?' || b[2] == b'.') && sep(b[3]) {
        return Some("a `\\\\?\\` or `\\\\.\\` prefix bypasses Windows path parsing and names devices".into());
    }
    if b.len() >= 2 && sep(b[0]) && sep(b[1]) && !allow_unc {
        return Some("a UNC path (`\\\\server\\share`) reaches another machine".into());
    }
    let drive = b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':';
    if drive && (b.len() == 2 || !sep(b[2])) {
        return Some(format!(
            "`{}` is drive-relative — it means the current directory of drive {}:, whatever that is",
            p,
            (b[0] as char).to_ascii_uppercase()
        ));
    }
    let body = if drive { &p[2..] } else { p };
    for comp in body.split(['\\', '/']) {
        if comp.is_empty() || comp == "." || comp == ".." {
            continue;
        }
        if comp.contains(':') {
            return Some(format!("`{comp}` names an alternate data stream (`:`)"));
        }
        if comp.ends_with('.') || comp.ends_with(' ') {
            return Some(format!(
                "`{comp}` ends in a dot or a space, which Windows strips — the file written would not be the one named"
            ));
        }
        let stem = comp.split('.').next().unwrap_or(comp).trim_end_matches(' ').to_ascii_uppercase();
        let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
            || ["COM", "LPT"].iter().any(|dev| {
                stem.strip_prefix(dev).is_some_and(|n| {
                    matches!(n, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "\u{b9}" | "\u{b2}" | "\u{b3}")
                })
            });
        if reserved {
            return Some(format!("`{comp}` is a Windows reserved device name"));
        }
    }
    None
}

/// Resolve `rel` inside a capability's filesystem scope, refusing escapes (DL0904).
///
/// Two gates, and the second is the one a link cannot walk past: the lexical check rejects `..`,
/// then [`contains_on_disk`] rejects anything that *resolves* outside the root (C84).
fn resolve_in_scope(root: &Path, rel: &str, span: Span) -> Result<PathBuf, Fault> {
    if let Some(why) = hostile_path(rel, false) {
        return Err(Fault::at("DL0904", format!("path `{}` is refused: {why}", rel.escape_debug()), span));
    }
    let joined = normalize(&root.join(rel));
    if joined.starts_with(root) && contains_on_disk(root, &joined) {
        Ok(joined)
    } else {
        // SYMLINK-DANGLE-1: name the cause when it is a link that does not resolve, so the refusal
        // reads as the deliberate decision it is rather than a puzzling scope error on a path that
        // "looks" inside the grant.
        let why = if unresolvable_link_on(&joined) {
            " — a component is a symbolic link whose target does not exist, so where a write would \
             land cannot be verified; refused rather than guessed"
        } else {
            ""
        };
        Err(Fault::at("DL0904", format!("path `{rel}` escapes the granted scope{why}"), span))
    }
}

/// The host component of an `https://` URL, per RFC 3986's authority grammar.
///
/// Campaign finding C86. The old extraction split the authority on `/` or `:` and took the first
/// field, so it never looked for `@` — and in RFC 3986 everything before the last `@` is *userinfo*,
/// not the host. `https://example.com:8080@evil.com/steal` therefore yielded `example.com`, so a
/// grant of `example.com` authorized a request whose real destination was `evil.com`, and the
/// hash-chained audit record attested the wrong host. The bytes never left (v1.x ships no HTTP
/// client), but the *decision* and the *record* were both wrong, which is the accountability the
/// system sells.
///
/// Public so the custody gate cannot drift from it: `interp.rs` used to re-implement this parse so
/// that "the broker's exact-set `net` check sees the same host string", which meant one bug in two
/// places (design rule 1 — the answer is one function referenced by both sides, not two lists kept
/// in step by hand).
pub fn host_of(url: &str) -> &str {
    let rest = url.strip_prefix("https://").unwrap_or(url);
    // The authority component ends at the first `/`, `?` or `#`.
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    // Userinfo is everything up to the LAST `@`; the host follows it.
    let hostport = match authority.rfind('@') {
        Some(i) => &authority[i + 1..],
        None => authority,
    };
    // An IPv6 literal is bracketed, and its colons are not port separators.
    if let Some(end) = hostport.strip_prefix('[').and_then(|r| r.find(']')) {
        return &hostport[..end + 2];
    }
    hostport.split(':').next().unwrap_or(hostport)
}

fn host_allowed(url: &str, allow: &[String]) -> bool {
    let host = host_of(url);
    allow.iter().any(|pat| host_matches(host, pat))
}

/// Does `host` fall under the allowlist pattern `pat` — exact, or `*.suffix` on a dot boundary?
///
/// Public because the egress client (PS-B-02) asks the same question of every redirect hop, and of
/// the `net.special=` subset, and two matchers kept in step by hand is how C85 happened.
pub fn host_matches(host: &str, pat: &str) -> bool {
    if let Some(suffix) = pat.strip_prefix("*.") {
        // Campaign finding C85 — the dot boundary is the whole point. A bare `ends_with` let
        // `*.example.com` match `evilexample.com`, which is a different registrable domain
        // owned by somebody else. The project's sibling matcher for Python imports
        // (`python::allowlist_allows`) already requires the separator, which is what makes this
        // an omission rather than a design choice: two namespace matchers, one correct.
        //
        // The apex (`example.com` itself) is deliberately NOT matched by `*.example.com`; that
        // is the ordinary reading of the pattern, and narrowing is the safe direction.
        host.len() > suffix.len()
            && host.ends_with(suffix)
            && host.as_bytes()[host.len() - suffix.len() - 1] == b'.'
    } else {
        host == pat
    }
}

// ----- Root: mint capabilities (attenuation — no effect) -------------------

pub fn call_root_method(root: &RootVal, method: &str, args: &[Value], span: Span) -> Result<Value, Fault> {
    // DL0703 is the first error nearly every newcomer meets: zero ambient authority means the
    // very first program that prints anything fails until a human grants the console. The
    // refusal is correct and is the point of the language — but a refusal that does not say
    // what to type teaches nothing, so every message below names the exact grant. (Prose only:
    // the code, the span, and the `--json` envelope are unchanged, per `for-agents.md`.)
    let refused = |what: &str, grant: &str| {
        Fault::at("DL0703", format!("`{what}` was not granted to this program — pass `--grant {grant}`"), span)
    };
    match method {
        "console" => {
            if root.console {
                Ok(cap(ResourceKind::Console, CapScope::Console))
            } else {
                Err(refused("console", "console"))
            }
        }
        "fs_read" => {
            let p = str_arg(args, 0, span)?;
            if let Some(why) = hostile_path(&p, false) {
                return Err(Fault::at("DL0904", format!("path `{}` is refused: {why}", p.escape_debug()), span));
            }
            let want = normalize(&std::env::current_dir().unwrap_or_default().join(&p));
            // Both gates, for the same reason as `resolve_in_scope`: minting the capability
            // *rooted at* a junction would otherwise put every later read lexically "inside" a
            // scope that resolves somewhere else entirely (C84, the second door).
            match root
                .fs_read
                .iter()
                .find(|g| want.starts_with(g.as_path()) && contains_on_disk(g, &want))
            {
                Some(_) => Ok(cap(ResourceKind::FsRead, CapScope::Fs { root: want, write: false })),
                None => Err(Fault::at("DL0703", format!("filesystem read of `{p}` was not granted — pass `--grant fs.read={p}`"), span)),
            }
        }
        "fs_write" => {
            let p = str_arg(args, 0, span)?;
            if let Some(why) = hostile_path(&p, false) {
                return Err(Fault::at("DL0904", format!("path `{}` is refused: {why}", p.escape_debug()), span));
            }
            let want = normalize(&std::env::current_dir().unwrap_or_default().join(&p));
            match root
                .fs_write
                .iter()
                .find(|g| want.starts_with(g.as_path()) && contains_on_disk(g, &want))
            {
                Some(_) => Ok(cap(ResourceKind::FsWrite, CapScope::Fs { root: want, write: true })),
                None => Err(Fault::at("DL0703", format!("filesystem write of `{p}` was not granted — pass `--grant fs.write={p}`"), span)),
            }
        }
        "http" => {
            let hosts = list_str_arg(args, 0, span)?;
            if hosts.iter().all(|h| root.net.iter().any(|g| g == h)) {
                // PS-B-02: the capability carries which of its hosts the operator granted with
                // `net.special=` — the same exact-string match the minting check above makes, so a
                // program cannot name its way into special-use authority it was not given.
                let special = hosts.iter().filter(|h| root.net_special.iter().any(|g| g == *h)).cloned().collect();
                Ok(cap(ResourceKind::Http, CapScope::Net { allow: hosts, special }))
            } else {
                Err(refused("network host", "net=HOST"))
            }
        }
        "clock" => {
            if root.clock {
                Ok(cap(ResourceKind::Clock, CapScope::Clock))
            } else {
                Err(refused("clock", "clock"))
            }
        }
        "rand" => {
            if root.rand {
                Ok(cap(ResourceKind::Rand, CapScope::Rand))
            } else {
                Err(refused("rand", "rand"))
            }
        }
        "declassify" => {
            if root.declassify {
                let names = root.secrets.keys().cloned().collect();
                Ok(cap(ResourceKind::Declassify, CapScope::Declassify { names }))
            } else {
                Err(refused("declassify", "declassify"))
            }
        }
        "secret" => {
            let name = str_arg(args, 0, span)?;
            // Daemon mode (Stage 5 phase 5g): a broker-held secret returns an opaque HANDLE — no
            // bytes cross into this process here (invariant 23); `expose` fetches them later.
            if root.broker_secrets.iter().any(|n| n == &name) {
                return Ok(Value::Secret(Rc::new(SecretVal::handle(name))));
            }
            match root.secrets.get(&name) {
                Some(v) => Ok(Value::Secret(Rc::new(SecretVal::new(v.clone())))),
                None => Err(Fault::at("DL0703", format!("secret `{name}` was not granted — pass `--grant secret:{name}=VALUE` (or `secret:{name}=env:VAR`)"), span)),
            }
        }
        "foreign_load" => {
            if root.foreign_load {
                Ok(cap(ResourceKind::ForeignLoad, CapScope::ForeignLoad))
            } else {
                Err(Fault::at("DL0703", "foreign loading was not granted (grant a `foreign.c` lib or `foreign.python`)", span))
            }
        }
        // T-Py binding (spec §5.1): `root.python(load: Cap[ForeignLoad]) -> Result[Cap[Python],
        // ForeignErr]`, PURE like `root.foreign` — deriving the handle is not an effect; *using* it
        // is. An ungranted `foreign.python` is `Err(NotGranted)` (DL1303's runtime face, behind the
        // CLI startup refusal); an interpreter that will not start is `Err(Unavailable)` (DL1307).
        // The interpreter is prepared lazily HERE, on this first grant-checked call.
        "python" => {
            if root.python_allowlist.is_empty() {
                return Ok(Value::err(Value::variant("NotGranted", vec![])));
            }
            match crate::python::ensure_available() {
                Ok(()) => Ok(Value::ok(cap(
                    ResourceKind::Python,
                    CapScope::Python { allowlist: root.python_allowlist.clone() },
                ))),
                Err(reason) => Ok(Value::err(Value::variant("Unavailable", vec![Value::str(reason)]))),
            }
        }
        // Stage 10 (10e): physical-device mints. Both are pure attenuation (deriving the handle is
        // not an effect; *using* it is), and both are deny-by-default: no matching grant, no cap —
        // the envelope/device list on `RootVal` is the whole authority story (spec §5.1).
        "actuator" => {
            let d = str_arg(args, 0, span)?;
            match root.actuators.iter().find(|e| e.device == d) {
                Some(e) => Ok(cap(ResourceKind::Actuator, CapScope::Actuator(e.clone()))),
                None => Err(Fault::at("DL0703", format!("actuator `{d}` was not granted — pass `--grant \"actuator={d}:DIM=LO..HI\"` with the envelope this machine may move in"), span)),
            }
        }
        "sensor" => {
            let d = str_arg(args, 0, span)?;
            if root.sensors.iter().any(|s| s == &d) {
                Ok(cap(ResourceKind::Sensor, CapScope::Sensor { device: d }))
            } else {
                Err(Fault::at("DL0703", format!("sensor `{d}` was not granted — pass `--grant sensor={d}`"), span))
            }
        }
        // Stage 10 (10h): the same shape for an accelerator. Deriving the handle is pure; the
        // `ForeignCall` effect is in dispatching through it.
        "compute" => {
            let d = str_arg(args, 0, span)?;
            match root.computes.iter().find(|e| e.device == d) {
                Some(e) => Ok(cap(ResourceKind::Compute, CapScope::Compute(e.clone()))),
                None => Err(Fault::at("DL0703", format!("compute device `{d}` was not granted — pass `--grant \"compute={d}:memory_bytes=N,...\"`"), span)),
            }
        }
        // P2 (D-V2-27): `root.plugin_host()` mints the capability that gates loading. Before P2 this
        // arm refused unconditionally with "not available in the Stage-1 runtime" — true then, and the
        // whole of NE-01.
        "plugin_host" => {
            if root.plugins.is_empty() {
                Err(refused("plugin loading", "plugin=PATH"))
            } else {
                Ok(cap(
                    ResourceKind::PluginHost,
                    CapScope::PluginHost {
                        roots: root.plugins.clone(),
                        allow_hashes: root.plugins_allow.clone(),
                    },
                ))
            }
        }
        _ => Err(Fault::at("DL0907", format!("unknown Root method `{method}` (checker bug)"), span)),
    }
}

fn cap(kind: ResourceKind, scope: CapScope) -> Value {
    Value::Cap(Rc::new(CapVal { kind, scope }))
}

// ----- capability operations (the effects themselves) ----------------------

pub fn call_cap_method(capv: &CapVal, method: &str, args: &[Value], span: Span) -> Result<Value, Fault> {
    // PS-A-02: a host-held capability carries no path, host or socket — only the number the host
    // minted. This process cannot perform it, and must not pretend to: a handle reaching the local
    // path means the guest was wired to the wrong sink, which is a failure, not a quiet no-op.
    if let CapScope::Handle(h) = capv.scope {
        return Err(Fault::at(
            "DL1401",
            format!("capability handle {h} belongs to the host: this process cannot perform it"),
            span,
        ));
    }
    match (capv.kind, method) {
        (ResourceKind::Console, "println") => {
            emit_console(&str_arg(args, 0, span)?, true);
            Ok(Value::Unit)
        }
        (ResourceKind::Console, "print") => {
            emit_console(&str_arg(args, 0, span)?, false);
            Ok(Value::Unit)
        }
        (ResourceKind::Console, "readline") => {
            let mut line = String::new();
            match std::io::stdin().read_line(&mut line) {
                Ok(0) => Ok(Value::err(io_err("NotFound"))),
                Ok(_) => Ok(Value::ok(Value::str(line.trim_end_matches(['\n', '\r']).to_string()))),
                Err(_) => Ok(Value::err(io_err("Other"))),
            }
        }
        (ResourceKind::FsRead, "read_text") => {
            let CapScope::Fs { root, .. } = &capv.scope else { return Err(scope_bug(span)) };
            let p = resolve_in_scope(root, &str_arg(args, 0, span)?, span)?;
            match std::fs::read_to_string(&p) {
                Ok(s) => Ok(Value::ok(Value::str(s))),
                Err(e) => Ok(Value::err(io_err_for(&e))),
            }
        }
        (ResourceKind::FsRead, "list_dir") => {
            let CapScope::Fs { root, .. } = &capv.scope else { return Err(scope_bug(span)) };
            let p = resolve_in_scope(root, &str_arg(args, 0, span)?, span)?;
            match std::fs::read_dir(&p) {
                Ok(rd) => {
                    let names: Vec<Value> = rd
                        .filter_map(|e| e.ok())
                        .map(|e| Value::str(e.file_name().to_string_lossy().to_string()))
                        .collect();
                    Ok(Value::ok(Value::List(Rc::new(std::cell::RefCell::new(names)))))
                }
                Err(e) => Ok(Value::err(io_err_for(&e))),
            }
        }
        (ResourceKind::FsRead, "narrow") => {
            let CapScope::Fs { root, write } = &capv.scope else { return Err(scope_bug(span)) };
            let sub = resolve_in_scope(root, &str_arg(args, 0, span)?, span)?;
            Ok(cap(ResourceKind::FsRead, CapScope::Fs { root: sub, write: *write }))
        }
        (ResourceKind::FsWrite, "write_text") | (ResourceKind::FsWrite, "append_text") => {
            let CapScope::Fs { root, .. } = &capv.scope else { return Err(scope_bug(span)) };
            let p = resolve_in_scope(root, &str_arg(args, 0, span)?, span)?;
            let body = str_arg(args, 1, span)?;
            let res = if method == "append_text" {
                use std::io::Write as _;
                std::fs::OpenOptions::new().create(true).append(true).open(&p).and_then(|mut f| f.write_all(body.as_bytes()))
            } else {
                std::fs::write(&p, body.as_bytes())
            };
            match res {
                Ok(()) => Ok(Value::ok(Value::Unit)),
                Err(e) => Ok(Value::err(io_err_for(&e))),
            }
        }
        (ResourceKind::Http, "get") => {
            let CapScope::Net { allow, special } = &capv.scope else { return Err(scope_bug(span)) };
            let url = str_arg(args, 0, span)?;
            if !url.starts_with("https://") {
                crate::egress::note_refusal(&url, crate::egress::Reason::Scheme);
                return Ok(Value::err(net_err("Refused")));
            }
            if !host_allowed(&url, allow) {
                return Err(Fault::at("DL0904", format!("host of `{url}` is not in the granted allowlist"), span));
            }
            // PS-B-02 (NE-17 closed): the egress client. It re-checks all of the above on its own
            // strict parse, resolves once, refuses special-use addresses unless `net.special=` named
            // the host, pins, and follows redirects through the same check. For a sandboxed guest this
            // line runs in the HOST — `HostChannel::decide` performs the request here — so one
            // implementation serves L0 and every guest.
            Ok(match crate::egress::get_for_program(&url, allow, special) {
                crate::egress::Answer::Body(b) => Value::ok(Value::str(b)),
                crate::egress::Answer::Refused => Value::err(net_err("Refused")),
                crate::egress::Answer::Timeout => Value::err(net_err("Timeout")),
                crate::egress::Answer::Other(m) => Value::err(Value::variant("Other", vec![Value::str(m)])),
            })
        }
        (ResourceKind::Clock, "now_ms") => {
            let ms = FIXED_CLOCK.with(|c| c.get()).unwrap_or_else(|| {
                SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
            });
            Ok(Value::Int(ms))
        }
        (ResourceKind::Rand, "int") => {
            let lo = int_arg(args, 0, span)?;
            let hi = int_arg(args, 1, span)?;
            if hi <= lo {
                return Err(Fault::at("DL0904", "rand.int requires lo < hi", span));
            }
            Ok(Value::Int(lo + (next_rand() % (hi - lo) as u64) as i64))
        }
        (ResourceKind::Rand, "float") => Ok(Value::Float((next_rand() as f64) / (u64::MAX as f64))),
        // Stage 10 (10e): the null sensor adapter — a read that cannot observe anything says so
        // honestly: `Err(NoDevice)`, never a fabricated measurement (invariant 50). 10f gave the
        // interpreter its own `Sensor` arm so a bound adapter can answer, and that arm returns
        // exactly this when no adapter is attached. This one stays as the floor for any caller
        // that reaches `call_cap_method` without a broker: the fallback for "no adapter" must be
        // absence in every path that can produce a reading, not just the one we remembered.
        (ResourceKind::Sensor, "read") => Ok(Value::err(Value::variant("NoDevice", vec![]))),
        _ => Err(Fault::at("DL0907", format!("unknown capability method `{method}` (checker bug)"), span)),
    }
}

pub fn call_secret_method(secret: &SecretVal, method: &str, args: &[Value], span: Span) -> Result<Value, Fault> {
    match method {
        "verify" => match args.first() {
            Some(Value::Secret(other)) => Ok(Value::Bool(secret.verify(other))),
            _ => Err(Fault::at("DL0907", "Secret.verify expects a Secret argument", span)),
        },
        // The only reader: expose. Reaching here means the checker admitted Cap[Declassify].
        "expose" => Ok(Value::str(secret.reveal())),
        _ => Err(Fault::at("DL0907", format!("unknown Secret method `{method}` (checker bug)"), span)),
    }
}

pub fn call_str_method(s: &str, method: &str, args: &[Value], span: Span) -> Result<Value, Fault> {
    match method {
        "len" => Ok(Value::Int(s.chars().count() as i64)),
        "trim" => Ok(Value::str(s.trim().to_string())),
        "contains" => Ok(Value::Bool(s.contains(&str_arg(args, 0, span)?))),
        "starts_with" => Ok(Value::Bool(s.starts_with(&str_arg(args, 0, span)?))),
        "split" => {
            let sep = str_arg(args, 0, span)?;
            let parts: Vec<Value> = if sep.is_empty() {
                s.chars().map(|c| Value::str(c.to_string())).collect()
            } else {
                s.split(&sep).map(|p| Value::str(p.to_string())).collect()
            };
            Ok(Value::List(Rc::new(std::cell::RefCell::new(parts))))
        }
        "slice" => {
            let lo = int_arg(args, 0, span)?.max(0) as usize;
            let hi = int_arg(args, 1, span)?.max(0) as usize;
            let chars: Vec<char> = s.chars().collect();
            let lo = lo.min(chars.len());
            let hi = hi.min(chars.len()).max(lo);
            Ok(Value::str(chars[lo..hi].iter().collect::<String>()))
        }
        // P3 (D-V2-29). Full Unicode, locale-independent — and NOT a normalization a security
        // decision may rest on: these do not round-trip (German sharp s uppercases to two letters),
        // and four of the P22 campaign's defects were comparisons made on a different spelling of
        // the same string. The reference says so where a reader will see it.
        "to_upper" => Ok(Value::str(s.to_uppercase())),
        "to_lower" => Ok(Value::str(s.to_lowercase())),
        // An EMPTY pattern returns the receiver unchanged (D-V2-29 decision 6). Rust's `replace`
        // would insert `to` between every character, which is a surprise rather than a semantic;
        // unlike `split("")`, an empty replacement pattern has no natural reading.
        "replace" => {
            let from = str_arg(args, 0, span)?;
            let to = str_arg(args, 1, span)?;
            Ok(Value::str(if from.is_empty() { s.to_string() } else { s.replace(&from, &to) }))
        }
        // DEFINED as `split("")`, and `chars_agrees_with_split_on_the_empty_separator` pins it, so
        // the second spelling of one operation cannot drift from the first.
        "chars" => call_str_method(s, "split", &[Value::str(String::new())], span),
        _ => Err(Fault::at("DL0907", format!("unknown Str method `{method}`"), span)),
    }
}

/// The ordered projection `List.sort` compares by. Deliberately NOT an `Ord` on `Value`: that would
/// define an order for every variant at once, including the opaque handles and `Float`, which is the
/// blanket answer D-V2-29 refused. `Bool` sorts false-before-true.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum SortKey {
    Int(i64),
    Str(String),
}

pub fn call_list_method(list: &Rc<std::cell::RefCell<Vec<Value>>>, method: &str, args: &[Value], span: Span) -> Result<Value, Fault> {
    match method {
        "len" => Ok(Value::Int(list.borrow().len() as i64)),
        "get" => {
            let i = int_arg(args, 0, span)?;
            match list.borrow().get(i as usize) {
                Some(v) => Ok(Value::variant("Some", vec![v.clone()])),
                None => Ok(Value::variant("None", vec![])),
            }
        }
        "push" => {
            if let Some(v) = args.first() {
                list.borrow_mut().push(v.clone());
            }
            Ok(Value::Unit)
        }
        // P3 (D-V2-29). `pop` MUTATES, like `push`, and is registered in `MUTATING_METHODS` so the
        // rcap pass refuses it through a `val` reference.
        "pop" => Ok(match list.borrow_mut().pop() {
            Some(v) => Value::variant("Some", vec![v]),
            None => Value::variant("None", vec![]),
        }),
        "is_empty" => Ok(Value::Bool(list.borrow().is_empty())),
        "reverse" => {
            let mut out = list.borrow().clone();
            out.reverse();
            Ok(Value::List(Rc::new(std::cell::RefCell::new(out))))
        }
        "concat" => {
            let mut out = list.borrow().clone();
            match args.first() {
                Some(Value::List(other)) => out.extend(other.borrow().iter().cloned()),
                _ => return Err(Fault::at("DL0907", "List.concat expects a List argument", span)),
            }
            Ok(Value::List(Rc::new(std::cell::RefCell::new(out))))
        }
        // The same clamping `Str.slice` uses, so the two agree: out-of-range is an empty slice, not
        // a fault, and `hi < lo` is empty rather than reversed.
        "slice" => {
            let lo = int_arg(args, 0, span)?.max(0) as usize;
            let hi = int_arg(args, 1, span)?.max(0) as usize;
            let items = list.borrow();
            let lo = lo.min(items.len());
            let hi = hi.min(items.len()).max(lo);
            Ok(Value::List(Rc::new(std::cell::RefCell::new(items[lo..hi].to_vec()))))
        }
        // `Value::eq`, the same comparison `==` performs. The checker has already refused an opaque
        // element type with DL0605, which is what keeps `contains` and `==` from disagreeing.
        "contains" => {
            let needle = args.first().ok_or_else(|| Fault::at("DL0907", "List.contains expects an argument", span))?;
            Ok(Value::Bool(list.borrow().iter().any(|v| v.eq(needle))))
        }
        "join" => {
            let sep = str_arg(args, 0, span)?;
            let items = list.borrow();
            let mut parts: Vec<String> = Vec::with_capacity(items.len());
            for v in items.iter() {
                match v {
                    Value::Str(s) => parts.push(s.to_string()),
                    other => {
                        return Err(Fault::at(
                            "DL0907",
                            format!("List.join is defined on List[Str] (found `{}`) — checker bug", other.display()),
                            span,
                        ))
                    }
                }
            }
            Ok(Value::str(parts.join(&sep)))
        }
        // STABLE, and only over the three element types that have a total order — the checker has
        // refused the rest, `Float` by name (NaN). A stable sort means equal elements keep their
        // input order, so the answer is reproducible across runs and platforms; an unstable one
        // would make two green runs disagree for no visible reason.
        "sort" => {
            let mut out = list.borrow().clone();
            let key = |v: &Value| -> Option<SortKey> {
                match v {
                    Value::Int(i) => Some(SortKey::Int(*i)),
                    Value::Bool(b) => Some(SortKey::Int(i64::from(*b))),
                    Value::Str(s) => Some(SortKey::Str(s.to_string())),
                    _ => None,
                }
            };
            if let Some(bad) = out.iter().find(|v| key(v).is_none()) {
                return Err(Fault::at(
                    "DL0907",
                    format!("List.sort has no ordering for `{}` (checker bug)", bad.display()),
                    span,
                ));
            }
            // `sort_by_key` is Rust's STABLE sort, which is the property D-V2-29 promises: equal
            // elements keep their input order, so the answer is reproducible across runs.
            out.sort_by_key(&key);
            Ok(Value::List(Rc::new(std::cell::RefCell::new(out))))
        }
        _ => Err(Fault::at("DL0907", format!("unknown List method `{method}`"), span)),
    }
}

/// `Map[K, V]` (P3, D-V2-29). Every key arrives through [`MapKey::of`]; a `None` from it means the
/// checker admitted a key type it should have refused, which is why that path is a named checker-bug
/// fault rather than a silent skip. Iteration is the `BTreeMap`'s, which is ascending by key.
pub fn call_map_method(
    map: &Rc<std::cell::RefCell<std::collections::BTreeMap<crate::value::MapKey, Value>>>,
    method: &str,
    args: &[Value],
    span: Span,
) -> Result<Value, Fault> {
    use crate::value::MapKey;
    // The key argument, for the four methods that take one.
    let key = |args: &[Value]| -> Result<MapKey, Fault> {
        let v = args.first().ok_or_else(|| Fault::at("DL0907", format!("Map.{method} expects a key"), span))?;
        MapKey::of(v).ok_or_else(|| {
            Fault::at(
                "DL0907",
                format!("`{}` cannot be a Map key — only Str, Int and Bool can (checker bug)", v.display()),
                span,
            )
        })
    };
    match method {
        "len" => Ok(Value::Int(map.borrow().len() as i64)),
        "is_empty" => Ok(Value::Bool(map.borrow().is_empty())),
        "get" => Ok(match map.borrow().get(&key(args)?) {
            Some(v) => Value::variant("Some", vec![v.clone()]),
            None => Value::variant("None", vec![]),
        }),
        "contains_key" => Ok(Value::Bool(map.borrow().contains_key(&key(args)?))),
        // MUTATING — registered in `MUTATING_MAP_METHODS`, so a `val` receiver refuses it.
        "insert" => {
            let k = key(args)?;
            let v = args.get(1).cloned().unwrap_or(Value::Unit);
            map.borrow_mut().insert(k, v);
            Ok(Value::Unit)
        }
        "remove" => Ok(match map.borrow_mut().remove(&key(args)?) {
            Some(v) => Value::variant("Some", vec![v]),
            None => Value::variant("None", vec![]),
        }),
        // `keys` and `values` iterate the SAME map in the SAME order, so a caller may zip them. That
        // is a promise the reference makes, and `keys_and_values_are_in_the_same_order` pins it.
        "keys" => Ok(Value::List(Rc::new(std::cell::RefCell::new(
            map.borrow().keys().map(MapKey::to_value).collect(),
        )))),
        "values" => Ok(Value::List(Rc::new(std::cell::RefCell::new(map.borrow().values().cloned().collect())))),
        _ => Err(Fault::at("DL0907", format!("unknown Map method `{method}`"), span)),
    }
}

/// Free builtins (§11). Returns None when `name` is not a builtin.
pub fn call_builtin(name: &str, args: &[Value], span: Span) -> Option<Result<Value, Fault>> {
    Some(match name {
        // `Map()` — the empty map. A free builtin for the same reason `Some`/`None` are.
        "Map" => Ok(Value::Map(Rc::new(std::cell::RefCell::new(std::collections::BTreeMap::new())))),
        "Ok" => Ok(Value::ok(args.first().cloned().unwrap_or(Value::Unit))),
        "Err" => Ok(Value::err(args.first().cloned().unwrap_or(Value::Unit))),
        "Some" => Ok(Value::variant("Some", vec![args.first().cloned().unwrap_or(Value::Unit)])),
        "None" => Ok(Value::variant("None", vec![])),
        "str" => Ok(Value::str(args.first().map(|v| v.display()).unwrap_or_default())),
        "len" => Ok(match args.first() {
            Some(Value::Str(s)) => Value::Int(s.chars().count() as i64),
            Some(Value::List(l)) => Value::Int(l.borrow().len() as i64),
            _ => Value::Int(0),
        }),
        "int" => Ok(match args.first() {
            Some(Value::Float(f)) => Value::Int(*f as i64),
            Some(Value::Int(i)) => Value::Int(*i),
            _ => Value::Int(0),
        }),
        "float" => Ok(match args.first() {
            Some(Value::Int(i)) => Value::Float(*i as f64),
            Some(Value::Float(f)) => Value::Float(*f),
            _ => Value::Float(0.0),
        }),
        "parse_int" => Ok(match args.first() {
            Some(Value::Str(s)) => match s.trim().parse::<i64>() {
                Ok(i) => Value::variant("Some", vec![Value::Int(i)]),
                Err(_) => Value::variant("None", vec![]),
            },
            _ => Value::variant("None", vec![]),
        }),
        // The float half of `parse_int`, and deliberately the SAME rule the lexer applies to a
        // literal (`delulu_syntax::num`) rather than a second one: text that names a magnitude
        // `Float` cannot hold is `None`, not a silent `inf` or `0.0`. That also closes the way in
        // for the non-finite words — without it, a data file containing `inf` could put infinity
        // into a program whose source is not allowed to write it.
        "parse_float" => Ok(match args.first() {
            Some(Value::Str(s)) => match delulu_syntax::num::float_from_text(s.trim()) {
                delulu_syntax::num::FloatText::Value(f) => {
                    Value::variant("Some", vec![Value::Float(f)])
                }
                _ => Value::variant("None", vec![]),
            },
            _ => Value::variant("None", vec![]),
        }),
        "range" => {
            let lo = args.first().and_then(as_int).unwrap_or(0);
            let hi = args.get(1).and_then(as_int).unwrap_or(0);
            let items: Vec<Value> = (lo..hi).map(Value::Int).collect();
            Ok(Value::List(Rc::new(std::cell::RefCell::new(items))))
        }
        "push" => {
            if let (Some(Value::List(l)), Some(v)) = (args.first(), args.get(1)) {
                l.borrow_mut().push(v.clone());
                Ok(Value::Unit)
            } else {
                Err(Fault::at("DL0907", "push expects (List, value)", span))
            }
        }
        // Stage 8 (phase 8a, spec §2): assertion failure is a PANIC — the fault message
        // carries the compared values, the span carries file/line (the 8g runner lifts both
        // into the structured JSON failure payload). Only non-opaque values reach here: the
        // checker refuses `assert_eq` on opaque types (DL0605, R-5) before anything runs.
        "assert" => match args.first() {
            Some(Value::Bool(true)) => Ok(Value::Unit),
            Some(Value::Bool(false)) => Err(Fault::at("DL1707", "assertion failed", span)),
            _ => Err(Fault::at("DL0907", "assert expects a Bool condition", span)),
        },
        "assert_eq" => match (args.first(), args.get(1)) {
            (Some(a), Some(b)) if a.eq(b) => Ok(Value::Unit),
            (Some(a), Some(b)) => Err(Fault::at(
                "DL1707",
                format!("assertion failed: `{}` != `{}`", a.display(), b.display()),
                span,
            )),
            _ => Err(Fault::at("DL0907", "assert_eq expects two values", span)),
        },
        _ => return None,
    })
}

// ----- argument helpers ----------------------------------------------------

fn str_arg(args: &[Value], i: usize, span: Span) -> Result<String, Fault> {
    match args.get(i) {
        Some(Value::Str(s)) => Ok(s.to_string()),
        _ => Err(Fault::at("DL0907", format!("expected a Str argument at position {i}"), span)),
    }
}
fn int_arg(args: &[Value], i: usize, span: Span) -> Result<i64, Fault> {
    match args.get(i) {
        Some(Value::Int(n)) => Ok(*n),
        _ => Err(Fault::at("DL0907", format!("expected an Int argument at position {i}"), span)),
    }
}
fn list_str_arg(args: &[Value], i: usize, span: Span) -> Result<Vec<String>, Fault> {
    match args.get(i) {
        Some(Value::List(l)) => Ok(l.borrow().iter().map(|v| v.display()).collect()),
        _ => Err(Fault::at("DL0907", "expected a List[Str] argument", span)),
    }
}
fn as_int(v: &Value) -> Option<i64> {
    match v {
        Value::Int(n) => Some(*n),
        _ => None,
    }
}

fn io_err(variant: &str) -> Value {
    Value::variant(variant, if variant == "Other" { vec![Value::str("error")] } else { vec![] })
}
fn net_err(variant: &str) -> Value {
    Value::variant(variant, vec![])
}
fn io_err_for(e: &std::io::Error) -> Value {
    use std::io::ErrorKind::*;
    match e.kind() {
        NotFound => io_err("NotFound"),
        PermissionDenied => io_err("Denied"),
        _ => Value::variant("Other", vec![Value::str(e.to_string())]),
    }
}
fn scope_bug(span: Span) -> Fault {
    Fault::at("DL0907", "capability scope mismatch (checker bug)", span)
}

// A small, non-cryptographic xorshift PRNG for `Cap[Rand]`. Deterministic replay (spec §6.2):
// `set_rand_seed` reseeds this thread-local generator so two runs with the same seed produce
// identical `Cap[Rand]` sequences (required by the fuzz harness and agent debugging loops).
use std::cell::Cell;
thread_local! {
    static RNG: Cell<u64> = Cell::new(seed());
    static FIXED_CLOCK: Cell<Option<i64>> = const { Cell::new(None) };
    /// When `Some`, `Cap[Console]` output is captured into this buffer instead of stdout — used by
    /// tests and by the WASM backend's two-engine parity harness (§9). Off by default.
    static CAPTURE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Turn Console output capture on (fresh empty buffer) or off (restore stdout).
pub fn set_capture(on: bool) {
    CAPTURE.with(|c| *c.borrow_mut() = if on { Some(String::new()) } else { None });
}

/// Take and clear the captured Console output, if capture is on.
pub fn take_capture() -> Option<String> {
    CAPTURE.with(|c| c.borrow_mut().take())
}

/// Emit a Console string: to the capture buffer if capturing, else to real stdout.
fn emit_console(s: &str, newline: bool) {
    let captured = CAPTURE.with(|c| {
        let mut b = c.borrow_mut();
        if let Some(buf) = b.as_mut() {
            buf.push_str(s);
            if newline {
                buf.push('\n');
            }
            true
        } else {
            false
        }
    });
    if !captured {
        if newline {
            println!("{s}");
        } else {
            print!("{s}");
        }
    }
}
fn seed() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0x9E3779B9) | 1
}
fn next_rand() -> u64 {
    RNG.with(|r| {
        let mut x = r.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        r.set(x);
        x
    })
}

/// Deterministic replay (spec §6.2): reseed `Cap[Rand]`'s thread-local xorshift generator. Two
/// runs on the same thread that call `set_rand_seed` with the same value produce identical
/// `rand.int`/`rand.float` sequences; different seeds (almost always) diverge. The xorshift
/// generator gets stuck at zero, so a zero seed is mapped to a fixed nonzero value, matching the
/// `| 1` bias `seed()` already applies for the non-deterministic default.
pub fn set_rand_seed(seed: u64) {
    let s = if seed == 0 { 0x9E3779B9 } else { seed } | 1;
    RNG.with(|r| r.set(s));
}

/// Deterministic replay (spec §6.2): fix `Cap[Clock].now_ms()` to always return `ms` on this
/// thread. `None` restores the real wall clock.
pub fn set_fixed_clock_ms(ms: Option<i64>) {
    FIXED_CLOCK.with(|c| c.set(ms));
}

#[cfg(test)]
mod determinism_tests {
    use super::*;

    #[test]
    fn same_seed_reproduces_the_same_rand_sequence() {
        set_rand_seed(42);
        let a: Vec<u64> = (0..8).map(|_| next_rand()).collect();
        set_rand_seed(42);
        let b: Vec<u64> = (0..8).map(|_| next_rand()).collect();
        assert_eq!(a, b, "same seed must reproduce the same xorshift sequence");
    }

    #[test]
    fn different_seeds_diverge() {
        set_rand_seed(1);
        let a: Vec<u64> = (0..8).map(|_| next_rand()).collect();
        set_rand_seed(2);
        let b: Vec<u64> = (0..8).map(|_| next_rand()).collect();
        assert_ne!(a, b, "different seeds should (overwhelmingly likely) diverge");
    }

    #[test]
    fn zero_seed_is_mapped_to_a_nonzero_deterministic_seed() {
        // The xorshift generator is a fixed point at zero (0 ^ ... == 0 forever); a literal
        // `--seed 0` must not silently produce an all-zero, non-random-looking sequence.
        set_rand_seed(0);
        let first = next_rand();
        assert_ne!(first, 0);
        set_rand_seed(0);
        let again = next_rand();
        assert_eq!(first, again, "seed 0 must still be deterministic");
    }

    #[test]
    fn fixed_clock_overrides_the_cap_clock_now_ms_primitive() {
        set_fixed_clock_ms(Some(1_700_000_000_123));
        let cap = CapVal { kind: ResourceKind::Clock, scope: CapScope::Clock };
        let span = Span::new(0, 0, 0);
        let v = call_cap_method(&cap, "now_ms", &[], span).expect("now_ms should not fault");
        set_fixed_clock_ms(None);
        match v {
            Value::Int(ms) => assert_eq!(ms, 1_700_000_000_123),
            other => panic!("expected Int, got {other:?}"),
        }
    }

    #[test]
    fn clearing_the_fixed_clock_restores_the_wall_clock() {
        set_fixed_clock_ms(Some(1));
        set_fixed_clock_ms(None);
        let cap = CapVal { kind: ResourceKind::Clock, scope: CapScope::Clock };
        let span = Span::new(0, 0, 0);
        let before = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;
        let v = call_cap_method(&cap, "now_ms", &[], span).expect("now_ms should not fault");
        match v {
            Value::Int(ms) => assert!((ms - before).abs() < 60_000, "expected a real wall-clock value, got {ms}"),
            other => panic!("expected Int, got {other:?}"),
        }
    }
}

/// Filesystem containment — the boundary [`contains_on_disk`] draws, and the one it does NOT.
///
/// Found by the P20 red team, 2026-08-08. C84 closed the *reparse-point* escape: a symlink or
/// junction inside a granted directory, pointing outside it, is refused because
/// `std::fs::canonicalize` resolves it and the resolved path no longer prefixes the root. These
/// tests lock that (regression) **and** pin the boundary a hardlink sits on the far side of, so a
/// future reader meets it as an executed fact rather than discovering it the way the red team did.
///
/// **Why a hardlink is different, and why this is a documented boundary rather than a fixed bug.** A
/// hardlink is not a reparse point: it is a second *directory entry* for one file record, and the
/// file genuinely resides at both names. `canonicalize` correctly reports a hardlink inside the
/// grant as inside the grant, because it *is* — there is no "real location" elsewhere to resolve to.
/// Three things make this narrower than the C84 escape it superficially resembles, and all three are
/// verified rather than asserted in `docs/design/HARDENING_CAMPAIGN.md` (finding P20-R1):
///
/// 1. **Not workspace-deliverable.** A hardlink does not survive `git`/archive: content is stored,
///    the link relation is not, so a clone cannot carry a hardlink aimed at the victim's files. The
///    C84 symlink/junction — which stores a path string — can, which is why *it* was the urgent one.
/// 2. **Requires prior local access.** Creating the link needs the attacker to open the target, so
///    they already reach the secret; delulu grants them nothing new.
/// 3. **No cheap, cross-platform fix exists.** Deciding "does this file also have a name outside the
///    grant?" needs enumerating every hardlink of an inode. Windows can (`FindFirstFileNameW`);
///    POSIX has no such call short of walking the whole filesystem. A Windows-only defense would make
///    containment platform-dependent — the one thing this project refuses, because the same program
///    would then confine differently on Linux and Windows.
///
/// # The other boundary: this is a check-then-open, so there is a race (CONTAIN-TOCTOU-1)
///
/// Named here because this doc claims to state the boundary `contains_on_disk` draws *and the one it
/// does not*, and until 2026-08-10 it listed only the hardlink. The containment decision is made by
/// resolving the path, and the operation that follows re-opens it **by name**. Between those two
/// moments the filesystem can change: anything able to write into the granted directory can replace
/// a checked plain file with a symlink and have the subsequent write follow it out.
///
/// What this does and does not mean:
///
/// - **It is not reachable by the confined program itself through this API.** A DeluluLang program
///   holding only `Cap[FsWrite]` cannot create a symlink — the primitive table exposes no such
///   operation — so winning this race requires a *second*, concurrent writer.
/// - That second writer is a same-uid process (**category 7**, already outside the proof boundary and
///   documented in `ROOT_ISSUANCE_TRUST_BOUNDARY.md`), **or** any other party who can write into the
///   granted directory — which is the case worth stating, because a grant aimed at a shared location
///   such as `/tmp` hands that ability to everyone on the machine.
/// - **Closing it properly needs the OS, not more path logic.** The fix is to open first and check
///   the opened handle (`O_NOFOLLOW`/`openat2` on Linux, `FILE_FLAG_OPEN_REPARSE_POINT` on Windows),
///   which is exactly the platform-dependent containment fact 3 above refuses. So it is stated, with
///   its threat model, rather than papered over — the same disposition the hardlink gets.
///
/// **Deployment consequence, in one sentence:** grant filesystem scopes that point at directories
/// only the program's own user can write, never at a shared or world-writable one.
#[cfg(test)]
mod containment_tests {
    use super::contains_on_disk;
    use std::fs;
    use std::path::Path;

    fn unique_dir(tag: &str) -> std::path::PathBuf {
        let base = std::env::temp_dir().join(format!(
            "delulu-contain-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir_all(base.join("grant")).unwrap();
        fs::create_dir_all(base.join("outside")).unwrap();
        base
    }

    /// A plain file inside the grant is inside it; a plain file outside is outside. The floor.
    #[test]
    fn a_plain_file_is_contained_iff_it_is_under_the_root() {
        let base = unique_dir("plain");
        let grant = base.join("grant");
        let inside = grant.join("ok.txt");
        let outside = base.join("outside").join("secret.txt");
        fs::write(&inside, b"ok").unwrap();
        fs::write(&outside, b"secret").unwrap();
        assert!(contains_on_disk(&grant, &inside), "a file under the grant must be contained");
        assert!(!contains_on_disk(&grant, &outside), "a file outside the grant must not be");
        let _ = fs::remove_dir_all(&base);
    }

    /// **C84 regression lock.** A symlink inside the grant, pointing outside it, must resolve and be
    /// rejected. Symlink creation needs privilege on Windows (Developer Mode / admin); when it is not
    /// available the assertion is skipped rather than failing, since the mechanism under test —
    /// `canonicalize` resolving a reparse point — is identical to the junction path exercised
    /// end-to-end elsewhere, and a POSIX runner exercises it every time.
    #[test]
    fn a_symlink_escaping_the_grant_is_not_contained() {
        let base = unique_dir("symlink");
        let grant = base.join("grant");
        let secret = base.join("outside").join("secret.txt");
        fs::write(&secret, b"secret").unwrap();
        let link = grant.join("escape.txt");

        #[cfg(unix)]
        let made = std::os::unix::fs::symlink(&secret, &link).is_ok();
        #[cfg(windows)]
        let made = std::os::windows::fs::symlink_file(&secret, &link).is_ok();

        if made {
            assert!(
                !contains_on_disk(&grant, &link),
                "C84: a symlink resolving outside the grant must not be reported contained"
            );
            // And the control: the same mechanism must still admit a link that stays inside.
            let inner = grant.join("inner.txt");
            fs::write(&inner, b"x").unwrap();
            let good = grant.join("good_link.txt");
            #[cfg(unix)]
            let good_made = std::os::unix::fs::symlink(&inner, &good).is_ok();
            #[cfg(windows)]
            let good_made = std::os::windows::fs::symlink_file(&inner, &good).is_ok();
            if good_made {
                assert!(contains_on_disk(&grant, &good), "a link that stays inside must be contained");
            }
        } else {
            eprintln!("skipped: symlink creation not permitted on this host (needs privilege)");
        }
        let _ = fs::remove_dir_all(&base);
    }

    /// **SYMLINK-DANGLE-1 regression lock.** A symlink inside the grant aimed OUTSIDE it, whose
    /// target does not exist yet, must not be reported contained. Before the fix this returned
    /// `true`: `canonicalize` fails on a dangling link exactly as it fails on an absent name, so the
    /// nearest-existing-ancestor walk re-appended the link's own name as a "plain name" and the
    /// prefix check passed — after which the caller opened the path, the OS followed the link, and
    /// the file was created outside the grant. Verified end-to-end before the fix (write returned
    /// `Ok`, file appeared outside); this is the unit-level lock.
    #[test]
    fn a_dangling_symlink_aimed_outside_the_grant_is_not_contained() {
        let base = unique_dir("dangling-out");
        let grant = base.join("grant");
        // Deliberately NOT created — the whole point is that the target does not exist.
        let victim = base.join("outside").join("pwned.txt");
        assert!(!victim.exists(), "the target must be absent for this to be the dangling case");
        let link = grant.join("log.txt");

        #[cfg(unix)]
        let made = std::os::unix::fs::symlink(&victim, &link).is_ok();
        #[cfg(windows)]
        let made = std::os::windows::fs::symlink_file(&victim, &link).is_ok();

        if made {
            assert!(
                !contains_on_disk(&grant, &link),
                "SYMLINK-DANGLE-1: a dangling symlink aimed outside the grant must not be reported \
                 contained — opening it creates the file at the target, outside the grant"
            );
        } else {
            eprintln!("skipped: symlink creation not permitted on this host (needs privilege)");
        }
        let _ = fs::remove_dir_all(&base);
    }

    /// The deliberate narrowing, pinned so it is a decision and not a surprise: a dangling link that
    /// points back INSIDE the grant is refused too. Telling it apart from the escaping one means
    /// resolving the link by hand, and when the alternative is guessing where a write lands,
    /// narrowing is the safe direction. If this is ever relaxed it must be a conscious change.
    #[test]
    fn a_dangling_symlink_aimed_inside_the_grant_is_also_refused() {
        let base = unique_dir("dangling-in");
        let grant = base.join("grant");
        let inside_target = grant.join("not_yet.txt"); // absent on purpose
        let link = grant.join("alias.txt");

        #[cfg(unix)]
        let made = std::os::unix::fs::symlink(&inside_target, &link).is_ok();
        #[cfg(windows)]
        let made = std::os::windows::fs::symlink_file(&inside_target, &link).is_ok();

        if made {
            assert!(
                !contains_on_disk(&grant, &link),
                "a dangling link is refused regardless of where it aims — narrowing is the safe \
                 direction (SYMLINK-DANGLE-1 disposition)"
            );
        } else {
            eprintln!("skipped: symlink creation not permitted on this host (needs privilege)");
        }
        let _ = fs::remove_dir_all(&base);
    }

    /// **The over-narrowing guard.** The SYMLINK-DANGLE-1 fix must not refuse ordinary programs: a
    /// file that simply does not exist yet — the common case for every `write_text` that creates its
    /// output — is still contained, and so is a not-yet-existing file in a not-yet-existing
    /// subdirectory. Without this, a fix aimed at the escape could quietly break every write.
    #[test]
    fn a_not_yet_existing_file_under_the_grant_is_still_contained() {
        let base = unique_dir("absent");
        let grant = base.join("grant");
        let fresh = grant.join("brand_new.txt");
        assert!(!fresh.exists());
        assert!(
            contains_on_disk(&grant, &fresh),
            "a write must still be able to create its own file inside the grant"
        );
        let deeper = grant.join("sub").join("deeper").join("out.txt");
        assert!(
            contains_on_disk(&grant, &deeper),
            "a not-yet-existing path under the grant stays contained (the open itself may still \
             fail if the parent is missing — that is an IO error, not a containment decision)"
        );
        let _ = fs::remove_dir_all(&base);
    }

    /// **The documented boundary, pinned as an executed fact.** A hardlink inside the grant whose
    /// content is shared with a file outside it IS reported contained — because the file genuinely
    /// has a name inside the grant. This is not a fix-required escape (see the module doc): it is not
    /// workspace-deliverable, it requires an attacker who already reaches the target, and no cheap
    /// cross-platform defense exists. If this behavior ever changes — in either direction — it must
    /// be a conscious decision that updates this test and the P20-R1 record, not a silent drift.
    #[test]
    fn a_hardlink_sharing_content_with_an_outside_file_is_still_contained() {
        let base = unique_dir("hardlink");
        let grant = base.join("grant");
        let outside = base.join("outside").join("secret.txt");
        fs::write(&outside, b"CROWN-JEWELS").unwrap();
        let link = grant.join("hl.txt");

        if fs::hard_link(&outside, &link).is_err() {
            // Cross-device or unsupported filesystem — nothing to characterize here.
            eprintln!("skipped: hard link creation not supported on this host/filesystem");
            let _ = fs::remove_dir_all(&base);
            return;
        }
        assert!(Path::new(&link).exists());
        assert!(
            contains_on_disk(&grant, &link),
            "a hardlink is a real member of the granted directory; canonicalize reports it inside \
             because it IS inside — this is the P20-R1 boundary, documented in HARDENING_CAMPAIGN.md, \
             not a containment escape in the C84 sense"
        );
        let _ = fs::remove_dir_all(&base);
    }
}

/// D-NE-29 (PS-0-06): the path spellings the primitive table refuses. The Windows rules are
/// compiled everywhere under test, so a Linux CI run pins them too.
#[cfg(test)]
mod hostile_path_tests {
    use super::*;

    #[test]
    fn windows_spellings_that_mean_something_else_are_refused() {
        for p in [
            "CON", "con", "NUL", "nul.txt", "PRN", "AUX.log", "COM1", "com9.dat", "LPT1", "lpt3.txt",
            "COM\u{b9}", "LPT\u{b2}.x", "CONIN$", "CONOUT$", "a/b/CON", r"a\NUL\b", "CON .txt",
            "trail.", "space ", "dir./x", "x.txt:stream", "x.txt::$DATA", "C:foo", "c:", r"D:x\y",
            r"\\?\C:\x", "//?/C:/x", r"\\.\PhysicalDrive0", "//./pipe/x", r"\\server\share\x",
        ] {
            assert!(windows_hostile_path(p, false).is_some(), "`{p}` must be refused");
        }
    }

    #[test]
    fn ordinary_names_still_pass() {
        for p in [
            "a.txt", "sub/file.txt", "./x", "a/../b", "console.log", "nullable", "com10", "LPT0x",
            "conx", r"C:\abs\file.txt", "D:/abs/file", "..", "a.b.c", "COMPANY.txt",
        ] {
            assert!(windows_hostile_path(p, false).is_none(), "`{p}` must pass: {:?}", windows_hostile_path(p, false));
        }
        // An operator's grant may name a share; a program's path may not.
        assert!(windows_hostile_path(r"\\server\share", true).is_none());
    }

    #[test]
    fn an_embedded_nul_is_refused_everywhere() {
        assert!(hostile_path("a\0b", false).is_some());
        assert!(hostile_path("a\0b", true).is_some());
    }
}
