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

    /// PS-A-07: the questions a reader actually has, answered from what was APPLIED.
    ///
    /// Every answer is derived from the guarantee list, never from a per-platform table written
    /// alongside it. That matters more than it looks: a table is a second copy of the truth, and a
    /// second copy of the truth in this phase has gone stale twice — once for the loader's
    /// environment allowlist, once for a per-platform jail report that claimed Linux enforced nothing
    /// on the very commit that made Linux enforce. Here, a guarantee that stops being applied stops
    /// being claimed in the same edit, because there is nowhere else to say it.
    ///
    /// `limitations` is the complement: every question whose answer is that nothing is enforcing it.
    /// A report that lists only guarantees reads as though the rest were covered.
    pub fn posture(guarantees: &[&str]) -> (serde_json::Value, Vec<&'static str>) {
        let has = |needle: &str| guarantees.iter().any(|g| g.contains(needle));
        // (question, the answer when it IS enforced, the guarantee that enforces it)
        let rows: [(&str, &str, bool); 7] = [
            ("filesystem_writes", "denied", has("no file writes")),
            ("filesystem_reads", "confined to the system paths", has("reads only from the system paths")),
            ("network", "only the channel", has("no TCP bind or connect") || has("no network but the channel")),
            ("new_programs", "denied", has("no new programs") || has("one process only")),
            ("memory", "capped", has("memory ceiling")),
            ("processor_time", "capped", has("processor-time ceiling")),
            ("privilege_escalation", "denied", has("no privilege escalation") || has("deny by default")),
        ];
        let mut obj = serde_json::Map::new();
        let mut limitations: Vec<&'static str> = Vec::new();
        for (question, answer, enforced) in rows {
            obj.insert(
                question.to_string(),
                serde_json::Value::String(if enforced { answer.to_string() } else { "not confined".to_string() }),
            );
            if !enforced {
                // The static name, so the list is machine-readable rather than a sentence.
                limitations.push(match question {
                    "filesystem_writes" => "filesystem_writes",
                    "filesystem_reads" => "filesystem_reads",
                    "network" => "network",
                    "new_programs" => "new_programs",
                    "memory" => "memory",
                    "processor_time" => "processor_time",
                    _ => "privilege_escalation",
                });
            }
        }
        // Identity is its own row because it is not a guarantee any platform gives today: the guest
        // runs as the same OS user, which is REMAINING_WORK 4.4 and proof category 7. Saying it here
        // stops a reader inferring separation from a long list of things that ARE enforced.
        obj.insert("identity".to_string(), serde_json::Value::String("same OS user".to_string()));
        limitations.push("identity_separation");
        (serde_json::Value::Object(obj), limitations)
    }

    /// The `sandbox` object of the run report (D-V2-21). `guarantees` are what the host actually
    /// applied, never what the profile hoped for.
    pub fn to_json(self, backend: &str, guarantees: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "backend": backend,
            "requested_level": 1,
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

    /// The object for a run that HAPPENED: the same fields plus the posture, the limitations, the
    /// fully-enforced flag and the refusals.
    ///
    /// The split is deliberate. `to_json` above is for the two cases where nothing ran —
    /// `sandbox policy` and `--mode audit` — and those must NOT carry a posture, because a posture is
    /// derived from the guarantees that were APPLIED, and an empty guarantee list would have them
    /// reporting "not confined" for everything as though that were a prediction about the run. It is
    /// not a prediction; it is a measurement, and a measurement of a run that did not happen does not
    /// exist.
    ///
    /// `denied` is every answer the host gave that was not `Ok` — a channel refusal, or a fault the
    /// host's own checks raised, such as a path outside the granted scope. Both are recorded, because
    /// from the guest's side both are "you may not"; telling them apart needs the code registry to say
    /// which codes are refusals, which is open work and is not pretended here. `denied_total` is
    /// larger than the list when a guest was refused more often than the bound keeps.
    pub fn to_json_with(
        self,
        backend: &str,
        guarantees: &[&str],
        denied: &[String],
        denied_total: u64,
    ) -> serde_json::Value {
        let (posture, limitations) = Self::posture(guarantees);
        // "Fully enforced" means every question above is answered by something that was applied —
        // except identity separation, which no platform gives today (RW 4.4) and which is therefore
        // named as a limitation on every honest run rather than quietly excluded from the total.
        let fully = self.level == 1 && limitations.iter().all(|l| *l == "identity_separation");
        serde_json::json!({
            "backend": backend,
            // Requested and ACTUAL, kept apart. A single `level` cannot say "you asked for a jailed
            // guest and this host gave you one" separately from "you asked and it did not".
            "requested_level": 1,
            "level": self.level,
            "fully_enforced": fully,
            "requested": self.profile.name(),
            "granted": if guarantees.is_empty() { "none" } else { self.profile.name() },
            "host_guarantees": guarantees,
            "posture": posture,
            "limitations": limitations,
            "limits": { "memory_bytes": self.limits.memory_bytes, "cpu_seconds": self.limits.cpu_seconds },
            "mode": self.mode.name(),
            "break_glass": self.break_glass,
            "denied": denied,
            "denied_total": denied_total,
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
    /// The posture is derived from what was applied, so a guarantee that goes away stops being
    /// claimed in the same edit. Both directions, because a function that answered "not confined" to
    /// everything would pass the first half alone.
    #[test]
    fn the_posture_answers_from_the_guarantees_and_names_what_nothing_enforces() {
        let (obj, lim) = SandboxPolicy::posture(&[
            "no file writes",
            "reads only from the system paths",
            "no TCP bind or connect",
            "no new programs",
            "memory ceiling",
            "processor-time ceiling",
            "no privilege escalation",
        ]);
        assert_eq!(obj["filesystem_writes"], "denied");
        assert_eq!(obj["filesystem_reads"], "confined to the system paths");
        assert_eq!(obj["network"], "only the channel");
        assert_eq!(obj["memory"], "capped");
        // Identity separation is never enforced by any platform today (RW 4.4), so it is the one
        // limitation a fully confined run still carries. A report that dropped it would let a long
        // list of real guarantees imply a boundary that is not there.
        assert_eq!(lim, vec!["identity_separation"], "a fully confined run named the wrong limitations");
        assert_eq!(obj["identity"], "same OS user");

        // And with nothing applied, every question must say so.
        let (obj, lim) = SandboxPolicy::posture(&[]);
        for q in ["filesystem_writes", "filesystem_reads", "network", "new_programs", "memory", "processor_time"] {
            assert_eq!(obj[q], "not confined", "`{q}` claimed a boundary nothing applied");
            assert!(lim.contains(&q), "`{q}` is unenforced and was not listed as a limitation");
        }
    }

    /// The two shapes are kept apart on purpose: a run that did not happen has no posture to report.
    #[test]
    fn a_preview_carries_no_posture_because_nothing_was_measured() {
        let p = SandboxPolicy::derive(1, Profile::Contained, None, Mode::Audit);
        let preview = p.to_json("none", &[]);
        assert!(preview.get("posture").is_none(), "a preview must not report a posture: {preview}");
        assert!(preview.get("limitations").is_none(), "{preview}");
        assert!(preview.get("denied").is_none(), "{preview}");
        // The run form does carry them, or the split above would just be a missing feature.
        let ran = p.to_json_with("process", &["memory ceiling"], &["DL1401 on `root.fs_write`: nope".into()], 1);
        assert!(ran.get("posture").is_some(), "{ran}");
        assert_eq!(ran["denied_total"], 1, "{ran}");
        assert_eq!(ran["fully_enforced"], false, "one guarantee is not full enforcement: {ran}");
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
