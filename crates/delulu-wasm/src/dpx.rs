//! The `.dpx` plugin artifact container (Stage 6 "Live", spec §2.2).
//!
//! A `.dpx` is a wasm-format container — the same custom-section technique as the `.dwx`
//! (`artifact.rs`), whose ULEB machinery it reuses — carrying the plugin's content entirely in
//! custom sections:
//!
//! | section         | Verified                      | Contained                       |
//! |-----------------|-------------------------------|---------------------------------|
//! | `delulu:plugin` | manifest JSON                 | manifest JSON                   |
//! | `delulu:dir`    | the DIR payload               | absent                          |
//! | `delulu:wasm`   | optional compilation cache    | the opaque module               |
//! | `delulu:sig`    | optional ed25519 signature    | optional ed25519 signature      |
//! | `delulu:lock`   | build lockfile (provenance)   | absent/optional                 |
//!
//! **Content binding (blake3, as in `.dwx`):** the `delulu:plugin` manifest JSON embeds the blake3
//! hash of every other section present (`dir_blake3`, `wasm_blake3`, `lock_blake3`), binding the
//! manifest's claims to those exact bytes. Verification semantics are class-aware and match the
//! spec's criterion 6 exactly:
//!
//! - a `delulu:dir` section that fails its binding is a **failed Verified re-check precondition**
//!   ([`DpxError::DirTampered`] → DL1504 — never DL1508, never a fallback to Contained);
//! - a Contained `delulu:wasm` that fails its binding is a corrupt artifact (DL1508);
//! - a **Verified** `delulu:wasm` is only a cache: a failed binding is NOT an error — the reader
//!   reports `wasm_cache_valid: false` and loaders recompile from DIR (spec §3.2: the verified
//!   guarantee never rests on shipped machine code).
//!
//! Reading is hostile-input hardened: bad magic, truncated sections, duplicate sections, an
//! unreadable manifest, and every single-byte corruption refuse cleanly — never a panic.

use serde_json::Value;

use crate::artifact::{read_uleb, write_uleb};

pub const PLUGIN_SECTION: &str = "delulu:plugin";
pub const DIR_SECTION: &str = "delulu:dir";
pub const WASM_SECTION: &str = "delulu:wasm";
pub const SIG_SECTION: &str = "delulu:sig";
pub const LOCK_SECTION: &str = "delulu:lock";

/// The `.dpx` container-format version this toolchain writes and understands.
pub const DPX_VERSION: u32 = 1;

/// Why a `.dpx` failed to read. `code()` maps each to its diagnostic: container corruption is
/// DL1508 (build-order deviation 2), an unsupported `plugin.api` is DL1507, and a DIR that fails
/// its content binding is DL1504 (criterion 6: a tampered DIR is a failed Verified re-check —
/// never "just" a container fault, never a silent fallback).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DpxError {
    NotDpx,
    Malformed(String),
    NoPluginSection,
    BadManifest(String),
    UnsupportedVersion(u32),
    UnsupportedApi(u32),
    /// The `delulu:dir` bytes do not match the manifest's `dir_blake3` binding.
    DirTampered,
    /// A **Contained** module's bytes do not match `wasm_blake3` (a Verified cache mismatch is
    /// not an error — see module docs).
    WasmTampered,
}

impl DpxError {
    pub fn code(&self) -> &'static str {
        match self {
            DpxError::UnsupportedApi(_) => "DL1507",
            DpxError::DirTampered => "DL1504",
            _ => "DL1508",
        }
    }

    pub fn message(&self) -> String {
        match self {
            DpxError::NotDpx => "not a `.dpx` plugin artifact (bad magic)".into(),
            DpxError::Malformed(w) => format!("malformed `.dpx` container: {w}"),
            DpxError::NoPluginSection => {
                format!("no `{PLUGIN_SECTION}` section — not a plugin artifact (build one with `delulu plugin build`)")
            }
            DpxError::BadManifest(w) => format!("`{PLUGIN_SECTION}` manifest is invalid: {w}"),
            DpxError::UnsupportedVersion(v) => format!(
                "`.dpx` container version {v} is newer than this toolchain supports (v{DPX_VERSION}) — upgrade `delulu` or rebuild the plugin"
            ),
            DpxError::UnsupportedApi(a) => format!(
                "plugin API version {a} is not supported by this toolchain — rebuild the plugin"
            ),
            DpxError::DirTampered => "the `delulu:dir` payload does not match its content binding — \
                 the Verified code cannot be re-checked as shipped (never falls back to Contained)"
                .into(),
            DpxError::WasmTampered => "the Contained module does not match its content binding — \
                 the artifact was altered after it was built"
                .into(),
        }
    }
}

/// A successfully read `.dpx`: the parsed manifest plus each section's bytes. `class`/`api` are
/// lifted out of the manifest for convenience (they are validated on read).
#[derive(Debug)]
pub struct Dpx {
    /// The parsed `delulu:plugin` manifest JSON (name, version, api, class, authority, exports,
    /// section hashes).
    pub manifest: Value,
    /// `"verified"` or `"contained"` — validated, never defaulted (invariant 29).
    pub class: String,
    pub api: u32,
    pub dir: Option<Vec<u8>>,
    pub wasm: Option<Vec<u8>>,
    pub sig: Option<Vec<u8>>,
    pub lock: Option<Vec<u8>>,
    /// For a Verified `.dpx` with a `delulu:wasm` cache: whether the cache matches its binding.
    /// An invalid cache is ignored by loaders (recompile from DIR — spec §3.2), never an error.
    /// Always `true` when no cache is present, and for Contained (a mismatch there refuses).
    pub wasm_cache_valid: bool,
}

/// Write a `.dpx`: a minimal wasm container (magic + version) holding only custom sections. The
/// manifest is augmented with the container version and the blake3 binding of every section
/// present, then serialized canonically (serde_json sorts keys) — identical inputs produce
/// byte-identical artifacts (house rule 8).
pub fn write_dpx(
    manifest: &Value,
    dir: Option<&[u8]>,
    wasm: Option<&[u8]>,
    sig: Option<&[u8]>,
    lock: Option<&[u8]>,
) -> Vec<u8> {
    let m = augmented_plugin_manifest(manifest, dir, wasm, lock);
    let manifest_bytes = serde_json::to_vec(&m).expect("manifest serializes");

    let mut out = b"\0asm\x01\0\0\0".to_vec();
    append_custom_section(&mut out, PLUGIN_SECTION, &manifest_bytes);
    if let Some(d) = dir {
        append_custom_section(&mut out, DIR_SECTION, d);
    }
    if let Some(w) = wasm {
        append_custom_section(&mut out, WASM_SECTION, w);
    }
    if let Some(s) = sig {
        append_custom_section(&mut out, SIG_SECTION, s);
    }
    if let Some(l) = lock {
        append_custom_section(&mut out, LOCK_SECTION, l);
    }
    out
}

/// The **augmented** `delulu:plugin` manifest that `write_dpx` serializes into the container: the
/// caller's manifest plus the `container` version and the blake3 binding of every present section.
/// Exposed (Stage 6 phase 6h) so `plugin build --sign` can sign over the EXACT canonical manifest
/// bytes the container will hold — `serde_json::to_vec` of this value equals the `delulu:plugin`
/// section a loader reads and re-serializes for signature verification (`sig_message`).
pub fn augmented_plugin_manifest(
    manifest: &Value,
    dir: Option<&[u8]>,
    wasm: Option<&[u8]>,
    lock: Option<&[u8]>,
) -> Value {
    let mut m = manifest.clone();
    let obj = m.as_object_mut().expect("plugin manifest is a JSON object");
    obj.insert("container".into(), Value::from(DPX_VERSION));
    if let Some(d) = dir {
        obj.insert("dir_blake3".into(), Value::from(blake3::hash(d).to_hex().to_string()));
    }
    if let Some(w) = wasm {
        obj.insert("wasm_blake3".into(), Value::from(blake3::hash(w).to_hex().to_string()));
    }
    if let Some(l) = lock {
        obj.insert("lock_blake3".into(), Value::from(blake3::hash(l).to_hex().to_string()));
    }
    m
}

/// Append one custom section (id 0): name as a length-prefixed UTF-8 string, then the payload —
/// the identical encoding `embed_authority` uses for the `.dwx`.
fn append_custom_section(out: &mut Vec<u8>, name: &str, payload: &[u8]) {
    let name_bytes = name.as_bytes();
    let mut body = Vec::with_capacity(1 + name_bytes.len() + payload.len());
    write_uleb(name_bytes.len() as u32, &mut body);
    body.extend_from_slice(name_bytes);
    body.extend_from_slice(payload);
    out.push(0x00);
    write_uleb(body.len() as u32, out);
    out.extend_from_slice(&body);
}

/// Read and structurally verify a `.dpx`. This is **the** container read path — `plugin inspect`,
/// `plugin verify`, and the loader all come through here (one code path, spec §9.9). See the
/// module docs for the class-aware binding semantics.
pub fn read_dpx(bytes: &[u8]) -> Result<Dpx, DpxError> {
    if bytes.len() < 8 || &bytes[0..4] != b"\0asm" {
        return Err(DpxError::NotDpx);
    }

    // Walk the sections. Every section must be a custom section (a `.dpx` carries content only in
    // custom sections), each known `delulu:` section may appear at most once, and every size must
    // land inside the buffer — hostile input never panics.
    let mut plugin_payload: Option<Vec<u8>> = None;
    let mut dir: Option<Vec<u8>> = None;
    let mut wasm: Option<Vec<u8>> = None;
    let mut sig: Option<Vec<u8>> = None;
    let mut lock: Option<Vec<u8>> = None;

    let mut i = 8usize;
    while i < bytes.len() {
        let id = bytes[i];
        i += 1;
        let (size, n) =
            read_uleb(&bytes[i..]).ok_or_else(|| DpxError::Malformed("truncated section size".into()))?;
        i += n;
        let body_start = i;
        let body_end = body_start
            .checked_add(size as usize)
            .filter(|&e| e <= bytes.len())
            .ok_or_else(|| DpxError::Malformed("section runs past end of artifact".into()))?;
        if id != 0 {
            return Err(DpxError::Malformed(format!(
                "unexpected non-custom section (id {id}) — a `.dpx` carries only custom sections"
            )));
        }
        let body = &bytes[body_start..body_end];
        let (name_len, m) =
            read_uleb(body).ok_or_else(|| DpxError::Malformed("truncated custom-section name".into()))?;
        let name_end = m
            .checked_add(name_len as usize)
            .filter(|&e| e <= body.len())
            .ok_or_else(|| DpxError::Malformed("custom-section name past end".into()))?;
        let name = &body[m..name_end];
        let payload = body[name_end..].to_vec();

        let slot = match name {
            b if b == PLUGIN_SECTION.as_bytes() => &mut plugin_payload,
            b if b == DIR_SECTION.as_bytes() => &mut dir,
            b if b == WASM_SECTION.as_bytes() => &mut wasm,
            b if b == SIG_SECTION.as_bytes() => &mut sig,
            b if b == LOCK_SECTION.as_bytes() => &mut lock,
            _ => {
                // Unknown custom sections are tolerated (forward compatibility), never trusted.
                i = body_end;
                continue;
            }
        };
        if slot.is_some() {
            return Err(DpxError::Malformed(format!(
                "duplicate `{}` section",
                String::from_utf8_lossy(name)
            )));
        }
        *slot = Some(payload);
        i = body_end;
    }

    let plugin_payload = plugin_payload.ok_or(DpxError::NoPluginSection)?;
    let manifest: Value =
        serde_json::from_slice(&plugin_payload).map_err(|e| DpxError::BadManifest(e.to_string()))?;

    let container = manifest
        .get("container")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| DpxError::BadManifest("missing `container` version".into()))? as u32;
    if container > DPX_VERSION {
        return Err(DpxError::UnsupportedVersion(container));
    }
    let api = manifest
        .get("api")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| DpxError::BadManifest("missing `api`".into()))? as u32;
    let class = manifest
        .get("class")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DpxError::BadManifest("missing `class`".into()))?
        .to_string();
    if class != "verified" && class != "contained" {
        // Invariant 29: the class is declared, exactly; an unknown class is never guessed at.
        return Err(DpxError::BadManifest(format!("unknown class `{class}`")));
    }

    // Class/section coherence (spec §2.2 table): Verified carries DIR; Contained carries the
    // module and — strictly — no DIR (a Contained artifact smuggling a DIR is refused, hostile
    // posture: there is no code path that would ever read it honestly).
    match class.as_str() {
        "verified" => {
            if dir.is_none() {
                return Err(DpxError::BadManifest("a verified plugin must carry a `delulu:dir` section".into()));
            }
        }
        _ => {
            if wasm.is_none() {
                return Err(DpxError::BadManifest("a contained plugin must carry a `delulu:wasm` section".into()));
            }
            if dir.is_some() {
                return Err(DpxError::BadManifest(
                    "a contained plugin must not carry a `delulu:dir` section (class is never inferred)".into(),
                ));
            }
        }
    }

    // Content bindings (blake3, class-aware — see module docs).
    let declared = |key: &str| manifest.get(key).and_then(|v| v.as_str()).map(str::to_string);
    if let Some(d) = &dir {
        match declared("dir_blake3") {
            Some(h) if h == blake3::hash(d).to_hex().to_string() => {}
            _ => return Err(DpxError::DirTampered),
        }
    }
    let mut wasm_cache_valid = true;
    if let Some(w) = &wasm {
        let ok = matches!(declared("wasm_blake3"), Some(h) if h == blake3::hash(w).to_hex().to_string());
        if !ok {
            if class == "contained" {
                return Err(DpxError::WasmTampered);
            }
            // Verified: the wasm is only a cache — ignored, never trusted (spec §3.2).
            wasm_cache_valid = false;
        }
    }
    if let Some(l) = &lock {
        match declared("lock_blake3") {
            Some(h) if h == blake3::hash(l).to_hex().to_string() => {}
            _ => return Err(DpxError::Malformed("the `delulu:lock` section does not match its content binding".into())),
        }
    }

    // NOTE: `api` is surfaced but not gated here — the *caller* (inspect/verify/load step 1)
    // refuses an unsupported api with DL1507 via `require_api`, so the reader stays usable for
    // forensics on artifacts from other toolchains.
    Ok(Dpx { manifest, class, api, dir, wasm, sig, lock, wasm_cache_valid })
}

/// Load-sequence step 1's api gate (spec §3.1): refuse a `.dpx` whose `plugin.api` this toolchain
/// does not support (DL1507). Split from [`read_dpx`] so `inspect` can still *describe* a
/// foreign-api artifact while `verify`/`load` refuse it.
pub fn require_api(dpx: &Dpx, supported: u32) -> Result<(), DpxError> {
    if dpx.api != supported {
        return Err(DpxError::UnsupportedApi(dpx.api));
    }
    Ok(())
}

// ===== the engine seam (playbook §1 / head-chef amendment) ===================================
//
// `delulu-wasm` depends on `delulu-runtime`, so the loader (which lives in `delulu-runtime`) cannot
// call this crate directly. The loader instead declares `PluginEngine` + the plain-data
// `PluginArtifact`, and THIS side implements them — the `delulu` crate injects the impl. That is
// what lets the container format live beside the `.dwx` machinery it reuses while the load sequence
// lives with the interpreter, without a dependency cycle.

use delulu_runtime::plugin::{dl1505, ImportSlice, WasmVal};
use delulu_runtime::{LoadRefusal, PluginArtifact, PluginEngine};

/// The Wasmtime-backed plugin engine: reads `.dpx` containers and (from phase 6f) instantiates
/// Contained modules. One instance is injected per run by the `delulu` crate.
#[derive(Debug, Default, Clone, Copy)]
pub struct WasmPluginEngine;

impl WasmPluginEngine {
    pub fn new() -> WasmPluginEngine {
        WasmPluginEngine
    }
}

impl PluginEngine for WasmPluginEngine {
    /// Load-sequence step 1's container read. `plugin verify` and a real load both arrive here, so
    /// a corrupt artifact refuses identically for both (spec §9.9 — one code path, not two).
    fn read_artifact(&self, bytes: &[u8]) -> Result<PluginArtifact, LoadRefusal> {
        let dpx = read_dpx(bytes).map_err(|e| LoadRefusal {
            code: e.code(),
            message: e.message(),
            intersection: None,
            // A tampered DIR (DL1504) is unverifiable Verified code — never machine-repairable and
            // never downgraded to Contained (invariant 29).
            requires_human: matches!(e, DpxError::DirTampered),
        })?;
        Ok(PluginArtifact {
            manifest: dpx.manifest,
            class: dpx.class,
            api: dpx.api,
            dir: dpx.dir,
            wasm: dpx.wasm,
            sig: dpx.sig,
            lock: dpx.lock,
            wasm_cache_valid: dpx.wasm_cache_valid,
        })
    }

    /// Step 5-Contained (DL1505). Wasmtime *validates and parses* the module for us — that part is
    /// its job and we should not hand-roll a wasm parser — but the **verdict is ours**: we decide
    /// name and type against the grant-derived slice here, before anything is instantiated. Letting
    /// instantiation be the gate would be "probably refused downstream", which is not the standard
    /// for a security rule.
    fn validate_imports(&self, wasm: &[u8], slice: &ImportSlice) -> Result<(), LoadRefusal> {
        let engine = wasmtime::Engine::default();
        // A module that does not validate cannot be reasoned about at all — refuse it.
        let module = wasmtime::Module::new(&engine, wasm)
            .map_err(|e| dl1505(format!("the module does not validate ({e})")))?;

        // A module importing NOTHING trivially fits every slice: it can reach nothing. That is the
        // flagship's shape — a pure text transform under `Grant { effects: [] }`.
        for imp in module.imports() {
            let key = (imp.module().to_string(), imp.name().to_string());
            let Some(expected) = slice.get(&key) else {
                // Covers the ungranted name, the unknown name, and — the one that matters — a
                // foreign namespace: `wasi_snapshot_preview1.fd_write` and friends are simply not in
                // any slice, so a Contained module can never reach WASI by the back door.
                return Err(dl1505(format!(
                    "`{}::{}` is not derivable from this grant",
                    imp.module(),
                    imp.name()
                )));
            };
            // Only functions cross. A memory/global/table import is refused rather than ignored.
            let wasmtime::ExternType::Func(ft) = imp.ty() else {
                return Err(dl1505(format!(
                    "`{}::{}` is not a function import — an opaque module may import only functions",
                    imp.module(),
                    imp.name()
                )));
            };
            // NAME IS NOT ENOUGH. A module can declare a granted name with a signature that suits
            // it; if we matched on name alone, only the engine's instantiation type-check would
            // stand between that and reality. We decide it here.
            let actual_params: Vec<WasmVal> = ft.params().map(val_of).collect::<Result<_, _>>().map_err(|t| {
                dl1505(format!("`{}::{}` uses unsupported parameter type `{t}`", imp.module(), imp.name()))
            })?;
            let actual_results: Vec<WasmVal> = ft.results().map(val_of).collect::<Result<_, _>>().map_err(|t| {
                dl1505(format!("`{}::{}` uses unsupported result type `{t}`", imp.module(), imp.name()))
            })?;
            if actual_params != expected.params || actual_results != expected.results {
                return Err(dl1505(format!(
                    "`{}::{}` is granted, but its declared signature does not match the host's — the name being in the slice is not enough",
                    imp.module(),
                    imp.name()
                )));
            }
        }
        Ok(())
    }
}

/// Map a Wasmtime value type onto the loader's. An exotic type (v128, refs) is not something the
/// host surface ever uses, so it can never match a slice entry — reported as unsupported.
fn val_of(t: wasmtime::ValType) -> Result<WasmVal, String> {
    Ok(match t {
        wasmtime::ValType::I32 => WasmVal::I32,
        wasmtime::ValType::I64 => WasmVal::I64,
        wasmtime::ValType::F32 => WasmVal::F32,
        wasmtime::ValType::F64 => WasmVal::F64,
        other => return Err(format!("{other:?}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn manifest(class: &str) -> Value {
        json!({
            "name": "summarize",
            "version": "0.1.0",
            "api": 1,
            "class": class,
            "authority": { "effects": ["Read"], "requires": ["Cap[FsRead]"] },
            "exports": { "summarize": "fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}" },
        })
    }

    #[test]
    fn verified_round_trips_with_bindings() {
        let dir = b"fake dir payload".to_vec();
        let lock = b"lockfile text".to_vec();
        let dpx = write_dpx(&manifest("verified"), Some(&dir), None, None, Some(&lock));
        let read = read_dpx(&dpx).expect("reads");
        assert_eq!(read.class, "verified");
        assert_eq!(read.api, 1);
        assert_eq!(read.dir.as_deref(), Some(&dir[..]));
        assert_eq!(read.lock.as_deref(), Some(&lock[..]));
        assert!(read.wasm.is_none() && read.sig.is_none());
        assert!(read.wasm_cache_valid);
        assert_eq!(read.manifest["name"], "summarize");
        assert_eq!(read.manifest["dir_blake3"], blake3::hash(&dir).to_hex().to_string());
    }

    #[test]
    fn contained_round_trips() {
        let wasm = b"\0asm fake module".to_vec();
        let dpx = write_dpx(&manifest("contained"), None, Some(&wasm), None, None);
        let read = read_dpx(&dpx).expect("reads");
        assert_eq!(read.class, "contained");
        assert_eq!(read.wasm.as_deref(), Some(&wasm[..]));
        assert!(read.dir.is_none());
    }

    #[test]
    fn writing_is_deterministic() {
        // House rule 8: identical inputs → byte-identical artifacts.
        let dir = b"dir".to_vec();
        let a = write_dpx(&manifest("verified"), Some(&dir), None, None, None);
        let b = write_dpx(&manifest("verified"), Some(&dir), None, None, None);
        assert_eq!(a, b);
    }

    #[test]
    fn tampered_dir_is_dl1504_dir_tampered() {
        // Criterion 6 shape: a byte flipped in the DIR body must surface as a failed Verified
        // re-check (DL1504) — the container layer itself reports it as DirTampered.
        let dir = b"the dir payload the manifest binds".to_vec();
        let dpx = write_dpx(&manifest("verified"), Some(&dir), None, None, None);
        // Find the dir payload inside the artifact and flip one byte of it.
        let pos = dpx.windows(dir.len()).position(|w| w == &dir[..]).expect("dir bytes present");
        let mut t = dpx.clone();
        t[pos] ^= 0x01;
        match read_dpx(&t) {
            Err(e @ DpxError::DirTampered) => assert_eq!(e.code(), "DL1504"),
            other => panic!("a tampered DIR must be DirTampered/DL1504, got {other:?}"),
        }
    }

    #[test]
    fn tampered_contained_wasm_is_refused() {
        let wasm = b"the contained module bytes".to_vec();
        let dpx = write_dpx(&manifest("contained"), None, Some(&wasm), None, None);
        let pos = dpx.windows(wasm.len()).position(|w| w == &wasm[..]).expect("wasm bytes present");
        let mut t = dpx.clone();
        t[pos] ^= 0x01;
        match read_dpx(&t) {
            Err(e @ DpxError::WasmTampered) => assert_eq!(e.code(), "DL1508"),
            other => panic!("a tampered Contained module must refuse, got {other:?}"),
        }
    }

    #[test]
    fn tampered_verified_wasm_cache_is_ignored_not_an_error() {
        // Criterion 6, second half: a corrupt cache never fails the read — the loader recompiles
        // from DIR (spec §3.2). The reader reports `wasm_cache_valid: false`.
        let dir = b"dir payload".to_vec();
        let cache = b"stale compiled cache".to_vec();
        let dpx = write_dpx(&manifest("verified"), Some(&dir), Some(&cache), None, None);
        let pos = dpx.windows(cache.len()).position(|w| w == &cache[..]).expect("cache present");
        let mut t = dpx.clone();
        t[pos] ^= 0x01;
        let read = read_dpx(&t).expect("a bad cache must not fail the read");
        assert!(!read.wasm_cache_valid, "the invalid cache must be flagged");
        assert_eq!(read.dir.as_deref(), Some(&dir[..]), "the DIR is untouched and intact");
    }

    #[test]
    fn a_signed_verified_dpx_verifies_end_to_end_through_the_loaders_message() {
        // Phase 6h byte-stability guard for the verify-from-manifest approach: sign over the
        // augmented manifest ‖ DIR (exactly as `plugin build --sign` does), write the .dpx, read it
        // back, and verify with the LOADER's own `verify_signature` — which re-serializes the PARSED
        // manifest. If serde_json's parse∘serialize were not byte-stable here, this would fail.
        let seed = [9u8; 32];
        let dir = b"the DIR payload of a verified plugin".to_vec();
        let m = manifest("verified");
        let augmented = augmented_plugin_manifest(&m, Some(&dir), None, None);
        let sig = delulu_runtime::sign_plugin(&seed, &augmented, Some(&dir));
        let dpx = write_dpx(&m, Some(&dir), None, Some(&sig), None);

        let read = read_dpx(&dpx).expect("a signed .dpx reads");
        assert!(read.sig.is_some(), "the signature section is present");
        let status =
            delulu_runtime::verify_signature(&read.manifest, read.dir.as_deref(), read.sig.as_deref());
        assert!(
            matches!(status, delulu_runtime::SignatureStatus::Valid { .. }),
            "the signed .dpx verifies end-to-end via the loader's message construction: {status:?}"
        );
    }

    #[test]
    fn contained_with_a_dir_section_is_refused() {
        // Hostile shape: a "contained" artifact smuggling a DIR. No honest code path reads it.
        let dpx = write_dpx(&manifest("contained"), Some(b"dir"), Some(b"wasm"), None, None);
        assert!(matches!(read_dpx(&dpx), Err(DpxError::BadManifest(_))));
    }

    #[test]
    fn verified_without_dir_is_refused() {
        let dpx = write_dpx(&manifest("verified"), None, None, None, None);
        assert!(matches!(read_dpx(&dpx), Err(DpxError::BadManifest(_))));
    }

    #[test]
    fn unknown_class_is_refused_never_guessed() {
        let dpx = write_dpx(&manifest("turbo"), Some(b"dir"), None, None, None);
        assert!(matches!(read_dpx(&dpx), Err(DpxError::BadManifest(_))));
    }

    #[test]
    fn unsupported_api_is_dl1507_at_the_gate() {
        let mut m = manifest("verified");
        m["api"] = json!(9);
        let dpx = write_dpx(&m, Some(b"dir"), None, None, None);
        let read = read_dpx(&dpx).expect("the reader describes it");
        match require_api(&read, 1) {
            Err(e @ DpxError::UnsupportedApi(9)) => assert_eq!(e.code(), "DL1507"),
            other => panic!("api 9 must be refused, got {other:?}"),
        }
    }

    #[test]
    fn garbage_and_truncations_refuse_cleanly_never_panic() {
        assert!(matches!(read_dpx(b""), Err(DpxError::NotDpx)));
        assert!(matches!(read_dpx(b"MZ\x90\x00 not wasm"), Err(DpxError::NotDpx)));
        assert!(read_dpx(b"\0asm\x01\0\0\0").is_err(), "no plugin section");

        let dpx = write_dpx(&manifest("verified"), Some(b"dir payload"), None, None, Some(b"lock"));
        for cut in 0..dpx.len() {
            let _ = read_dpx(&dpx[..cut]); // must never panic
        }
        for i in 0..dpx.len() {
            let mut t = dpx.clone();
            t[i] ^= 0xff;
            let _ = read_dpx(&t); // must never panic
        }
    }

    #[test]
    fn the_engine_reads_an_artifact_into_the_loader_s_plain_data() {
        // The seam: delulu-wasm parses the container; delulu-runtime's loader consumes plain data.
        let dir = b"dir payload".to_vec();
        let dpx = write_dpx(&manifest("verified"), Some(&dir), None, None, None);
        let art = WasmPluginEngine::new().read_artifact(&dpx).expect("reads");
        assert_eq!(art.class, "verified");
        assert_eq!(art.api, 1);
        assert_eq!(art.name(), "summarize");
        assert_eq!(art.dir.as_deref(), Some(&dir[..]));
        // The ceiling projects out of the manifest for load step 3.
        assert!(art.ceiling().effects.contains(&delulu_check::Effect::Read));
        assert_eq!(art.exports().len(), 1);
    }

    #[test]
    fn the_engine_maps_a_tampered_dir_to_dl1504_requires_human() {
        let dir = b"the dir payload".to_vec();
        let dpx = write_dpx(&manifest("verified"), Some(&dir), None, None, None);
        let pos = dpx.windows(dir.len()).position(|w| w == &dir[..]).expect("dir present");
        let mut t = dpx.clone();
        t[pos] ^= 0x01;
        let e = WasmPluginEngine::new().read_artifact(&t).expect_err("tampered");
        assert_eq!(e.code, "DL1504");
        assert!(e.requires_human, "a tampered DIR is never machine-repaired, never downgraded");
    }

    // ----- DL1505: the Contained import slice, and its "couldn't tell" cases -----------------
    //
    // Kitchen rule: the "what if we couldn't tell" cases are written FIRST and refuse.

    fn grant_with(effects: &[&str]) -> delulu_runtime::Grant {
        delulu_runtime::Grant { effects: effects.iter().map(|s| s.to_string()).collect(), ..Default::default() }
    }

    /// Build a tiny module importing `(ns, name)` with the given signature. Uses `wasm-encoder`,
    /// already a dependency of this crate — no new dep for a test fixture.
    fn module_importing(
        ns: &str,
        name: &str,
        params: Vec<wasm_encoder::ValType>,
        results: Vec<wasm_encoder::ValType>,
    ) -> Vec<u8> {
        use wasm_encoder::*;
        let mut m = Module::new();
        let mut types = TypeSection::new();
        types.ty().function(params, results);
        m.section(&types);
        let mut imports = ImportSection::new();
        imports.import(ns, name, EntityType::Function(0));
        m.section(&imports);
        m.finish()
    }

    /// A module that imports a MEMORY under a granted name (only functions may cross).
    fn module_importing_memory(ns: &str, name: &str) -> Vec<u8> {
        use wasm_encoder::*;
        let mut m = Module::new();
        let mut imports = ImportSection::new();
        imports.import(
            ns,
            name,
            EntityType::Memory(MemoryType { minimum: 1, maximum: None, memory64: false, shared: false, page_size_log2: None }),
        );
        m.section(&imports);
        m.finish()
    }

    /// A module that imports nothing at all.
    fn module_importing_nothing() -> Vec<u8> {
        use wasm_encoder::*;
        let mut m = Module::new();
        let mut types = TypeSection::new();
        types.ty().function([ValType::I32], [ValType::I32]);
        m.section(&types);
        let mut funcs = FunctionSection::new();
        funcs.function(0);
        m.section(&funcs);
        let mut exports = ExportSection::new();
        exports.export("shout", ExportKind::Func, 0);
        m.section(&exports);
        let mut code = CodeSection::new();
        let mut f = Function::new([]);
        f.instruction(&Instruction::LocalGet(0));
        f.instruction(&Instruction::End);
        code.function(&f);
        m.section(&code);
        m.finish()
    }

    #[test]
    fn dl1505_zero_imports_fits_every_slice() {
        // A module that imports nothing can reach nothing — trivially within any slice. This is the
        // flagship's shape: a pure text transform under Grant { effects: [] }.
        let wasm = module_importing_nothing();
        let e = WasmPluginEngine::new();
        assert!(e.validate_imports(&wasm, &delulu_runtime::plugin::cap_slice(&grant_with(&[]))).is_ok());
        assert!(e.validate_imports(&wasm, &delulu_runtime::plugin::cap_slice(&grant_with(&["Read", "Net"]))).is_ok());
    }

    #[test]
    fn dl1505_an_import_resolving_to_nothing_is_refused() {
        // A name derivable from NO grant: refused by whitelist, never "unknown ⇒ harmless".
        let wasm =
            module_importing("delulu:cap", "definitely_not_a_host_fn", vec![wasm_encoder::ValType::I32], vec![]);
        let e = WasmPluginEngine::new()
            .validate_imports(&wasm, &delulu_runtime::plugin::cap_slice(&grant_with(&["Read", "Write"])))
            .expect_err("an unknown import must refuse");
        assert_eq!(e.code, "DL1505");
    }

    #[test]
    fn dl1505_an_ungranted_but_real_host_import_is_refused() {
        // `console_println` is real, but this grant has no Write — so it is not in the slice.
        let wasm =
            module_importing("delulu:cap", "console_println", vec![wasm_encoder::ValType::I32; 5], vec![]);
        let e = WasmPluginEngine::new()
            .validate_imports(&wasm, &delulu_runtime::plugin::cap_slice(&grant_with(&["Read"])))
            .expect_err("Write is not granted");
        assert_eq!(e.code, "DL1505");
        // With Write granted, the same module fits.
        assert!(WasmPluginEngine::new()
            .validate_imports(&wasm, &delulu_runtime::plugin::cap_slice(&grant_with(&["Write"])))
            .is_ok());
    }

    #[test]
    fn dl1505_wasi_by_the_back_door_is_refused() {
        // A Contained module must not reach WASI. The namespace is in no slice at any grant.
        let wasm = module_importing(
            "wasi_snapshot_preview1",
            "fd_write",
            vec![wasm_encoder::ValType::I32; 4],
            vec![wasm_encoder::ValType::I32],
        );
        for g in [grant_with(&[]), grant_with(&["Read", "Write", "Clock", "Rand"])] {
            let e = WasmPluginEngine::new()
                .validate_imports(&wasm, &delulu_runtime::plugin::cap_slice(&g))
                .expect_err("WASI must never be reachable");
            assert_eq!(e.code, "DL1505");
        }
    }

    /// THE HEAD-CHEF CASE — the DL1505 analogue of the DL1509 bug. The name IS in the slice; the
    /// TYPE is not what the host provides. Matching on name alone would leave the engine's
    /// instantiation type-check as the only gate — "probably refused downstream", which is exactly
    /// the standard rejected on R-6a. Step 5 decides it.
    #[test]
    fn dl1505_a_granted_name_with_the_wrong_type_is_refused_instantiation_is_not_the_gate() {
        // `console_println` is granted under Write, but declared here with a different signature.
        let wasm = module_importing(
            "delulu:cap",
            "console_println",
            vec![wasm_encoder::ValType::I64],
            vec![wasm_encoder::ValType::I64],
        );
        let e = WasmPluginEngine::new()
            .validate_imports(&wasm, &delulu_runtime::plugin::cap_slice(&grant_with(&["Write"])))
            .expect_err("a granted name with a forged signature must refuse at step 5");
        assert_eq!(e.code, "DL1505");
        assert!(
            e.message.contains("the name being in the slice is not enough"),
            "the refusal says WHY name-matching is insufficient: {}",
            e.message
        );
    }

    #[test]
    fn dl1505_a_root_constructor_is_never_in_any_slice() {
        // Invariant 31's shape: a plugin receives capability VALUES from its host and never holds a
        // Root, so it can never mint a capability. `root_console` is in no slice at any grant —
        // the operation does not exist in the plugin's world.
        let wasm = module_importing(
            "delulu:cap",
            "root_console",
            vec![wasm_encoder::ValType::I32],
            vec![wasm_encoder::ValType::I32],
        );
        for g in [grant_with(&["Write"]), grant_with(&["Read", "Write", "Clock", "Rand"])] {
            let e = WasmPluginEngine::new()
                .validate_imports(&wasm, &delulu_runtime::plugin::cap_slice(&g))
                .expect_err("a plugin can never mint a capability");
            assert_eq!(e.code, "DL1505");
        }
    }

    #[test]
    fn dl1505_a_malformed_module_is_refused_not_assumed_harmless() {
        let e = WasmPluginEngine::new()
            .validate_imports(b"\0asm\x01\0\0\0garbage-tail", &delulu_runtime::plugin::cap_slice(&grant_with(&[])))
            .expect_err("a module that does not validate must refuse");
        assert_eq!(e.code, "DL1505");
        assert!(WasmPluginEngine::new()
            .validate_imports(b"not wasm", &delulu_runtime::plugin::cap_slice(&grant_with(&[])))
            .is_err());
    }

    #[test]
    fn dl1505_a_non_function_import_is_refused() {
        // Only functions cross. A memory import is refused rather than ignored.
        let wasm = module_importing_memory("delulu:cap", "console_println");
        let e = WasmPluginEngine::new()
            .validate_imports(&wasm, &delulu_runtime::plugin::cap_slice(&grant_with(&["Write"])))
            .expect_err("a non-function import must refuse");
        assert_eq!(e.code, "DL1505");
    }

    #[test]
    fn duplicate_sections_are_refused() {
        let mut dpx = write_dpx(&manifest("verified"), Some(b"dir"), None, None, None);
        // Append a second delulu:dir section by hand.
        let mut body = Vec::new();
        write_uleb(DIR_SECTION.len() as u32, &mut body);
        body.extend_from_slice(DIR_SECTION.as_bytes());
        body.extend_from_slice(b"second dir");
        dpx.push(0x00);
        write_uleb(body.len() as u32, &mut dpx);
        dpx.extend_from_slice(&body);
        assert!(matches!(read_dpx(&dpx), Err(DpxError::Malformed(_))));
    }
}
