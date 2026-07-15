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
