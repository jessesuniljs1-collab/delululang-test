//! The `.dwx` artifact (spec §5): a WebAssembly module that carries its compiler-computed authority
//! as a `delulu:authority` custom section, hash-bound to the code it describes. `delulu build
//! --target wasm` writes one; `delulu run <file>.dwx` re-verifies it before running.
//!
//! Honesty (Constitution): the embedded blake3 hash binds the authority claim to *these* code
//! bytes, so corruption or a naive code/authority swap is detected (DL1202). It is **not** a
//! cryptographic signature — it does not prove *who* produced the artifact. Publisher signing is a
//! later stage; we don't claim tamper-proofness against an adversary who simply re-runs the builder.

use serde_json::Value;

/// The wasm custom-section name that carries the authority manifest.
pub const AUTHORITY_SECTION: &str = "delulu:authority";
/// The `.dwx` manifest format version this toolchain writes and understands.
pub const DWX_VERSION: u32 = 1;

/// Why a `.dwx` failed to load or re-verify. Everything maps to DL1202 except an artifact from a
/// newer toolchain (DL1204 — version this toolchain doesn't support).
#[derive(Debug, Clone)]
pub enum ArtifactError {
    NotWasm,
    Malformed(String),
    NoSection,
    BadJson(String),
    Tampered { declared: String, actual: String },
    UnsupportedVersion(u32),
}

impl ArtifactError {
    pub fn message(&self) -> String {
        match self {
            ArtifactError::NotWasm => "not a WebAssembly module (bad magic)".to_string(),
            ArtifactError::Malformed(w) => format!("malformed module: {w}"),
            ArtifactError::NoSection => format!(
                "no `{AUTHORITY_SECTION}` section — not a Delulu artifact (build one with `delulu build --target wasm`)"
            ),
            ArtifactError::BadJson(w) => format!("`{AUTHORITY_SECTION}` section is not valid authority JSON: {w}"),
            ArtifactError::Tampered { declared, actual } => format!(
                "code does not match its authority manifest (manifest binds code {}…, actual code hashes to {}…) — the artifact was altered after its authority was computed",
                short(declared), short(actual)
            ),
            ArtifactError::UnsupportedVersion(v) => format!(
                "artifact manifest version {v} is newer than this toolchain supports (v{DWX_VERSION}) — upgrade `delulu`"
            ),
        }
    }

    /// The diagnostic code the CLI reports for this fault.
    pub fn code(&self) -> &'static str {
        match self {
            ArtifactError::UnsupportedVersion(_) => "DL1204",
            _ => "DL1202",
        }
    }
}

fn short(h: &str) -> &str {
    &h[..h.len().min(12)]
}

/// A verified `.dwx`: its declared authority and the runnable wasm (custom sections are ignored at
/// run time, so the full artifact bytes run directly under Wasmtime).
#[derive(Debug)]
pub struct Artifact {
    pub version: u32,
    /// The authority report the artifact declares — the same shape as `delulu authority --json`.
    pub authority: Value,
    /// The full artifact bytes, runnable as-is.
    pub wasm: Vec<u8>,
}

/// Wrap a bare wasm module (a `compile_module` result) into `.dwx` bytes: append a
/// `delulu:authority` custom section carrying `{version, code_blake3, authority}`, where
/// `code_blake3` hashes the bare module — binding the manifest to the exact code it describes.
pub fn embed_authority(bare_wasm: &[u8], authority: &Value) -> Vec<u8> {
    let code_hash = blake3::hash(bare_wasm).to_hex().to_string();
    let manifest = serde_json::json!({
        "version": DWX_VERSION,
        "code_blake3": code_hash,
        "authority": authority,
    });
    let payload = serde_json::to_vec(&manifest).expect("manifest serializes");

    // Custom section body = name (a length-prefixed UTF-8 "name") followed by the payload bytes.
    let name = AUTHORITY_SECTION.as_bytes();
    let mut body = Vec::new();
    write_uleb(name.len() as u32, &mut body);
    body.extend_from_slice(name);
    body.extend_from_slice(&payload);

    // A custom section (id 0) appended after the last section keeps the module valid.
    let mut out = bare_wasm.to_vec();
    out.push(0x00);
    write_uleb(body.len() as u32, &mut out);
    out.extend_from_slice(&body);
    out
}

/// Parse and re-verify a `.dwx`: locate the `delulu:authority` section, check its format/version,
/// and confirm the embedded code hash matches the module's actual code (the tamper check).
pub fn read_and_verify(wasm: &[u8]) -> Result<Artifact, ArtifactError> {
    if wasm.len() < 8 || &wasm[0..4] != b"\0asm" {
        return Err(ArtifactError::NotWasm);
    }

    // Reconstruct the "bare" module (header + every non-authority section, original order) to hash.
    let mut bare = wasm[0..8].to_vec();
    let mut payload: Option<Vec<u8>> = None;
    let mut i = 8usize;
    while i < wasm.len() {
        let sec_start = i;
        let id = wasm[i];
        i += 1;
        let (size, n) = read_uleb(&wasm[i..]).ok_or_else(|| ArtifactError::Malformed("truncated section size".into()))?;
        i += n;
        let body_start = i;
        let body_end = body_start
            .checked_add(size as usize)
            .filter(|&e| e <= wasm.len())
            .ok_or_else(|| ArtifactError::Malformed("section runs past end of module".into()))?;

        if id == 0 {
            let body = &wasm[body_start..body_end];
            let (name_len, m) = read_uleb(body).ok_or_else(|| ArtifactError::Malformed("truncated custom-section name".into()))?;
            let name_end = m
                .checked_add(name_len as usize)
                .filter(|&e| e <= body.len())
                .ok_or_else(|| ArtifactError::Malformed("custom-section name past end".into()))?;
            if &body[m..name_end] == AUTHORITY_SECTION.as_bytes() {
                payload = Some(body[name_end..].to_vec());
                i = body_end;
                continue; // exclude the authority section itself from the hashed code
            }
        }
        bare.extend_from_slice(&wasm[sec_start..body_end]);
        i = body_end;
    }

    let payload = payload.ok_or(ArtifactError::NoSection)?;
    let manifest: Value = serde_json::from_slice(&payload).map_err(|e| ArtifactError::BadJson(e.to_string()))?;
    let version = manifest
        .get("version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ArtifactError::BadJson("missing `version`".into()))? as u32;
    if version > DWX_VERSION {
        return Err(ArtifactError::UnsupportedVersion(version));
    }
    let declared = manifest
        .get("code_blake3")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ArtifactError::BadJson("missing `code_blake3`".into()))?
        .to_string();
    let authority = manifest
        .get("authority")
        .cloned()
        .ok_or_else(|| ArtifactError::BadJson("missing `authority`".into()))?;

    let actual = blake3::hash(&bare).to_hex().to_string();
    if actual != declared {
        return Err(ArtifactError::Tampered { declared, actual });
    }
    Ok(Artifact { version, authority, wasm: wasm.to_vec() })
}

fn write_uleb(mut v: u32, out: &mut Vec<u8>) {
    loop {
        let mut byte = (v & 0x7f) as u8;
        v >>= 7;
        if v != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if v == 0 {
            break;
        }
    }
}

fn read_uleb(bytes: &[u8]) -> Option<(u32, usize)> {
    let mut result: u32 = 0;
    let mut shift = 0u32;
    let mut i = 0usize;
    loop {
        let byte = *bytes.get(i)?;
        result |= ((byte & 0x7f) as u32).checked_shl(shift)?;
        i += 1;
        if byte & 0x80 == 0 {
            return Some((result, i));
        }
        shift += 7;
        if shift >= 32 {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compile_module, run_main_console};
    use delulu_check::check_source;
    use serde_json::json;

    fn hello_wasm() -> Vec<u8> {
        let src = "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n out.println(\"hi from dwx\") }\n";
        let checked = check_source(0, src);
        assert!(!checked.has_errors(), "{:?}", checked.diagnostics);
        compile_module(&checked.module).expect("compile")
    }

    #[test]
    fn round_trip_embeds_and_verifies_authority() {
        let bare = hello_wasm();
        let authority = json!({ "program": "m", "effects": ["Write"], "capabilities": [{ "kind": "Console", "scopes": ["stdio"] }] });
        let dwx = embed_authority(&bare, &authority);
        assert!(dwx.len() > bare.len(), "the artifact must carry the extra section");

        let art = read_and_verify(&dwx).expect("verifies");
        assert_eq!(art.version, DWX_VERSION);
        assert_eq!(art.authority, authority, "the declared authority round-trips exactly");
    }

    #[test]
    fn verified_artifact_still_runs_under_wasmtime() {
        // The custom section must not disturb execution: the full .dwx runs `main` normally.
        let bare = hello_wasm();
        let dwx = embed_authority(&bare, &json!({ "program": "m", "effects": ["Write"] }));
        let art = read_and_verify(&dwx).expect("verifies");
        assert_eq!(run_main_console(&art.wasm, true).expect("run"), "hi from dwx\n");
    }

    #[test]
    fn tampered_code_is_detected() {
        // Bind an authority manifest to one module, then splice that (still-valid) section onto a
        // DIFFERENT valid module. The module parses cleanly, but its code no longer matches the
        // hash the manifest declares — exactly the swap the hash binding exists to catch.
        let bare1 = hello_wasm();
        let dwx1 = embed_authority(&bare1, &json!({ "effects": ["Write"] }));
        let section = dwx1[bare1.len()..].to_vec(); // the appended `delulu:authority` section

        let src2 = "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n out.println(\"different code\") }\n";
        let checked2 = check_source(0, src2);
        let bare2 = compile_module(&checked2.module).expect("compile");
        assert_ne!(bare1, bare2, "the two modules must differ so their hashes differ");

        let mut tampered = bare2.clone();
        tampered.extend_from_slice(&section);
        match read_and_verify(&tampered) {
            Err(ArtifactError::Tampered { .. }) => {}
            other => panic!("swapped code must be caught as Tampered, got {other:?}"),
        }
    }

    #[test]
    fn missing_section_is_no_section() {
        let bare = hello_wasm();
        match read_and_verify(&bare) {
            Err(ArtifactError::NoSection) => {}
            other => panic!("a bare module has no authority section, got {other:?}"),
        }
        assert_eq!(ArtifactError::NoSection.code(), "DL1202");
    }

    #[test]
    fn non_wasm_bytes_are_rejected() {
        assert!(matches!(read_and_verify(b"not wasm at all"), Err(ArtifactError::NotWasm)));
    }
}
