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

    /// The next seq this sink would give a record, when the sink is shared with other writers and
    /// can say (AUDIT-SEQ-1): a writer that keeps its own count — the broker — takes it as a floor
    /// before each seq, so neither a restart nor a record another writer appended can make it repeat
    /// one. `None` — the default, and every in-memory sink — leaves the writer's own count alone.
    fn next_seq(&mut self) -> Option<u64> {
        None
    }
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
    /// The highest seq any record in the chain carries (0 for none) — what the next record must
    /// exceed (AUDIT-SEQ-1). Kept with the head: recovered when another writer moved the chain.
    max_seq: u64,
    /// How many records `current_day`'s file holds, and every older day file's bytes together — the
    /// anchor's [`AnchorAt`], so the next opening can take the anchor's word (RW 4.53).
    day_records: usize,
    older_bytes: u64,
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
    /// The chain this lock is for — a held lock is a token ([`AuditLog::append_under`]), and a token for
    /// another chain proves nothing about this one.
    dir: PathBuf,
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
                Ok(()) => return Ok(AppendLock { _file: file, dir: dir.to_path_buf() }),
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
fn write_anchor(dir: &Path, head: &str, records: usize, at: Option<&AnchorAt>) -> Result<(), AuditError> {
    let body = match at {
        Some(a) => format!(
            "{{\"head\":\"{head}\",\"records\":{records},\"day\":\"{}\",\"day_len\":{},\"day_records\":{},\"older_bytes\":{},\"max_seq\":{}}}\n",
            a.day, a.day_len, a.day_records, a.older_bytes, a.max_seq
        ),
        None => format!("{{\"head\":\"{head}\",\"records\":{records}}}\n"),
    };
    fs::write(anchor_path(dir), body).map_err(AuditError::io)
}

/// Where the chain stood when its anchor was written (RW 4.53, D-V2-160): the newest day file, its length and how many
/// records it holds, every older day file's bytes together, and the highest seq. An opening whose files still measure so
/// takes the anchor's count after checking that the newest file's last record is the anchor's head, and reads nothing
/// else; any other measure sends it to the whole recovery, which refuses what it always refused.
#[derive(Clone, Debug, PartialEq, Eq)]
struct AnchorAt {
    day: String,
    day_len: u64,
    day_records: usize,
    older_bytes: u64,
    max_seq: u64,
}

/// The anchor's [`AnchorAt`], when it has one (an anchor written before RW 4.53 has none).
fn read_anchor_at(dir: &Path) -> Option<AnchorAt> {
    let v: Value = serde_json::from_str(&fs::read_to_string(anchor_path(dir)).ok()?).ok()?;
    Some(AnchorAt {
        day: v.get("day")?.as_str()?.to_string(),
        day_len: v.get("day_len")?.as_u64()?,
        day_records: v.get("day_records")?.as_u64()? as usize,
        older_bytes: v.get("older_bytes")?.as_u64()?,
        max_seq: v.get("max_seq")?.as_u64()?,
    })
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

/// What a recovery found: the chain's head, its newest day file and that file's length, the record count, the highest
/// seq, and the two counts an [`AnchorAt`] carries.
#[derive(Debug, PartialEq, Eq)]
struct Recovered {
    head: String,
    current_day: Option<String>,
    records: usize,
    max_seq: u64,
    tail_len: Option<u64>,
    day_records: usize,
    older_bytes: u64,
}

/// What the files say — the head, the latest day file, the record count — checked against the
/// anchor, and the anchor written if there is none. Called under the append lock.
fn recover(dir: &Path) -> Result<Recovered, AuditError> {
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
    let mut max_seq = 0u64;
    let (mut day_records, mut older_bytes) = (0usize, 0u64);
    for (i, day) in days.iter().enumerate() {
        let (n, top) = count_records(&day_path(dir, day))?;
        records += n;
        max_seq = max_seq.max(top);
        if i + 1 < days.len() {
            older_bytes += file_len(&day_path(dir, day)).unwrap_or(0);
        } else {
            day_records = n;
        }
    }
    let tail_len = current_day.as_deref().and_then(|d| file_len(&day_path(dir, d)));
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
        _ => {
            let at = current_day.clone().zip(tail_len).map(|(day, day_len)| AnchorAt {
                day,
                day_len,
                day_records,
                older_bytes,
                max_seq,
            });
            write_anchor(dir, &head, records, at.as_ref())?
        }
    }
    Ok(Recovered { head, current_day, records, max_seq, tail_len, day_records, older_bytes })
}

/// RW 4.53 (D-V2-160): an opening that reads the newest day file's last line and nothing else, when every file still
/// measures as the anchor says — the newest file named and of its length, the older files' bytes together unchanged — and
/// that line is the anchor's head. `None` for anything else, and the whole recovery decides. What it does not re-read: an
/// older record changed in place to the same length — never the opening's to find (`verify` recomputes every hash).
fn recover_fast(dir: &Path) -> Result<Option<Recovered>, AuditError> {
    let (Some((head, records)), Some(at)) = (read_anchor(dir), read_anchor_at(dir)) else { return Ok(None) };
    let days = list_day_files(dir)?;
    let Some((newest, older)) = days.split_last() else { return Ok(None) };
    if *newest != at.day || at.day_records == 0 || file_len(&day_path(dir, newest)) != Some(at.day_len) {
        return Ok(None);
    }
    let mut older_bytes = 0u64;
    for day in older {
        let Some(len) = file_len(&day_path(dir, day)) else { return Ok(None) };
        older_bytes += len;
    }
    if older_bytes != at.older_bytes || last_line_hash(&day_path(dir, newest), at.day_len)?.as_deref() != Some(head.as_str()) {
        return Ok(None);
    }
    Ok(Some(Recovered {
        head,
        current_day: Some(at.day),
        records,
        max_seq: at.max_seq,
        tail_len: Some(at.day_len),
        day_records: at.day_records,
        older_bytes,
    }))
}

/// The hash of a day file's last record line, read from its end: the last 64 KiB, or the whole file when its last line is
/// longer. `None` when the last line is not a record.
fn last_line_hash(path: &Path, len: u64) -> Result<Option<String>, AuditError> {
    let from = len.saturating_sub(64 * 1024);
    let mut bytes = read_day_from(path, from).map_err(AuditError::io)?;
    let complete = complete_lines(&bytes).len();
    bytes.truncate(complete);
    let body = bytes.strip_suffix(b"\n").unwrap_or(&bytes);
    let line = match body.iter().rposition(|b| *b == b'\n') {
        Some(i) => &body[i + 1..],
        None if from == 0 => body,
        None => return last_line_hash(path, 0),
    };
    let v: Value = match serde_json::from_slice(line) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    Ok(v.get("seq").and(v.get("hash")).and_then(Value::as_str).map(str::to_string))
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
        // RW 4.53: an opening read every day file twice; when the files measure as the anchor says, it reads one line.
        let r = match recover_fast(&dir)? {
            Some(r) => r,
            None => recover(&dir)?,
        };
        let mut log = AuditLog {
            dir,
            head: String::new(),
            current_day: None,
            records: 0,
            tail_len: None,
            max_seq: 0,
            day_records: 0,
            older_bytes: 0,
        };
        log.take(r);
        Ok(log)
    }

    fn take(&mut self, r: Recovered) {
        self.head = r.head;
        self.current_day = r.current_day;
        self.records = r.records;
        self.max_seq = r.max_seq;
        self.tail_len = r.tail_len;
        self.day_records = r.day_records;
        self.older_bytes = r.older_bytes;
    }

    /// Where this handle's chain stands, for the anchor it writes.
    fn anchor_at(&self) -> Option<AnchorAt> {
        Some(AnchorAt {
            day: self.current_day.clone()?,
            day_len: self.tail_len?,
            day_records: self.day_records,
            older_bytes: self.older_bytes,
            max_seq: self.max_seq,
        })
    }

    /// The current chain head (for tests / cross-links).
    pub fn head(&self) -> &str {
        &self.head
    }

    /// Catch up with any other writer (AUDIT-WRITERS-1). Called under the append lock. The fast
    /// path is two small reads — the anchor, and the tail file's length — both of which every
    /// writer moves on every append; only when either disagrees with what this handle last wrote is
    /// the chain read again: first only what was appended since ([`AuditLog::read_appended`]), and
    /// the whole chain when that does not account for it. A chain that disagrees with its own anchor
    /// is refused here exactly as `open` refuses it.
    fn catch_up(&mut self) -> Result<(), AuditError> {
        let anchor = read_anchor(&self.dir);
        let anchor_agrees = anchor.as_ref().is_some_and(|(h, n)| *h == self.head && *n == self.records);
        let tail_agrees = match &self.current_day {
            Some(d) => file_len(&day_path(&self.dir, d)) == self.tail_len,
            None => list_day_files(&self.dir)?.is_empty(),
        };
        if anchor_agrees && tail_agrees {
            return Ok(());
        }
        if self.read_appended(anchor.as_ref())? {
            return Ok(());
        }
        let r = recover(&self.dir)?;
        self.take(r);
        Ok(())
    }

    /// RW 4.53 (MONITOR-CHAIN-1's class, on the writing side): catch up by reading only what other
    /// writers appended since this handle last wrote — the rest of the day file it wrote, from the byte
    /// it stopped at, and every later day file — each record naming the one before it and its hash its
    /// own, and then the anchor agreeing with the head and count that reading gives. A catch-up re-read
    /// and re-counted the WHOLE chain under the append lock (twice: every head, then every count), so on
    /// a long chain each daemon request after a sandboxed host's record — an operator's e-stop revoke
    /// among them — paid for the chain's whole history. `false` when anything else is the case — no
    /// file this handle wrote, a file cut short, a line that is not a record or does not verify, an
    /// anchor that disagrees (a writer puts down its line, then its line break, then the anchor, all
    /// under the lock) — and the whole recovery decides, refusing what it refused before. Nothing is
    /// changed unless the reading accounts for what the anchor says.
    fn read_appended(&mut self, anchor: Option<&(String, usize)>) -> Result<bool, AuditError> {
        let (Some(day), Some(from)) = (self.current_day.clone(), self.tail_len) else { return Ok(false) };
        let days = list_day_files(&self.dir)?;
        if !days.contains(&day) || file_len(&day_path(&self.dir, &day)).is_none_or(|l| l < from) {
            return Ok(false);
        }
        let (mut head, mut records, mut max_seq) = (self.head.clone(), self.records, self.max_seq);
        let (mut day_records, mut older_bytes) = (self.day_records, self.older_bytes);
        let mut tail = (day.clone(), from);
        let later = days.iter().filter(|d| d.as_str() > day.as_str()).map(|d| (d.clone(), 0));
        for (i, (d, start)) in std::iter::once((day.clone(), from)).chain(later).enumerate() {
            if i > 0 {
                // A later file: the one before it is closed — its bytes are an older file's now.
                older_bytes += tail.1;
                day_records = 0;
            }
            let bytes = read_day_from(&day_path(&self.dir, &d), start).map_err(AuditError::io)?;
            for line in bytes.split(|b| *b == b'\n') {
                if line.iter().all(u8::is_ascii_whitespace) {
                    continue;
                }
                let Ok(v) = serde_json::from_slice::<Value>(line) else { return Ok(false) };
                if v.get("seq").is_none() {
                    continue; // a day file's header
                }
                let Some((hash, seq)) = verified_link(&v, &head) else { return Ok(false) };
                head = hash;
                records += 1;
                day_records += 1;
                max_seq = max_seq.max(seq);
            }
            tail = (d, start + bytes.len() as u64);
        }
        if anchor.is_none_or(|(h, n)| *h != head || *n != records) {
            return Ok(false);
        }
        self.head = head;
        self.records = records;
        self.max_seq = max_seq;
        self.current_day = Some(tail.0);
        self.tail_len = Some(tail.1);
        self.day_records = day_records;
        self.older_bytes = older_bytes;
        Ok(true)
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

impl AuditLog {
    /// Take this chain's append lock, to hold across several operations (RW 4.48): a writer that hands
    /// out a seq BEFORE its record exists — the daemon, which stamps it into a node and an answer — holds
    /// the lock from [`AuditLog::next_seq_under`] to its last [`AuditLog::append_under`], so no other
    /// writer can take that seq in between. Dropping the lock releases it.
    pub fn take_lock(&self) -> Result<AppendLock, AuditError> {
        AppendLock::take(&self.dir)
    }

    fn check_token(&self, lock: &AppendLock) -> Result<(), AuditError> {
        if lock.dir == self.dir {
            Ok(())
        } else {
            Err(AuditError::Io(format!(
                "an append lock for `{}` was offered for the chain at `{}` — nothing was written",
                lock.dir.display(),
                self.dir.display()
            )))
        }
    }

    /// [`AuditSink::next_seq`] under a lock the caller holds: the seq no other writer can take while it
    /// is held.
    pub fn next_seq_under(&mut self, lock: &AppendLock) -> Result<u64, AuditError> {
        self.check_token(lock)?;
        self.catch_up()?;
        Ok(self.max_seq + 1)
    }

    /// [`AuditSink::append`] under a lock the caller holds (see [`AuditLog::take_lock`]).
    pub fn append_under(&mut self, lock: &AppendLock, mut entry: AuditEntry) -> Result<AuditRecord, AuditError> {
        self.check_token(lock)?;
        self.catch_up()?;
        // AUDIT-SEQ-1: and never a seq the chain already holds. The number is settled HERE, under the
        // lock: a writer that asks for none (0) gets the next, and one whose own count is behind the
        // chain — the daemon, after a sandboxed run's host wrote beside it — is moved past it.
        entry.seq = entry.seq.max(self.max_seq + 1);
        // AUDIT-DAY-1: the FILE follows the chain, never falls behind it. A record is stamped before it
        // waits for this lock, and writers share the chain, so near midnight one stamped 23:59:59.9 can be
        // written after one stamped 00:00:00.5 — chosen by its own stamp it went into the earlier day's
        // file, behind records it chains after, and the whole chain read as broken. The record keeps its
        // own stamp; only where it is filed is the later of its day and the chain's (`YYYYMMDD` sorts as
        // dates do).
        let day = std::cmp::max(day_string(entry.ts), self.current_day.clone().unwrap_or_default());
        if self.current_day.as_deref().is_some_and(|d| d != day) {
            // The file this handle wrote is closed: its bytes are an older file's now (RW 4.53).
            self.older_bytes += self.tail_len.unwrap_or(0);
            self.day_records = 0;
        }
        self.ensure_day_file(&day)?;
        let rec = entry.into_record(&self.head);
        let path = day_path(&self.dir, &day);
        append_line(&path, &rec.to_line())?;
        self.tail_len = file_len(&path);
        self.head = rec.hash.clone();
        self.records += 1;
        self.day_records += 1;
        self.max_seq = rec.seq;
        // The anchor is refreshed AFTER the record lands, so a crash between the two leaves the
        // anchor one behind — which `verify` reports as a mismatch. Fail-closed: a log that may
        // have lost its last record says so rather than passing.
        write_anchor(&self.dir, &self.head, self.records, self.anchor_at().as_ref())?;
        Ok(rec)
    }
}

impl AuditSink for AuditLog {
    fn append(&mut self, entry: AuditEntry) -> Result<AuditRecord, AuditError> {
        // AUDIT-WRITERS-1: one writer at a time, and never onto a head another writer has moved.
        let lock = AppendLock::take(&self.dir)?;
        self.append_under(&lock, entry)
    }

    fn next_seq(&mut self) -> Option<u64> {
        let lock = AppendLock::take(&self.dir).ok()?;
        self.next_seq_under(&lock).ok()
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
            let hit = r.actor_node.as_deref() == Some(node.as_str()) || r.target.as_deref() == Some(node.as_str());
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
            let v: Value = serde_json::from_str(line).map_err(|e| AuditError::corrupt(0, format!("malformed JSON line: {e}")))?;
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
                return Err(AuditError::corrupt(seq, format!("prev_hash chain break: expected {expected_prev}, found {prev}")));
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

// ----- following the chain (MONITOR-CHAIN-1, RW 4.49) ------------------------------------------------

/// A reader that follows the chain from a point and reads only what was appended after it, each record checked against the
/// one before it (MONITOR-CHAIN-1, RW 4.49).
///
/// `delulu monitor watch` read the WHOLE chain on every poll — about 3 µs a record in a release build, 11 in a debug one —
/// and it beats its dead-man between polls, so once one read outlasted the dead-man's period a healthy monitor was revoked
/// by its own broker, and every run under it with it, for nothing but the chain's length (witnessed by hand at 250,000
/// records and the default 2 s period). A follower costs what was appended since its last read: the rest of the day file it
/// stopped in, from the byte it stopped at, and every later day file — a record is never filed behind the chain's newest
/// file (AUDIT-DAY-1).
///
/// Only COMPLETE lines are read: a writer puts down a record and then its line break, so a line without its break may
/// still be growing, and is read by the next call. Each record must name the previous record's hash as its `prev_hash`,
/// and its own hash must recompute — the link check [`verify`] makes. A line that does not is returned as UNVERIFIED and
/// never as a record: what a follower's reader acts on is what the chain vouches for. That catches a damaged line and one
/// forged without the format; a writer of the operator's own user who computes the hash is category 7, as for every file
/// in this directory, and [`verify`] with the anchor stays the whole chain's check.
#[derive(Clone, Debug)]
pub struct Follower {
    dir: PathBuf,
    /// The day file the follower stopped in (`None`: there was none), and how many of its bytes it has read.
    day: Option<String>,
    offset: u64,
    /// The hash of the last record read — [`GENESIS_HASH`] before the first.
    head: String,
}

/// What one [`Follower::read_new`] found.
#[derive(Clone, Debug, Default)]
pub struct Followed {
    /// The records appended since the last read whose link and hash verify, in chain order.
    pub records: Vec<AuditRecord>,
    /// Each line that did not verify, by its seq (`None`: a line that is not a record at all) — never in `records`.
    pub unverified: Vec<Option<u64>>,
    /// The day file the follower stopped in is gone, or shorter than what it had read: the chain was rewritten under it.
    /// The follower moved to the chain's end and read nothing in between.
    pub rewritten: bool,
}

impl Follower {
    /// A follower at the chain's end now: its first [`Follower::read_new`] returns what is appended after this call.
    /// Reads the newest day file once (and older ones only back to the last record, when the newest holds none).
    pub fn at_end(dir: impl Into<PathBuf>) -> Result<Follower, AuditError> {
        let dir = dir.into();
        let days = list_day_files(&dir)?;
        let (mut at, mut head) = (None, None);
        for day in days.iter().rev() {
            let bytes = read_day_from(&day_path(&dir, day), 0).map_err(AuditError::io)?;
            let complete = complete_lines(&bytes);
            if at.is_none() {
                at = Some((day.clone(), complete.len() as u64));
            }
            // The last record line, read from the end: only it is parsed on a long day.
            head = complete
                .rsplit(|b| *b == b'\n')
                .filter_map(|line| serde_json::from_slice::<Value>(line).ok())
                .find_map(|v| v.get("seq").and(v.get("hash")).and_then(Value::as_str).map(str::to_string));
            if head.is_some() {
                break;
            }
        }
        let (day, offset) = at.map_or((None, 0), |(d, o)| (Some(d), o));
        Ok(Follower { dir, day, offset, head: head.unwrap_or_else(|| GENESIS_HASH.to_string()) })
    }

    /// The hash of the last record this follower read — [`GENESIS_HASH`] when it has read none on an empty chain.
    pub fn head(&self) -> &str {
        &self.head
    }

    /// Everything appended since the last call (or since [`Follower::at_end`]), each line verified against the one
    /// before it. A line that does not verify is named in [`Followed::unverified`], and the follower goes on from its
    /// stored hash, so one bad line does not blind it to the records written after.
    pub fn read_new(&mut self) -> Result<Followed, AuditError> {
        let days = list_day_files(&self.dir)?;
        let mut out = Followed::default();
        let mut todo: Vec<(String, u64)> = Vec::new();
        match &self.day {
            None => todo.extend(days.iter().map(|d| (d.clone(), 0))),
            Some(cur) => {
                let len = fs::metadata(day_path(&self.dir, cur)).map(|m| m.len()).ok();
                if !days.contains(cur) || len.is_none_or(|l| l < self.offset) {
                    *self = Follower::at_end(self.dir.clone())?;
                    out.rewritten = true;
                    return Ok(out);
                }
                todo.push((cur.clone(), self.offset));
                todo.extend(days.iter().filter(|d| d.as_str() > cur.as_str()).map(|d| (d.clone(), 0)));
            }
        }
        for (day, from) in todo {
            let bytes = read_day_from(&day_path(&self.dir, &day), from).map_err(AuditError::io)?;
            let complete = complete_lines(&bytes);
            for line in complete.split(|b| *b == b'\n') {
                if line.iter().all(u8::is_ascii_whitespace) {
                    continue;
                }
                let Ok(v) = serde_json::from_slice::<Value>(line) else {
                    out.unverified.push(None);
                    continue;
                };
                if v.get("seq").is_none() {
                    continue; // a day file's header — not part of the chain
                }
                let (seq, stored) = (v.get("seq").and_then(Value::as_u64), v.get("hash").and_then(Value::as_str));
                match AuditRecord::from_value(&v) {
                    Some(rec) if verified_link(&v, &self.head).is_some() => {
                        self.head = rec.hash.clone();
                        out.records.push(rec);
                    }
                    _ => {
                        out.unverified.push(seq);
                        if let Some(h) = stored {
                            self.head = h.to_string();
                        }
                    }
                }
            }
            self.day = Some(day);
            self.offset = from + complete.len() as u64;
        }
        Ok(out)
    }
}

/// A stored record's two checks, as [`verify`] makes them: it names `head` as its `prev_hash`, and its hash recomputes
/// from its own body. Its hash and seq when both hold.
fn verified_link(v: &Value, head: &str) -> Option<(String, u64)> {
    let (prev, stored, seq) = (v.get("prev_hash")?.as_str()?, v.get("hash")?.as_str()?, v.get("seq")?.as_u64()?);
    let mut body = v.clone();
    body.as_object_mut()?.remove("hash");
    (prev == head && chain_hash(prev, &canonical_json(&body)) == stored).then(|| (stored.to_string(), seq))
}

/// The bytes of a day file from `from` on — [`read_regular`]'s rule (a FIFO or any other non-file is refused, never
/// waited on), and a seek, so a follower reads what was appended and nothing before it.
fn read_day_from(path: &Path, from: u64) -> std::io::Result<Vec<u8>> {
    use std::io::{Read as _, Seek as _};
    let mut open = fs::OpenOptions::new();
    open.read(true);
    #[cfg(all(unix, not(miri)))]
    std::os::unix::fs::OpenOptionsExt::custom_flags(&mut open, libc::O_NONBLOCK);
    let mut f = open.open(path)?;
    if !f.metadata()?.is_file() {
        return Err(std::io::Error::other(NotRegular(path.to_path_buf(), "an audit day log")));
    }
    f.seek(std::io::SeekFrom::Start(from))?;
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// `bytes` up to and including its last line break — the lines a writer has finished.
fn complete_lines(bytes: &[u8]) -> &[u8] {
    bytes.iter().rposition(|b| *b == b'\n').map_or(&[][..], |i| &bytes[..=i])
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
        let v: Value =
            serde_json::from_str(line).map_err(|e| AuditError::corrupt(i as u64, format!("line {i} is not JSON: {e}")))?;
        let r = AuditRecord::from_value(&v).ok_or_else(|| AuditError::corrupt(i as u64, format!("line {i} is not a record")))?;
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
    read_regular(path, "an audit day log")
}

/// [`read_day`]'s rule for any file the broker keeps: read `path` only if what was opened is a regular
/// file, and refuse anything else in words naming `what` it had to be. The secret store reads through it
/// too (RW 4.38's remainder).
pub(crate) fn read_regular(path: &Path, what: &'static str) -> std::io::Result<String> {
    use std::io::Read as _;
    let mut open = fs::OpenOptions::new();
    open.read(true);
    // Not under Miri, whose `open` takes a short list of flags and which never makes a FIFO: there the
    // open is the blocking one this replaced (the nightly's `miri-slow` interprets these reads).
    #[cfg(all(unix, not(miri)))]
    std::os::unix::fs::OpenOptionsExt::custom_flags(&mut open, libc::O_NONBLOCK);
    let mut f = open.open(path)?;
    if !f.metadata()?.is_file() {
        return Err(std::io::Error::other(NotRegular(path.to_path_buf(), what)));
    }
    let mut text = String::new();
    f.read_to_string(&mut text)?;
    Ok(text)
}

/// [`read_regular`]'s refusal: named, so the readers that treat an unreadable day log as empty do not treat
/// this one so.
#[derive(Debug)]
pub(crate) struct NotRegular(pub(crate) PathBuf, pub(crate) &'static str);

impl std::fmt::Display for NotRegular {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` is not a regular file, and {} must be one", self.0.display(), self.1)
    }
}

impl std::error::Error for NotRegular {}

fn not_regular(e: &std::io::Error) -> bool {
    e.get_ref().is_some_and(|inner| inner.is::<NotRegular>())
}

/// How many CHAIN RECORDS a day file holds. Uses exactly `last_record_hash`'s notion of a record —
/// a JSON line carrying `seq` — so a header line is not miscounted and the anchor's count means the
/// same thing `verify` counts.
/// How many records one day's file holds, and the highest seq among them (AUDIT-SEQ-1).
fn count_records(path: &Path) -> Result<(usize, u64), AuditError> {
    let text = match read_day(path) {
        Ok(t) => t,
        Err(e) if not_regular(&e) => return Err(AuditError::io(e)),
        Err(_) => return Ok((0, 0)),
    };
    let (mut n, mut top) = (0usize, 0u64);
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            if let Some(seq) = v.get("seq") {
                n += 1;
                top = top.max(seq.as_u64().unwrap_or(0));
            }
        }
    }
    Ok((n, top))
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
    let mut f = fs::OpenOptions::new().create(true).append(true).open(path).map_err(AuditError::io)?;
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

    /// AUDIT-DAY-1 (routine run 16): a record's day FILE follows the chain, never falls behind it. Two
    /// writers share the chain, and a record is stamped before it waits for the append lock (the
    /// daemon's waits up to 5 s) — so near midnight a record stamped 23:59:59.9 can be written after one
    /// stamped 00:00:00.5. Chosen by its own stamp, it went into the EARLIER day's file, behind records
    /// it chains after: `verify` and every later writer then read the whole chain as truncated.
    #[test]
    fn a_record_stamped_before_midnight_but_written_after_keeps_the_chain_whole() {
        let dir = tmp_dir("midnight");
        let midnight = 20_736 * 86_400_000i64;
        let mut log = AuditLog::open(&dir).expect("open");
        log.append(entry(0, midnight - 5_000, "issue", "g_a", "allow")).expect("before midnight");
        log.append(entry(0, midnight + 500, "use", "g_a", "allow")).expect("after midnight");
        log.append(entry(0, midnight - 100, "revoke", "g_a", "allow")).expect("stamped before, written after");
        let stats = verify(&dir).expect("the chain verifies across the midnight it was written over");
        assert_eq!(stats.records, 3);
        let reopened = AuditLog::open(&dir).expect("and another writer can still open it");
        assert_eq!(reopened.head(), stats.head);
        let q = query(&dir, &QueryFilter::default()).expect("query");
        let seqs: Vec<u64> = q.iter().map(|r| r.seq).collect();
        assert_eq!(seqs, vec![1, 2, 3], "the chain reads in its written order");
        assert_eq!(q[2].ts, midnight - 100, "a record keeps its own stamp; only its file follows the chain");
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
        let readers: [(&str, Reader); 4] = [
            ("verify", |d| verify(d).map(|_| ())),
            ("tail", |d| tail(d, 3).map(|_| ())),
            ("open", |d| AuditLog::open(d).map(|_| ())),
            ("follow", |d| Follower::at_end(d).map(|_| ())),
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

    /// AUDIT-SEQ-1 (routine run 15): ONE numbering, whoever writes. A writer with no counter of its own
    /// (a sandboxed run's host, an adapter's provenance record) asks for none and is given the next; a
    /// writer whose counter is behind the chain (the daemon, after another wrote) is moved past it; and
    /// a log opened again — a daemon restarting — resumes after the highest seq the chain holds. Before
    /// the fix the daemon started at 1 on every start and the host numbered "last + 1" outside the lock,
    /// so one verified chain read `1, 2, 3, 1, 2, 3` and `audit export --since` dropped records.
    #[test]
    fn every_writer_continues_one_numbering_and_a_reopened_log_resumes_it() {
        let dir = tmp_dir("one-numbering");
        let mut daemon = AuditLog::open(&dir).unwrap();
        assert_eq!(daemon.next_seq(), Some(1), "an empty chain starts at 1");
        assert_eq!(daemon.append(entry(1, 1000, "issue", "g_a", "allow")).unwrap().seq, 1);
        assert_eq!(daemon.append(entry(2, 1001, "use", "g_a", "allow")).unwrap().seq, 2);
        let mut run = AuditLog::open(&dir).unwrap();
        let launched = run.append(entry(0, 1002, "sandbox-launch", "g_a", "allow")).unwrap();
        assert_eq!(launched.seq, 3, "a writer that asks for no number is given the next one");
        // The daemon's own counter says 3 — it never saw the run's record. It must not repeat it.
        let used = daemon.append(entry(3, 1003, "use", "g_a", "allow")).unwrap();
        assert_eq!(used.seq, 4, "a writer behind the chain is moved past it, never onto a seq in use");
        assert_eq!(daemon.next_seq(), Some(5));
        // A daemon restarting opens the chain again and resumes after its highest seq.
        let mut restarted = AuditLog::open(&dir).unwrap();
        assert_eq!(restarted.next_seq(), Some(5), "a reopened chain resumes, it does not start again at 1");
        assert_eq!(restarted.append(entry(1, 1004, "root-policy-mode", "g_a", "allow")).unwrap().seq, 5);
        let seqs: Vec<u64> = tail(&dir, 100).unwrap().iter().map(|r| r.seq).collect();
        assert_eq!(seqs, vec![1, 2, 3, 4, 5], "unique and strictly increasing in chain order");
        assert_eq!(verify(&dir).expect("and it verifies").records, 5);
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

    /// MONITOR-CHAIN-1 (RW 4.49): a follower reads what was appended after it — across a day's file into the next — and
    /// nothing from before it: the record it had already passed is overwritten here, in place, with a line that would not
    /// verify, and nothing the follower returns is about it.
    #[test]
    fn a_follower_reads_only_what_was_appended_after_it_across_days() {
        let dir = tmp_dir("follow");
        let (day1, day2) = (86_400_000 + 500, 2 * 86_400_000 + 500);
        let mut log = AuditLog::open(&dir).unwrap();
        log.append(entry(0, day1, "issue", "g_a", "allow")).unwrap();
        let mut f = Follower::at_end(&dir).unwrap();
        assert_eq!(f.head(), log.head(), "at the chain's end");
        assert!(f.read_new().unwrap().records.is_empty(), "nothing appended yet");
        log.append(entry(0, day1 + 1, "use", "g_a", "allow")).unwrap();
        log.append(entry(0, day2, "use", "g_a", "deny")).unwrap();
        assert_eq!(list_day_files(&dir).unwrap(), vec!["19700102", "19700103"], "the third went into the next day's file");
        let path = day_path(&dir, "19700102");
        let text = fs::read_to_string(&path).unwrap();
        let passed = text.lines().nth(1).unwrap().to_string();
        fs::write(&path, text.replacen(&passed, &passed.replace("\"issue\"", "\"ISSUE\""), 1)).unwrap();
        let got = f.read_new().unwrap();
        assert_eq!(got.records.iter().map(|r| r.seq).collect::<Vec<_>>(), vec![2, 3], "{got:?}");
        assert!(got.unverified.is_empty() && !got.rewritten, "nothing from before it was read: {got:?}");
        assert_eq!(f.head(), log.head());
        assert!(f.read_new().unwrap().records.is_empty(), "each record is read once");
        let _ = fs::remove_dir_all(&dir);
    }

    /// A writer puts down a record and then its line break: a line without its break may still be growing, so neither a
    /// follower's read nor the end it starts from takes it — both read it once it is whole.
    #[test]
    fn a_follower_leaves_a_line_without_its_break_for_the_next_read() {
        let dir = tmp_dir("follow-partial");
        let mut log = AuditLog::open(&dir).unwrap();
        log.append(entry(0, 1000, "issue", "g_a", "allow")).unwrap();
        let mut f = Follower::at_end(&dir).unwrap();
        let rec = entry(2, 1001, "use", "g_a", "allow").into_record(log.head());
        let mut file = fs::OpenOptions::new().append(true).open(day_path(&dir, &day_string(1000))).unwrap();
        file.write_all(rec.to_line().as_bytes()).unwrap();
        let got = f.read_new().unwrap();
        assert!(got.records.is_empty() && got.unverified.is_empty(), "a line still growing is not read: {got:?}");
        let mut g = Follower::at_end(&dir).unwrap();
        assert_eq!(g.head(), log.head(), "nor is it the end a follower starts from");
        file.write_all(b"\n").unwrap();
        for (name, follower) in [("f", &mut f), ("g", &mut g)] {
            let got = follower.read_new().unwrap();
            assert_eq!(got.records.iter().map(|r| r.seq).collect::<Vec<_>>(), vec![2], "{name}: {got:?}");
            assert!(got.unverified.is_empty(), "{name}: {got:?}");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    /// What a follower returns as a record is what the chain vouches for. A deny whose hash does not recompute, a line that
    /// is not a record, and a record that names another predecessor are each UNVERIFIED — and the follower reads on from
    /// the last one's stored hash, so the record written after them is read.
    #[test]
    fn a_follower_returns_lines_that_do_not_verify_as_unverified_and_reads_on() {
        let dir = tmp_dir("follow-forged");
        let mut log = AuditLog::open(&dir).unwrap();
        let first = log.append(entry(0, 1000, "issue", "g_a", "allow")).unwrap();
        let mut f = Follower::at_end(&dir).unwrap();
        let mut altered = entry(2, 1001, "use", "g_a", "allow").into_record(&first.hash);
        altered.decision = "deny".into(); // changed after it was hashed
        let elsewhere = entry(3, 1002, "use", "g_a", "deny").into_record(GENESIS_HASH);
        let after = entry(4, 1003, "use", "g_a", "deny").into_record(&elsewhere.hash);
        let lines = format!("{}\nnot a record\n{}\n{}\n", altered.to_line(), elsewhere.to_line(), after.to_line());
        append_line(&day_path(&dir, &day_string(1000)), lines.trim_end()).unwrap();
        let got = f.read_new().unwrap();
        assert_eq!(got.unverified, vec![Some(2), None, Some(3)], "{got:?}");
        assert_eq!(got.records.iter().map(|r| r.seq).collect::<Vec<_>>(), vec![4], "{got:?}");
        let _ = fs::remove_dir_all(&dir);
    }

    /// A day file cut short under a follower is a chain rewritten under it: it says so, reads nothing in between, and goes
    /// on from the chain's end as it now is.
    #[test]
    fn a_follower_whose_file_was_cut_short_says_the_chain_was_rewritten() {
        let dir = tmp_dir("follow-cut");
        let mut log = AuditLog::open(&dir).unwrap();
        let first = log.append(entry(0, 1000, "issue", "g_a", "allow")).unwrap();
        log.append(entry(0, 1001, "use", "g_a", "allow")).unwrap();
        let mut f = Follower::at_end(&dir).unwrap();
        let path = day_path(&dir, &day_string(1000));
        let text = fs::read_to_string(&path).unwrap();
        fs::write(&path, text.lines().take(2).map(|l| format!("{l}\n")).collect::<String>()).unwrap();
        let got = f.read_new().unwrap();
        assert!(got.rewritten && got.records.is_empty() && got.unverified.is_empty(), "{got:?}");
        assert_eq!(f.head(), first.hash, "from the chain's end as it now is");
        let _ = fs::remove_dir_all(&dir);
    }

    /// RW 4.53: a writer catching up after another reads what that writer appended — the rest of its day and the next
    /// day's file — and not the chain behind it. A record behind both is made unreadable here, in place (so nothing after
    /// it moves): the catching-up append still chains onto the other writer's last record, and `verify` still finds the
    /// damage. Before, every catch-up re-read and re-counted the whole chain under the append lock, and refused it here.
    #[test]
    fn a_writer_catching_up_reads_only_what_another_writer_appended() {
        let dir = tmp_dir("catchup");
        let (day1, day2) = (86_400_000 + 500, 2 * 86_400_000 + 500);
        let mut a = AuditLog::open(&dir).unwrap();
        a.append(entry(0, day1, "issue", "g_a", "allow")).unwrap();
        a.append(entry(0, day1 + 1, "use", "g_a", "allow")).unwrap();
        let mut b = AuditLog::open(&dir).unwrap();
        b.append(entry(0, day1 + 2, "use", "g_b", "allow")).unwrap();
        let last = b.append(entry(0, day2, "use", "g_b", "allow")).unwrap();
        assert_eq!(list_day_files(&dir).unwrap(), vec!["19700102", "19700103"], "b's second record began the next day");
        let path = day_path(&dir, "19700102");
        let text = fs::read_to_string(&path).unwrap();
        let behind = text.lines().nth(1).unwrap().to_string();
        fs::write(&path, text.replacen(&behind, &format!("X{}", &behind[1..]), 1)).unwrap();
        let rec = a.append(entry(0, day2 + 1, "revoke", "g_a", "allow")).expect("a catches up past b's records alone");
        assert_eq!((rec.prev_hash.as_str(), rec.seq), (last.hash.as_str(), last.seq + 1), "onto b's last record");
        assert!(verify(&dir).is_err(), "the damage behind is still `verify`'s to find");
        let _ = fs::remove_dir_all(&dir);
    }

    /// What a catch-up reads only past its point must still account for everything the anchor says: a line appended by a
    /// writer that did not anchor it — a forged record, well formed and linked — is refused as the whole recovery refused it.
    #[test]
    fn a_line_appended_without_its_anchor_is_refused_as_before() {
        let dir = tmp_dir("catchup-unanchored");
        let mut a = AuditLog::open(&dir).unwrap();
        let first = a.append(entry(0, 1000, "issue", "g_a", "allow")).unwrap();
        let forged = entry(2, 1001, "use", "g_x", "deny").into_record(&first.hash);
        append_line(&day_path(&dir, &day_string(1000)), &forged.to_line()).unwrap();
        let e = a.append(entry(0, 1002, "use", "g_a", "allow")).expect_err("the anchor does not hold the forged line");
        assert!(format!("{e:?}").contains("anchor disagrees"), "{e:?}");
        let _ = fs::remove_dir_all(&dir);
    }

    /// A day file cut short under a writer is refused as the whole recovery refused it (P17-C1) — never read as "nothing was
    /// appended", which would chain the next record onto a head the file no longer holds.
    #[test]
    fn a_file_cut_short_under_a_writer_is_refused_as_before() {
        let dir = tmp_dir("catchup-cut");
        let mut a = AuditLog::open(&dir).unwrap();
        a.append(entry(0, 1000, "issue", "g_a", "allow")).unwrap();
        a.append(entry(0, 1001, "use", "g_a", "allow")).unwrap();
        let path = day_path(&dir, &day_string(1000));
        let text = fs::read_to_string(&path).unwrap();
        fs::write(&path, text.lines().take(2).map(|l| format!("{l}\n")).collect::<String>()).unwrap();
        let e = a.append(entry(0, 1002, "use", "g_a", "allow")).expect_err("the anchor holds a record the file lost");
        assert!(format!("{e:?}").contains("anchor disagrees"), "{e:?}");
        let _ = fs::remove_dir_all(&dir);
    }

    /// RW 4.53 (D-V2-160): an opening whose files measure as the anchor says reads the newest file's last line and
    /// nothing else. A record in an OLDER day is made unreadable here, in place (its length unchanged): the opening still
    /// takes the anchor's word and the next record chains onto the head, while `verify` finds the damage. Before, every
    /// opening re-read and re-counted every day file, and refused here.
    #[test]
    fn an_opening_reads_only_the_newest_files_last_line_when_the_files_measure_as_the_anchor_says() {
        // The newest record short, and longer than the 64 KiB read from the file's end.
        for target_len in [0usize, 70_000] {
            let dir = tmp_dir(&format!("open-fast-{target_len}"));
            let (day1, day2) = (86_400_000 + 500, 2 * 86_400_000 + 500);
            let last = {
                let mut log = AuditLog::open(&dir).unwrap();
                log.append(entry(0, day1, "issue", "g_a", "allow")).unwrap();
                log.append(entry(0, day1 + 1, "use", "g_a", "allow")).unwrap();
                let mut newest = entry(0, day2, "use", "g_a", "allow");
                newest.target = Some("t".repeat(target_len));
                log.append(newest).unwrap()
            };
            let path = day_path(&dir, "19700102");
            let text = fs::read_to_string(&path).unwrap();
            let older = text.lines().nth(1).unwrap().to_string();
            fs::write(&path, text.replacen(&older, &format!("X{}", &older[1..]), 1)).unwrap();
            let mut log = AuditLog::open(&dir)
                .unwrap_or_else(|e| panic!("{target_len}: the files measure as the anchor says, and it was read whole: {e:?}"));
            let rec = log.append(entry(0, day2 + 1, "revoke", "g_a", "allow")).unwrap();
            assert_eq!((rec.prev_hash.as_str(), rec.seq), (last.hash.as_str(), last.seq + 1), "onto the head, past the top seq");
            assert!(verify(&dir).is_err(), "the damage behind is still `verify`'s to find");
            let _ = fs::remove_dir_all(&dir);
        }
    }

    /// The fast opening is the whole recovery's answer, or none: after every append of a history two writers make across
    /// three days — each catching up after the other, by reading what was appended — the anchor's note lets an opening
    /// take it, and what it takes equals what reading every file gives.
    #[test]
    fn the_fast_opening_agrees_with_the_whole_recovery_after_every_append() {
        let dir = tmp_dir("open-agree");
        let mut a = AuditLog::open(&dir).unwrap();
        let mut b = AuditLog::open(&dir).unwrap();
        let mut ts = 86_400_000i64 + 500;
        for i in 0..24u64 {
            if i % 8 == 7 {
                ts += 86_400_000; // the next day's file
            }
            ts += 1;
            let w = if (i * 7) % 3 == 0 { &mut a } else { &mut b };
            w.append(entry(0, ts, "use", "g_a", "allow")).unwrap();
            let fast = recover_fast(&dir)
                .unwrap()
                .unwrap_or_else(|| panic!("after append {i} the opening could not take the anchor's word"));
            let whole = recover(&dir).unwrap();
            assert_eq!(fast, whole, "after append {i}");
        }
        assert_eq!(list_day_files(&dir).unwrap().len(), 4, "the history crossed three midnights");
        let _ = fs::remove_dir_all(&dir);
    }

    /// What an opening refused before it still refuses: a line deleted from an older day (its bytes change), a line
    /// deleted from the middle of the newest day (its last line, the head, unchanged), and the newest record replaced in
    /// place by one whose hash is not the anchor's head.
    #[test]
    fn an_opening_still_refuses_an_older_line_deleted_or_the_newest_record_replaced() {
        for case in ["older line deleted", "newest middle line deleted", "newest replaced"] {
            let dir = tmp_dir(&format!("open-refuse-{}", case.replace(' ', "-")));
            let (day1, day2) = (86_400_000 + 500, 2 * 86_400_000 + 500);
            {
                let mut log = AuditLog::open(&dir).unwrap();
                log.append(entry(0, day1, "issue", "g_a", "allow")).unwrap();
                log.append(entry(0, day1 + 1, "use", "g_a", "allow")).unwrap();
                log.append(entry(0, day2, "use", "g_a", "allow")).unwrap();
                if case == "newest middle line deleted" {
                    log.append(entry(0, day2 + 1, "use", "g_a", "allow")).unwrap();
                }
            }
            let (path, text) = if case == "newest middle line deleted" {
                let path = day_path(&dir, "19700103");
                let text = fs::read_to_string(&path).unwrap();
                let line = text.lines().nth(1).unwrap().to_string();
                (path, text.replacen(&format!("{line}\n"), "", 1))
            } else if case == "older line deleted" {
                let path = day_path(&dir, "19700102");
                let text = fs::read_to_string(&path).unwrap();
                let line = text.lines().nth(1).unwrap().to_string();
                (path, text.replacen(&format!("{line}\n"), "", 1))
            } else {
                let path = day_path(&dir, "19700103");
                let text = fs::read_to_string(&path).unwrap();
                // The same length, so only the head's check can tell: another record in its place.
                let rec = entry(3, day2, "use", "g_b", "allow").into_record(GENESIS_HASH);
                let line = text.lines().last().unwrap().to_string();
                assert_eq!(rec.to_line().len(), line.len());
                (path, text.replacen(&line, &rec.to_line(), 1))
            };
            fs::write(&path, text).unwrap();
            let e = AuditLog::open(&dir).err().unwrap_or_else(|| panic!("{case}: the opening must refuse"));
            assert!(format!("{e:?}").contains("anchor disagrees"), "{case}: {e:?}");
            let _ = fs::remove_dir_all(&dir);
        }
    }

    /// An anchor written before RW 4.53 has no note of where the chain stood: the opening reads every file, as before, and
    /// the anchor it re-affirms carries the note, so the next opening can take its word.
    #[test]
    fn an_anchor_without_its_note_is_read_the_whole_way_once() {
        let dir = tmp_dir("open-old-anchor");
        let head = {
            let mut log = AuditLog::open(&dir).unwrap();
            log.append(entry(0, 1000, "issue", "g_a", "allow")).unwrap();
            log.append(entry(0, 1001, "use", "g_a", "allow")).unwrap().hash
        };
        fs::write(anchor_path(&dir), format!("{{\"head\":\"{head}\",\"records\":2}}\n")).unwrap();
        assert!(read_anchor_at(&dir).is_none() && recover_fast(&dir).unwrap().is_none());
        let log = AuditLog::open(&dir).expect("the old anchor agrees with the files");
        assert_eq!(log.head(), head);
        assert!(recover_fast(&dir).unwrap().is_some(), "the re-affirmed anchor carries its note");
        let _ = fs::remove_dir_all(&dir);
    }
}
