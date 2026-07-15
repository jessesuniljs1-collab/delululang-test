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
use delulu_check::check::lower_export_signature;
use delulu_check::resolve::{resolve, DeclTable};
use delulu_check::ty::Type;
use delulu_check::Effect;
use delulu_syntax::parse_type_string;

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

// ===== the Contained import slice (DL1505, spec §3.1 step 5-Contained) =======================

/// A wasm value type — enough to match an import's signature exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum WasmVal {
    I32,
    I64,
    F32,
    F64,
}

/// One host import's exact signature. Name matching alone is **not** sufficient (see
/// [`cap_slice`]), so the slice carries types and step 5 checks them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportSig {
    pub params: Vec<WasmVal>,
    pub results: Vec<WasmVal>,
}

/// The set of imports a Contained module may declare: `(namespace, name) → exact signature`.
pub type ImportSlice = BTreeMap<(String, String), ImportSig>;

/// Derive the `delulu:cap` slice from a grant (spec §3.1 step 5-Contained). **A whitelist**: only
/// names derivable from the grant are permitted; everything else is DL1505. Never a blacklist, and
/// never "an unknown namespace defaults to harmless" — that is how WASI walks in the back door.
///
/// Two properties worth stating, because both are invariants rather than conveniences:
///
/// - **No `root_*` constructors, ever.** A plugin receives capability *values* from its host; it
///   never holds a `Root`, so it can never mint a capability. The cap-minting imports
///   (`root_console`, `root_fs_read`, …) are therefore absent from every slice at every grant —
///   invariant 31's shape again: the operation does not exist in the plugin's world.
/// - **Signatures are part of the slice.** A module may declare a granted *name* with a signature
///   that suits it; if step 5 matched on name alone, the only thing standing between that and
///   reality would be the engine's instantiation type-check. That is "probably refused downstream",
///   and this stage does not accept that standard for a security rule.
pub fn cap_slice(grant: &Grant) -> ImportSlice {
    use WasmVal::{I32, I64};
    let mut slice = ImportSlice::new();
    let effects = grant.to_authority().effects;
    let mut add = |name: &str, params: Vec<WasmVal>, results: Vec<WasmVal>| {
        slice.insert(("delulu:cap".to_string(), name.to_string()), ImportSig { params, results });
    };
    // Each arm mirrors the host function `delulu-wasm::host` actually provides for that effect —
    // the cap OPERATION only, never the Root constructor that mints the capability.
    for e in &effects {
        match e {
            Effect::Write => add("console_println", vec![I32, I32, I32, I32, I32], vec![]),
            Effect::Read => add("fs_read_text", vec![I32, I32, I32, I32, I32], vec![I32]),
            Effect::Clock => add("clock_now_ms", vec![I32, I32, I32, I32], vec![I64]),
            Effect::Rand => add("rand_int", vec![I32, I64, I64, I32, I32, I32], vec![I64]),
            // Net/Declassify/Load/ForeignCall/user effects have no Contained host import in v0.6: a
            // module granted them still imports nothing for them, so it can reach nothing by them.
            // Silence here is fail-closed — an unlisted effect grants no import.
            _ => {}
        }
    }
    slice
}

/// The engine seam (playbook §1 / head-chef amendment): everything the loader needs that lives
/// above it in the crate graph. `delulu-wasm` implements this; the `delulu` crate injects it.
pub trait PluginEngine {
    /// Read `.dpx` bytes into a [`PluginArtifact`]. The implementation owns the container format
    /// (and its blake3 content bindings); a corrupt artifact refuses here, identically for
    /// `plugin verify` and a real load.
    fn read_artifact(&self, bytes: &[u8]) -> Result<PluginArtifact, LoadRefusal>;

    /// **Step 5 (Contained)** — validate the module's imports against the grant-derived slice
    /// (DL1505). Decides name **and** type; a module whose imports do not fit its slice is refused
    /// here, before any instantiation. The engine's own instantiation type-check is *not* the gate.
    fn validate_imports(&self, wasm: &[u8], slice: &ImportSlice) -> Result<(), LoadRefusal>;
}

/// The DL1505 refusal, in one place so every path words it identically.
pub fn dl1505(detail: impl Into<String>) -> LoadRefusal {
    LoadRefusal {
        code: "DL1505",
        message: format!(
            "contained module imports outside its grant slice: {} — an opaque module may import only what its grant derives",
            detail.into()
        ),
        intersection: None,
        requires_human: true,
    }
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

// ===== step 5 — class-specific verification ==================================================

/// A Verified plugin's re-proved content: the replayed DIR plus the declaration table its exports
/// were lowered against. Produced only when the **whole** §5-Verified check passed.
#[derive(Debug)]
pub struct VerifiedPlugin {
    pub dir: delulu_check::Dir,
    pub table: DeclTable,
}

/// **Step 5 (Verified)** — replay-check the DIR in full, then check every export's verified row
/// against its manifest string (spec §3.1). Any failure is **DL1504**: the node is revoked and
/// nothing is instantiated, and — invariant 29 — it **never** falls back to Contained.
///
/// The replay is `delulu_check::dir::verify`: the *same* `resolve` + `check_module` the compiler
/// ran (the Deviation-1-approved construction), so a plugin whose code is unsound, stale, or lies
/// about its authority is refuted by the checker itself, not by a second implementation.
///
/// The export check is `verified_row ⊆ manifest_row` with **exact** parameter/return types. It is
/// deliberately *weaker* than the build-time fence (which demands equality, DL1501): at build the
/// author is held to an exact manifest; at load, soundness only requires that the code cannot
/// exceed what the manifest advertises to the host. Code narrower than its manifest is safe.
pub fn step5_verified(art: &PluginArtifact) -> Result<VerifiedPlugin, LoadRefusal> {
    let Some(dir_bytes) = art.dir.as_deref() else {
        return Err(LoadRefusal {
            code: "DL1504",
            message: format!("plugin `{}` declares class `verified` but carries no DIR", art.name()),
            intersection: None,
            requires_human: true,
        });
    };

    // The full replay — types, rows, R-rules, opacity: the whole Stage-1 §6 judgment, re-run.
    let dir = delulu_check::dir::verify(dir_bytes).map_err(|e| LoadRefusal {
        code: e.code(),
        message: format!("plugin `{}`: {}", art.name(), e.message()),
        intersection: None,
        requires_human: e.requires_human(),
    })?;

    // `dir::verify` already proved this module resolves and checks clean, so re-deriving its table
    // is deterministic and diagnostic-free; it is what the manifest signatures lower against.
    let (table, _diags) = resolve(&dir.module);

    for (name, sig_str) in art.exports() {
        let refuse = |msg: String| LoadRefusal {
            code: "DL1504",
            message: format!("plugin `{}` export `{name}`: {msg}", art.name()),
            intersection: None,
            requires_human: true,
        };
        let Some(code_ty) = dir.fn_types.get(&name) else {
            return Err(refuse("the manifest advertises it, but the verified code declares no such function".into()));
        };
        let (te, pdiags) = parse_type_string(u32::MAX, &sig_str);
        if pdiags.iter().any(|d| d.is_error()) {
            return Err(refuse(format!("the manifest signature `{sig_str}` does not parse")));
        }
        let manifest_ty = lower_export_signature(&te, &table)
            .map_err(|e| refuse(format!("the manifest signature `{sig_str}` does not lower: {e}")))?;
        check_export_row(code_ty, &manifest_ty).map_err(refuse)?;
    }

    Ok(VerifiedPlugin { dir, table })
}

/// `verified ⊆ manifest`: identical parameter/return types, and the verified row's effects a
/// **subset** of the manifest's declared row. A code row that exceeds its manifest is the lie this
/// check exists to catch.
fn check_export_row(code: &Type, manifest: &Type) -> Result<(), String> {
    let (Type::Fn { params: cp, ret: cr, row: crow }, Type::Fn { params: mp, ret: mr, row: mrow }) =
        (code, manifest)
    else {
        return Err("the manifest signature is not a function type".into());
    };
    if cp != mp || cr != mr {
        return Err(format!("the verified type `{code}` does not match the manifest signature `{manifest}`"));
    }
    if !crow.effects.is_subset(&mrow.effects) {
        let extra: Vec<&str> =
            crow.effects.difference(&mrow.effects).map(|e| e.name()).collect();
        return Err(format!(
            "the verified code performs `{}`, which its manifest row `{mrow}` does not declare — the code exceeds what the manifest advertises",
            extra.join(", ")
        ));
    }
    Ok(())
}

// ===== R-Get — the per-class export check at `p.get` (spec §3.3, runtime) ====================
//
// The compile-time half of `get` is already done and needs no rule: `F`'s row flows into the
// caller's row through T-Call, and DL0803/DL1509 fence R-6a at the call site. What remains is the
// runtime half, here — and it is written FAIL-CLOSED throughout: every "the loader could not tell"
// case refuses, because R-Get is a security rule and an undecidable answer is not an allow.

/// Verified R-Get: `row(export) ⊆ row(F)` with **exact** parameter/return types (spec §3.3).
///
/// The export type comes from the re-proved DIR, so it is real. The three "couldn't tell" cases —
/// the export is absent, the export's stored type is not a function, or `F` is not a function —
/// all **refuse**; none of them is a shrug.
pub fn r_get_verified(export: Option<&Type>, f: &Type, name: &str) -> Result<(), PluginErr> {
    let Some(export) = export else {
        // Couldn't tell: the plugin exports no such name. Refuse — never synthesize a stub.
        return Err(PluginErr::NotGranted(format!(
            "plugin exports no `{name}` — the verified code declares no such function"
        )));
    };
    let Type::Fn { params: ep, ret: er, row: erow } = export else {
        // Couldn't tell: the stored export type is malformed (not a function). A DIR that reaches
        // here has been re-verified, so this is a can't-happen — which is exactly why it refuses
        // rather than assumes.
        return Err(PluginErr::VerifyFailed(format!(
            "export `{name}` has a non-function type `{export}` — exports are functions only"
        )));
    };
    let Type::Fn { params: fp, ret: fr, row: frow } = f else {
        return Err(PluginErr::VerifyFailed(format!(
            "`get` requires a function type; `{f}` is not one"
        )));
    };
    if ep != fp || er != fr {
        return Err(PluginErr::VerifyFailed(format!(
            "export `{name}` has type `{export}`, which does not match the requested `{f}`"
        )));
    }
    // R-Get: the export's row must fit inside the row the caller declared for it. An export that
    // performs MORE than `F` admits would perform effects no caller row accounts for.
    if !erow.effects.is_subset(&frow.effects) {
        let extra: Vec<&str> = erow.effects.difference(&frow.effects).map(|e| e.name()).collect();
        return Err(PluginErr::VerifyFailed(format!(
            "export `{name}` performs `{}`, which the requested type `{f}` does not admit — its row must cover the export's",
            extra.join(", ")
        )));
    }
    Ok(())
}

/// Contained R-Get: **R-1** — every export is typed at `effects(grant)`, whatever the manifest
/// claims (audit F-1). So `effects(grant) ⊆ row(F)`, plus a scalar/`Str`/`Cap`-parameter signature
/// match (spec §3.3).
///
/// This is the honesty keystone: a Contained plugin's "read-only" export **types as everything its
/// module was granted**, because containment is module-granular — the WASI import set derives from
/// the grant, not from which export you call. A manifest's per-export row is documentation and
/// never enters the type system.
pub fn r_get_contained(grant: &Grant, f: &Type, name: &str) -> Result<(), PluginErr> {
    let Type::Fn { params, row: frow, .. } = f else {
        return Err(PluginErr::VerifyFailed(format!(
            "`get` requires a function type; `{f}` is not one"
        )));
    };
    // R-1: the row the caller asks for must cover the WHOLE grant, not the advertised export row.
    let granted = grant.to_authority().effects;
    if !granted.is_subset(&frow.effects) {
        let missing: Vec<&str> = granted.difference(&frow.effects).map(|e| e.name()).collect();
        return Err(PluginErr::VerifyFailed(format!(
            "contained export `{name}` is typed at its module's FULL grant (rule R-1): the requested type `{f}` must admit `{}`. \
             Containment is module-granular — an opaque module's exports are not bounded per-export, so a \"read-only\" export types as everything the module was granted",
            missing.join(", ")
        )));
    }
    // The marshalling fence: only scalars, Str, and capabilities cross into an opaque module.
    // R-6a's compile-time fence (DL0803/DL1509) already rejects function-typed parameters; this
    // refuses everything else that cannot cross, fail-closed rather than by omission.
    for (i, p) in params.iter().enumerate() {
        if !is_contained_marshallable(p) {
            return Err(PluginErr::VerifyFailed(format!(
                "contained export `{name}` parameter {} has type `{p}`, which cannot cross into an opaque module (only Int, Float, Bool, Str, Unit, and Cap[_] may)",
                i + 1
            )));
        }
    }
    Ok(())
}

/// What may cross into a Contained (opaque WASM) export. Deliberately a whitelist: an unknown or
/// composite type is refused, never assumed marshallable.
fn is_contained_marshallable(t: &Type) -> bool {
    matches!(t, Type::Int | Type::Float | Type::Bool | Type::Str | Type::Unit | Type::Cap(_))
}

// ===== R-6b — host values die when the export call returns ===================================

/// The call-scoped handle table (R-6b, audit F-6).
///
/// A host value passed into a plugin export lives **only for the synchronous duration of that
/// call**. The plugin never receives the value itself — it receives an opaque `u64` handle, and
/// every use is resolved through this table. `enter` opens a call scope; `leave` invalidates every
/// handle minted in it. A plugin that squirrels a handle away and uses it on a later call finds it
/// **dead**, which is the whole point: no persistent host references across calls.
///
/// **Ids are monotonic and NEVER reused.** That is load-bearing, not tidiness: if ids restarted per
/// call, a handle retained from call *N* would silently alias a *different* value in call *N+1* —
/// an authority-confusion bug strictly worse than a dangling reference, because it would succeed.
/// `handles_are_never_reused_across_calls` witnesses it.
///
/// **"Couldn't tell" is a refusal.** An id that was never minted, one minted in an earlier scope,
/// and any id at all when no call is in flight are all refused. The table never guesses that an
/// unknown id might be host-owned.
#[derive(Debug, Default)]
pub struct HandleTable {
    /// Monotonic; never reset, never reused for the lifetime of the table.
    next_id: u64,
    /// Ids valid in the CURRENT call scope only.
    live: std::collections::BTreeSet<u64>,
    /// Whether a call is in flight. Outside a call, every id is dead.
    in_call: bool,
}

impl HandleTable {
    pub fn new() -> HandleTable {
        HandleTable::default()
    }

    /// Open a call scope. Any handle from a previous call is already dead (`leave` cleared it).
    pub fn enter(&mut self) {
        self.live.clear();
        self.in_call = true;
    }

    /// Mint a handle for a host value crossing into the plugin for this call.
    ///
    /// **DO NOT "recycle" ids for tidiness — `next_id` must never decrease or wrap.** Reusing an id
    /// is not a space optimization, it is an authority-confusion hole: a handle retained from an
    /// earlier call would then resolve to a *different* live value and the call would **succeed**.
    /// A security bug that succeeds is worse than one that fails. `handles_are_never_reused_across_calls`
    /// exists to fail the moment anyone tries.
    pub fn mint(&mut self) -> u64 {
        let id = self.next_id;
        // Monotonic: an id is never handed out twice, so a stale handle can never alias a live one.
        self.next_id += 1;
        self.live.insert(id);
        id
    }

    /// Resolve a handle the plugin presented. Fail-closed: the three "couldn't tell" cases —
    /// no call in flight, an id never minted, an id from an earlier call — all refuse.
    pub fn resolve(&self, id: u64) -> Result<(), PluginErr> {
        if !self.in_call {
            return Err(PluginErr::Revoked(0));
        }
        if !self.live.contains(&id) {
            // Either forged, or retained across the call boundary. Both are R-6b violations, and
            // we deliberately do NOT distinguish them to the plugin: it learns only "dead".
            return Err(PluginErr::Revoked(0));
        }
        Ok(())
    }

    /// Close the call scope — **invalidate every host value passed into this call** (R-6b). Called
    /// on return, including on a faulting/trapping return: a plugin must never keep a live host
    /// handle by failing.
    pub fn leave(&mut self) {
        self.live.clear();
        self.in_call = false;
    }

    /// How many handles are live right now (host-side introspection/tests).
    pub fn live_count(&self) -> usize {
        self.live.len()
    }
}

/// The result of a completed load: the plugin's node and its class-specific content.
#[derive(Debug)]
pub enum LoadedPlugin {
    Verified { grant_id: GrantId, authority: Authority, verified: Box<VerifiedPlugin> },
}

/// Steps 1–5 for a **Verified** plugin, in the normative order, with the refusal discipline the
/// spec demands: if step 5 fails, the plugin's node is **revoked** and nothing is instantiated.
///
/// A step-5 failure never degrades the request to Contained (invariant 29) — the only outcomes are
/// a fully re-proved Verified plugin or a DL1504-class refusal.
pub fn load_verified(
    art: &PluginArtifact,
    grant: &Grant,
    custody: &mut dyn Custody,
) -> Result<LoadedPlugin, LoadRefusal> {
    let prepared = load_prepare(art, PluginClass::Verified, grant, custody)?;
    match step5_verified(art) {
        Ok(verified) => Ok(LoadedPlugin::Verified {
            grant_id: prepared.grant_id,
            authority: prepared.authority,
            verified: Box::new(verified),
        }),
        Err(e) => {
            // The node was minted at step 4; a failed verification must not leave it alive. The
            // revoke is best-effort in the sense that its own failure cannot rescue the load — the
            // refusal stands either way, and it is the refusal the caller sees.
            let _ = custody.revoke_node(&prepared.grant_id);
            Err(e)
        }
    }
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

    // ----- step 5 (Verified): the replay + the export row check ------------------------------

    /// A Verified artifact carrying real DIR built from `code`, with `exports` in its manifest.
    fn verified_artifact(code: &str, exports: serde_json::Value, ceiling: &[&str]) -> PluginArtifact {
        let c = delulu_check::check_source(0, code);
        assert!(!c.has_errors(), "test plugin code must check clean: {:?}", c.diagnostics);
        let dir = delulu_check::dir_serialize(&c.module, &c.result);
        PluginArtifact {
            manifest: json!({
                "name": "p", "version": "0.1.0", "api": 1, "class": "verified",
                "authority": { "effects": ceiling, "requires": [] },
                "exports": exports,
            }),
            class: "verified".into(),
            api: 1,
            dir: Some(dir),
            wasm: None,
            sig: None,
            lock: None,
            wasm_cache_valid: true,
        }
    }

    const PURE_CODE: &str = "module p\npub fn shout(s: Str) -> Str { s }\n";

    #[test]
    fn step5_verified_accepts_an_honest_plugin() {
        let art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        let v = step5_verified(&art).expect("an honest Verified plugin re-checks");
        assert!(v.dir.fn_types.contains_key("shout"));
    }

    #[test]
    fn step5_verified_refuses_a_tampered_dir_as_dl1504_requires_human() {
        // The DIR replay is the trust anchor: corrupt bytes cannot be re-checked → DL1504.
        let mut art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        art.dir = Some(b"not a dir at all".to_vec());
        let e = step5_verified(&art).expect_err("must refuse");
        assert_eq!(e.code, "DL1504");
        assert!(e.requires_human, "DL1504 is requires_human and never falls back to Contained");
    }

    #[test]
    fn step5_verified_refuses_code_whose_row_exceeds_its_manifest() {
        // The export check's whole purpose: the code Writes but the manifest advertises purity.
        let code = "module p\npub fn shout(out: Cap[Console], s: Str) ! {Write} { out.println(s) }\n";
        let art = verified_artifact(code, json!({ "shout": "fn(Cap[Console], Str)" }), &["Write"]);
        let e = step5_verified(&art).expect_err("the code exceeds its manifest row");
        assert_eq!(e.code, "DL1504");
        assert!(e.message.contains("Write"), "{}", e.message);
        assert!(e.message.contains("exceeds what the manifest advertises"), "{}", e.message);
    }

    #[test]
    fn step5_verified_allows_code_narrower_than_its_manifest() {
        // `verified ⊆ manifest`: a plugin that advertises Write but is actually pure is SAFE.
        // (The build-time fence demands equality — DL1501 — but load-time soundness needs only ⊆.)
        let code = "module p\npub fn shout(out: Cap[Console], s: Str) { }\n";
        let art = verified_artifact(code, json!({ "shout": "fn(Cap[Console], Str) ! {Write}" }), &["Write"]);
        assert!(step5_verified(&art).is_ok(), "code narrower than its manifest is safe");
    }

    #[test]
    fn step5_verified_refuses_a_manifest_export_the_code_lacks() {
        let art = verified_artifact(PURE_CODE, json!({ "ghost": "fn(Str) -> Str" }), &[]);
        let e = step5_verified(&art).expect_err("must refuse");
        assert_eq!(e.code, "DL1504");
        assert!(e.message.contains("no such function"), "{}", e.message);
    }

    #[test]
    fn step5_verified_refuses_a_type_mismatched_export() {
        let art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Int) -> Int" }), &[]);
        let e = step5_verified(&art).expect_err("must refuse");
        assert_eq!(e.code, "DL1504");
        assert!(e.message.contains("does not match the manifest signature"), "{}", e.message);
    }

    #[test]
    fn a_verified_class_never_falls_back_when_step5_fails_and_the_node_is_revoked() {
        // Trap 1 / invariant 29 + the spec's "node revoked, nothing instantiated".
        let mut art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        art.dir = Some(b"garbage".to_vec());
        let mut c = host_custody(&["Read"]);
        let e = load_verified(&art, &grant(&[]), &mut c).expect_err("must refuse");
        assert_eq!(e.code, "DL1504");
        // The node minted at step 4 must be dead — a failed verification leaves nothing alive.
        let tree = c.broker().expect("tree");
        let live: Vec<_> = tree
            .nodes()
            .iter()
            .filter(|n| n.holder.kind == "plugin" && matches!(tree.effective_state(&n.id), Some(delulu_broker::EffState::Live)))
            .map(|n| n.id.clone())
            .collect();
        assert!(live.is_empty(), "a DL1504 refusal must revoke the plugin's node: {live:?}");
    }

    #[test]
    fn load_verified_happy_path_keeps_the_node_live() {
        let art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        let mut c = host_custody(&["Read"]);
        let loaded = load_verified(&art, &grant(&[]), &mut c).expect("loads");
        let LoadedPlugin::Verified { grant_id, .. } = loaded;
        assert_eq!(
            c.broker().expect("tree").effective_state(&grant_id),
            Some(delulu_broker::EffState::Live),
            "a successful load leaves the plugin's node live"
        );
    }

    #[test]
    fn a_zero_authority_plugin_loads_and_its_node_confers_nothing() {
        // The flagship's load-side shape (criterion 1): Grant { effects: [] } → a live node that
        // confers no effect at all.
        let art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        let mut c = host_custody(&["Read", "Net"]);
        let loaded = load_verified(&art, &grant(&[]), &mut c).expect("loads");
        let LoadedPlugin::Verified { grant_id, authority, .. } = loaded;
        assert!(authority.effects.is_empty(), "a zero-authority grant confers nothing");
        let node = c.broker().expect("tree").inspect(&grant_id).expect("node exists");
        assert!(node.authority.effects.is_empty(), "and the broker node agrees");
    }

    // ----- R-Get (runtime), and its "couldn't tell" cases -------------------------------------
    //
    // Kitchen rule: the "what if the checker couldn't tell" case is written FIRST. The skip branch
    // is where security rules go to die.

    fn fnty(params: Vec<Type>, ret: Type, effects: &[&str]) -> Type {
        Type::Fn {
            params,
            ret: Box::new(ret),
            row: delulu_check::ty::Row::closed(effects.iter().map(|e| effect_from_name(e)).collect()),
        }
    }

    #[test]
    fn r_get_verified_couldnt_tell_cases_all_refuse() {
        let f = fnty(vec![Type::Str], Type::Str, &[]);
        // (1) the export is ABSENT — refuse, never synthesize a stub.
        let e = r_get_verified(None, &f, "shout").expect_err("absent export must refuse");
        assert_eq!(e.variant(), "NotGranted");
        // (2) the export's stored type is MALFORMED (not a function) — refuse, never assume.
        let e = r_get_verified(Some(&Type::Int), &f, "shout").expect_err("non-fn export must refuse");
        assert_eq!(e.variant(), "VerifyFailed");
        // (3) `F` itself is not a function — refuse.
        let export = fnty(vec![Type::Str], Type::Str, &[]);
        let e = r_get_verified(Some(&export), &Type::Int, "shout").expect_err("non-fn F must refuse");
        assert_eq!(e.variant(), "VerifyFailed");
    }

    #[test]
    fn r_get_verified_enforces_row_subset_and_exact_types() {
        // Exact types + row(export) ⊆ row(F).
        let export = fnty(vec![Type::Str], Type::Str, &["Read"]);
        // The requested row admits Read: OK.
        assert!(r_get_verified(Some(&export), &fnty(vec![Type::Str], Type::Str, &["Read"]), "s").is_ok());
        // A WIDER requested row is fine — the export's row still fits inside it.
        assert!(r_get_verified(Some(&export), &fnty(vec![Type::Str], Type::Str, &["Read", "Net"]), "s").is_ok());
        // A NARROWER requested row is the refusal: the export would perform Read that no caller
        // row accounts for.
        let e = r_get_verified(Some(&export), &fnty(vec![Type::Str], Type::Str, &[]), "s")
            .expect_err("a pure F cannot receive a Read export");
        assert!(e.variant() == "VerifyFailed");
        // Type mismatch refuses regardless of rows.
        assert!(r_get_verified(Some(&export), &fnty(vec![Type::Int], Type::Str, &["Read"]), "s").is_err());
    }

    #[test]
    fn r_get_contained_couldnt_tell_cases_all_refuse() {
        let g = grant(&[]);
        // `F` is not a function — refuse rather than shrug.
        let e = r_get_contained(&g, &Type::Int, "scan").expect_err("non-fn F must refuse");
        assert_eq!(e.variant(), "VerifyFailed");
        // A parameter that cannot cross the boundary — refused by whitelist, not by omission.
        let e = r_get_contained(&g, &fnty(vec![Type::List(Box::new(Type::Int))], Type::Str, &[]), "scan")
            .expect_err("List cannot cross into an opaque module");
        assert_eq!(e.variant(), "VerifyFailed");
    }

    #[test]
    fn r_get_contained_types_every_export_at_the_full_grant_r1() {
        // THE AUDIT F-1 SCENARIO, at the R-Get gate (criterion 2). The manifest advertises
        // `scan : fn() -> Str ! {Read}` — an advisory lie. Under a {Read, Net} grant, R-1 types the
        // export at the module's FULL grant, so asking for `!{Read}` is REFUSED.
        let g = grant(&["Read", "Net"]);
        let e = r_get_contained(&g, &fnty(vec![], Type::Str, &["Read"]), "scan")
            .expect_err("R-1: a {Read,Net}-granted module's export is not `!{Read}`");
        assert_eq!(e.variant(), "VerifyFailed");
        assert!(e.variant() == "VerifyFailed");
        let PluginErr::VerifyFailed(msg) = &e else { unreachable!() };
        assert!(msg.contains("Net"), "the refusal names the effect the request omits: {msg}");
        assert!(msg.contains("R-1"), "and cites the rule: {msg}");

        // The HONEST annotation — covering the whole grant — succeeds. The program then only
        // compiles if the caller's row includes Net (that half is compile-time, T-Call).
        assert!(
            r_get_contained(&g, &fnty(vec![], Type::Str, &["Read", "Net"]), "scan").is_ok(),
            "the honest annotation covering the full grant is accepted"
        );
    }

    #[test]
    fn r_get_contained_on_a_zero_authority_grant_admits_a_pure_export() {
        // The flagship's Contained mirror: nothing granted ⇒ `effects(grant)` is empty ⇒ a pure `F`
        // covers it. The plugin still cannot read, clock, or net — it was granted nothing.
        assert!(r_get_contained(&grant(&[]), &fnty(vec![Type::Str], Type::Str, &[]), "shout").is_ok());
    }

    #[test]
    fn r_get_contained_allows_only_scalars_str_and_caps_across_the_boundary() {
        let g = grant(&[]);
        for ok in [Type::Int, Type::Float, Type::Bool, Type::Str, Type::Unit, Type::Cap(delulu_check::ResourceKind::FsRead)] {
            assert!(
                r_get_contained(&g, &fnty(vec![ok.clone()], Type::Str, &[]), "f").is_ok(),
                "{ok} must be allowed to cross"
            );
        }
        for bad in [
            Type::List(Box::new(Type::Int)),
            Type::Option(Box::new(Type::Int)),
            Type::Secret(Box::new(Type::Str)),
            Type::Root,
        ] {
            assert!(
                r_get_contained(&g, &fnty(vec![bad.clone()], Type::Str, &[]), "f").is_err(),
                "{bad} must not cross into an opaque module"
            );
        }
    }

    // ----- R-6b: host values die on return, and "couldn't tell" refuses -----------------------

    #[test]
    fn r6b_a_handle_retained_across_the_call_boundary_is_dead() {
        // THE R-6b PROPERTY (audit F-6): a host value passed into an export lives only for the
        // synchronous duration of that call.
        let mut t = HandleTable::new();
        t.enter();
        let h = t.mint();
        assert!(t.resolve(h).is_ok(), "live during its own call");
        t.leave();
        assert!(t.resolve(h).is_err(), "dead the instant the export returns");
        // And still dead inside a LATER call — the plugin cannot revive it by calling again.
        t.enter();
        assert!(t.resolve(h).is_err(), "a retained handle stays dead in a later call");
    }

    #[test]
    fn r6b_couldnt_tell_cases_all_refuse() {
        // The head-chef question: what if the table cannot tell a value is host-owned?
        let mut t = HandleTable::new();
        // (1) No call in flight — every id is dead, including plausible ones.
        assert!(t.resolve(0).is_err(), "outside a call, nothing resolves");
        t.enter();
        // (2) An id that was NEVER minted (forged out of thin air) — refused, never guessed.
        assert!(t.resolve(9_999).is_err(), "a forged id is never assumed host-owned");
        // (3) A plausible-looking id adjacent to a live one — still refused.
        let h = t.mint();
        assert!(t.resolve(h + 1).is_err(), "adjacency confers nothing");
        assert!(t.resolve(h).is_ok());
    }

    #[test]
    fn handles_are_never_reused_across_calls() {
        // Load-bearing, not tidiness: if ids restarted per call, a handle retained from call N
        // would ALIAS a different value in call N+1 — an authority confusion that would SUCCEED,
        // which is strictly worse than a dangling reference that fails.
        let mut t = HandleTable::new();
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..5 {
            t.enter();
            for _ in 0..3 {
                let h = t.mint();
                assert!(seen.insert(h), "handle {h} was reused across calls — R-6b aliasing hole");
            }
            t.leave();
        }
        assert_eq!(seen.len(), 15);
    }

    #[test]
    fn r6b_leave_invalidates_every_handle_even_on_a_faulting_return() {
        // A plugin must never keep a live host handle by failing mid-call.
        let mut t = HandleTable::new();
        t.enter();
        let a = t.mint();
        let b = t.mint();
        assert_eq!(t.live_count(), 2);
        t.leave(); // as on a fault/trap return
        assert_eq!(t.live_count(), 0, "a faulting return still invalidates every host value");
        assert!(t.resolve(a).is_err() && t.resolve(b).is_err());
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
