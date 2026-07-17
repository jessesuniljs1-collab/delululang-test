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

use crate::custody::{Custody, CustodyDenial, Liveness};
use crate::interp::{Interp, DEFAULT_INTERP_MEM_BYTES, DEFAULT_INTERP_STEPS};
use crate::value::{Fault, Value};

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

    /// Parse a declared class name. Named `from_name` (not `from_str`) so it is not mistaken for
    /// `std::str::FromStr::from_str`, whose `Result` contract this `Option`-returning parser does not
    /// follow (clippy `should_implement_trait`).
    pub fn from_name(s: &str) -> Option<PluginClass> {
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
    /// **Boxed** so a `LoadRefusal` stays small: it is the `Err` of every load `Result`, and an
    /// `Authority` inline would bloat every one (clippy `result_large_err`). Mirrors the broker's
    /// own `Denial::intersection: Box<Authority>`.
    pub intersection: Option<Box<Authority>>,
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
            // DL1511 (unsigned but required) is a grant-policy refusal — `NotGranted`.
            "DL1502" | "DL0802" | "DL1511" => PluginErr::NotGranted(self.message.clone()),
            // DL1510 (badly-signed) is a verification failure — `VerifyFailed`.
            "DL1504" | "DL1505" | "DL1510" => PluginErr::VerifyFailed(self.message.clone()),
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
    PluginClass::from_name(&art.class).ok_or_else(|| {
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
            intersection: Some(Box::new(intersection)),
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

// ===== DL1506 (trap 5) and R-6c: a plugin can die, and dead is forever =======================

/// **Trap 5 / DL1506** — terminate a plugin that exceeded its granted limits.
///
/// A limit-killed plugin is **gone, not wounded**. This is one act: the instance is dropped *here*
/// (it is taken by value, so it cannot outlive the call) **and** its grant node is revoked in the
/// same breath. The host's next observation is therefore deterministic — the plugin does not
/// exist, its authority does not exist, and nothing can be resumed. Do not try to "recover" a
/// fuel-exhausted plugin: there is nothing left to recover.
pub fn kill_on_limit<I>(
    custody: &mut dyn Custody,
    grant_id: &GrantId,
    instance: I,
    limit: &str,
) -> PluginErr {
    // The instance dies first and unconditionally — taking it by value means the caller cannot
    // keep a copy, and dropping it here means no engine state survives the refusal.
    drop(instance);
    // ...and its authority dies with it, in the same act. A revoke failure cannot resurrect the
    // plugin: the refusal stands either way, which is exactly why the result is discarded.
    let _ = custody.revoke_node(grant_id);
    PluginErr::LimitExceeded(format!(
        "plugin exceeded its granted {limit} — the instance was dropped and its grant node revoked; a limit-killed plugin is gone, not wounded"
    ))
}

/// **R-6c** — `p.unload()`: revoke the plugin's node (transitively, if it loaded sub-plugins) and
/// return the **revoking audit seq**, which every later call through a retained reference reports.
pub fn unload(custody: &mut dyn Custody, grant_id: &GrantId) -> Result<u64, PluginErr> {
    custody.revoke_node(grant_id).map_err(|d| PluginErr::BadArtifact(d.message))
}

/// A retained reference to a plugin export. **It binds the load-time `GrantId`** (R-6c) — not the
/// plugin's name, not its path — which is what makes an unload/reload authority swap impossible: a
/// reload mints a *fresh* node, so an old reference can never be silently re-bound to a plugin with
/// different authority. It stays dead forever.
#[derive(Clone, Debug)]
pub struct PluginRef {
    pub grant_id: GrantId,
    pub export: String,
}

impl PluginRef {
    pub fn new(grant_id: GrantId, export: impl Into<String>) -> PluginRef {
        PluginRef { grant_id, export: Into::into(export) }
    }

    /// The per-call re-check (R-6c). **DL0801** carries the revoking audit seq, so the error says
    /// why and when this reference's authority died. Fail-closed: a custody that cannot answer is
    /// treated exactly as revoked — an unknown node confers nothing.
    pub fn check_call(&self, custody: &dyn Custody) -> Result<(), PluginErr> {
        match custody.liveness(&self.grant_id) {
            Liveness::Live => Ok(()),
            Liveness::Revoked(seq) => Err(PluginErr::Revoked(seq as i64)),
            Liveness::Unknown => Err(PluginErr::Revoked(0)),
        }
    }
}

/// The diagnostic code a [`PluginErr`] surfaces as. `Revoked` is **DL0801** — a call through a
/// revoked plugin reference — and it is `requires_human` (spec §7: no machine repair).
pub fn plugin_err_code(e: &PluginErr) -> &'static str {
    match e {
        PluginErr::Revoked(_) => "DL0801",
        PluginErr::LimitExceeded(_) => "DL1506",
        PluginErr::ApiMismatch(_) => "DL1507",
        PluginErr::NotGranted(_) => "DL1502",
        PluginErr::VerifyFailed(_) => "DL1504",
        PluginErr::BadArtifact(_) => "DL1508",
    }
}

/// The result of a completed load: the plugin's node and its class-specific content.
#[derive(Debug)]
pub enum LoadedPlugin {
    Verified {
        grant_id: GrantId,
        authority: Authority,
        verified: Box<VerifiedPlugin>,
        /// The signer's ed25519 public-key identity (lowercase hex) if the artifact carried a valid
        /// signature, else `None` (spec §3.1 step 6). Recorded in the audit log at load.
        signer: Option<String>,
    },
}

/// Steps 1–6 for a **Verified** plugin, in the normative order, with the refusal discipline the
/// spec demands: if any step fails, the plugin's node is **revoked** and nothing is instantiated.
///
/// - **Step 5** — the DIR replay + export-row check (DL1504). A failure never degrades the request
///   to Contained (invariant 29).
/// - **Step 6** — the signature policy (spec §3.1): a present-but-invalid signature is **DL1510**,
///   an unsigned plugin under `require_signed` is **DL1511** (a *different* fault), and a valid
///   signature records the signer's identity in the audit log via [`Custody::note_plugin_signature`].
pub fn load_verified(
    art: &PluginArtifact,
    grant: &Grant,
    custody: &mut dyn Custody,
) -> Result<LoadedPlugin, LoadRefusal> {
    let prepared = load_prepare(art, PluginClass::Verified, grant, custody)?;
    // Steps 5–6 together, so a refusal at either revokes the step-4 node in one place. Neither step
    // touches custody, so it is free for the note/revoke that follows.
    let outcome = (|| {
        let verified = step5_verified(art)?;
        let status = verify_signature(&art.manifest, art.dir.as_deref(), art.sig.as_deref());
        let signer = step6_signature(&status, grant.require_signed)?;
        Ok::<_, LoadRefusal>((verified, signer))
    })();
    match outcome {
        Ok((verified, signer)) => {
            // Record the signer identity in the audit log (spec §3.1 step 6). No-op if the custody
            // has no audit sink; the identity also travels on the returned handle either way.
            if let Some(s) = &signer {
                custody.note_plugin_signature(&prepared.grant_id, s);
            }
            Ok(LoadedPlugin::Verified {
                grant_id: prepared.grant_id,
                authority: prepared.authority,
                verified: Box::new(verified),
                signer,
            })
        }
        Err(e) => {
            // The node was minted at step 4; a failed verification/signature check must not leave it
            // alive. The revoke's own failure cannot rescue the load — the refusal stands either way.
            let _ = custody.revoke_node(&prepared.grant_id);
            Err(e)
        }
    }
}

// ===== 6i — `delulu plugin verify` (steps 1, 2, 5 — identical verdicts to a load) =============

/// What `verify_plugin` reports for a well-formed artifact (spec §6). Descriptive only — a `verify`
/// never instantiates and never mints a node.
#[derive(Clone, Debug)]
pub struct VerifyReport {
    pub class: PluginClass,
    pub signature: SignatureStatus,
    /// Verified: the re-proved export names. Contained: the manifest-declared exports (documentation
    /// — a Contained export types at `effects(grant)`, R-1).
    pub exports: Vec<String>,
}

/// `delulu plugin verify` — run load-sequence **steps 1, 2, 5** (and the signature check) WITHOUT
/// instantiating and WITHOUT a grant (spec §6). It calls the EXACT functions a real load calls
/// ([`step1_container_api`], [`step5_verified`], [`verify_signature`], [`step6_signature`]), so a
/// `verify` gives **identical verdicts to a real load** — criterion 9, no verify/load divergence,
/// guaranteed by there being one code path, not two (playbook trap 3).
///
/// Steps 3/4 (ceiling/holder) and 7 (instantiate) are grant/runtime concerns and are deliberately
/// not run here — `verify` answers "does this artifact pass verification?", the grant-independent
/// half of a load. A Contained plugin's import-slice check (step 5-Contained) is grant-dependent, so
/// for Contained `verify` covers container + api + signature and leaves the slice check to a real
/// load with a grant (stated in the report, never faked).
pub fn verify_plugin(art: &PluginArtifact) -> Result<VerifyReport, LoadRefusal> {
    let class = step1_container_api(art, PLUGIN_API_SUPPORTED)?;
    let (signature, exports) = match class {
        PluginClass::Verified => {
            let v = step5_verified(art)?;
            let sig = verify_signature(&art.manifest, art.dir.as_deref(), art.sig.as_deref());
            // A present-but-invalid signature refuses at verify EXACTLY as at load (DL1510).
            // `require_signed` is a grant policy, not applicable to a grant-free verify.
            step6_signature(&sig, false)?;
            (sig, v.dir.fn_types.keys().cloned().collect())
        }
        PluginClass::Contained => {
            let sig = verify_signature(&art.manifest, art.wasm.as_deref(), art.sig.as_deref());
            step6_signature(&sig, false)?;
            (sig, art.exports().keys().cloned().collect())
        }
    };
    Ok(VerifyReport { class, signature, exports })
}

// ===== 6e.5 — Verified interpreter instantiation (the flagship RUNS) ==========================
//
// A Verified plugin ships DIR — re-proved source-grade semantics — and is executed by the HOST's
// own engine (spec §5.2). On the interpreter host, "instantiate" means: build an [`Interp`] over the
// re-verified `dir.module` and call an export through it. Because `step5_verified` re-ran the whole
// checker over that exact module at load, the interpreter's well-typedness assumption holds — so a
// runtime failure is a *defined* DL09xx fault, never UB and (crucially, per the kitchen rule) never
// a panic and never a silent `Unit`.
//
// **Verified on the WASM engine** (spec §5.2, playbook 6h) compiles the DIR to a module and runs it
// through `delulu-wasm`'s `run_contained_export` — the SAME limits machinery as a Contained plugin,
// which on Windows returns an honest `EnforcementUnsupported` refusal *before any store exists*
// (build-order deviation 7). So a Verified-on-WASM run inherits that refusal by construction; the
// interpreter path below is unaffected by it and runs on every platform (a Verified plugin was
// re-proved safe by type, so the interpreter is a legitimate engine for it — spec §5.4).

/// The effective best-effort interpreter limits for a load (§5.4): `0` → profile default, **never
/// "unlimited"**. `fuel` → a step budget; `mem_mb` → an allocation-accounting byte budget.
fn effective_interp_limits(limits: &Limits) -> (u64, u64) {
    let steps = if limits.fuel > 0 { limits.fuel as u64 } else { DEFAULT_INTERP_STEPS };
    let mem_bytes = if limits.mem_mb > 0 {
        (limits.mem_mb as u64).saturating_mul(1024 * 1024)
    } else {
        DEFAULT_INTERP_MEM_BYTES
    };
    (steps, mem_bytes)
}

/// A Verified plugin instantiated on the interpreter host (spec §5.2), ready to run exports.
///
/// Holds an [`Interp`] built over the **re-proved** `dir.module` — the exact AST the checker
/// validated and `step5_verified` re-verified at load — under a **best-effort** budget (§5.4).
/// Best-effort is not a hedge: the interpreter is a courtesy engine, not the containment boundary.
/// A *Contained* plugin never runs here (it runs on the WASM engine); this runs *Verified* plugins,
/// which were re-proved safe by type, and the budget only bounds accidental runaway.
pub struct VerifiedInterpInstance {
    interp: Interp,
    fn_types: BTreeMap<String, Type>,
}

impl VerifiedInterpInstance {
    /// Instantiate a re-proved Verified plugin on the interpreter under its granted best-effort
    /// limits. Pure construction — **no plugin code runs** until [`VerifiedInterpInstance::call_export`].
    pub fn instantiate(verified: &VerifiedPlugin, limits: &Limits) -> VerifiedInterpInstance {
        let (steps, mem_bytes) = effective_interp_limits(limits);
        let interp = Interp::new(&verified.dir.module).with_plugin_budget(steps, mem_bytes);
        VerifiedInterpInstance { interp, fn_types: verified.dir.fn_types.clone() }
    }

    /// The re-verified static type of an export (for a caller re-confirming R-Get before a call).
    pub fn export_type(&self, name: &str) -> Option<&Type> {
        self.fn_types.get(name)
    }

    /// Call a Verified export with host-provided argument values, under the best-effort budget.
    ///
    /// The budget is **reset per call** — fresh fuel/mem each invocation, so a plugin cannot starve
    /// a later call by spending an earlier one's budget. The outcome is honest and *total*:
    /// - `Ok(value)` — the export returned its result.
    /// - `Err(Limit)` — a best-effort limit (step/mem) tripped; DL1506, **LABELED best-effort**.
    /// - `Err(Faulted)` — the plugin's export hit a **defined** runtime fault (overflow, div-by-zero,
    ///   out-of-bounds): the plugin's own bug, surfaced cleanly — not a host failure, not a limit.
    /// - `Err(Unevaluable)` — the DIR verified, yet evaluation reached something the interpreter
    ///   cannot evaluate (a missing primitive, an unexpected node). **The couldn't-tell case**:
    ///   honest, **never a panic, never a silent `Unit`**.
    pub fn call_export(&self, name: &str, args: Vec<Value>) -> Result<Value, PluginRunError> {
        self.interp.reset_plugin_budget();
        self.interp.call_with(name, args).map_err(classify_run_fault)
    }
}

/// The outcome of a failed Verified-export call (§5.2/§5.4). Kept **distinct from** the load-time
/// [`PluginErr`]: loading a plugin and running one are different phases, and conflating their error
/// codes would make a message a lie (the kitchen rule — a skipped check and a violated check are
/// different faults and earn different codes).
#[derive(Clone, Debug)]
pub enum PluginRunError {
    /// A best-effort interpreter limit tripped (DL1506, spec §5.4). The message LABELS best-effort:
    /// the interpreter is not the enforcement-grade sandbox.
    Limit(Fault),
    /// The plugin's export hit a defined runtime fault (its own bug), surfaced cleanly.
    Faulted(Fault),
    /// The DIR verified, but the interpreter could not evaluate some node/primitive — the
    /// couldn't-tell case. Never a panic, never a silent `Unit`.
    Unevaluable(Fault),
}

impl PluginRunError {
    /// The registered diagnostic code this outcome surfaces as.
    pub fn code(&self) -> &'static str {
        self.fault().code
    }
    /// The human-facing message (for `Limit`, it carries the best-effort label).
    pub fn message(&self) -> &str {
        &self.fault().message
    }
    /// The underlying interpreter fault.
    pub fn fault(&self) -> &Fault {
        match self {
            PluginRunError::Limit(f) | PluginRunError::Faulted(f) | PluginRunError::Unevaluable(f) => f,
        }
    }
}

/// Classify an interpreter [`Fault`] from a plugin export call into an honest [`PluginRunError`].
///
/// This is a security-honesty distinction, so it is its own function and tested directly (the
/// kitchen rule: the couldn't-tell branch is where honesty dies):
/// - **DL1506** is a best-effort limit — the only code the interpreter budget emits (§5.4).
/// - **DL0907** is the interpreter's "unexpected / cannot evaluate" catch-all: an unknown function,
///   a missing method/primitive, a non-callable value, `?` on a non-`Result`, an unexpected node.
///   In a re-verified (well-typed) module these are *can't-happens* — so if one nonetheless occurs,
///   the honest report is "the interpreter could not evaluate this", **not** a claim about the
///   plugin's behavior and **not** a limit.
/// - Every **other** DL09xx/DL07xx/DL14xx is a **defined** runtime fault the plugin's own code hit
///   (overflow DL0901, div-by-zero DL0902, bounds DL0903, depth DL0905, an ungranted or
///   again-revoked capability): the plugin faulted, honestly, and the host learns exactly which.
fn classify_run_fault(f: Fault) -> PluginRunError {
    match f.code {
        "DL1506" => PluginRunError::Limit(f),
        "DL0907" => PluginRunError::Unevaluable(f),
        _ => PluginRunError::Faulted(f),
    }
}

// ===== 6h — ed25519 plugin signatures (spec §2.2 / §3.1 step 6) ===============================
//
// A `.dpx` may carry a `delulu:sig` section: a 32-byte ed25519 public key followed by a 64-byte
// signature over the SIGNED MESSAGE (the canonical `delulu:plugin` manifest bytes ‖ the class
// payload — DIR for Verified, the module for Contained; spec §2.2). Verification (load step 6) and
// signing (`plugin build --sign`) share [`sig_message`], so the two can never drift.
//
// **Signatures authenticate ORIGIN, not behavior** (spec §10, playbook trap 7): a signed plugin is
// not a safe plugin. v0.6 has no trust *policy* (governance, Stage 9) — a signature says only
// "whoever holds this key signed these exact bytes". The recorded identity is the public key.
//
// The kitchen rule travels to signatures: every "couldn't tell" case refuses honestly with the
// right code, and — the head-chef point — an UNSIGNED-when-required plugin (DL1511) is a DIFFERENT
// fault from a BADLY-signed one (DL1510). Reusing one code would make a message a lie.

/// Load-refusal code for a **present but invalid** signature — tampered content, a wrong key, or a
/// malformed section (build-order deviation 8). `requires_human`: a broken signature is never
/// machine-repairable, and it is refused regardless of `require_signed`.
pub const DL_SIGNATURE_INVALID: &str = "DL1510";
/// Load-refusal code for an **unsigned** plugin under a `require_signed` grant (build-order
/// deviation 8) — a *policy* refusal, deliberately a different fault from a bad signature.
pub const DL_SIGNATURE_REQUIRED: &str = "DL1511";

/// The signature-verification outcome for an artifact (spec §3.1 step 6).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignatureStatus {
    /// No `delulu:sig` section. Not a fault by itself — only under `require_signed` (DL1511).
    Unsigned,
    /// A valid signature; carries the signer's ed25519 public key as lowercase hex (the identity).
    Valid { signer: String },
    /// A signature is present but does NOT verify — tampered content, a wrong key, or a malformed
    /// section. A DIFFERENT fault from unsigned: refused (DL1510) regardless of policy, because a
    /// broken signature means the artifact is not what it claims.
    Invalid { reason: String },
}

/// The exact bytes an ed25519 signature covers (spec §2.2): the canonical `delulu:plugin` manifest
/// JSON followed by the class payload (DIR for Verified, the module for Contained). Both signing and
/// verification call THIS, so the message is constructed exactly one way.
pub fn sig_message(manifest: &serde_json::Value, payload: Option<&[u8]>) -> Vec<u8> {
    let mut msg = serde_json::to_vec(manifest).unwrap_or_default();
    if let Some(p) = payload {
        msg.extend_from_slice(p);
    }
    msg
}

/// Verify a `.dpx`'s signature (load step 6). Reads only existing artifact fields — the manifest
/// (re-serialized canonically), the class payload, and the `delulu:sig` section — so an UNSIGNED
/// artifact (every prior load path, `sig = None`) is `Unsigned` with no change in behavior.
///
/// The couldn't-tell cases each refuse honestly rather than skip: a section that is not 96 bytes, a
/// public key that is not a valid ed25519 point, and a signature that does not verify are all
/// `Invalid` with a distinct reason — never silently treated as unsigned (that would let a tampered
/// artifact through).
pub fn verify_signature(
    manifest: &serde_json::Value,
    payload: Option<&[u8]>,
    sig: Option<&[u8]>,
) -> SignatureStatus {
    let Some(sig) = sig else {
        return SignatureStatus::Unsigned;
    };
    if sig.len() != 96 {
        return SignatureStatus::Invalid {
            reason: format!(
                "signature section is {} bytes, not the expected 96 (32-byte key ‖ 64-byte signature)",
                sig.len()
            ),
        };
    }
    let key_bytes: [u8; 32] = sig[..32].try_into().expect("32 bytes");
    let sig_bytes: [u8; 64] = sig[32..].try_into().expect("64 bytes");
    let vk = match ed25519_dalek::VerifyingKey::from_bytes(&key_bytes) {
        Ok(vk) => vk,
        Err(_) => {
            return SignatureStatus::Invalid {
                reason: "the signing public key is not a valid ed25519 key".into(),
            }
        }
    };
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    let msg = sig_message(manifest, payload);
    match vk.verify_strict(&msg, &signature) {
        Ok(()) => SignatureStatus::Valid { signer: hex_lower(&key_bytes) },
        Err(_) => SignatureStatus::Invalid {
            reason: "the signature does not verify over the plugin's bytes (tampered content, or a key that did not sign it)".into(),
        },
    }
}

/// Load-sequence **step 6** (spec §3.1): apply the signature POLICY. Returns the signer identity to
/// record on success, or a refusal. The two refusals are deliberately DIFFERENT faults:
/// - **DL1510** — a present signature that does not verify. Refused regardless of `require_signed`:
///   a broken signature means the artifact is not what it claims.
/// - **DL1511** — an unsigned plugin under a `require_signed` grant (a policy refusal), distinct
///   from DL1510 so the diagnostic is honest about which fault occurred.
pub fn step6_signature(
    status: &SignatureStatus,
    require_signed: bool,
) -> Result<Option<String>, LoadRefusal> {
    match status {
        SignatureStatus::Invalid { reason } => Err(LoadRefusal {
            code: DL_SIGNATURE_INVALID,
            message: format!(
                "the plugin carries a signature that does not verify: {reason}. A badly-signed artifact is refused — this is NOT the same fault as an unsigned one"
            ),
            intersection: None,
            requires_human: true,
        }),
        SignatureStatus::Unsigned if require_signed => Err(LoadRefusal {
            code: DL_SIGNATURE_REQUIRED,
            message:
                "this grant sets `require_signed`, but the plugin carries no signature — sign it (`plugin build --sign`) or clear `require_signed`. (An unsigned plugin is a different fault from a badly-signed one.)"
                    .into(),
            intersection: None,
            requires_human: true,
        }),
        SignatureStatus::Unsigned => Ok(None),
        SignatureStatus::Valid { signer } => Ok(Some(signer.clone())),
    }
}

/// Sign a plugin's bytes with an ed25519 private key (the `plugin build --sign` helper). `key_seed`
/// is a raw 32-byte ed25519 seed; the returned section is `public key (32) ‖ signature (64)` — the
/// exact `delulu:sig` layout [`verify_signature`] reads. Signs the SAME message verification checks
/// ([`sig_message`]), so signing and verification can never disagree.
pub fn sign_plugin(key_seed: &[u8; 32], manifest: &serde_json::Value, payload: Option<&[u8]>) -> Vec<u8> {
    use ed25519_dalek::Signer;
    let sk = ed25519_dalek::SigningKey::from_bytes(key_seed);
    let signature = sk.sign(&sig_message(manifest, payload));
    let mut section = Vec::with_capacity(96);
    section.extend_from_slice(sk.verifying_key().as_bytes());
    section.extend_from_slice(&signature.to_bytes());
    section
}

/// Lowercase-hex a byte slice (the signer-identity rendering — the key is short, no dep needed).
fn hex_lower(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
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

    // ----- trap 5 (DL1506) and R-6c (DL0801) ---------------------------------------------------

    #[test]
    fn trap5_a_limit_killed_plugin_is_gone_not_wounded() {
        // DL1506 drops the instance AND revokes the node in ONE act.
        let art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        let mut c = host_custody(&["Read"]);
        let loaded = load_verified(&art, &grant(&[]), &mut c).expect("loads");
        let LoadedPlugin::Verified { grant_id, .. } = loaded;
        assert_eq!(c.liveness(&grant_id), Liveness::Live);

        let fake_instance = vec![0u8; 8]; // stands in for the engine instance
        let e = kill_on_limit(&mut c, &grant_id, fake_instance, "fuel");
        assert_eq!(e.variant(), "LimitExceeded");
        assert_eq!(plugin_err_code(&e), "DL1506");
        // NO LIVE NODE SURVIVES IT. The plugin does not exist; nothing can be resumed.
        assert!(
            matches!(c.liveness(&grant_id), Liveness::Revoked(_)),
            "a limit-killed plugin's node must be revoked in the same act"
        );
        // And a retained reference through it is dead — the host's view is deterministic.
        let r = PluginRef::new(grant_id, "shout");
        assert_eq!(plugin_err_code(&r.check_call(&c).expect_err("dead")), "DL0801");
    }

    #[test]
    fn r6c_unload_then_a_retained_reference_is_dl0801_with_the_revoking_seq() {
        let art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        let mut c = host_custody(&["Read"]);
        let LoadedPlugin::Verified { grant_id, .. } = load_verified(&art, &grant(&[]), &mut c).expect("loads");
        let retained = PluginRef::new(grant_id.clone(), "shout");
        assert!(retained.check_call(&c).is_ok(), "live before unload");

        let seq = unload(&mut c, &grant_id).expect("unload revokes");
        let e = retained.check_call(&c).expect_err("dead after unload");
        assert_eq!(plugin_err_code(&e), "DL0801");
        // The error carries the REVOKING AUDIT SEQ — why and when this authority died.
        assert_eq!(e, PluginErr::Revoked(seq as i64), "DL0801 carries the revoking audit seq");
        assert!(seq > 0, "a real audit seq, not a placeholder");
    }

    #[test]
    fn r6c_reload_mints_a_fresh_node_and_the_old_reference_stays_dead_forever() {
        // THE AUTHORITY-SWAP CLOSER (criterion 3). A reload is a NEW node with a NEW GrantId; the
        // old reference is never silently re-bound to it.
        let art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        let mut c = host_custody(&["Read"]);
        let LoadedPlugin::Verified { grant_id: first, .. } =
            load_verified(&art, &grant(&[]), &mut c).expect("loads");
        let old_ref = PluginRef::new(first.clone(), "shout");
        unload(&mut c, &first).expect("unload");

        let LoadedPlugin::Verified { grant_id: second, .. } =
            load_verified(&art, &grant(&[]), &mut c).expect("reloads");
        assert_ne!(first, second, "a reload mints a FRESH GrantId");
        // The new handle works...
        assert!(PluginRef::new(second, "shout").check_call(&c).is_ok(), "the new handle is live");
        // ...and the old reference is STILL dead. Dead is forever.
        assert_eq!(plugin_err_code(&old_ref.check_call(&c).expect_err("still dead")), "DL0801");
    }

    #[test]
    fn r6c_check_call_is_fail_closed_when_custody_cannot_answer() {
        // "Couldn't tell" is dead, never alive: a custody with no tree answers Unknown, and an
        // unknown node confers nothing.
        let c = EmbeddedCustody::new(); // no grant tree at all
        let r = PluginRef::new(delulu_broker::GrantId::from_trusted("g_nonexistent"), "shout");
        assert_eq!(plugin_err_code(&r.check_call(&c).expect_err("unknown ⇒ dead")), "DL0801");
    }

    /// CRITERION 4 — R-7 composition across three levels.
    #[test]
    fn criterion4_r7_composition_a_sub_plugin_can_never_exceed_its_parent() {
        let art_read = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &["Read", "Net"]);
        // Host holds {Read, Net}; it grants plugin A only {Read}.
        let mut c = host_custody(&["Read", "Net"]);
        let LoadedPlugin::Verified { grant_id: node_a, .. } =
            load_verified(&art_read, &grant(&["Read"]), &mut c).expect("A loads under the host");

        // Now A is the holder: a sub-plugin loads under A's OWN node (R-7 composition).
        c.set_holder(node_a.clone());

        // A granted {Read} tries to hand its child {Read, Net} — MORE than A holds. DL0802.
        let e = load_verified(&art_read, &grant(&["Read", "Net"]), &mut c)
            .expect_err("a sub-plugin may not exceed its parent");
        assert_eq!(e.code, "DL0802", "R-7: no grantee exceeds its grantor, at any depth");

        // A conforming sub-plugin ({Read} ⊑ {Read}) loads.
        let LoadedPlugin::Verified { grant_id: node_b, .. } =
            load_verified(&art_read, &grant(&["Read"]), &mut c).expect("B loads under A");
        assert_eq!(c.liveness(&node_a), Liveness::Live);
        assert_eq!(c.liveness(&node_b), Liveness::Live);

        // Host revocation kills all three levels TRANSITIVELY. Revoking is done from the host's own
        // node (a caller may only revoke its own node or a descendant — §3.2, no upward reach).
        let host_node = c.broker().unwrap().nodes().iter().find(|n| n.parent.is_none()).unwrap().id.clone();
        c.set_holder(host_node.clone());
        unload(&mut c, &host_node).expect("the host revokes its own subtree");
        assert!(matches!(c.liveness(&node_a), Liveness::Revoked(_)), "A dies with the host");
        assert!(matches!(c.liveness(&node_b), Liveness::Revoked(_)), "and so does B, transitively");
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

    // ----- 6e.5: Verified interpreter instantiation — the flagship RUNS -----------------------

    /// Complete a load and pull out the re-proved plugin (the shape a host embedding holds).
    fn load_and_get_verified(
        code: &str,
        exports: serde_json::Value,
        ceiling: &[&str],
        g: &Grant,
        c: &mut EmbeddedCustody,
    ) -> Box<VerifiedPlugin> {
        let art = verified_artifact(code, exports, ceiling);
        let LoadedPlugin::Verified { verified, .. } = load_verified(&art, g, c).expect("loads");
        verified
    }

    #[test]
    fn flagship_a_zero_authority_text_transform_loads_gets_and_runs() {
        // CRITERION 1 (the flagship, Constitution §4 Possibility 2) at the INTERPRETER engine. A
        // third-party text-transform plugin loads with `Grant { effects: [] }`; the host asks for a
        // PURE `fn(Str) -> Str ! {}` (annotation form — build-order deviation 4, not bracket
        // syntax); R-Get succeeds; calling it returns the transformed string; and the host's row is
        // unchanged — the export is pure, so a pure `F` covers it and nothing is added to any caller
        // row (that "host row unchanged" is exactly R-Get accepting a pure export under a pure F).
        let code = "module p\npub fn shout(s: Str) -> Str { s + \"!\" }\n";
        let mut c = host_custody(&["Read", "Net"]); // the host holds authority the plugin will NOT get
        let verified = load_and_get_verified(code, json!({ "shout": "fn(Str) -> Str" }), &[], &grant(&[]), &mut c);

        // R-Get, annotation form: `let f: fn(Str) -> Str ! {} = p.get("shout")?`.
        let pure_f = fnty(vec![Type::Str], Type::Str, &[]);
        r_get_verified(verified.dir.fn_types.get("shout"), &pure_f, "shout")
            .expect("R-Get accepts a pure export under a pure F — the host row is unchanged");

        // Instantiate on the interpreter and RUN. The transformed string comes back.
        let inst = VerifiedInterpInstance::instantiate(verified.as_ref(), &Limits::default());
        let out = inst.call_export("shout", vec![Value::str("hello")]).expect("the export runs");
        assert_eq!(out.display(), "hello!", "the plugin transformed the string");

        // It is re-runnable and deterministic — a fresh budget each call.
        assert_eq!(
            inst.call_export("shout", vec![Value::str("world")]).unwrap().display(),
            "world!"
        );
    }

    #[test]
    fn flagship_a_rigged_variant_that_tries_to_tell_the_clock_is_refused_at_load() {
        // CRITERION 1's rigged half. A plugin that ATTEMPTS an effect (here reading the clock) while
        // hiding it behind a pure manifest row is refused AT LOAD — a DL1504-class ROW violation —
        // so the effect never reaches the point of running. A file-read or a net-reach rig is
        // refused identically: the manifest row cannot conceal what the code's row performs, and
        // Verified NEVER falls back to Contained (invariant 29).
        let code = "module p\npub fn shout(clk: Cap[Clock], s: Str) -> Str ! {Clock} { let _t = clk.now_ms()\n s }\n";
        // The manifest LIES — same shape, but a PURE row.
        let art = verified_artifact(code, json!({ "shout": "fn(Cap[Clock], Str) -> Str ! {}" }), &["Clock"]);
        let mut c = host_custody(&["Clock"]);
        let e = load_verified(&art, &grant(&["Clock"]), &mut c).expect_err("the hidden effect is caught at load");
        assert_eq!(e.code, "DL1504", "a hidden effect is a DL1504 row violation: {}", e.message);
        assert!(e.message.contains("Clock"), "the refusal names the hidden effect: {}", e.message);
        assert!(e.requires_human, "DL1504 is requires_human and never falls back to Contained");
    }

    #[test]
    fn couldnt_tell_an_unevaluable_export_is_an_honest_error_never_a_panic_or_silent_unit() {
        // THE KITCHEN RULE for 6e.5. A DIR that verified, but whose evaluation reaches something the
        // interpreter cannot evaluate, must yield an honest error — NEVER a panic, NEVER a silent
        // `Unit`. We provoke it deterministically: ask the instance to run a name the interpreter's
        // function table does not hold. The interpreter faults DL0907 ("unknown function"), its
        // couldn't-tell catch-all, and the runner reports `Unevaluable` — an `Err`, not `Ok(Unit)`.
        let code = "module p\npub fn shout(s: Str) -> Str { s }\n";
        let mut c = host_custody(&[]);
        let verified = load_and_get_verified(code, json!({ "shout": "fn(Str) -> Str" }), &[], &grant(&[]), &mut c);
        let inst = VerifiedInterpInstance::instantiate(verified.as_ref(), &Limits::default());

        let r = inst.call_export("ghost", vec![Value::str("x")]);
        assert!(r.is_err(), "an unevaluable export must be an Err, never a silent Ok(Unit)");
        let e = r.unwrap_err();
        assert!(matches!(e, PluginRunError::Unevaluable(_)), "the couldn't-tell case is Unevaluable: {e:?}");
        assert_eq!(e.code(), "DL0907");
    }

    #[test]
    fn classify_run_fault_maps_each_class_honestly() {
        // The couldn't-tell classification, tested at the seam (the kitchen rule, written first).
        // DL1506 → Limit; DL0907 → Unevaluable; every other defined fault → Faulted.
        assert!(matches!(classify_run_fault(Fault::new("DL1506", "x")), PluginRunError::Limit(_)));
        assert!(matches!(classify_run_fault(Fault::new("DL0907", "x")), PluginRunError::Unevaluable(_)));
        for code in ["DL0901", "DL0902", "DL0903", "DL0905", "DL0703", "DL1403"] {
            assert!(
                matches!(classify_run_fault(Fault::new(code, "x")), PluginRunError::Faulted(_)),
                "{code} is the plugin's own runtime fault, not a limit or a couldn't-tell"
            );
        }
    }

    #[test]
    fn a_plugin_that_faults_at_runtime_surfaces_cleanly_as_faulted() {
        // The plugin's own runtime bug (division by zero) surfaces as a clean Faulted(DL0902) —
        // never a panic, never a silent `Unit`, and NOT a limit (more authority cannot fix a bug).
        let code = "module p\npub fn crash(x: Int) -> Int { x / 0 }\n";
        let mut c = host_custody(&[]);
        let verified = load_and_get_verified(code, json!({ "crash": "fn(Int) -> Int" }), &[], &grant(&[]), &mut c);
        let inst = VerifiedInterpInstance::instantiate(verified.as_ref(), &Limits::default());
        let e = inst.call_export("crash", vec![Value::Int(10)]).expect_err("div by zero faults");
        assert!(matches!(e, PluginRunError::Faulted(_)), "a runtime bug is Faulted: {e:?}");
        assert_eq!(e.code(), "DL0902");
    }

    #[test]
    fn best_effort_fuel_kills_an_infinite_loop_and_is_labeled_best_effort() {
        // §5.4: on the interpreter `fuel` is a best-effort STEP COUNTER. An infinite loop is killed
        // by it — but the message says plainly it is best-effort and names the WASM engine as the
        // enforcement-grade path. We never claim the interpreter *contains* hostile code. (An
        // *unbounded loop*, not unbounded recursion: the latter grows the interpreter's own native
        // call stack, which the depth guard DL0905 bounds — a different, orthogonal limit.)
        let code = "module p\npub fn spin(s: Str) -> Str { while true { let _k = 1 }\n s }\n";
        let mut c = host_custody(&[]);
        let verified = load_and_get_verified(code, json!({ "spin": "fn(Str) -> Str" }), &[], &grant(&[]), &mut c);
        let inst =
            VerifiedInterpInstance::instantiate(verified.as_ref(), &Limits { fuel: 5_000, mem_mb: 0, wall_ms: 0 });
        let e = inst.call_export("spin", vec![Value::str("x")]).expect_err("an infinite loop is killed");
        assert!(matches!(e, PluginRunError::Limit(_)), "a step-budget kill is a Limit: {e:?}");
        assert_eq!(e.code(), "DL1506");
        assert!(e.message().contains("best-effort"), "the message LABELS best-effort: {}", e.message());
        assert!(e.message().contains("WASM engine"), "and names the enforcement-grade path: {}", e.message());
        // The budget resets per call — a second call gets fresh fuel and also trips (never wedged).
        assert!(matches!(inst.call_export("spin", vec![Value::str("y")]), Err(PluginRunError::Limit(_))));
    }

    #[test]
    fn best_effort_memory_accounting_stops_a_runaway_allocation_labeled_best_effort() {
        // §5.4: `mem_mb` maps to allocator accounting at value-construction sites. A doubling string
        // concatenation would blow real memory; the byte budget trips first — and is honestly
        // labeled, with generous fuel so the STEP counter is not what caught it.
        let code =
            "module p\npub fn grow(s: Str, n: Int) -> Str { if n <= 0 { s } else { grow(s + s, n - 1) } }\n";
        let mut c = host_custody(&[]);
        let verified =
            load_and_get_verified(code, json!({ "grow": "fn(Str, Int) -> Str" }), &[], &grant(&[]), &mut c);
        let inst = VerifiedInterpInstance::instantiate(
            verified.as_ref(),
            &Limits { fuel: 10_000_000, mem_mb: 1, wall_ms: 0 },
        );
        let e = inst
            .call_export("grow", vec![Value::str("padpadpadpadpadpad"), Value::Int(40)])
            .expect_err("a runaway allocation is stopped");
        assert!(matches!(e, PluginRunError::Limit(_)), "a memory-budget kill is a Limit: {e:?}");
        assert_eq!(e.code(), "DL1506");
        assert!(
            e.message().contains("best-effort") && e.message().contains("mem_mb"),
            "labeled best-effort memory: {}",
            e.message()
        );
    }

    #[test]
    fn a_host_program_run_is_byte_identical_no_budget_attached() {
        // Criterion 11 shape: the budget is opt-in. A plugin instance attaches one; the ordinary
        // `Interp::new` path used by every host program does NOT — so its behavior is unchanged.
        // (Witnessed by every pre-existing interpreter test still passing; asserted here directly.)
        let checked = delulu_check::check_source(0, "module m\nfn f(n: Int) -> Int { n + 1 }\n");
        let interp = Interp::new(&checked.module);
        assert_eq!(interp.call_with("f", vec![Value::Int(41)]).unwrap().display(), "42");
    }

    // ----- 6h: ed25519 signatures (spec §2.2 / §3.1 step 6) -----------------------------------
    //
    // Kitchen rule for signatures (head-chef): the couldn't-tell cases each refuse honestly, and an
    // UNSIGNED-when-required plugin (DL1511) is a DIFFERENT fault from a BADLY-signed one (DL1510).

    /// Attach an ed25519 signature over this artifact's (manifest ‖ DIR), the Verified signing shape.
    fn sign_verified(art: &mut PluginArtifact, seed: &[u8; 32]) {
        let sig = sign_plugin(seed, &art.manifest, art.dir.as_deref());
        art.sig = Some(sig);
    }

    #[test]
    fn verify_signature_couldnt_tell_cases_all_refuse_never_skip() {
        // Written FIRST (kitchen rule). Each ambiguous case is a distinct, honest outcome — a
        // present-but-unusable signature is NEVER read as "unsigned" (that would pass a tamper).
        let m = json!({ "name": "p", "version": "0.1.0" });
        let payload = Some(&b"dir bytes"[..]);
        assert_eq!(verify_signature(&m, payload, None), SignatureStatus::Unsigned, "no section ⇒ Unsigned");
        assert!(
            matches!(verify_signature(&m, payload, Some(&[0u8; 10])), SignatureStatus::Invalid { .. }),
            "a wrong-length section is Invalid, never skipped"
        );
        assert!(
            matches!(verify_signature(&m, payload, Some(&[0u8; 96])), SignatureStatus::Invalid { .. }),
            "a bogus 96-byte section (bad key/sig) is Invalid"
        );
        let seed = [3u8; 32];
        let good = sign_plugin(&seed, &m, payload);
        assert!(matches!(verify_signature(&m, payload, Some(&good)), SignatureStatus::Valid { .. }), "a real signature is Valid");
        assert!(
            matches!(verify_signature(&m, Some(&b"other payload"[..]), Some(&good)), SignatureStatus::Invalid { .. }),
            "a valid signature over DIFFERENT content does not verify"
        );
    }

    #[test]
    fn sign_then_verify_round_trips_and_the_identity_is_the_public_key() {
        // verify ≡ sign: signing and verification share `sig_message`, so a fresh signature always
        // verifies, and the recorded identity is the signer's public key (hex).
        let seed = [42u8; 32];
        let m = json!({ "name": "p" });
        let sig = sign_plugin(&seed, &m, Some(b"dir"));
        match verify_signature(&m, Some(b"dir"), Some(&sig)) {
            SignatureStatus::Valid { signer } => {
                assert_eq!(signer.len(), 64, "the identity is a 32-byte ed25519 key in hex");
                assert!(signer.chars().all(|c| c.is_ascii_hexdigit()));
            }
            other => panic!("a fresh signature must verify, got {other:?}"),
        }
    }

    #[test]
    fn criterion8_a_signed_plugin_verifies_and_its_identity_lands_in_the_audit_log() {
        // CRITERION 8, first half. A signed plugin loads, carries its signer identity on the handle,
        // and that identity is written to the audit log (spec §3.1 step 6).
        let mut art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        sign_verified(&mut art, &[7u8; 32]);

        let mut c = host_custody(&[]);
        let sink = delulu_broker::MemSink::new();
        c.broker_mut().expect("embedded tree").set_sink(Box::new(sink.clone()));

        let LoadedPlugin::Verified { signer, .. } =
            load_verified(&art, &grant(&[]), &mut c).expect("a validly-signed plugin loads");
        let signer = signer.expect("a signed plugin carries its signer identity");
        assert_eq!(signer.len(), 64, "identity = 32-byte key in hex");

        let records = sink.records();
        let sig_rec = records
            .iter()
            .find(|r| r.action == "plugin-signature")
            .expect("the signature identity landed in the audit log");
        assert_eq!(
            sig_rec.authority.as_ref().and_then(|a| a.get("signed_by")).and_then(|v| v.as_str()),
            Some(signer.as_str()),
            "the audit record names the signer"
        );
    }

    #[test]
    fn criterion8_require_signed_refuses_an_unsigned_plugin_as_dl1511() {
        // CRITERION 8, second half. `require_signed: true` refuses an UNSIGNED plugin — DL1511, a
        // policy refusal distinct from a bad signature.
        let art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]); // no sig
        let mut c = host_custody(&[]);
        let mut g = grant(&[]);
        g.require_signed = true;
        let e = load_verified(&art, &g, &mut c).expect_err("require_signed refuses an unsigned plugin");
        assert_eq!(e.code, "DL1511", "unsigned-but-required is DL1511");
        assert_eq!(e.to_plugin_err().variant(), "NotGranted");
        // Refusal honesty: the node minted at step 4 is revoked (no live plugin node survives).
        let tree = c.broker().expect("tree");
        let live = tree.nodes().iter().any(|n| {
            n.holder.kind == "plugin"
                && matches!(tree.effective_state(&n.id), Some(delulu_broker::EffState::Live))
        });
        assert!(!live, "a DL1511 refusal must revoke the plugin's node");
    }

    #[test]
    fn a_badly_signed_plugin_is_dl1510_a_different_fault_from_unsigned() {
        // The head-chef point: a present-but-invalid signature is DL1510, NOT DL1511. We sign, then
        // tamper the content — the signature no longer verifies. Refused regardless of require_signed.
        let mut art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        sign_verified(&mut art, &[7u8; 32]);
        art.manifest["version"] = json!("9.9.9-tampered"); // content changed after signing

        let mut c = host_custody(&[]);
        let e = load_verified(&art, &grant(&[]), &mut c).expect_err("a broken signature is refused");
        assert_eq!(e.code, "DL1510", "badly-signed is DL1510");
        assert_ne!(e.code, "DL1511", "and it is NOT the unsigned fault");
        assert!(e.requires_human, "a broken signature is never machine-repairable");
        assert_eq!(e.to_plugin_err().variant(), "VerifyFailed");
    }

    #[test]
    fn a_malformed_signature_section_is_dl1510_not_a_silent_skip() {
        // A present but wrong-length signature section is a fault, never quietly ignored.
        let mut art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        art.sig = Some(vec![0u8; 10]); // not 96 bytes
        let mut c = host_custody(&[]);
        let e = load_verified(&art, &grant(&[]), &mut c).expect_err("a malformed signature is refused");
        assert_eq!(e.code, "DL1510");
    }

    #[test]
    fn an_unsigned_plugin_without_require_signed_loads_and_records_no_identity() {
        // Positive control: the signature machinery does not disturb the ordinary unsigned load path.
        let art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        let mut c = host_custody(&[]);
        let LoadedPlugin::Verified { signer, .. } = load_verified(&art, &grant(&[]), &mut c).expect("loads");
        assert!(signer.is_none(), "an unsigned plugin has no signer identity");
    }

    #[test]
    fn criterion9_verify_gives_identical_verdicts_to_a_real_load_across_the_corpus() {
        // CRITERION 9: `plugin verify` (steps 1, 2, 5 + signature) must NEVER diverge from a real
        // load. It is guaranteed by ONE code path (both call step1_container_api / step5_verified /
        // verify_signature / step6_signature), and proven here across a corpus of tricky artifacts.
        fn verdict_verify(art: &PluginArtifact) -> String {
            verify_plugin(art).map(|_| "ok".to_string()).unwrap_or_else(|e| e.code.to_string())
        }
        fn verdict_load(art: &PluginArtifact) -> String {
            // A grant that passes steps 3/4 (empty ⊑ any ceiling; empty ⊑ the holder) and no
            // require_signed — so the load's verdict is decided by steps 1, 5, 6: exactly what
            // verify runs. Any divergence is then a real verify/load bug.
            let mut c = host_custody(&["Read", "Write", "Net", "Clock"]);
            load_verified(art, &grant(&[]), &mut c)
                .map(|_| "ok".to_string())
                .unwrap_or_else(|e| e.code.to_string())
        }

        let mut corpus: Vec<(&str, PluginArtifact)> = Vec::new();
        corpus.push(("pure", verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[])));
        let eff = "module p\npub fn w(out: Cap[Console], s: Str) ! {Write} { out.println(s) }\n";
        corpus.push((
            "effectful_honest",
            verified_artifact(eff, json!({ "w": "fn(Cap[Console], Str) ! {Write}" }), &["Write"]),
        ));
        let mut tampered = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        tampered.dir = Some(b"garbage dir bytes".to_vec());
        corpus.push(("tampered_dir", tampered));
        corpus.push(("type_mismatch", verified_artifact(PURE_CODE, json!({ "shout": "fn(Int) -> Int" }), &[])));
        corpus.push(("missing_export", verified_artifact(PURE_CODE, json!({ "ghost": "fn(Str) -> Str" }), &[])));
        let mut api = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        api.api = 9;
        api.manifest["api"] = json!(9);
        corpus.push(("bad_api", api));
        let over = "module p\npub fn shout(clk: Cap[Clock], s: Str) -> Str ! {Clock} { let _t = clk.now_ms()\n s }\n";
        corpus.push((
            "row_exceeds_manifest",
            verified_artifact(over, json!({ "shout": "fn(Cap[Clock], Str) -> Str ! {}" }), &["Clock"]),
        ));
        let mut signed = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        signed.sig = Some(sign_plugin(&[5u8; 32], &signed.manifest, signed.dir.as_deref()));
        corpus.push(("signed", signed));
        let mut bad = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        bad.sig = Some(sign_plugin(&[5u8; 32], &bad.manifest, bad.dir.as_deref()));
        bad.manifest["version"] = json!("tampered-after-signing");
        corpus.push(("badly_signed", bad));

        for (name, art) in &corpus {
            let (v, l) = (verdict_verify(art), verdict_load(art));
            assert_eq!(v, l, "verify/load divergence on `{name}`: verify={v}, load={l}");
        }
        // The corpus really exercised both accept and several distinct refusals (not a vacuous pass).
        let verdicts: std::collections::BTreeSet<String> =
            corpus.iter().map(|(_, a)| verdict_verify(a)).collect();
        for expected in ["ok", "DL1504", "DL1507", "DL1510"] {
            assert!(verdicts.contains(expected), "the corpus must exercise `{expected}`: {verdicts:?}");
        }
    }

    #[test]
    fn criterion6_load_succeeds_with_an_invalid_wasm_cache_never_rests_on_machine_code() {
        // CRITERION 6 (loader side, spec §3.2): the Verified guarantee never rests on shipped machine
        // code. A load SUCCEEDS even with a corrupt `delulu:wasm` cache — the loader never touches it
        // (a WASM host recompiles from DIR; the interpreter host ignores it entirely). The DIR-flip
        // half is DL1504 at the container read (delulu-wasm `criterion6_dir_flip_...`).
        let mut art = verified_artifact(PURE_CODE, json!({ "shout": "fn(Str) -> Str" }), &[]);
        art.wasm = Some(b"a corrupt, stale compilation cache".to_vec());
        art.wasm_cache_valid = false; // as the container reader flags a Verified cache mismatch
        let mut c = host_custody(&[]);
        assert!(
            load_verified(&art, &grant(&[]), &mut c).is_ok(),
            "an invalid wasm cache must not fail a Verified load"
        );
    }
}
