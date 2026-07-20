//! Phase 5d — the append-only, hash-chained audit log (spec §7, invariant 26).
//!
//! Every issue/attenuate/delegate/revoke/deny and every synchronous-class capability *use* produces
//! exactly one record (ruling 5: each of those already consumed one audit seq in chunk 1, so the log
//! numbers 1,2,3,… with no renumbering). Epoch-class uses produce no record. Records are hash-chained:
//!
//! ```text
//! hash = blake3( prev_hash ‖ canonical_record )
//! ```
//!
//! **Canonicalization (defined precisely).** `canonical_record` is the record's JSON with the `hash`
//! field itself EXCLUDED and every object key sorted lexicographically at every depth (see
//! [`canonical_json`]); absent optional fields (`actor_node`/`authority`/`span`/`target`) are omitted
//! entirely. `prev_hash` is fed to blake3 as the ASCII bytes of its 64-char lowercase-hex string,
//! followed by the canonical-record UTF-8 bytes. The very first record's `prev_hash` is
//! [`GENESIS_HASH`] (64 hex zeros).
//!
//! **The log is OBSERVABILITY, NOT ENFORCEMENT** (playbook trap 6, spec §7). No enforcement logic
//! ever reads it; the first line of every day file states that phrase verbatim. Verification detects
//! tampering *after the fact* (DL1405) — it does not prevent it.
//!
//! Storage is one JSONL file per UTC day, `YYYYMMDD.jsonl`, under an INJECTED base directory (ruling
//! 2 — the library never hardcodes `~/.delulu`; the CLI resolves `~/.delulu/audit`). A new day's file
//! cross-links the previous day's head: the running head hash persists across the day boundary, so the
//! first record of each new file has `prev_hash` = the previous file's last record hash.

use std::cell::RefCell;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::{json, Value};

use crate::diag::Denial;

/// The chain anchor: the `prev_hash` of the very first record ever written (64 hex zeros).
pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// The exact phrase every day file's header line must contain (spec §7).
pub const OBSERVABILITY_PHRASE: &str = "observability, not enforcement";

// ----- record + entry ----------------------------------------------------------------------------

/// The semantic fields of an audit event, before chaining. The broker builds one of these (stamping
/// `ts` from its clock and reusing the `seq` the operation already consumed) and hands it to a sink,
/// which assigns `prev_hash`/`hash`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditEntry {
    pub seq: u64,
    pub ts: i64,
    pub actor_node: Option<String>,
    pub action: String,
    pub target: Option<String>,
    pub authority: Option<Value>,
    pub span: Option<String>,
    pub decision: String,
}

impl AuditEntry {
    /// Chain this entry onto `prev_hash`, producing the full stored record.
    pub fn into_record(self, prev_hash: &str) -> AuditRecord {
        let mut rec = AuditRecord {
            seq: self.seq,
            ts: self.ts,
            prev_hash: prev_hash.to_string(),
            hash: String::new(),
            actor_node: self.actor_node,
            action: self.action,
            target: self.target,
            authority: self.authority,
            span: self.span,
            decision: self.decision,
        };
        let body = rec.body_value();
        rec.hash = chain_hash(prev_hash, &canonical_json(&body));
        rec
    }
}

/// A stored, hash-chained audit record (spec §7 shape).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditRecord {
    pub seq: u64,
    pub ts: i64,
    pub prev_hash: String,
    pub hash: String,
    pub actor_node: Option<String>,
    pub action: String,
    pub target: Option<String>,
    pub authority: Option<Value>,
    pub span: Option<String>,
    pub decision: String,
}

impl AuditRecord {
    /// The canonical value of every field EXCEPT `hash` (the hashed body). Absent optionals omitted.
    fn body_value(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("action".into(), json!(self.action));
        if let Some(a) = &self.actor_node {
            m.insert("actor_node".into(), json!(a));
        }
        if let Some(a) = &self.authority {
            m.insert("authority".into(), a.clone());
        }
        m.insert("decision".into(), json!(self.decision));
        m.insert("prev_hash".into(), json!(self.prev_hash));
        m.insert("seq".into(), json!(self.seq));
        if let Some(s) = &self.span {
            m.insert("span".into(), json!(s));
        }
        if let Some(t) = &self.target {
            m.insert("target".into(), json!(t));
        }
        m.insert("ts".into(), json!(self.ts));
        Value::Object(m)
    }

    /// The full canonical JSON line (body + `hash`) written to disk.
    pub fn to_line(&self) -> String {
        let mut v = self.body_value();
        v.as_object_mut().unwrap().insert("hash".into(), json!(self.hash));
        canonical_json(&v)
    }

    /// Parse a record from an on-disk JSON value. Returns `None` for a header line (no `seq`).
    fn from_value(v: &Value) -> Option<AuditRecord> {
        let obj = v.as_object()?;
        Some(AuditRecord {
            seq: obj.get("seq")?.as_u64()?,
            ts: obj.get("ts")?.as_i64()?,
            prev_hash: obj.get("prev_hash")?.as_str()?.to_string(),
            hash: obj.get("hash")?.as_str()?.to_string(),
            actor_node: obj.get("actor_node").and_then(|x| x.as_str()).map(str::to_string),
            action: obj.get("action")?.as_str()?.to_string(),
            target: obj.get("target").and_then(|x| x.as_str()).map(str::to_string),
            authority: obj.get("authority").cloned(),
            span: obj.get("span").and_then(|x| x.as_str()).map(str::to_string),
            decision: obj.get("decision")?.as_str()?.to_string(),
        })
    }
}

// ----- the sink trait + implementations ----------------------------------------------------------

/// Where the broker sends audit records. Default: none (chunk-1 behavior). Two impls ship: an
/// in-memory [`MemSink`] for unit tests and the file-backed hash-chained [`AuditLog`].
pub trait AuditSink {
    /// Append one entry, chaining it onto the sink's current head. Returns the stored record.
    /// Errors are I/O-class only: the log is observability, so the broker logs a failure and
    /// continues rather than changing any enforcement decision.
    fn append(&mut self, entry: AuditEntry) -> Result<AuditRecord, AuditError>;
}

/// An in-memory, hash-chained sink for unit tests. Cheaply cloneable and shares state, so a test can
/// keep a handle, attach a clone to a [`crate::Broker`], run ops, then read back the records.
#[derive(Clone, Default)]
pub struct MemSink {
    inner: Rc<RefCell<MemState>>,
}

struct MemState {
    records: Vec<AuditRecord>,
    head: String,
}

impl Default for MemState {
    fn default() -> Self {
        MemState { records: Vec::new(), head: GENESIS_HASH.to_string() }
    }
}

impl MemSink {
    pub fn new() -> MemSink {
        MemSink::default()
    }

    /// A snapshot copy of every record appended so far.
    pub fn records(&self) -> Vec<AuditRecord> {
        self.inner.borrow().records.clone()
    }

    pub fn len(&self) -> usize {
        self.inner.borrow().records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.borrow().records.is_empty()
    }
}

impl AuditSink for MemSink {
    fn append(&mut self, entry: AuditEntry) -> Result<AuditRecord, AuditError> {
        let mut st = self.inner.borrow_mut();
        let rec = entry.into_record(&st.head);
        st.head = rec.hash.clone();
        st.records.push(rec.clone());
        Ok(rec)
    }
}

/// The file-backed, hash-chained log: one `YYYYMMDD.jsonl` per UTC day under `dir`, cross-linked
/// across days by a persistent running head.
pub struct AuditLog {
    dir: PathBuf,
    /// The hash of the last record written (or [`GENESIS_HASH`] if none) — the chain head.
    head: String,
    /// The `YYYYMMDD` of the file currently being appended to (so we only write a header once per
    /// file). `None` until the first append or if no day file exists yet.
    current_day: Option<String>,
}

impl AuditLog {
    /// Open (creating `dir` if needed) and recover the chain head from any existing day files.
    pub fn open(dir: impl Into<PathBuf>) -> Result<AuditLog, AuditError> {
        let dir = dir.into();
        fs::create_dir_all(&dir).map_err(AuditError::io)?;
        let days = list_day_files(&dir)?;
        // Head = the hash of the last record across all files, in day order (robust to a trailing
        // header-only file). current_day = the latest existing file, so same-day appends don't
        // rewrite the header.
        let mut head = GENESIS_HASH.to_string();
        for day in &days {
            if let Some(h) = last_record_hash(&day_path(&dir, day))? {
                head = h;
            }
        }
        let current_day = days.last().cloned();
        Ok(AuditLog { dir, head, current_day })
    }

    /// The current chain head (for tests / cross-links).
    pub fn head(&self) -> &str {
        &self.head
    }

    fn ensure_day_file(&mut self, day: &str) -> Result<(), AuditError> {
        if self.current_day.as_deref() == Some(day) {
            return Ok(());
        }
        let path = day_path(&self.dir, day);
        let fresh = match fs::metadata(&path) {
            Ok(m) => m.len() == 0,
            Err(_) => true,
        };
        if fresh {
            append_line(&path, &header_line(day))?;
        }
        self.current_day = Some(day.to_string());
        Ok(())
    }
}

impl AuditSink for AuditLog {
    fn append(&mut self, entry: AuditEntry) -> Result<AuditRecord, AuditError> {
        let day = day_string(entry.ts);
        self.ensure_day_file(&day)?;
        let rec = entry.into_record(&self.head);
        append_line(&day_path(&self.dir, &day), &rec.to_line())?;
        self.head = rec.hash.clone();
        Ok(rec)
    }
}

// ----- verification / read side ------------------------------------------------------------------

/// The outcome of a successful [`verify`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedStats {
    pub files: usize,
    pub records: usize,
    /// The chain head after the last verified record.
    pub head: String,
}

/// A filter for [`query`]. Simple by design (spec §7): by node id (matches `actor_node` OR `target`),
/// by `action`, and by effect (present in the record's `authority.effects`). `None` fields match all.
#[derive(Clone, Debug, Default)]
pub struct QueryFilter {
    pub node: Option<String>,
    pub action: Option<String>,
    pub effect: Option<String>,
}

impl QueryFilter {
    fn matches(&self, r: &AuditRecord) -> bool {
        if let Some(node) = &self.node {
            let hit = r.actor_node.as_deref() == Some(node.as_str())
                || r.target.as_deref() == Some(node.as_str());
            if !hit {
                return false;
            }
        }
        if let Some(action) = &self.action {
            if &r.action != action {
                return false;
            }
        }
        if let Some(effect) = &self.effect {
            let hit = r
                .authority
                .as_ref()
                .and_then(|a| a.get("effects"))
                .and_then(|e| e.as_array())
                .map(|arr| arr.iter().any(|x| x.as_str() == Some(effect.as_str())))
                .unwrap_or(false);
            if !hit {
                return false;
            }
        }
        true
    }
}

/// Errors from the file-backed log. `Corrupt` carries a DL1405 [`Denial`] (spec §8: `requires_human`).
#[derive(Debug)]
pub enum AuditError {
    /// An I/O error (create/read/append). Never a policy signal.
    Io(String),
    /// A hash-chain verification failure — the DL1405 denial (with the failing seq).
    Corrupt(Denial),
}

impl AuditError {
    fn io(e: std::io::Error) -> AuditError {
        AuditError::Io(e.to_string())
    }
    fn corrupt(seq: u64, detail: impl Into<String>) -> AuditError {
        AuditError::Corrupt(Denial::AuditChainBroken { seq, detail: detail.into() })
    }
    /// The DL1405 denial, if this is a corruption error.
    pub fn denial(&self) -> Option<&Denial> {
        match self {
            AuditError::Corrupt(d) => Some(d),
            _ => None,
        }
    }
}

impl std::fmt::Display for AuditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditError::Io(m) => write!(f, "audit i/o error: {m}"),
            AuditError::Corrupt(d) => write!(f, "{}", d.to_diagnostic().message),
        }
    }
}

/// Verify the whole chain across every day file in `dir`, in order, recomputing each record's hash
/// and every cross-record (and cross-file) `prev_hash` linkage. Any mismatch is DL1405 at the failing
/// seq (spec §8, `requires_human`).
pub fn verify(dir: impl AsRef<Path>) -> Result<VerifiedStats, AuditError> {
    let dir = dir.as_ref();
    let days = list_day_files(dir)?;
    let mut expected_prev = GENESIS_HASH.to_string();
    let mut files = 0usize;
    let mut records = 0usize;
    for day in &days {
        files += 1;
        let text = fs::read_to_string(day_path(dir, day)).map_err(AuditError::io)?;
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let v: Value = serde_json::from_str(line)
                .map_err(|e| AuditError::corrupt(0, format!("malformed JSON line: {e}")))?;
            if v.get("seq").is_none() {
                // Header line (or any non-record) — not part of the chain.
                continue;
            }
            let seq = v.get("seq").and_then(|x| x.as_u64());
            let stored_hash = v.get("hash").and_then(|x| x.as_str());
            let prev = v.get("prev_hash").and_then(|x| x.as_str());
            let (Some(seq), Some(stored_hash), Some(prev)) = (seq, stored_hash, prev) else {
                return Err(AuditError::corrupt(seq.unwrap_or(0), "record missing seq/hash/prev_hash"));
            };
            if prev != expected_prev {
                return Err(AuditError::corrupt(
                    seq,
                    format!("prev_hash chain break: expected {expected_prev}, found {prev}"),
                ));
            }
            let mut body = v.clone();
            body.as_object_mut().unwrap().remove("hash");
            let recomputed = chain_hash(prev, &canonical_json(&body));
            if recomputed != stored_hash {
                return Err(AuditError::corrupt(seq, "record hash mismatch (possible tamper)"));
            }
            expected_prev = stored_hash.to_string();
            records += 1;
        }
    }
    Ok(VerifiedStats { files, records, head: expected_prev })
}

/// The last `n` records across every day file in `dir`, in chain order (oldest → newest).
pub fn tail(dir: impl AsRef<Path>, n: usize) -> Result<Vec<AuditRecord>, AuditError> {
    let all = read_all_records(dir.as_ref())?;
    let start = all.len().saturating_sub(n);
    Ok(all[start..].to_vec())
}

/// Every record in `dir` matching `filter`, in chain order.
pub fn query(dir: impl AsRef<Path>, filter: &QueryFilter) -> Result<Vec<AuditRecord>, AuditError> {
    let all = read_all_records(dir.as_ref())?;
    Ok(all.into_iter().filter(|r| filter.matches(r)).collect())
}

// ----- internals ---------------------------------------------------------------------------------

/// Read every record (skipping headers) across sorted day files, in order. Does NOT verify the chain
/// (that is [`verify`]) — this is the read surface for `tail`/`query`.
fn read_all_records(dir: &Path) -> Result<Vec<AuditRecord>, AuditError> {
    let days = list_day_files(dir)?;
    let mut out = Vec::new();
    for day in &days {
        let text = fs::read_to_string(day_path(dir, day)).map_err(AuditError::io)?;
        for line in text.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let v: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if let Some(rec) = AuditRecord::from_value(&v) {
                out.push(rec);
            }
        }
    }
    Ok(out)
}

/// The sorted list of `YYYYMMDD` day names present in `dir` (files named `YYYYMMDD.jsonl`).
fn list_day_files(dir: &Path) -> Result<Vec<String>, AuditError> {
    let mut days = Vec::new();
    let rd = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(days), // a missing dir has no records
    };
    for entry in rd {
        let entry = entry.map_err(AuditError::io)?;
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(stem) = name.strip_suffix(".jsonl") {
            if stem.len() == 8 && stem.chars().all(|c| c.is_ascii_digit()) {
                days.push(stem.to_string());
            }
        }
    }
    days.sort(); // lexical == chronological for zero-padded YYYYMMDD
    Ok(days)
}

fn day_path(dir: &Path, day: &str) -> PathBuf {
    dir.join(format!("{day}.jsonl"))
}

fn last_record_hash(path: &Path) -> Result<Option<String>, AuditError> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return Ok(None),
    };
    let mut last = None;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if let Some(h) = v.get("hash").and_then(|x| x.as_str()) {
                if v.get("seq").is_some() {
                    last = Some(h.to_string());
                }
            }
        }
    }
    Ok(last)
}

fn append_line(path: &Path, line: &str) -> Result<(), AuditError> {
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(AuditError::io)?;
    f.write_all(line.as_bytes()).map_err(AuditError::io)?;
    f.write_all(b"\n").map_err(AuditError::io)?;
    Ok(())
}

fn header_line(day: &str) -> String {
    // Contains OBSERVABILITY_PHRASE verbatim (spec §7). No `seq` key → skipped by the record parser.
    canonical_json(&json!({
        "audit": "delulu",
        "day": day,
        "note": OBSERVABILITY_PHRASE,
        "version": 1,
    }))
}

/// The content hash of an arbitrary byte string, `blake3:<64 hex>`. Stage 10 phase 10f uses it
/// for the sim-to-hardware artifact gate (spec §5.4, DL1905): the bytes that were exercised in
/// simulation are the bytes a hardware grant is checked against. Exposed here rather than
/// re-derived at the call site so every hash in the system comes from one implementation.
pub fn content_hash(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

/// `hash = blake3( prev_hash_ascii_bytes ‖ canonical_body_utf8_bytes )`, lowercase hex.
fn chain_hash(prev_hash: &str, canonical_body: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(canonical_body.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// A deterministic canonical JSON encoding: object keys sorted lexicographically at every depth,
/// arrays in order, no insignificant whitespace. Independent of any serde_json `preserve_order`
/// feature, so the hash is stable regardless of workspace feature unification.
pub fn canonical_json(v: &Value) -> String {
    let mut s = String::new();
    write_canonical(v, &mut s);
    s
}

fn write_canonical(v: &Value, out: &mut String) {
    match v {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(k).expect("string key encodes"));
                out.push(':');
                write_canonical(&map[*k], out);
            }
            out.push('}');
        }
        Value::Array(arr) => {
            out.push('[');
            for (i, e) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(e, out);
            }
            out.push(']');
        }
        other => out.push_str(&serde_json::to_string(other).expect("scalar encodes")),
    }
}

// ----- calendar (epoch millis → YYYYMMDD, no datetime dep) ----------------------------------------

/// The `YYYYMMDD` UTC day of an epoch-millis timestamp.
fn day_string(ts_millis: i64) -> String {
    let days = ts_millis.div_euclid(86_400_000);
    let (y, m, d) = civil_from_days(days);
    let mut s = String::with_capacity(8);
    let _ = write!(s, "{y:04}{m:02}{d:02}");
    s
}

/// Render an epoch-millis timestamp as `YYYY-MM-DDTHH:MM:SS.mmmZ` (UTC). DISPLAY ONLY (chunk-1
/// ruling: stored/serialized timestamps are epoch millis; ISO is a CLI/human rendering concern).
pub fn render_ts_utc(ts_millis: i64) -> String {
    let days = ts_millis.div_euclid(86_400_000);
    let rem = ts_millis.rem_euclid(86_400_000);
    let (y, m, d) = civil_from_days(days);
    let (h, min, s, ms) = (rem / 3_600_000, rem / 60_000 % 60, rem / 1000 % 60, rem % 1000);
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{min:02}:{s:02}.{ms:03}Z")
}

/// Howard Hinnant's `civil_from_days`: days-since-1970-01-01 → (year, month, day). Pure integer math.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("delulu_audit_{}_{}", std::process::id(), tag));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn entry(seq: u64, ts: i64, action: &str, actor: &str, decision: &str) -> AuditEntry {
        AuditEntry {
            seq,
            ts,
            actor_node: Some(actor.into()),
            action: action.into(),
            target: None,
            authority: None,
            span: None,
            decision: decision.into(),
        }
    }

    #[test]
    fn civil_from_days_known_dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        // 2026-07-11 is 20645 days after the epoch (56y = 20454d to 2026-01-01, +191d to Jul 11).
        assert_eq!(day_string(20_645 * 86_400_000), "20260711");
    }

    #[test]
    fn render_ts_utc_is_iso_shaped() {
        assert_eq!(render_ts_utc(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(render_ts_utc(20_645 * 86_400_000 + 3_723_456), "2026-07-11T01:02:03.456Z");
    }

    #[test]
    fn canonical_json_sorts_keys_deeply() {
        let v = json!({ "b": 1, "a": { "z": 2, "y": 3 } });
        assert_eq!(canonical_json(&v), r#"{"a":{"y":3,"z":2},"b":1}"#);
    }

    #[test]
    fn mem_sink_chains_and_first_prev_is_genesis() {
        let mut s = MemSink::new();
        let r1 = s.append(entry(1, 1000, "issue", "g_a", "allow")).unwrap();
        let r2 = s.append(entry(2, 1001, "revoke", "g_a", "allow")).unwrap();
        assert_eq!(r1.prev_hash, GENESIS_HASH);
        assert_eq!(r2.prev_hash, r1.hash, "each record chains onto the previous hash");
        assert_ne!(r1.hash, r2.hash);
    }

    #[test]
    fn write_then_verify_passes_and_header_has_the_phrase() {
        let dir = tmp_dir("verify_ok");
        {
            let mut log = AuditLog::open(&dir).unwrap();
            for i in 1..=5 {
                log.append(entry(i, 1000 + i as i64, "use", "g_x", "allow")).unwrap();
            }
        }
        let stats = verify(&dir).unwrap();
        assert_eq!(stats.records, 5);
        assert_eq!(stats.files, 1);
        // The header line states the exact phrase.
        let day = day_string(1001);
        let text = fs::read_to_string(day_path(&dir, &day)).unwrap();
        let first = text.lines().next().unwrap();
        assert!(first.contains(OBSERVABILITY_PHRASE), "header states the phrase: {first}");
        assert!(!first.contains("\"seq\""), "header is not a record");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupting_one_byte_fails_verify_at_the_correct_seq() {
        let dir = tmp_dir("verify_tamper");
        {
            let mut log = AuditLog::open(&dir).unwrap();
            for i in 1..=4 {
                log.append(entry(i, 1000 + i as i64, "use", "g_deadbeef", "allow")).unwrap();
            }
        }
        // Corrupt one byte inside the actor_node of the seq-3 record — keeps the line valid JSON, so
        // the reported seq is exact; the recomputed hash no longer matches the stored hash.
        let day = day_string(1001);
        let path = day_path(&dir, &day);
        let text = fs::read_to_string(&path).unwrap();
        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        // lines[0] = header, then records 1..=4; the seq-3 record is lines[3].
        let target = &mut lines[3];
        assert!(target.contains("\"seq\":3"), "sanity: line 3 is the seq-3 record: {target}");
        *target = target.replacen("g_deadbeef", "g_deadb3ef", 1);
        fs::write(&path, lines.join("\n") + "\n").unwrap();

        let err = verify(&dir).unwrap_err();
        let denial = err.denial().expect("corruption yields a DL1405 denial");
        assert_eq!(denial.code(), "DL1405");
        assert_eq!(denial.audit_break_seq(), Some(3), "DL1405 registers the failing seq");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn daily_rotation_cross_links_heads() {
        let dir = tmp_dir("rotation");
        // Day 1: two records at ts on 1970-01-02; Day 2: two records the next day. No sleeps — the
        // timestamps are supplied directly (in the broker these come from the injected clock).
        let day1_ts = 86_400_000 + 500; // 1970-01-02
        let day2_ts = 2 * 86_400_000 + 500; // 1970-01-03
        let (h_day1_last, h_day2_first_prev);
        {
            let mut log = AuditLog::open(&dir).unwrap();
            log.append(entry(1, day1_ts, "issue", "g_a", "allow")).unwrap();
            let r2 = log.append(entry(2, day1_ts + 1, "use", "g_a", "allow")).unwrap();
            h_day1_last = r2.hash.clone();
            let r3 = log.append(entry(3, day2_ts, "use", "g_a", "allow")).unwrap();
            h_day2_first_prev = r3.prev_hash.clone();
        }
        assert_eq!(list_day_files(&dir).unwrap(), vec!["19700102", "19700103"]);
        assert_eq!(h_day2_first_prev, h_day1_last, "new day's first record cross-links the previous head");
        // The whole two-file chain verifies.
        let stats = verify(&dir).unwrap();
        assert_eq!(stats.records, 3);
        assert_eq!(stats.files, 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn reopening_recovers_the_head_and_continues_the_chain() {
        let dir = tmp_dir("reopen");
        let first_hash;
        {
            let mut log = AuditLog::open(&dir).unwrap();
            first_hash = log.append(entry(1, 1000, "issue", "g_a", "allow")).unwrap().hash;
        }
        // Reopen: the second record must chain onto the recovered head, not GENESIS.
        let mut log = AuditLog::open(&dir).unwrap();
        assert_eq!(log.head(), first_hash);
        let r2 = log.append(entry(2, 1001, "use", "g_a", "allow")).unwrap();
        assert_eq!(r2.prev_hash, first_hash);
        assert!(verify(&dir).is_ok());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn tail_and_query_filter() {
        let dir = tmp_dir("query");
        {
            let mut log = AuditLog::open(&dir).unwrap();
            log.append(AuditEntry {
                seq: 1,
                ts: 1000,
                actor_node: Some("g_a".into()),
                action: "issue".into(),
                target: None,
                authority: Some(json!({ "effects": ["Read", "Net"], "scopes": {} })),
                span: None,
                decision: "allow".into(),
            })
            .unwrap();
            log.append(entry(2, 1001, "revoke", "g_a", "allow")).unwrap();
            log.append(entry(3, 1002, "use", "g_b", "deny")).unwrap();
        }
        assert_eq!(tail(&dir, 2).unwrap().len(), 2);
        assert_eq!(tail(&dir, 100).unwrap().len(), 3);

        let by_action = query(&dir, &QueryFilter { action: Some("revoke".into()), ..Default::default() }).unwrap();
        assert_eq!(by_action.len(), 1);
        assert_eq!(by_action[0].seq, 2);

        let by_node = query(&dir, &QueryFilter { node: Some("g_a".into()), ..Default::default() }).unwrap();
        assert_eq!(by_node.len(), 2, "g_a appears as actor in the issue and revoke records");

        let by_effect = query(&dir, &QueryFilter { effect: Some("Net".into()), ..Default::default() }).unwrap();
        assert_eq!(by_effect.len(), 1, "only the issue record's authority carries Net");
        let _ = fs::remove_dir_all(&dir);
    }
}
