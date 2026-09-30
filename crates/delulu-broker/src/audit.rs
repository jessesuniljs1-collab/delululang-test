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
        canonical_json(&self.to_value())
    }

    /// The record as a JSON value: the body and its `hash` (PS-E-06 embeds it in each OCSF event).
    pub fn to_value(&self) -> Value {
        let mut v = self.body_value();
        v.as_object_mut().unwrap().insert("hash".into(), json!(self.hash));
        v
    }

    /// Parse a record from an on-disk JSON value. Returns `None` for a header line (no `seq`).
    pub(crate) fn from_value(v: &Value) -> Option<AuditRecord> {
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
    /// How many records the chain holds. Written to the anchor alongside the head so that a
    /// truncation is detectable even in the (impossible-by-hash but cheap-to-check) case where a
    /// shortened chain somehow ended on the same head.
    records: usize,
    /// The byte length of `current_day`'s file after this handle last touched it. With the anchor,
    /// this is how an append notices that ANOTHER writer moved the chain since (AUDIT-WRITERS-1).
    tail_len: Option<u64>,
}

/// The append lock's file name, beside the day files and the anchor (neither reader lists it).
pub const APPEND_LOCK_FILE: &str = "append.lock";

/// Exclusive access to an audit directory for the length of one append or one recovery.
///
/// # Campaign finding AUDIT-WRITERS-1 (2026-09-27)
///
/// The chain was designed single-writer — the broker daemon owns it — and it stopped being that the
/// day sandboxed runs began recording `sandbox-launch`/`sandbox-death` in it, as `reconcile` and a
/// run's own records already did. Each writer caches the head it opened with. So a sandboxed run
/// under a lease appended two records, and the daemon's next record chained onto the head it had
/// cached BEFORE them: `audit verify` failed DL1405 "prev_hash chain break" on a chain nobody had
/// tampered with. Found by the end-to-end Guard test (`sandbox_guard_e2e.rs`), the first test to
/// put both writers on one chain.
///
/// Now every write goes through this lock, and under it [`AuditLog::append`] first catches up with
/// what any other writer did.
///
/// # Campaign finding AUDIT-LOCK-TAKEOVER-1 (2026-09-28)
///
/// The first lock was a `create_new` file, taken over after five seconds so that a writer killed
/// mid-append could not block the chain for ever. But a holder that is only SLOW is not dead. Under
/// Miri, a hundred times slower, a waiter took the lock from a live writer mid-append, both wrote, and
/// the chain disagreed with its own anchor — which `verify` reports as truncation (the `miri-slow`
/// run of `4b583e4`); a paused process, a slow disk or a swapping host does the same natively. And two
/// waiters timing out together could each delete the other's fresh lock and both hold it.
///
/// Now the lock is the operating system's — `flock` on Unix, `LockFileEx` on Windows, through
/// `File::try_lock` — on a file that is never deleted. The OS releases it when its holder's handle
/// closes, including when the process dies, so a dead writer needs no takeover and a live one is never
/// taken over. A wait that runs out FAILS the append, in words, rather than writing beside another
/// writer; the daemon then refuses a synchronous operation it could not record (invariant 26).
pub struct AppendLock {
    /// Held open for as long as the lock is: the OS lock belongs to this handle and ends with it.
    _file: fs::File,
}

/// How long a writer waits for another: far longer than any real append, short enough that an audit
/// record cannot hang a run. Under Miri every append is a hundred times slower and the wall clock is
/// not what it examines, so the bound there is an hour.
const LOCK_WAIT: std::time::Duration =
    if cfg!(miri) { std::time::Duration::from_secs(3600) } else { std::time::Duration::from_secs(5) };

impl AppendLock {
    /// Take the lock on `dir`, waiting for another writer as long as [`LOCK_WAIT`].
    pub fn take(dir: &Path) -> Result<AppendLock, AuditError> {
        AppendLock::take_within(dir, LOCK_WAIT)
    }

    fn take_within(dir: &Path, wait: std::time::Duration) -> Result<AppendLock, AuditError> {
        let path = dir.join(APPEND_LOCK_FILE);
        let file = fs::OpenOptions::new().read(true).write(true).create(true).truncate(false).open(&path).map_err(|e| {
            AuditError::Io(format!("the audit chain's append lock `{}` could not be opened: {e}", path.display()))
        })?;
        let deadline = std::time::Instant::now() + wait;
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(AppendLock { _file: file }),
                Err(fs::TryLockError::WouldBlock) if std::time::Instant::now() < deadline => {
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                Err(fs::TryLockError::WouldBlock) => {
                    return Err(AuditError::Io(format!(
                        "another writer held the audit chain's append lock `{}` for {:.1} s — nothing was written",
                        path.display(),
                        wait.as_secs_f64()
                    )))
                }
                Err(fs::TryLockError::Error(e)) => {
                    return Err(AuditError::Io(format!(
                        "the audit chain's append lock `{}` could not be taken: {e}",
                        path.display()
                    )))
                }
            }
        }
    }
}

/// The anchor file's name. Deliberately NOT a `.jsonl` day file, so `list_day_files` never sees it.
const ANCHOR_FILE: &str = "ANCHOR.json";

fn anchor_path(dir: &Path) -> PathBuf {
    dir.join(ANCHOR_FILE)
}

/// Write the chain anchor: the head hash and the record count, outside the log itself.
///
/// # What this closes, and what it does not — stated because an anchor is easy to oversell
///
/// `verify` checks each record's `prev_hash` and recomputes its hash, and **every one of those
/// checks is local to a link**. Deleting the last *k* records therefore leaves a chain in which
/// every remaining link is still correct, so verification passed on a log with its history cut off
/// (campaign finding P17-C1). Nothing anchored the head: `AuditLog::open` *recovers* it by reading
/// the files, so the broker resumed chaining from the truncated head and every later record was
/// genuinely valid.
///
/// The anchor makes that detectable. It does **not** make the log tamper-proof: an attacker with
/// write access to this directory can rewrite the anchor as well as the log. What it buys is real
/// but bounded:
///
/// * **Accidental truncation is caught** — a partial write, a full disk, a botched rotation, a
///   half-finished sync. That is an ordinary operational failure and it used to pass silently.
/// * **Naive tampering is caught** — deleting lines from a `.jsonl` needs a text editor; producing
///   a consistent anchor needs the format.
/// * **The head becomes exportable.** `verify` returns it and it is now written down, so an
///   operator can witness it elsewhere and check a later run against a value the attacker never
///   had. That is the only route to genuine tamper-evidence, and it requires an external witness —
///   which no file inside this directory can be.
fn write_anchor(dir: &Path, head: &str, records: usize) -> Result<(), AuditError> {
    let body = format!("{{\"head\":\"{head}\",\"records\":{records}}}\n");
    fs::write(anchor_path(dir), body).map_err(AuditError::io)
}

/// Read the anchor, if one exists. `None` means an unanchored log (one written before anchoring
/// existed) — reported by [`verify`] rather than treated as agreement.
fn read_anchor(dir: &Path) -> Option<(String, usize)> {
    let text = fs::read_to_string(anchor_path(dir)).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    let head = v.get("head")?.as_str()?.to_string();
    let records = v.get("records")?.as_u64()? as usize;
    Some((head, records))
}

/// What the files say — the head, the latest day file, the record count — checked against the
/// anchor, and the anchor written if there is none. Called under the append lock.
fn recover(dir: &Path) -> Result<(String, Option<String>, usize), AuditError> {
    let days = list_day_files(dir)?;
    // Head = the hash of the last record across all files, in day order (robust to a trailing
    // header-only file). current_day = the latest existing file, so same-day appends don't
    // rewrite the header.
    let mut head = GENESIS_HASH.to_string();
    for day in &days {
        if let Some(h) = last_record_hash(&day_path(dir, day))? {
            head = h;
        }
    }
    let current_day = days.last().cloned();
    // Count what is on disk so the anchor can be refreshed. A log opened for the first time
    // gets its anchor here; an existing one gets it re-affirmed only if it already AGREES —
    // silently rewriting a disagreeing anchor would erase the very evidence it exists to keep.
    let mut records = 0usize;
    for day in &days {
        records += count_records(&day_path(dir, day))?;
    }
    match read_anchor(dir) {
        Some((a_head, a_records)) if a_head != head || a_records != records => {
            return Err(AuditError::corrupt(
                records as u64,
                format!(
                    "audit anchor disagrees with the log: anchor says {a_records} record(s) \
                     ending {a_head}, the files hold {records} ending {head} — the chain has \
                     been truncated, replaced or rolled back"
                ),
            ));
        }
        _ => write_anchor(dir, &head, records)?,
    }
    Ok((head, current_day, records))
}

fn file_len(path: &Path) -> Option<u64> {
    fs::metadata(path).ok().map(|m| m.len())
}

impl AuditLog {
    /// Open (creating `dir` if needed) and recover the chain head from any existing day files.
    pub fn open(dir: impl Into<PathBuf>) -> Result<AuditLog, AuditError> {
        let dir = dir.into();
        fs::create_dir_all(&dir).map_err(AuditError::io)?;
        // Under the lock: a recovery that raced another writer's append would see its record
        // without its anchor, and refuse a chain that is fine.
        let _lock = AppendLock::take(&dir)?;
        let (head, current_day, records) = recover(&dir)?;
        let tail_len = current_day.as_deref().and_then(|d| file_len(&day_path(&dir, d)));
        Ok(AuditLog { dir, head, current_day, records, tail_len })
    }

    /// The current chain head (for tests / cross-links).
    pub fn head(&self) -> &str {
        &self.head
    }

    /// Catch up with any other writer (AUDIT-WRITERS-1). Called under the append lock. The fast
    /// path is two small reads — the anchor, and the tail file's length — both of which every
    /// writer moves on every append; only when either disagrees with what this handle last wrote is
    /// the chain re-read. A chain that disagrees with its own anchor is refused here exactly as
    /// `open` refuses it.
    fn catch_up(&mut self) -> Result<(), AuditError> {
        let anchor_agrees = read_anchor(&self.dir).is_some_and(|(h, n)| h == self.head && n == self.records);
        let tail_agrees = match &self.current_day {
            Some(d) => file_len(&day_path(&self.dir, d)) == self.tail_len,
            None => list_day_files(&self.dir)?.is_empty(),
        };
        if anchor_agrees && tail_agrees {
            return Ok(());
        }
        let (head, current_day, records) = recover(&self.dir)?;
        self.tail_len = current_day.as_deref().and_then(|d| file_len(&day_path(&self.dir, d)));
        self.head = head;
        self.current_day = current_day;
        self.records = records;
        Ok(())
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
        // AUDIT-WRITERS-1: one writer at a time, and never onto a head another writer has moved.
        let _lock = AppendLock::take(&self.dir)?;
        self.catch_up()?;
        let day = day_string(entry.ts);
        self.ensure_day_file(&day)?;
        let rec = entry.into_record(&self.head);
        let path = day_path(&self.dir, &day);
        append_line(&path, &rec.to_line())?;
        self.tail_len = file_len(&path);
        self.head = rec.hash.clone();
        self.records += 1;
        // The anchor is refreshed AFTER the record lands, so a crash between the two leaves the
        // anchor one behind — which `verify` reports as a mismatch. Fail-closed: a log that may
        // have lost its last record says so rather than passing.
        write_anchor(&self.dir, &self.head, self.records)?;
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
    pub(crate) fn corrupt(seq: u64, detail: impl Into<String>) -> AuditError {
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
///
/// A pass that fails is read again under the append lock before it is believed (AUDIT-WRITERS-1):
/// a live broker writes a record and THEN its anchor, and a reader that lands between the two sees a
/// chain one ahead of its anchor, or a half-written last line. A clean pass never touches the lock,
/// so a read-only copy of a log verifies as before.
pub fn verify(dir: impl AsRef<Path>) -> Result<VerifiedStats, AuditError> {
    let dir = dir.as_ref();
    match verify_unlocked(dir) {
        Err(AuditError::Corrupt(first)) => match AppendLock::take_within(dir, std::time::Duration::from_secs(2)) {
            Ok(_lock) => verify_unlocked(dir),
            Err(_) => Err(AuditError::Corrupt(first)),
        },
        other => other,
    }
}

fn verify_unlocked(dir: &Path) -> Result<VerifiedStats, AuditError> {
    let days = list_day_files(dir)?;
    let mut expected_prev = GENESIS_HASH.to_string();
    let mut files = 0usize;
    let mut records = 0usize;
    for day in &days {
        files += 1;
        let text = read_day(&day_path(dir, day)).map_err(AuditError::io)?;
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
    // P17-C1: every check above is LOCAL TO A LINK, so a truncated chain still verifies — each
    // surviving link is correct and the deleted suffix leaves no trace. The anchor is the only
    // thing that can notice, because it was written when the chain was longer.
    match read_anchor(dir) {
        Some((a_head, a_records)) if a_head != expected_prev || a_records != records => {
            return Err(AuditError::corrupt(
                records as u64,
                format!(
                    "audit anchor disagrees with the log: anchor says {a_records} record(s) \
                     ending {a_head}, the chain holds {records} ending {expected_prev} — records \
                     have been removed, replaced or rolled back"
                ),
            ));
        }
        Some(_) => {}
        // An unanchored log is REPORTED, not silently accepted: it is what every log written before
        // anchoring existed looks like, and it is also what a log looks like after someone deletes
        // the anchor. `verify` cannot tell those apart, and does not pretend to.
        None => {}
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

// ----- RFC 0001 phase F5: audit reconciliation ---------------------------------------------------

/// The bundle wire magic. A bundle is a *transcript*, never a second chain.
pub const BUNDLE_MAGIC: &str = "dlbundle1";

/// What a verified [`Bundle`] proved, for the `reconcile` record the receiver writes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BundleSummary {
    /// The bundle's own content hash — the value the receiver's chain records as a cross-link.
    pub digest: String,
    pub records: usize,
    pub first_seq: u64,
    pub last_seq: u64,
    /// The head hash of the sender's chain at the end of this segment.
    pub head: String,
}

/// A transcript of another broker's audit segment.
///
/// # Why chains are cross-linked and never merged
///
/// A chain is `hash = blake3(prev_hash ‖ canonical_record)` over one broker's monotone `seq`. Two
/// brokers produce two chains, and splicing them would invalidate every hash after the splice
/// point — so a "merged" log would either be a lie or a rewrite, and this is the one artifact whose
/// entire value is that it is neither.
///
/// Instead each broker keeps its own chain and the receiver records a `reconcile` entry naming the
/// bundle's digest and seq range. **The mechanism is already in the tree**: a new day file's first
/// record carries the previous file's last hash as its `prev_hash`, so day files are already
/// cross-linked rather than concatenated. This generalizes that across brokers instead of days.
///
/// Two honesty properties, both load-bearing:
///
/// - **`seq` is per-broker.** Records from two brokers are NOT globally ordered, and any tool that
///   renders a combined timeline has to say so.
/// - **Verification failure is an INCIDENT, not a denial.** The log is observability, not
///   enforcement (spec §7, playbook trap 6). A bundle that does not verify is reported and
///   recorded; it never blocks a vehicle from operating. Making reconciliation gate anything would
///   quietly convert the audit log into an enforcement input, which is exactly the rule this
///   project keeps.
pub struct Bundle {
    pub records: Vec<AuditRecord>,
}

impl Bundle {
    /// Serialize as a bundle document: the magic, then each record's canonical line.
    pub fn to_wire(&self) -> String {
        let mut s = String::from(BUNDLE_MAGIC);
        s.push('\n');
        for r in &self.records {
            s.push_str(&r.to_line());
            s.push('\n');
        }
        s
    }

    /// The bundle's content hash — `blake3` over the canonical lines, independent of the framing.
    pub fn digest(&self) -> String {
        let mut h = blake3::Hasher::new();
        for r in &self.records {
            h.update(r.to_line().as_bytes());
            h.update(b"\n");
        }
        h.finalize().to_hex().to_string()
    }

    /// Re-verify the bundle's internal chain: every record's hash recomputed from its own body, and
    /// every `prev_hash` matching the previous record's `hash`.
    ///
    /// `expected_start` is the hash the segment must chain onto — [`GENESIS_HASH`] for a sender's
    /// first-ever segment, or the head recorded by the previous reconciliation. Passing `None`
    /// accepts any starting point and is only appropriate for a first contact; a caller that knows
    /// the previous head and does not pass it has thrown away the property that makes a *sequence*
    /// of bundles tamper-evident (a dropped middle segment would go unnoticed).
    pub fn verify(&self, expected_start: Option<&str>) -> Result<BundleSummary, AuditError> {
        if self.records.is_empty() {
            return Err(AuditError::corrupt(0, "bundle contains no records"));
        }
        let mut prev = expected_start.map(str::to_string);
        for r in &self.records {
            if let Some(p) = &prev {
                if &r.prev_hash != p {
                    return Err(AuditError::corrupt(
                        r.seq,
                        format!(
                            "prev_hash `{}` does not chain onto `{p}` — a segment is missing, \
                             reordered, or altered",
                            r.prev_hash
                        ),
                    ));
                }
            }
            let recomputed = chain_hash(&r.prev_hash, &canonical_json(&r.body_value()));
            if recomputed != r.hash {
                return Err(AuditError::corrupt(r.seq, "record hash does not match its own body (tampered)"));
            }
            prev = Some(r.hash.clone());
        }
        Ok(BundleSummary {
            digest: self.digest(),
            records: self.records.len(),
            first_seq: self.records[0].seq,
            last_seq: self.records[self.records.len() - 1].seq,
            head: self.records[self.records.len() - 1].hash.clone(),
        })
    }
}

/// Parse a bundle document. A malformed line is a hard error: a transcript this build can only
/// partly read is a transcript it must not summarize.
pub fn parse_bundle(text: &str) -> Result<Bundle, AuditError> {
    let mut lines = text.lines();
    match lines.next().map(str::trim) {
        Some(BUNDLE_MAGIC) => {}
        other => return Err(AuditError::corrupt(0, format!("not an audit bundle (magic {other:?})"))),
    }
    let mut records = Vec::new();
    for (i, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: Value = serde_json::from_str(line)
            .map_err(|e| AuditError::corrupt(i as u64, format!("line {i} is not JSON: {e}")))?;
        let r = AuditRecord::from_value(&v)
            .ok_or_else(|| AuditError::corrupt(i as u64, format!("line {i} is not a record")))?;
        records.push(r);
    }
    Ok(Bundle { records })
}

/// Read every record in `dir` as a [`Bundle`] — the sender's side of reconciliation.
pub fn bundle(dir: impl AsRef<Path>) -> Result<Bundle, AuditError> {
    Ok(Bundle { records: read_all_records(dir.as_ref())? })
}

// ----- internals ---------------------------------------------------------------------------------

/// Read every record (skipping headers) across sorted day files, in order. Does NOT verify the chain
/// (that is [`verify`]) — this is the read surface for `tail`/`query`.
fn read_all_records(dir: &Path) -> Result<Vec<AuditRecord>, AuditError> {
    let days = list_day_files(dir)?;
    let mut out = Vec::new();
    for day in &days {
        let text = read_day(&day_path(dir, day)).map_err(AuditError::io)?;
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

/// A day log, read only if it is a regular file (RW 4.38, the red-team pass on FRAME-DRIP-1's F6). It was
/// `read_to_string` by name: a FIFO named like a day log hung every reader of the chain — `verify`, `tail`,
/// `query` and the log's own open — waiting for a writer that never came. Opened non-blocking, so opening
/// a FIFO returns at once, and judged by what was OPENED (ATTEST-FIFO-1's rule).
fn read_day(path: &Path) -> std::io::Result<String> {
    use std::io::Read as _;
    let mut open = fs::OpenOptions::new();
    open.read(true);
    // Not under Miri, whose `open` takes a short list of flags and which never makes a FIFO: there the
    // open is the blocking one this replaced (the nightly's `miri-slow` interprets these reads).
    #[cfg(all(unix, not(miri)))]
    std::os::unix::fs::OpenOptionsExt::custom_flags(&mut open, libc::O_NONBLOCK);
    let mut f = open.open(path)?;
    if !f.metadata()?.is_file() {
        return Err(std::io::Error::other(NotRegular(path.to_path_buf())));
    }
    let mut text = String::new();
    f.read_to_string(&mut text)?;
    Ok(text)
}

/// [`read_day`]'s refusal: named, so the readers that treat an unreadable day log as empty do not treat
/// this one so.
#[derive(Debug)]
struct NotRegular(PathBuf);

impl std::fmt::Display for NotRegular {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` is not a regular file, and an audit day log must be one", self.0.display())
    }
}

impl std::error::Error for NotRegular {}

fn not_regular(e: &std::io::Error) -> bool {
    e.get_ref().is_some_and(|inner| inner.is::<NotRegular>())
}

/// How many CHAIN RECORDS a day file holds. Uses exactly `last_record_hash`'s notion of a record —
/// a JSON line carrying `seq` — so a header line is not miscounted and the anchor's count means the
/// same thing `verify` counts.
fn count_records(path: &Path) -> Result<usize, AuditError> {
    let text = match read_day(path) {
        Ok(t) => t,
        Err(e) if not_regular(&e) => return Err(AuditError::io(e)),
        Err(_) => return Ok(0),
    };
    let mut n = 0usize;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if v.get("seq").is_some() {
                n += 1;
            }
        }
    }
    Ok(n)
}

fn last_record_hash(path: &Path) -> Result<Option<String>, AuditError> {
    let text = match read_day(path) {
        Ok(t) => t,
        Err(e) if not_regular(&e) => return Err(AuditError::io(e)),
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

    /// **Phase M — a fuzzing-style battery: every adversarial bundle is an error, never a panic.** A
    /// bundle crosses a trust boundary as a file; `audit reconcile` runs `parse_bundle(...).and_then(|b|
    /// b.verify(...))` on bytes a remote party produced. Neither step may panic: not an index out of
    /// bounds on an EMPTY bundle (`verify` guards `is_empty` before `records[0]`), not a stack overflow
    /// on a deeply-nested field (serde_json's recursion limit turns it into a parse error), not an
    /// unbounded allocation from a count field (`parse_bundle` grows a Vec line-by-line and trusts no
    /// declared length). Every case must be `Err`; reaching the end proves nothing panicked.
    #[test]
    fn adversarial_bundles_are_errors_never_panics() {
        // 600 deep — past serde_json's 128 recursion limit, so this is a parse error, not a crash.
        let nested = format!("{}{}", "[".repeat(600), "]".repeat(600));
        let cases: Vec<String> = vec![
            String::new(),                                 // no magic
            "dlbundle1".to_string(),                       // magic only -> empty bundle -> verify Err
            "dlbundle1\n".to_string(),
            "dlbundle1\nnot json at all".to_string(),      // a non-JSON record line
            "dlbundle1\n{}".to_string(),                   // JSON object but not a record (no seq)
            "dlbundle1\n[]".to_string(),                   // JSON but not even an object
            "dlbundle1\n{\"seq\":\"not-a-number\"}".to_string(), // seq is the wrong type
            format!("dlbundle1\n{{\"seq\":1,\"ts\":0,\"prev_hash\":\"x\",\"hash\":\"y\",\"action\":\"use\",\"decision\":\"allow\",\"authority\":{nested}}}"),
            "dlbundle1\n{\"seq\":18446744073709551615,\"ts\":0,\"prev_hash\":\"x\",\"hash\":\"y\",\"action\":\"a\",\"decision\":\"d\"}".to_string(), // u64::MAX seq
        ];
        for c in &cases {
            let result = parse_bundle(c).and_then(|b| b.verify(None));
            assert!(result.is_err(), "expected an AuditError (not a panic/Ok) for:\n{c}");
        }

        // The empty bundle specifically must be the guarded `is_empty` error, never a `records[0]` panic.
        let empty = parse_bundle("dlbundle1").expect("magic-only parses to an empty bundle");
        assert!(empty.records.is_empty(), "magic-only is a zero-record bundle");
        assert!(empty.verify(None).is_err(), "an empty bundle is a clean error, never an index panic");
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

    /// `verify` reads a file an attacker may have rewritten, and it must **report** corruption
    /// rather than die on it — a verifier that panics is a verifier that can be silenced.
    ///
    /// The subtle one is the JSON array. `verify` does
    /// `body.as_object_mut().unwrap().remove("hash")`, which is sound only because reaching that
    /// line requires `v.get("seq")` to have returned `Some`, and a *string* index never matches a
    /// non-object. That is a real invariant but an **implicit** one, held together by serde_json's
    /// indexing rules rather than by anything local — so it is pinned here rather than re-derived by
    /// the next reader of that `unwrap`.
    #[test]
    fn a_line_that_is_not_an_object_is_reported_not_panicked_on() {
        for (tag, line) in [
            ("array", "[1,2,3]"),
            ("scalar", "42"),
            ("string", "\"seq\""),
            ("null", "null"),
            ("truncated", "{\"seq\":1,\"hash\":"),
        ] {
            let dir = tmp_dir(&format!("verify_nonobject_{tag}"));
            {
                let mut log = AuditLog::open(&dir).unwrap();
                log.append(entry(1, 1000, "use", "g_x", "allow")).unwrap();
            }
            let day = day_string(1000);
            let path = day_path(&dir, &day);
            let text = fs::read_to_string(&path).unwrap();
            fs::write(&path, format!("{text}{line}\n")).unwrap();

            // Either outcome is acceptable — refused, or ignored as a non-record. A PANIC is not.
            match verify(&dir) {
                Ok(_) | Err(_) => {}
            }
            let _ = fs::remove_dir_all(&dir);
        }
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

    /// RW 4.38 (the red-team pass on FRAME-DRIP-1, F6): a FIFO named like a day log hung every reader of
    /// the chain — `verify`, `tail`, and the log's own open (so `broker start`) — in `read_to_string`,
    /// waiting for a writer that never came: red on `2734ba9`, each still waiting at 5 s. Each refuses it
    /// now, in words, at once. (A FIFO and the wall clock: not under Miri.)
    #[cfg(unix)]
    #[test]
    #[cfg_attr(miri, ignore)]
    fn a_day_log_that_is_not_a_regular_file_is_refused_not_waited_on() {
        let dir = tmp_dir("fifo");
        fs::create_dir_all(&dir).unwrap();
        let day = dir.join("20260930.jsonl");
        let made = std::process::Command::new("mkfifo").arg(&day).status().expect("mkfifo runs");
        assert!(made.success());
        type Reader = fn(PathBuf) -> Result<(), AuditError>;
        let readers: [(&str, Reader); 3] = [
            ("verify", |d| verify(d).map(|_| ())),
            ("tail", |d| tail(d, 3).map(|_| ())),
            ("open", |d| AuditLog::open(d).map(|_| ())),
        ];
        for (name, read) in readers {
            let (tx, rx) = std::sync::mpsc::channel();
            let d = dir.clone();
            std::thread::spawn(move || {
                let _ = tx.send(read(d));
            });
            let r = rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap_or_else(|_| panic!("`{name}` was still waiting on the FIFO after 5 s"));
            let e = r.expect_err("a day log that is not a regular file is refused");
            assert!(format!("{e:?}").contains("not a regular file"), "`{name}`, in words: {e:?}");
        }
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

    /// AUDIT-WRITERS-1: two writers on one chain — the broker daemon and a sandboxed run — each
    /// holding its own handle. Before the fix the first handle's next record chained onto the head
    /// it had cached before the second one wrote, and `verify` failed DL1405 on an untampered chain.
    #[test]
    fn two_handles_on_one_chain_interleave_without_breaking_it() {
        let dir = tmp_dir("two-writers");
        let mut daemon = AuditLog::open(&dir).unwrap();
        daemon.append(entry(1, 1000, "issue", "g_a", "allow")).unwrap();
        let mut run = AuditLog::open(&dir).unwrap();
        run.append(entry(2, 1001, "sandbox-launch", "g_a", "allow")).unwrap();
        run.append(entry(3, 1002, "sandbox-death", "g_a", "allow")).unwrap();
        // The daemon's handle is stale here; its record must chain onto the run's last one.
        let r = daemon.append(entry(4, 1003, "use", "g_a", "allow")).unwrap();
        assert_eq!(r.prev_hash, tail(&dir, 2).unwrap()[0].hash, "chained onto the other writer's head");
        let stats = verify(&dir).expect("an interleaved chain verifies");
        assert_eq!(stats.records, 4);
        assert_eq!(daemon.head(), stats.head);
        // And across a day boundary another writer opened.
        run.append(entry(5, 1000 + 86_400_000, "sandbox-launch", "g_a", "allow")).unwrap();
        daemon.append(entry(6, 1001 + 86_400_000, "use", "g_a", "allow")).unwrap();
        assert_eq!(verify(&dir).expect("across days too").records, 6);
        assert!(AppendLock::take_within(&dir, std::time::Duration::ZERO).is_ok(), "no lock is left held");
        let _ = fs::remove_dir_all(&dir);
    }

    /// The same, as it happens: writers in parallel threads, each with its own handle, as parallel
    /// sandboxed runs beside a daemon are. Every record lands, on its own line, in one chain.
    #[test]
    fn parallel_writers_leave_one_verifiable_chain() {
        let dir = tmp_dir("parallel-writers");
        drop(AuditLog::open(&dir).unwrap());
        let handles: Vec<_> = (0..4u64)
            .map(|w| {
                let dir = dir.clone();
                std::thread::spawn(move || {
                    let mut log = AuditLog::open(&dir).unwrap();
                    for i in 0..25u64 {
                        log.append(entry(w * 100 + i, 5000, "use", &format!("g_{w}"), "allow")).unwrap();
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(verify(&dir).expect("parallel writers leave a sound chain").records, 100);
        let _ = fs::remove_dir_all(&dir);
    }

    /// AUDIT-LOCK-TAKEOVER-1: a holder that is only SLOW is waited for, never taken over.
    #[test]
    fn a_slow_holder_is_never_taken_over() {
        let dir = tmp_dir("slow-holder");
        fs::create_dir_all(&dir).unwrap();
        let first = AppendLock::take_within(&dir, std::time::Duration::from_millis(50)).expect("a free lock");
        let second = AppendLock::take_within(&dir, std::time::Duration::from_millis(50));
        assert!(second.is_err(), "a second writer took the lock while its holder still held it");
        assert!(format!("{:?}", second.err().unwrap()).contains("nothing was written"), "refused in words");
        drop(first);
        assert!(AppendLock::take_within(&dir, std::time::Duration::from_millis(50)).is_ok(), "free again");
        let _ = fs::remove_dir_all(&dir);
    }

    /// What the first lock's takeover was for, without one: a lock FILE a writer left behind (the old
    /// format's, or anything else) blocks nobody — only a held OS lock does.
    #[test]
    fn a_lock_file_left_behind_blocks_nobody() {
        let dir = tmp_dir("stale-lock");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(APPEND_LOCK_FILE), "a writer that died").unwrap();
        AppendLock::take_within(&dir, std::time::Duration::ZERO).expect("an unheld file is no lock");
        let _ = fs::remove_dir_all(&dir);
    }

    /// A writer KILLED while holding the lock does not block the next one: the operating system
    /// releases it with the process. The holder is this test binary, re-entered as a child.
    #[cfg(not(miri))]
    #[test]
    fn a_writer_killed_holding_the_lock_does_not_block_the_next() {
        const CHILD: &str = "DELULU_AUDIT_LOCK_HOLDER";
        if let Ok(dir) = std::env::var(CHILD) {
            let _held = AppendLock::take(Path::new(&dir)).expect("the child takes the lock");
            std::thread::sleep(std::time::Duration::from_secs(120));
            return;
        }
        let dir = tmp_dir("killed-holder");
        fs::create_dir_all(&dir).unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "audit::tests::a_writer_killed_holding_the_lock_does_not_block_the_next", "--nocapture"])
            .env(CHILD, &dir)
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        // Wait until the child holds it — a test that never saw it held would prove nothing.
        let seen_held = (0..2000).any(|_| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            AppendLock::take_within(&dir, std::time::Duration::ZERO).is_err()
        });
        assert!(seen_held, "the child never held the lock");
        child.kill().unwrap();
        child.wait().unwrap();
        AppendLock::take_within(&dir, std::time::Duration::from_secs(5)).expect("released with the process that held it");
        let _ = fs::remove_dir_all(&dir);
    }
}
