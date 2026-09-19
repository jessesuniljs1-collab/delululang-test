//! PS-A-06: `SandboxPolicy` — what a run's confinement IS, as one value.
//!
//! Derived from the profile the operator chose and the limits they set, and from nothing else. It is
//! pure, so the same inputs give the same policy; stable in JSON, so an agent can compare two runs;
//! and hashable, so an audit record can name the policy a run actually used rather than describing it
//! again in prose.
//!
//! The three profile names are the owner's ruling D-V2-25: `dev` for your own code, `contained` for
//! the ordinary case, `hostile-agent` for code you did not write and do not trust. They differ in how
//! much a guest may consume, never in who performs its effects — that is always the host.

use crate::jail::Limits;

/// A named policy. Profiles exist so an operator, or an agent, can say what KIND of code this is
/// rather than tuning numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    Dev,
    Contained,
    HostileAgent,
}

impl Profile {
    pub fn parse(name: &str) -> Option<Profile> {
        match name {
            "dev" => Some(Profile::Dev),
            "contained" => Some(Profile::Contained),
            "hostile-agent" => Some(Profile::HostileAgent),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Profile::Dev => "dev",
            Profile::Contained => "contained",
            Profile::HostileAgent => "hostile-agent",
        }
    }

    /// What this profile allows a guest to consume. Never unlimited, on any profile.
    pub fn limits(self) -> Limits {
        match self {
            // Your own code, on your own machine: generous, still bounded.
            Profile::Dev => Limits { memory_bytes: 4 * 1024 * 1024 * 1024, cpu_seconds: 1_800 },
            // The ordinary case, and the owner's defaults (D-V2-25).
            Profile::Contained => Limits::default(),
            // Code you did not write: tight enough that a runaway is stopped quickly.
            Profile::HostileAgent => Limits { memory_bytes: 256 * 1024 * 1024, cpu_seconds: 60 },
        }
    }
}

/// How a run treats effects. AUDIT performs none of them and reports what would have happened
/// (PS-A-10); STRICT is the ordinary run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Strict,
    Audit,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Strict => "strict",
            Mode::Audit => "audit",
        }
    }
}

/// The whole confinement of one run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxPolicy {
    /// 0 = in process, no OS boundary. 1 = the guest runs as a jailed child process.
    pub level: u8,
    pub profile: Profile,
    pub limits: Limits,
    pub mode: Mode,
    pub break_glass: bool,
}

impl SandboxPolicy {
    /// Derive the policy for a run. Pure: the same request gives the same policy, which is what
    /// makes the hash below worth recording.
    pub fn derive(level: u8, profile: Profile, limits: Option<Limits>, mode: Mode) -> SandboxPolicy {
        SandboxPolicy {
            level,
            profile,
            limits: limits.unwrap_or_else(|| profile.limits()),
            mode,
            // Break-glass is never derived: it is an authorized external decision (PS-B), and a
            // policy that could turn it on by itself would not be a boundary.
            break_glass: false,
        }
    }

    /// The `sandbox` object of the run report (D-V2-21). `guarantees` are what the host actually
    /// applied, never what the profile hoped for.
    pub fn to_json(self, backend: &str, guarantees: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "backend": backend,
            "level": self.level,
            "requested": self.profile.name(),
            "granted": if guarantees.is_empty() { "none" } else { self.profile.name() },
            "host_guarantees": guarantees,
            "limits": { "memory_bytes": self.limits.memory_bytes, "cpu_seconds": self.limits.cpu_seconds },
            "mode": self.mode.name(),
            "break_glass": self.break_glass,
            "policy_hash": self.hash(),
        })
    }

    /// A stable name for this policy: the same policy hashes the same in any run, on any machine, so
    /// an audit record can say WHICH policy was in force without repeating it.
    pub fn hash(self) -> String {
        let canonical = format!(
            "delulu-sandbox-policy/1 level={} profile={} mem={} cpu={} mode={} break_glass={}",
            self.level,
            self.profile.name(),
            self.limits.memory_bytes,
            self.limits.cpu_seconds,
            self.mode.name(),
            self.break_glass
        );
        blake3::hash(canonical.as_bytes()).to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_profile_round_trips_and_an_unknown_name_is_refused() {
        for p in [Profile::Dev, Profile::Contained, Profile::HostileAgent] {
            assert_eq!(Profile::parse(p.name()), Some(p));
        }
        assert_eq!(Profile::parse("whatever"), None, "an unknown profile must be refused, never defaulted");
    }

    /// No profile is unlimited, and the untrusted one is the tightest. A profile that let a guest
    /// consume everything would be a name for no policy at all.
    #[test]
    fn every_profile_is_bounded_and_hostile_agent_is_the_tightest() {
        for p in [Profile::Dev, Profile::Contained, Profile::HostileAgent] {
            let l = p.limits();
            assert!(l.memory_bytes > 0 && l.cpu_seconds > 0, "{} is unbounded", p.name());
        }
        let hostile = Profile::HostileAgent.limits();
        for p in [Profile::Dev, Profile::Contained] {
            assert!(hostile.memory_bytes <= p.limits().memory_bytes);
            assert!(hostile.cpu_seconds <= p.limits().cpu_seconds);
        }
    }

    #[test]
    fn the_hash_names_the_policy_and_changes_when_the_policy_does() {
        let a = SandboxPolicy::derive(1, Profile::Contained, None, Mode::Strict);
        let same = SandboxPolicy::derive(1, Profile::Contained, None, Mode::Strict);
        assert_eq!(a.hash(), same.hash(), "the same policy must hash the same");
        for other in [
            SandboxPolicy::derive(0, Profile::Contained, None, Mode::Strict),
            SandboxPolicy::derive(1, Profile::Dev, None, Mode::Strict),
            SandboxPolicy::derive(1, Profile::Contained, None, Mode::Audit),
            SandboxPolicy::derive(1, Profile::Contained, Some(Limits { memory_bytes: 1, cpu_seconds: 1 }), Mode::Strict),
        ] {
            assert_ne!(a.hash(), other.hash(), "a different policy must hash differently: {other:?}");
        }
    }

    /// Break-glass is an authorized external decision, never something a derivation turns on.
    #[test]
    fn deriving_a_policy_never_turns_on_break_glass() {
        for profile in [Profile::Dev, Profile::Contained, Profile::HostileAgent] {
            for mode in [Mode::Strict, Mode::Audit] {
                assert!(!SandboxPolicy::derive(1, profile, None, mode).break_glass);
            }
        }
    }

    /// The report says what was APPLIED: with nothing enforced, nothing is granted.
    #[test]
    fn the_report_grants_nothing_when_nothing_was_enforced() {
        let p = SandboxPolicy::derive(1, Profile::Contained, None, Mode::Strict);
        assert_eq!(p.to_json("process", &[])["granted"], "none");
        let applied = p.to_json("process", &["memory ceiling"]);
        assert_eq!(applied["granted"], "contained");
        assert_eq!(applied["host_guarantees"], serde_json::json!(["memory ceiling"]));
        assert_eq!(applied["policy_hash"], p.hash());
    }
}
