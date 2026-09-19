//! `delulu sandbox probe [--json]` (PS-0-04): which isolation levels this host can give a program
//! today, and for each absent one the FIRST missing prerequisite.
//!
//! **Every line is an attempt.** The probe opens `/dev/kvm`, asks the kernel for a Landlock ABI,
//! creates a Job Object and a restricted token, spawns a Seatbelt-confined child — it never reads a
//! version string and infers. A level is `present` only when every attempt for it succeeded; the
//! levels whose backend is not built yet say so as the last prerequisite, because "the host could"
//! and "this build does" are different facts (`V2_SECURITY_MODEL.md` §6).

use serde_json::{json, Value as Json};

/// One attempted prerequisite.
pub struct Attempt {
    pub what: &'static str,
    pub ok: bool,
    pub detail: String,
}

/// One isolation level's verdict.
pub struct Level {
    pub level: u8,
    pub name: &'static str,
    pub attempts: Vec<Attempt>,
}

impl Level {
    /// Present exactly when it was attempted and every attempt succeeded.
    pub fn available(&self) -> bool {
        !self.attempts.is_empty() && self.attempts.iter().all(|a| a.ok)
    }
    pub fn first_missing(&self) -> Option<&Attempt> {
        self.attempts.iter().find(|a| !a.ok)
    }
}

fn attempt(what: &'static str, r: Result<String, String>) -> Attempt {
    match r {
        Ok(detail) => Attempt { what, ok: true, detail },
        Err(detail) => Attempt { what, ok: false, detail },
    }
}

/// Open `/dev/kvm` read-write, as a VMM must. Public so a test can compare the probe with its own
/// independent attempt.
pub fn attempt_open_kvm() -> Result<String, String> {
    match std::fs::OpenOptions::new().read(true).write(true).open("/dev/kvm") {
        Ok(_) => Ok("opened /dev/kvm read-write".into()),
        Err(e) => Err(format!("cannot open /dev/kvm read-write: {e}")),
    }
}

/// Run every attempt and return the five levels.
pub fn probe() -> Vec<Level> {
    let l0 = Level {
        level: 0,
        name: "none",
        attempts: vec![attempt(
            "the in-process runtime",
            Ok("this probe is running in it: the language and custody, in-process — no OS boundary".into()),
        )],
    };

    let mut l1 = os_primitive_attempts();
    l1.push(Attempt {
        what: "the L1 guest launcher",
        ok: false,
        detail: "not in this build — `--isolation process` isolates foreign code only (PS-A builds the jail)".into(),
    });

    let l2 = vec![
        attempt("KVM", attempt_open_kvm()),
        attempt("a microVM monitor on PATH", find_vmm()),
        Attempt { what: "the L2 guest launch", ok: false, detail: "not in this build (PS-C)".into() },
    ];

    let l3 = vec![Attempt {
        what: "an operator-supplied external launcher",
        ok: false,
        detail: "none can be configured in this build (PS-D)".into(),
    }];

    let l4 = vec![Attempt {
        what: "an attester for the guest image",
        ok: false,
        detail: "attestation is deferred (PS-D-02)".into(),
    }];

    vec![
        l0,
        Level { level: 1, name: "process", attempts: l1 },
        Level { level: 2, name: "microvm", attempts: l2 },
        Level { level: 3, name: "external", attempts: l3 },
        Level { level: 4, name: "attested", attempts: l4 },
    ]
}

fn find_vmm() -> Result<String, String> {
    const VMMS: &[&str] = &["firecracker", "cloud-hypervisor"];
    let path = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path) {
        for v in VMMS {
            let p = dir.join(v);
            if p.is_file() {
                return Ok(format!("found {}", p.display()));
            }
        }
    }
    Err(format!("none of {} on PATH", VMMS.join(", ")))
}

#[cfg(target_os = "linux")]
fn os_primitive_attempts() -> Vec<Attempt> {
    // landlock_create_ruleset(NULL, 0, LANDLOCK_CREATE_RULESET_VERSION) returns the ABI version.
    let landlock = {
        const SYS_LANDLOCK_CREATE_RULESET: libc::c_long = 444;
        const LANDLOCK_CREATE_RULESET_VERSION: libc::c_uint = 1;
        // SAFETY: the documented version query; no pointer is dereferenced (attr is NULL, size 0).
        let v = unsafe {
            libc::syscall(SYS_LANDLOCK_CREATE_RULESET, std::ptr::null::<libc::c_void>(), 0usize, LANDLOCK_CREATE_RULESET_VERSION)
        };
        if v >= 1 {
            Ok(format!("the kernel answered Landlock ABI {v}"))
        } else {
            Err(format!("the kernel refused the Landlock query: {}", std::io::Error::last_os_error()))
        }
    };
    let rlimit = {
        let mut r = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
        // SAFETY: getrlimit writes into the struct we own.
        if unsafe { libc::getrlimit(libc::RLIMIT_AS, &mut r) } == 0 {
            Ok("getrlimit(RLIMIT_AS) answered".to_string())
        } else {
            Err(format!("getrlimit failed: {}", std::io::Error::last_os_error()))
        }
    };
    vec![attempt("Landlock", landlock), attempt("resource limits", rlimit)]
}

#[cfg(target_os = "macos")]
fn os_primitive_attempts() -> Vec<Attempt> {
    // A Seatbelt-confined child: spawn `/usr/bin/true` under the most permissive profile. It proves
    // `sandbox-exec` can apply a profile here, nothing more.
    let seatbelt = match std::process::Command::new("/usr/bin/sandbox-exec")
        .args(["-p", "(version 1)(allow default)", "/usr/bin/true"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(s) if s.success() => Ok("a child ran under a Seatbelt profile".to_string()),
        Ok(s) => Err(format!("sandbox-exec exited {s}")),
        Err(e) => Err(format!("cannot spawn sandbox-exec: {e}")),
    };
    vec![attempt("Seatbelt", seatbelt)]
}

#[cfg(windows)]
fn os_primitive_attempts() -> Vec<Attempt> {
    vec![attempt("a Job Object with limits", win::job_object()), attempt("a restricted token", win::restricted_token())]
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn os_primitive_attempts() -> Vec<Attempt> {
    vec![Attempt { what: "an OS confinement primitive", ok: false, detail: "no probe for this OS".into() }]
}

#[cfg(windows)]
mod win {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        CreateRestrictedToken, DISABLE_MAX_PRIVILEGE, TOKEN_ASSIGN_PRIMARY, TOKEN_DUPLICATE, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::JobObjects::{
        CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOB_OBJECT_LIMIT_PROCESS_MEMORY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    pub fn job_object() -> Result<String, String> {
        // SAFETY: plain Win32 calls on handles this function creates and closes.
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return Err(format!("CreateJobObjectW failed: {}", std::io::Error::last_os_error()));
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags =
                JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
            info.BasicLimitInformation.ActiveProcessLimit = 1;
            info.ProcessMemoryLimit = 256 * 1024 * 1024;
            let ok = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            let err = std::io::Error::last_os_error();
            CloseHandle(job);
            if ok == 0 {
                return Err(format!("SetInformationJobObject refused the limits: {err}"));
            }
            Ok("created a Job Object and set kill-on-close, one-process and memory limits".into())
        }
    }

    pub fn restricted_token() -> Result<String, String> {
        // SAFETY: plain Win32 calls on handles this function opens and closes.
        unsafe {
            let mut token: HANDLE = std::ptr::null_mut();
            if OpenProcessToken(GetCurrentProcess(), TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ASSIGN_PRIMARY, &mut token) == 0 {
                return Err(format!("OpenProcessToken failed: {}", std::io::Error::last_os_error()));
            }
            let mut restricted: HANDLE = std::ptr::null_mut();
            let ok = CreateRestrictedToken(
                token,
                DISABLE_MAX_PRIVILEGE,
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                0,
                std::ptr::null(),
                &mut restricted,
            );
            let err = std::io::Error::last_os_error();
            CloseHandle(token);
            if ok == 0 {
                return Err(format!("CreateRestrictedToken failed: {err}"));
            }
            CloseHandle(restricted);
            Ok("created a restricted token (all privileges but one removed)".into())
        }
    }
}

pub fn to_json(levels: &[Level]) -> Json {
    json!({
        "host": { "os": std::env::consts::OS, "arch": std::env::consts::ARCH },
        "highest_available": levels.iter().filter(|l| l.available()).map(|l| l.level).max(),
        "levels": levels.iter().map(|l| json!({
            "level": l.level,
            "name": l.name,
            "available": l.available(),
            "first_missing": l.first_missing().map(|a| format!("{}: {}", a.what, a.detail)),
            "attempts": l.attempts.iter().map(|a| json!({ "what": a.what, "ok": a.ok, "detail": a.detail })).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

/// `delulu sandbox probe [--json]`.
/// `--sandbox-profile <name>` for the `policy` verb; `Err(name)` when the name is not one of the
/// three (D-V2-25). Refused, never defaulted: a typo must not silently report a different policy
/// from the one a run would use.
fn profile_flag(rest: &[String]) -> Result<crate::policy::Profile, String> {
    let mut it = rest.iter();
    while let Some(a) = it.next() {
        let name = if a == "--sandbox-profile" {
            it.next().cloned()
        } else {
            a.strip_prefix("--sandbox-profile=").map(str::to_string)
        };
        if let Some(name) = name {
            return crate::policy::Profile::parse(&name).ok_or(name);
        }
    }
    Ok(crate::policy::Profile::Contained)
}

pub fn cmd_sandbox(rest: &[String]) -> i32 {
    let json = rest.iter().any(|a| a == "--json");
    // A flag's VALUE is not a verb: `policy f.delulu --sandbox-profile dev` has one verb and one
    // file, and counting `dev` as a third made the whole command fall through to the usage line.
    let mut verbs: Vec<&str> = Vec::new();
    let mut skip_next = false;
    for a in rest.iter().map(String::as_str) {
        if skip_next {
            skip_next = false;
            continue;
        }
        if a == "--sandbox-profile" {
            skip_next = true;
            continue;
        }
        if !a.starts_with('-') {
            verbs.push(a);
        }
    }
    if let Some(bad) = rest
        .iter()
        .find(|a| a.starts_with('-') && *a != "--json" && *a != "--sandbox-profile" && !a.starts_with("--sandbox-profile="))
    {
        eprintln!("error: `sandbox` does not know this option: {bad}");
        eprintln!("  nothing was done — an option nobody understood is refused, never ignored");
        return 2;
    }
    match verbs.as_slice() {
        ["probe"] => {
            let levels = probe();
            if json {
                crate::cli::print_success_envelope("sandbox", to_json(&levels));
            } else {
                println!("sandbox levels on this host ({} {}) — every line is an attempt:", std::env::consts::OS, std::env::consts::ARCH);
                for l in &levels {
                    match l.first_missing() {
                        None => println!("  L{} {:<9} present — {}", l.level, l.name, l.attempts.iter().map(|a| a.detail.as_str()).collect::<Vec<_>>().join("; ")),
                        Some(a) => println!("  L{} {:<9} absent  — first missing: {}: {}", l.level, l.name, a.what, a.detail),
                    }
                }
            }
            0
        }
        // PS-A-06: what policy WOULD hold for this program, without running it. The derivation is
        // pure, so this answers with the same policy a real run would use, hash and all — which is
        // what makes it worth reading before letting unfamiliar code run.
        ["policy", file] => {
            let profile = match profile_flag(rest) {
                Ok(p) => p,
                Err(name) => {
                    eprintln!("error: `{name}` is not a sandbox profile (dev, contained, hostile-agent)");
                    return 2;
                }
            };
            let program = match std::fs::read_to_string(file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot read `{file}`: {e}");
                    return 2;
                }
            };
            let policy = crate::policy::SandboxPolicy::derive(1, profile, None, crate::policy::Mode::Strict);
            let carried = crate::guest::unsupported_surface(&program);
            if json {
                let mut obj = policy.to_json("process", &[]);
                obj["unsupported_surface"] = match &carried {
                    Some(s) => serde_json::json!(s),
                    None => serde_json::Value::Null,
                };
                crate::cli::print_success_envelope("sandbox", serde_json::json!({ "policy": obj }));
            } else {
                println!("policy for `{file}` under `{}`:", profile.name());
                println!("  level      1 (a jailed guest process)");
                println!("  memory     {} bytes", policy.limits.memory_bytes);
                println!("  processor  {} seconds", policy.limits.cpu_seconds);
                println!("  mode       {}", policy.mode.name());
                println!("  hash       {}", policy.hash());
                match &carried {
                    None => println!("  this program's surface is carried by the sandbox channel"),
                    Some(s) => println!("  NOT carried yet: {s} — `--sandbox` would refuse this program"),
                }
            }
            0
        }
        _ => {
            eprintln!("error: `sandbox` needs a verb: probe [--json] | policy <file.delulu> [--sandbox-profile P] [--json]");
            2
        }
    }
}
