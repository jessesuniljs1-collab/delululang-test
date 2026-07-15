//! The plugin loader (Stage 6 "Live", spec §3.1) — steps 1–4 land in phase 6d.
//!
//! The load sequence is an **ORDER, not a set**. Each step is its own testable function, and
//! [`load_prepare`] runs them in the normative order. The order is observable: a grant that
//! violates *both* the manifest ceiling and the holder's node reports **DL1502**, because step 3
//! fires before step 4 (witnessed by `ceiling_is_checked_before_the_holder_step_order_matters`).
//!
//! ```text
//! 1. read artifact; validate container + plugin.api            → DL1507
//! 2. class check against C (invariant 29 — never inferred)     → refuse, never fall back
//! 3. manifest ceiling: grant ⊑ plugin.authority                → DL1502 (intersection = exact repair)
//! 4. holder check: broker attenuate(host's node, grant)        → DL0802; child node + fresh GrantId
//! 5. class-specific verification                               (phase 6e / 6f)
//! 6. signature policy                                          (phase 6h)
//! 7. instantiate                                               (phase 6e / 6f)
//! ```
//!
//! **Invariant 31 — no plugin reaches the broker — is guaranteed by *absence*, not refusal.** The
//! `GrantId` minted at step 4 is held *here*, host-side, in [`PreparedLoad`]; it is never a value in
//! the plugin's world. The plugin's world (spec §4) contains only `Grant`/`Limits`/`PluginErr` —
//! ordinary records that *describe* authority — plus capability values the host passes explicitly.
//! There is no `attenuate`, no `revoke`, no lease token, and no IPC path in the plugin's value
//! vocabulary: those operations **do not exist** for it, so there is nothing to refuse.
//!
//! **Crate topology (playbook §1, head-chef amendment).** `delulu-wasm` already depends on
//! `delulu-runtime`, so the loader here cannot reach `delulu-wasm` directly. Instead the container
//! is reified as plain data ([`PluginArtifact`]) and the engine as a trait ([`PluginEngine`]),
//! both defined *here*; `delulu-wasm` implements them, and the `delulu` crate injects the
//! implementation. `plugin verify` and a real load therefore share one code path by construction
//! (spec §9.9) — they call these very functions.

use std::collections::BTreeMap;

use delulu_broker::{attenuation_check, Authority, GrantId, Holder, Scopes};
use delulu_check::Effect;

use crate::custody::{Custody, CustodyDenial};

/// The plugin ABI version this runtime loads. Kept in lockstep with
/// `delulu_check::plugin::PLUGIN_API_SUPPORTED`; a mismatch is DL1507.
pub const PLUGIN_API_SUPPORTED: u32 = 1;

/// `std.plugin.Limits` (spec §4). `0` means "broker/profile default" — never "unlimited".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Limits {
    pub fuel: i64,
    pub mem_mb: i64,
    pub wall_ms: i64,
}

/// `std.plugin.Grant` (spec §4) — an **ordinary record**: it *describes* authority, it does not
/// confer it. Conferral happens only at `load`, under the step-4 holder check.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Grant {
    pub effects: Vec<String>,
    pub fs_read: Vec<String>,
    pub fs_write: Vec<String>,
    pub net: Vec<String>,
    pub secrets: Vec<String>,
    pub declassify: Vec<String>,
    pub limits: Limits,
    pub require_signed: bool,
}

impl Grant {
    /// Project to the broker's `Authority` — the shape the `⊑` lattice speaks. Unknown effect names
    /// become `Effect::User`, exactly as the checker lowers a user-declared effect.
    pub fn to_authority(&self) -> Authority {
        Authority {
            effects: self.effects.iter().map(|e| effect_from_name(e)).collect(),
            scopes: Scopes {
                fs_read: self.fs_read.iter().cloned().collect(),
                fs_write: self.fs_write.iter().cloned().collect(),
                net: self.net.iter().cloned().collect(),
                secrets: self.secrets.iter().cloned().collect(),
                declassify: self.declassify.iter().cloned().collect(),
                foreign_c: Default::default(),
                foreign_python: Default::default(),
            },
        }
    }

    /// The effect names this grant confers, sorted — `effects(grant)`, the row every **Contained**
    /// export is typed at (R-1, audit F-1).
    pub fn effect_names(&self) -> Vec<String> {
        let a = self.to_authority();
        a.effects.iter().map(|e| e.name().to_string()).collect()
    }
}

fn effect_from_name(name: &str) -> Effect {
    Effect::core_from_name(name).unwrap_or_else(|| Effect::User(name.to_string()))
}

/// `std.plugin.PluginErr` (spec §4) — the in-language error sum. Every load refusal reaches the
/// program as one of these; the CLI additionally renders the registered DL code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginErr {
    NotGranted(String),
    VerifyFailed(String),
    BadArtifact(String),
    Revoked(i64),
    LimitExceeded(String),
    ApiMismatch(String),
}

impl PluginErr {
    /// The sum-variant name, as the in-language value renders.
    pub fn variant(&self) -> &'static str {
        match self {
            PluginErr::NotGranted(_) => "NotGranted",
            PluginErr::VerifyFailed(_) => "VerifyFailed",
            PluginErr::BadArtifact(_) => "BadArtifact",
            PluginErr::Revoked(_) => "Revoked",
            PluginErr::LimitExceeded(_) => "LimitExceeded",
            PluginErr::ApiMismatch(_) => "ApiMismatch",
        }
    }
}

/// The declared plugin class. **Never inferred** (invariant 29): the artifact declares it and the
/// loader verifies the declaration against the requested `C`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginClass {
    Verified,
    Contained,
}

impl PluginClass {
    pub fn as_str(self) -> &'static str {
        match self {
            PluginClass::Verified => "verified",
            PluginClass::Contained => "contained",
        }
    }

    pub fn from_str(s: &str) -> Option<PluginClass> {
        match s {
            "verified" => Some(PluginClass::Verified),
            "contained" => Some(PluginClass::Contained),
            _ => None,
        }
    }
}

/// A `.dpx` container, already parsed into plain data. Produced by a [`PluginEngine`] (implemented
/// in `delulu-wasm`, which owns the container format) and consumed by the load steps here — the
/// seam that keeps the loader in `delulu-runtime` without a dependency cycle.
#[derive(Clone, Debug)]
pub struct PluginArtifact {
    /// The `delulu:plugin` manifest JSON.
    pub manifest: serde_json::Value,
    /// `"verified"` | `"contained"`, as declared (validated by the container reader).
    pub class: String,
    pub api: u32,
    pub dir: Option<Vec<u8>>,
    pub wasm: Option<Vec<u8>>,
    pub sig: Option<Vec<u8>>,
    pub lock: Option<Vec<u8>>,
    /// Verified only: whether the `delulu:wasm` compilation cache matched its content binding. An
    /// invalid cache is ignored and recompiled from DIR (spec §3.2) — never an error.
    pub wasm_cache_valid: bool,
}

impl PluginArtifact {
    pub fn name(&self) -> &str {
        self.manifest.get("name").and_then(|v| v.as_str()).unwrap_or("<unnamed>")
    }

    pub fn version(&self) -> &str {
        self.manifest.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0")
    }

    /// The `[plugin.authority]` hard ceiling as an `Authority` (Stage-1 §8.1): the most a host may
    /// ever grant this plugin. Missing/garbled fields read as *empty* — fail closed: an unreadable
    /// ceiling grants nothing, it never defaults to permissive.
    pub fn ceiling(&self) -> Authority {
        let a = self.manifest.get("authority");
        let list = |key: &str| -> Vec<String> {
            a.and_then(|a| a.get(key))
                .and_then(|v| v.as_array())
                .map(|xs| xs.iter().filter_map(|x| x.as_str().map(str::to_string)).collect())
                .unwrap_or_default()
        };
        Authority {
            effects: list("effects").iter().map(|e| effect_from_name(e)).collect(),
            scopes: Scopes {
                fs_read: list("fs_read").into_iter().collect(),
                fs_write: list("fs_write").into_iter().collect(),
                net: list("net").into_iter().collect(),
                secrets: list("secrets").into_iter().collect(),
                declassify: list("declassify").into_iter().collect(),
                foreign_c: Default::default(),
                foreign_python: Default::default(),
            },
        }
    }

    /// The manifest's declared exports (name → signature string). For a **Contained** plugin these
    /// are documentation only and never enter the type system (R-1).
    pub fn exports(&self) -> BTreeMap<String, String> {
        self.manifest
            .get("exports")
            .and_then(|v| v.as_object())
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// A structured load refusal: the registered code, the human message, and — for DL1502 — the
/// **intersection**, which is the exact, never-widening repair (spec §7). Converts to the
/// in-language [`PluginErr`] so a program sees a `Result` while the CLI renders a diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadRefusal {
    pub code: &'static str,
    pub message: String,
    /// DL1502 only: `grant ⊓ ceiling` — narrowing by construction (`authority_widening: false`).
    pub intersection: Option<Authority>,
    pub requires_human: bool,
}

impl LoadRefusal {
    fn new(code: &'static str, message: impl Into<String>) -> LoadRefusal {
        LoadRefusal { code, message: message.into(), intersection: None, requires_human: false }
    }

    /// The in-language error value this refusal becomes (spec §4).
    pub fn to_plugin_err(&self) -> PluginErr {
        match self.code {
            "DL1507" => PluginErr::ApiMismatch(self.message.clone()),
            "DL1502" | "DL0802" => PluginErr::NotGranted(self.message.clone()),
            "DL1504" | "DL1505" => PluginErr::VerifyFailed(self.message.clone()),
            "DL1506" => PluginErr::LimitExceeded(self.message.clone()),
            _ => PluginErr::BadArtifact(self.message.clone()),
        }
    }
}

/// The engine seam (playbook §1 / head-chef amendment): everything the loader needs that lives
/// above it in the crate graph. `delulu-wasm` implements this; the `delulu` crate injects it.
/// Phase 6d needs only the container reader; instantiation arrives in 6e/6f.
pub trait PluginEngine {
    /// Read `.dpx` bytes into a [`PluginArtifact`]. The implementation owns the container format
    /// (and its blake3 content bindings); a corrupt artifact refuses here, identically for
    /// `plugin verify` and a real load.
    fn read_artifact(&self, bytes: &[u8]) -> Result<PluginArtifact, LoadRefusal>;
}

/// The product of steps 1–4: everything decided *before* any code is verified or instantiated.
/// The `grant_id` is the plugin's fresh broker node — held **host-side, here**; it is never a value
/// the plugin can see (invariant 31).
#[derive(Clone, Debug)]
pub struct PreparedLoad {
    pub class: PluginClass,
    pub grant_id: GrantId,
    pub authority: Authority,
}

// ===== the steps, in order ===================================================================

/// **Step 1** — validate the container and `plugin.api` (DL1507). The container itself was parsed
/// by the engine; what remains is the api gate and the class declaration's well-formedness.
pub fn step1_container_api(art: &PluginArtifact, supported_api: u32) -> Result<PluginClass, LoadRefusal> {
    if art.api != supported_api {
        return Err(LoadRefusal::new(
            "DL1507",
            format!(
                "plugin `{}` declares API version {} — this runtime supports {supported_api}; rebuild the plugin",
                art.name(),
                art.api
            ),
        ));
    }
    // Invariant 29: the class is *declared*. An unreadable declaration is refused, never guessed.
    PluginClass::from_str(&art.class).ok_or_else(|| {
        LoadRefusal::new(
            "DL1508",
            format!("plugin `{}` declares unknown class `{}`", art.name(), art.class),
        )
    })
}

/// **Step 2** — the class check against the requested `C` (invariant 29). A `.dpx` whose declared
/// class differs from the one the host asked for is **refused** — it never loads as the other
/// class, and a Verified request is never silently satisfied by a Contained artifact.
pub fn step2_class(declared: PluginClass, requested: PluginClass) -> Result<(), LoadRefusal> {
    if declared != requested {
        return Err(LoadRefusal::new(
            "DL1508",
            format!(
                "artifact declares class `{}` but the host requested `Plugin[{}]` — the class is never inferred and never substituted",
                declared.as_str(),
                requested.as_str()
            ),
        ));
    }
    Ok(())
}

/// **Step 3** — the manifest ceiling: `grant ⊑ plugin.authority` (DL1502). The host may grant
/// *less* than the ceiling, never more. Reuses the broker's own `⊑` lattice, so the refusal carries
/// the **intersection** — the exact, narrowing repair (`authority_widening: false`), never wider
/// than either side by construction.
pub fn step3_ceiling(grant: &Authority, ceiling: &Authority) -> Result<(), LoadRefusal> {
    match attenuation_check(grant, ceiling) {
        Ok(()) => Ok(()),
        Err(intersection) => Err(LoadRefusal {
            code: "DL1502",
            message: format!(
                "the requested grant exceeds the plugin's declared ceiling — requested {}; ceiling {}; the host may grant less than the ceiling, never more",
                grant.render_compact(),
                ceiling.render_compact()
            ),
            intersection: Some(intersection),
            requires_human: false,
        }),
    }
}

/// **Step 4** — the holder check: `attenuate(host's node, grant)` at the broker (R-7). On success a
/// **child node** with a **fresh `GrantId`** (the R-6c binding, invariant 28); a request wider than
/// the holder's own grant is **DL0802**. Routed through [`Custody`], so it works identically in
/// embedded and daemon custody.
pub fn step4_holder(
    custody: &mut dyn Custody,
    authority: Authority,
    plugin_name: &str,
) -> Result<GrantId, LoadRefusal> {
    let holder = Holder::new("plugin", plugin_name, "");
    custody.attenuate(authority, holder).map_err(|d: CustodyDenial| LoadRefusal {
        code: d.code,
        message: d.message,
        intersection: None,
        requires_human: false,
    })
}

/// Steps 1–4 in the **normative order** (spec §3.1). Stops before class verification (step 5) —
/// phases 6e/6f continue from here.
///
/// The order is load-bearing and observable: a grant violating both the ceiling *and* the holder's
/// node reports DL1502, because step 3 runs before step 4.
pub fn load_prepare(
    art: &PluginArtifact,
    requested: PluginClass,
    grant: &Grant,
    custody: &mut dyn Custody,
) -> Result<PreparedLoad, LoadRefusal> {
    let declared = step1_container_api(art, PLUGIN_API_SUPPORTED)?;
    step2_class(declared, requested)?;
    let authority = grant.to_authority();
    step3_ceiling(&authority, &art.ceiling())?;
    let grant_id = step4_holder(custody, authority.clone(), art.name())?;
    Ok(PreparedLoad { class: declared, grant_id, authority })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::custody::EmbeddedCustody;
    use serde_json::json;

    fn artifact(class: &str, api: u32, ceiling_effects: &[&str]) -> PluginArtifact {
        PluginArtifact {
            manifest: json!({
                "name": "summarize",
                "version": "0.1.0",
                "api": api,
                "class": class,
                "authority": { "effects": ceiling_effects, "requires": [], "fs_read": ["./docs"] },
                "exports": { "summarize": "fn(Str) -> Str" },
            }),
            class: class.to_string(),
            api,
            dir: Some(b"dir".to_vec()),
            wasm: None,
            sig: None,
            lock: None,
            wasm_cache_valid: true,
        }
    }

    fn grant(effects: &[&str]) -> Grant {
        Grant { effects: effects.iter().map(|s| s.to_string()).collect(), ..Default::default() }
    }

    /// A host custody holding `effects` at its own node — the holder a plugin attenuates under.
    fn host_custody(effects: &[&str]) -> EmbeddedCustody {
        let auth = Authority {
            effects: effects.iter().map(|e| effect_from_name(e)).collect(),
            scopes: Scopes { fs_read: ["./docs".to_string()].into_iter().collect(), ..Default::default() },
        };
        EmbeddedCustody::with_root(auth)
    }

    #[test]
    fn step1_rejects_an_unsupported_api_as_dl1507() {
        let art = artifact("verified", 9, &["Read"]);
        let e = step1_container_api(&art, 1).expect_err("api 9 must refuse");
        assert_eq!(e.code, "DL1507");
        assert_eq!(e.to_plugin_err().variant(), "ApiMismatch");
    }

    #[test]
    fn step1_accepts_the_supported_api_and_reads_the_declared_class() {
        let art = artifact("contained", 1, &[]);
        assert_eq!(step1_container_api(&art, 1).unwrap(), PluginClass::Contained);
    }

    #[test]
    fn step2_refuses_a_class_mismatch_and_never_substitutes() {
        // Invariant 29: asking for Verified and being handed Contained is a refusal, not a downgrade.
        let e = step2_class(PluginClass::Contained, PluginClass::Verified).expect_err("must refuse");
        assert!(e.message.contains("never inferred and never substituted"), "{}", e.message);
        assert!(step2_class(PluginClass::Verified, PluginClass::Verified).is_ok());
    }

    #[test]
    fn step3_refuses_a_grant_over_the_ceiling_with_the_intersection_as_exact_repair() {
        let art = artifact("verified", 1, &["Read"]);
        let g = grant(&["Read", "Net"]).to_authority();
        let e = step3_ceiling(&g, &art.ceiling()).expect_err("Net exceeds the ceiling");
        assert_eq!(e.code, "DL1502");
        assert_eq!(e.to_plugin_err().variant(), "NotGranted");
        let inter = e.intersection.clone().expect("DL1502 carries the intersection");
        // The repair narrows: the intersection is within BOTH sides — never wider than either.
        assert!(inter.effects.contains(&Effect::Read));
        assert!(!inter.effects.contains(&Effect::Net), "the repair never widens: {inter:?}");
    }

    #[test]
    fn step3_allows_granting_less_than_the_ceiling() {
        // The host may grant less than the ceiling — that is the whole point of a ceiling.
        let art = artifact("verified", 1, &["Read", "Net"]);
        assert!(step3_ceiling(&grant(&["Read"]).to_authority(), &art.ceiling()).is_ok());
        assert!(step3_ceiling(&grant(&[]).to_authority(), &art.ceiling()).is_ok());
    }

    #[test]
    fn step4_mints_a_child_node_with_a_fresh_grant_id() {
        // Invariant 28: every load creates a child node under the holder's, with a fresh GrantId.
        let mut c = host_custody(&["Read"]);
        let host_node = c.holder_node().expect("the host holds a node");
        let id = step4_holder(&mut c, grant(&["Read"]).to_authority(), "summarize").expect("attenuates");
        assert_ne!(id, host_node, "the plugin's node is a FRESH id, never the holder's own");
        let id2 = step4_holder(&mut c, grant(&["Read"]).to_authority(), "summarize").expect("attenuates");
        assert_ne!(id, id2, "every load mints a fresh GrantId (the R-6c binding)");
    }

    #[test]
    fn step4_refuses_a_grant_wider_than_the_holder_as_dl0802() {
        // R-7: no grantee exceeds its grantor — the Stage-5 tree doing its job.
        let mut c = host_custody(&["Read"]);
        let e = step4_holder(&mut c, grant(&["Read", "Net"]).to_authority(), "greedy")
            .expect_err("Net exceeds the holder's grant");
        assert_eq!(e.code, "DL0802");
        assert_eq!(e.to_plugin_err().variant(), "NotGranted");
    }

    #[test]
    fn ceiling_is_checked_before_the_holder_step_order_matters() {
        // THE ORDERING WITNESS. This grant violates BOTH the manifest ceiling (step 3) and the
        // holder's node (step 4). The spec's sequence is an ORDER, not a set: step 3 fires first,
        // so the refusal is DL1502 — never DL0802.
        let art = artifact("verified", 1, &["Read"]); // ceiling: {Read}
        let mut c = host_custody(&["Read"]); // holder:  {Read}
        let g = grant(&["Read", "Net"]); // violates both
        let e = load_prepare(&art, PluginClass::Verified, &g, &mut c).expect_err("must refuse");
        assert_eq!(e.code, "DL1502", "step 3 (ceiling) precedes step 4 (holder): {}", e.message);
        assert!(e.intersection.is_some(), "and it carries the ceiling intersection");
    }

    #[test]
    fn api_is_checked_before_the_class_step_order_matters() {
        // A second ordering fact: a bad api on a class-mismatched artifact reports DL1507 (step 1),
        // not the step-2 class refusal.
        let art = artifact("contained", 9, &[]);
        let mut c = host_custody(&["Read"]);
        let e = load_prepare(&art, PluginClass::Verified, &grant(&[]), &mut c).expect_err("must refuse");
        assert_eq!(e.code, "DL1507", "step 1 (api) precedes step 2 (class)");
    }

    #[test]
    fn load_prepare_runs_the_happy_path_to_a_child_node() {
        let art = artifact("verified", 1, &["Read"]);
        let mut c = host_custody(&["Read"]);
        let p = load_prepare(&art, PluginClass::Verified, &grant(&["Read"]), &mut c).expect("prepares");
        assert_eq!(p.class, PluginClass::Verified);
        assert!(p.authority.effects.contains(&Effect::Read));
        // The node is live and is a child of the host's node.
        assert!(c.broker().expect("tree").inspect(&p.grant_id).is_some());
    }

    #[test]
    fn a_failed_prepare_mints_no_node() {
        // Refusal honesty: a refused load leaves no grant node behind.
        let art = artifact("verified", 1, &["Read"]);
        let mut c = host_custody(&["Read"]);
        let before = c.broker().expect("tree").len();
        let _ = load_prepare(&art, PluginClass::Verified, &grant(&["Read", "Net"]), &mut c).expect_err("refuses");
        assert_eq!(c.broker().expect("tree").len(), before, "a ceiling refusal must mint no node");
    }

    #[test]
    fn effect_names_are_the_contained_export_row_r1() {
        // R-1: `effects(grant)` is the row every Contained export types at.
        assert_eq!(grant(&["Net", "Read"]).effect_names(), vec!["Read", "Net"], "sorted, canonical");
    }

    #[test]
    fn an_unreadable_ceiling_grants_nothing_fail_closed() {
        // A garbled/missing [plugin.authority] must read as EMPTY, never as permissive.
        let mut art = artifact("verified", 1, &["Read"]);
        art.manifest["authority"] = json!("not a table");
        let ceiling = art.ceiling();
        assert!(ceiling.effects.is_empty(), "an unreadable ceiling confers nothing");
        assert!(step3_ceiling(&grant(&["Read"]).to_authority(), &ceiling).is_err());
    }
}
