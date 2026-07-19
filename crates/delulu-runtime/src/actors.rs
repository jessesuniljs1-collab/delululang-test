//! The native actor runtime (Stage 7 phase 7g, spec §6).
//!
//! Topology (build-order deviation 3, recorded honestly): **worker-owned actors**, not
//! stealing deques. Each actor is pinned to one worker thread at spawn (round-robin) and
//! never migrates; each worker owns its actors' heaps outright. Consequences:
//! - **Zero `unsafe`.** `Value` is `Rc`-based and not `Send`; because cells never cross
//!   threads and messages cross only as [`MsgValue`] (an owned, `Send`-by-construction
//!   representation), no `unsafe impl Send` exists anywhere in this runtime.
//! - **Per-sender-pair FIFO** falls out of `mpsc` (each sender's messages to one worker
//!   arrive in order, and an actor's worker is fixed) — exactly the spec §6.1 guarantee,
//!   which never promised global ordering.
//! - Load balance is round-robin at spawn, not steal-at-runtime; a stealing scheduler is a
//!   throughput optimization deferred with multi-threaded WASM (Stage 10).
//!
//! Message passing (spec §6.2 / deviation 7): an `iso` graph MOVES by rebuild — the
//! sender's binding is statically dead (`consume`), so rebuild vs pointer handoff is
//! observationally identical; the debug lane asserts the moved graph really was unaliased
//! (`Rc::strong_count == 1`, the §7.4 check). `val` data converts by structure; `tag`
//! (actor references) copies the address.
//!
//! Quiescence (spec §6.1): program exit when `main` has returned AND all mailboxes are
//! empty AND no turn is running — tracked as one atomic pending-count (enqueue increments,
//! turn completion decrements; a turn's own enqueues happen before its decrement, so the
//! count can never falsely reach zero).
//!
//! Failures (spec §6.6): a fault in a behavior poisons its actor — subsequent sends are
//! dropped and counted, reported at exit; `--on-actor-death abort` opts into whole-program
//! abort. Supervision is post-1.0.

use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};

use delulu_check::ResourceKind;
use delulu_syntax::ast::{Block, Module};

use crate::trace::{Cause, TraceRecord, TraceSink};
use crate::value::{CapScope, CapVal, Closure, Env, Fault, Scope, SecretVal, Value};

/// The causal identity of a send (spec §6.3): who sent, from which member, from where.
/// Carried on every job so the receiving turn's trace records can name their cause.
#[derive(Clone, Debug)]
pub struct SendCause {
    pub sender: String,
    pub sender_member: Option<String>,
    pub send_span: Option<(u32, u32, u32)>,
}

/// A process-wide actor address: the owning worker plus a unique id. The id is never
/// reused (monotonic mint), so a reference to a dead actor stays dead forever.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ActorId {
    pub worker: usize,
    pub id: u64,
}

/// The Send-safe actor-boundary value representation. Nothing here is `Rc`: this enum IS
/// `Send` by construction, which is the whole safety argument of the runtime.
#[derive(Debug)]
pub enum MsgValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Unit,
    List(Vec<MsgValue>),
    Record { name: String, fields: Vec<(String, MsgValue)> },
    Variant { name: String, fields: Vec<MsgValue> },
    /// A sendable (`val`-inferred) closure: body AST plus the converted capture scopes,
    /// outermost first (the module-global scope is NOT carried — the receiver reattaches
    /// its own identical globals).
    Closure { params: Vec<String>, body: Block, scopes: Vec<Vec<(String, MsgValue)>> },
    Cap { kind: ResourceKind, scope: CapScope },
    SecretLocal(String),
    SecretHandle(String),
    Root(RootMsg),
    Actor { id: ActorId, actor: String },
}

/// A root slice crossing an actor boundary (value-level authority passing, spec §7).
#[derive(Debug)]
pub struct RootMsg {
    pub console: bool,
    pub fs_read: Vec<PathBuf>,
    pub fs_write: Vec<PathBuf>,
    pub net: Vec<String>,
    pub clock: bool,
    pub rand: bool,
    pub declassify: bool,
    pub secrets: Vec<(String, String)>,
    pub foreign_load: bool,
    pub python_allowlist: Vec<String>,
    pub broker_secrets: Vec<String>,
}

enum Job {
    Create { id: u64, actor: String, args: Vec<MsgValue>, cause: SendCause },
    Send { id: u64, behavior: String, args: Vec<MsgValue>, cause: SendCause },
    Shutdown,
}

/// A bounded mailbox's live state (Stage 10 phase 10c, spec §3 B2). Unconfigured actors have no
/// entry and keep the 1.0 unbounded behavior — bounding is opt-in, so no existing program
/// changes meaning (stability contract).
struct MailboxState {
    actor: String,
    bound: usize,
    /// `true` = `drop-new` (overflow drops the NEW message, counted); `false` = `block`
    /// (the sending turn waits at the send site until space frees — spec §3's default).
    drop_new: bool,
    depth: AtomicI64,
    peak: AtomicU64,
    drops: AtomicU64,
}

/// What a send did (Stage 10): delivery, or an overflow drop under `drop-new`. The interpreter
/// turns a drop into DL1902 when the program runs in abort mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SendOutcome {
    Delivered,
    DroppedOverflow,
}

/// Per-actor mailbox telemetry, reported at quiescence under `--trace-memory` (spec §3 B3 —
/// the mailbox half; heap bytes and collection counts arrive with the 10d collector).
#[derive(Clone, Debug)]
pub struct MailboxStat {
    pub actor: String,
    pub bound: usize,
    pub peak: u64,
    pub drops: u64,
}

/// State shared by every thread that can spawn or send.
struct Shared {
    pending: AtomicI64,
    idle: Mutex<bool>,
    quiesced: Condvar,
    next_actor: AtomicU64,
    round_robin: AtomicUsize,
    live_actors: AtomicI64,
    dead_actors: AtomicU64,
    dead_sends: AtomicU64,
    total_turns: AtomicU64,
    abort_on_death: bool,
    died: AtomicBool,
    /// Stage 10 (10c): the manifest-level default mailbox bound (`[actors] mailbox = N`);
    /// `None` = unbounded, the 1.0 behavior.
    default_bound: Option<usize>,
    /// Stage 10 (10c): the manifest-level overflow policy (`[actors] overflow = "drop-new"`);
    /// `false` = `block`, the spec default for bounded mailboxes.
    overflow_drop_new: bool,
    /// Live bounded-mailbox states, keyed by actor id. Only configured actors appear.
    mailboxes: Mutex<HashMap<u64, Arc<MailboxState>>>,
    /// Blocked senders park here; every Send-job completion (and program death) notifies.
    space_lock: Mutex<()>,
    space: Condvar,
    overflow_drops: AtomicU64,
}

impl Shared {
    fn enqueue(&self) {
        self.pending.fetch_add(1, Ordering::SeqCst);
    }
    fn done(&self) {
        if self.pending.fetch_sub(1, Ordering::SeqCst) == 1 {
            let _g = self.idle.lock().unwrap();
            self.quiesced.notify_all();
        }
    }
}

/// The per-thread handle through which an [`crate::interp::Interp`] spawns and sends.
/// Cheap to clone per thread; `mpsc::Sender` is `Send`, so worker threads get their own.
pub struct ActorHost {
    shared: Arc<Shared>,
    senders: Vec<mpsc::Sender<Job>>,
    /// Which worker this host belongs to (`None` = the main thread). Stage 10 (10c): a `block`
    /// send from a worker to an actor it OWNS can never wait — the only thread that could drain
    /// that mailbox is the one that would be waiting. Structural self-deadlock, refused by
    /// construction: same-worker sends bypass the bound, and the exemption is documented and
    /// witnessed rather than discovered in production.
    me: Option<usize>,
}

impl ActorHost {
    /// Spawn, resolving the mailbox config (Stage 10, 10c): the declaration's
    /// `(mailbox = N)` wins, else the manifest default; no config = unbounded (1.0 behavior).
    pub fn spawn(
        &self,
        actor: &str,
        args: Vec<MsgValue>,
        cause: SendCause,
        decl_bound: Option<u64>,
    ) -> ActorId {
        let id = self.shared.next_actor.fetch_add(1, Ordering::SeqCst);
        let worker = self.shared.round_robin.fetch_add(1, Ordering::SeqCst) % self.senders.len();
        if let Some(bound) = decl_bound.map(|b| b as usize).or(self.shared.default_bound) {
            let st = Arc::new(MailboxState {
                actor: actor.to_string(),
                bound: bound.max(1),
                drop_new: self.shared.overflow_drop_new,
                depth: AtomicI64::new(0),
                peak: AtomicU64::new(0),
                drops: AtomicU64::new(0),
            });
            self.shared.mailboxes.lock().unwrap().insert(id, st);
        }
        self.shared.live_actors.fetch_add(1, Ordering::SeqCst);
        self.shared.enqueue();
        let _ = self.senders[worker].send(Job::Create { id, actor: actor.to_string(), args, cause });
        ActorId { worker, id }
    }

    /// Send, applying the receiver's bound (Stage 10, 10c). The reservation is a CAS loop, so
    /// the bound is exact under concurrent senders. `block` waits here — at the send site, the
    /// turn's last-resort suspension point — until a Send-job completion frees a slot or the
    /// program dies; cross-worker cycles of full mailboxes can therefore deadlock, which spec §3
    /// documents as a non-guarantee (backpressure bounds MEMORY, never liveness).
    pub fn send(&self, to: ActorId, behavior: &str, args: Vec<MsgValue>, cause: SendCause) -> SendOutcome {
        let state = self.shared.mailboxes.lock().unwrap().get(&to.id).cloned();
        if let Some(st) = &state {
            loop {
                let d = st.depth.load(Ordering::SeqCst);
                if (d.max(0) as usize) >= st.bound {
                    if st.drop_new {
                        st.drops.fetch_add(1, Ordering::SeqCst);
                        self.shared.overflow_drops.fetch_add(1, Ordering::SeqCst);
                        return SendOutcome::DroppedOverflow;
                    }
                    // Same-worker or post-mortem sends may not wait (self-deadlock / shutdown);
                    // they deliver past the bound, and the depth counter records the excess.
                    if self.me == Some(to.worker) || self.shared.died.load(Ordering::SeqCst) {
                        st.depth.fetch_add(1, Ordering::SeqCst);
                        break;
                    }
                    let g = self.shared.space_lock.lock().unwrap();
                    // Re-check under the lock so a completion between the load and the wait
                    // cannot strand us (the missed-notification race, closed the classic way).
                    if (st.depth.load(Ordering::SeqCst).max(0) as usize) < st.bound
                        || self.shared.died.load(Ordering::SeqCst)
                    {
                        continue;
                    }
                    let _g = self.shared.space.wait(g).unwrap();
                    continue;
                }
                if st.depth.compare_exchange(d, d + 1, Ordering::SeqCst, Ordering::SeqCst).is_ok() {
                    break;
                }
            }
            let now = st.depth.load(Ordering::SeqCst).max(0) as u64;
            st.peak.fetch_max(now, Ordering::SeqCst);
        }
        self.shared.enqueue();
        let _ = self
            .senders[to.worker]
            .send(Job::Send { id: to.id, behavior: behavior.to_string(), args, cause });
        SendOutcome::Delivered
    }

    /// Whether the program runs in abort mode (`--on-actor-death abort`) — the interpreter
    /// consults this to turn an overflow drop into DL1902 (spec §10).
    pub fn abort_on_death(&self) -> bool {
        self.shared.abort_on_death
    }
}

/// What the system reports at quiescence (`--on-quiesce report` / exit accounting).
#[derive(Clone, Debug)]
pub struct QuiesceReport {
    pub surviving_actors: i64,
    pub dead_actors: u64,
    pub dropped_sends: u64,
    pub total_turns: u64,
    /// True when a behavior fault occurred and `--on-actor-death abort` was set.
    pub aborted: bool,
    /// Stage 10 (10c): messages dropped by `drop-new` mailbox overflow, program-wide.
    pub overflow_drops: u64,
    /// Stage 10 (10c): per-bounded-actor mailbox telemetry (empty when nothing is bounded).
    pub mailbox: Vec<MailboxStat>,
}

/// The actor system: worker threads, their mailboxes, and the quiescence machinery.
pub struct ActorSystem {
    shared: Arc<Shared>,
    senders: Vec<mpsc::Sender<Job>>,
    handles: Vec<std::thread::JoinHandle<()>>,
}

impl ActorSystem {
    /// Start `threads` workers for `module` (its actor declarations are cloned into each
    /// worker, which builds its own single-threaded `Interp` — cells never cross threads).
    pub fn start(module: &Module, threads: usize, abort_on_death: bool) -> ActorSystem {
        ActorSystem::start_with(module, threads, abort_on_death, None, None, None, false)
    }

    /// [`ActorSystem::start`] plus the phase-7i instrumentation: a shared trace collector
    /// (workers stamp their records with actor/member/turn/cause and ship them here) and
    /// the checker's iso-move node set for `--debug-rcaps` (DL1610 on a violated
    /// uniqueness claim — a compiler-bug detector, never the guarantee).
    pub fn start_with(
        module: &Module,
        threads: usize,
        abort_on_death: bool,
        trace: Option<Arc<Mutex<Vec<TraceRecord>>>>,
        debug_rcaps: Option<Arc<std::collections::HashSet<delulu_syntax::ast::NodeId>>>,
        default_mailbox: Option<usize>,
        overflow_drop_new: bool,
    ) -> ActorSystem {
        let threads = threads.max(1);
        let shared = Arc::new(Shared {
            pending: AtomicI64::new(0),
            idle: Mutex::new(false),
            quiesced: Condvar::new(),
            next_actor: AtomicU64::new(1),
            round_robin: AtomicUsize::new(0),
            live_actors: AtomicI64::new(0),
            dead_actors: AtomicU64::new(0),
            dead_sends: AtomicU64::new(0),
            total_turns: AtomicU64::new(0),
            abort_on_death,
            died: AtomicBool::new(false),
            default_bound: default_mailbox,
            overflow_drop_new,
            mailboxes: Mutex::new(HashMap::new()),
            space_lock: Mutex::new(()),
            space: Condvar::new(),
            overflow_drops: AtomicU64::new(0),
        });
        let (senders, receivers): (Vec<_>, Vec<_>) = (0..threads).map(|_| mpsc::channel::<Job>()).unzip();
        let mut handles = Vec::new();
        for (wi, rx) in receivers.into_iter().enumerate() {
            let module = module.clone();
            let shared_w = shared.clone();
            let senders_w = senders.clone();
            let trace_w = trace.clone();
            let debug_w = debug_rcaps.clone();
            handles.push(
                std::thread::Builder::new()
                    .name(format!("delulu-actor-{wi}"))
                    .spawn(move || worker_loop(wi, rx, module, shared_w, senders_w, trace_w, debug_w))
                    .expect("spawn actor worker"),
            );
        }
        ActorSystem { shared, senders, handles }
    }

    /// A host handle for the calling thread's interpreter.
    pub fn host(&self) -> ActorHost {
        ActorHost { shared: self.shared.clone(), senders: self.senders.clone(), me: None }
    }

    /// Block until quiescence (all mailboxes empty, no turn running), then shut down and
    /// join the workers. Called after `main` returns — the spec §6.1 exit condition.
    pub fn finish(self) -> QuiesceReport {
        {
            let guard = self.shared.idle.lock().unwrap();
            let _guard = self
                .shared
                .quiesced
                .wait_while(guard, |_| {
                    self.shared.pending.load(Ordering::SeqCst) > 0
                        && !self.shared.died.load(Ordering::SeqCst)
                })
                .unwrap();
        }
        for s in &self.senders {
            let _ = s.send(Job::Shutdown);
        }
        for h in self.handles {
            let _ = h.join();
        }
        let mut mailbox: Vec<MailboxStat> = self
            .shared
            .mailboxes
            .lock()
            .unwrap()
            .values()
            .map(|st| MailboxStat {
                actor: st.actor.clone(),
                bound: st.bound,
                peak: st.peak.load(Ordering::SeqCst),
                drops: st.drops.load(Ordering::SeqCst),
            })
            .collect();
        mailbox.sort_by(|a, b| a.actor.cmp(&b.actor).then(b.peak.cmp(&a.peak)));
        QuiesceReport {
            surviving_actors: self.shared.live_actors.load(Ordering::SeqCst),
            dead_actors: self.shared.dead_actors.load(Ordering::SeqCst),
            dropped_sends: self.shared.dead_sends.load(Ordering::SeqCst),
            total_turns: self.shared.total_turns.load(Ordering::SeqCst),
            aborted: self.shared.died.load(Ordering::SeqCst),
            overflow_drops: self.shared.overflow_drops.load(Ordering::SeqCst),
            mailbox,
        }
    }
}

struct Cell {
    actor: String,
    /// The actor's fields as a Record value — `self` in member bodies, so the ordinary
    /// field read/write machinery IS the actor-state machinery.
    state: Value,
    dead: bool,
}

fn worker_loop(
    wi: usize,
    rx: mpsc::Receiver<Job>,
    module: Module,
    shared: Arc<Shared>,
    senders: Vec<mpsc::Sender<Job>>,
    trace: Option<Arc<Mutex<Vec<TraceRecord>>>>,
    debug_rcaps: Option<Arc<std::collections::HashSet<delulu_syntax::ast::NodeId>>>,
) {
    let mut interp = crate::interp::Interp::new(&module)
        .with_actors(ActorHost { shared: shared.clone(), senders, me: Some(wi) });
    let local_sink = trace.as_ref().map(|_| TraceSink::new());
    if let Some(s) = &local_sink {
        interp = interp.with_trace(s.clone());
    }
    if let Some(d) = &debug_rcaps {
        interp = interp.with_debug_rcaps(d.clone());
    }
    // Stamp this turn's records with turn id + cause and ship them to the collector
    // (actor/member were stamped by the interpreter at record time).
    let ship = |turn_id: u64, cause: &SendCause| {
        if let (Some(collector), Some(sink)) = (&trace, &local_sink) {
            let c = Cause {
                sender: cause.sender.clone(),
                sender_member: cause.sender_member.clone(),
                send_span: cause.send_span,
            };
            let mut recs = sink.take_records();
            for r in &mut recs {
                r.turn = Some(turn_id);
                r.cause = Some(c.clone());
            }
            collector.lock().unwrap().extend(recs);
        }
    };
    let mut cells: HashMap<u64, Cell> = HashMap::new();

    while let Ok(job) = rx.recv() {
        match job {
            Job::Shutdown => break,
            Job::Create { id, actor, args, cause } => {
                let Some(decl) = interp.actor_decl(&actor).cloned() else {
                    shared.dead_sends.fetch_add(1, Ordering::SeqCst);
                    shared.done();
                    continue;
                };
                // Fields pre-seed Unit; the checker's definite-initialization rule makes an
                // uninitialized read unreachable in a checked program.
                let fields: Vec<(String, Value)> =
                    decl.fields.iter().map(|f| (f.name.name.clone(), Value::Unit)).collect();
                let state = Value::Record {
                    name: Rc::from(actor.as_str()),
                    fields: Rc::new(std::cell::RefCell::new(fields)),
                };
                let self_ref = Value::ActorRef { id: ActorId { worker: wi, id }, actor: Rc::from(actor.as_str()) };
                let params: Vec<String> = decl.ctor.params.iter().map(|p| p.name.name.clone()).collect();
                let argvals: Vec<Value> = args.into_iter().map(|m| interp.msg_to_value(m)).collect();
                let turn_id = shared.total_turns.fetch_add(1, Ordering::SeqCst);
                let dead = match interp.run_actor_turn(&decl.ctor.body, "new", &params, argvals, state.clone(), self_ref) {
                    Ok(()) => false,
                    Err(fault) => {
                        actor_died(&shared, &actor, "new", &fault);
                        true
                    }
                };
                ship(turn_id, &cause);
                cells.insert(id, Cell { actor, state, dead });
                shared.done();
            }
            Job::Send { id, behavior, args, cause } => {
                // Stage 10 (10c): this job leaving the queue is what frees a mailbox slot —
                // on EVERY path out of this arm (dead, unknown behavior, delivered), so the
                // depth pairs exactly with the send-site increment. Blocked senders wake here.
                let _slot = MailboxSlot::release_on_drop(&shared, id);
                let deliverable = matches!(cells.get(&id), Some(c) if !c.dead);
                if !deliverable {
                    // Poisoned or unknown: dropped silently, counted, reported at exit
                    // (spec §6.6 — the system stays live by default).
                    shared.dead_sends.fetch_add(1, Ordering::SeqCst);
                    shared.done();
                    continue;
                }
                let (actor_name, state) = {
                    let c = &cells[&id];
                    (c.actor.clone(), c.state.clone())
                };
                let decl = interp.actor_decl(&actor_name).cloned().expect("cell's actor is declared");
                let Some(beh) = decl.behaviors.iter().find(|b| b.name.name == behavior) else {
                    shared.dead_sends.fetch_add(1, Ordering::SeqCst);
                    shared.done();
                    continue;
                };
                let self_ref =
                    Value::ActorRef { id: ActorId { worker: wi, id }, actor: Rc::from(actor_name.as_str()) };
                let params: Vec<String> = beh.params.iter().map(|p| p.name.name.clone()).collect();
                let argvals: Vec<Value> = args.into_iter().map(|m| interp.msg_to_value(m)).collect();
                let turn_id = shared.total_turns.fetch_add(1, Ordering::SeqCst);
                if let Err(fault) = interp.run_actor_turn(&beh.body, &behavior, &params, argvals, state, self_ref) {
                    actor_died(&shared, &actor_name, &behavior, &fault);
                    if let Some(c) = cells.get_mut(&id) {
                        c.dead = true;
                    }
                }
                ship(turn_id, &cause);
                shared.done();
            }
        }
    }
}

fn actor_died(shared: &Shared, actor: &str, member: &str, fault: &Fault) {
    shared.live_actors.fetch_sub(1, Ordering::SeqCst);
    shared.dead_actors.fetch_add(1, Ordering::SeqCst);
    eprintln!(
        "actor {actor} died in `{member}`: {} [{}] (its later messages will be dropped and counted)",
        fault.message, fault.code
    );
    if shared.abort_on_death {
        shared.died.store(true, Ordering::SeqCst);
        let _g = shared.idle.lock().unwrap();
        shared.quiesced.notify_all();
        // Blocked senders must not outlive the program: wake them so they observe `died`.
        drop(_g);
        let _s = shared.space_lock.lock().unwrap();
        shared.space.notify_all();
    }
}

/// The Send-job slot release (Stage 10, 10c): decrements the receiver's mailbox depth and wakes
/// blocked senders when the job leaves the queue — via Drop, so no early `continue` in the job
/// arm can ever skip it (the skip branch, closed by construction).
struct MailboxSlot<'a> {
    shared: &'a Shared,
    state: Option<Arc<MailboxState>>,
}

impl<'a> MailboxSlot<'a> {
    fn release_on_drop(shared: &'a Shared, id: u64) -> MailboxSlot<'a> {
        let state = shared.mailboxes.lock().unwrap().get(&id).cloned();
        MailboxSlot { shared, state }
    }
}

impl Drop for MailboxSlot<'_> {
    fn drop(&mut self) {
        if let Some(st) = &self.state {
            st.depth.fetch_sub(1, Ordering::SeqCst);
            let _g = self.shared.space_lock.lock().unwrap();
            self.shared.space.notify_all();
        }
    }
}

// ----- Value ⇄ MsgValue conversion (the boundary) ---------------------------------------

/// Convert a value for an actor boundary. `Err` carries an honest fault; unreachable for a
/// CHECKED program (the rcap pass fails closed on everything unconvertible), so the code is
/// the checker-bug class.
pub fn value_to_msg(v: &Value, self_state: Option<(&Value, ActorId, &str)>) -> Result<MsgValue, Fault> {
    // `self` crossing a boundary is the actor handing out its own ADDRESS (ref <: tag) —
    // the classic reply-to pattern. Detected by heap identity with the current turn's state.
    if let (Value::Record { fields, .. }, Some((Value::Record { fields: sf, .. }, id, name))) = (v, self_state) {
        if Rc::ptr_eq(fields, sf) {
            return Ok(MsgValue::Actor { id, actor: name.to_string() });
        }
    }
    Ok(match v {
        Value::Int(i) => MsgValue::Int(*i),
        Value::Float(f) => MsgValue::Float(*f),
        Value::Bool(b) => MsgValue::Bool(*b),
        Value::Str(s) => MsgValue::Str(s.to_string()),
        Value::Unit => MsgValue::Unit,
        Value::List(items) => {
            // Debug lane (§7.4 / deviation 7): an iso move's graph must be unaliased. At
            // this layer we can only see the spine; `--debug-rcaps` (7i) deepens this.
            debug_assert!(
                Rc::strong_count(items) <= 2, // the binding + this borrow's clone path
                "actor-boundary list with {} strong refs — the static uniqueness proof failed",
                Rc::strong_count(items)
            );
            MsgValue::List(items.borrow().iter().map(|x| value_to_msg(x, self_state)).collect::<Result<_, _>>()?)
        }
        Value::Record { name, fields } => MsgValue::Record {
            name: name.to_string(),
            fields: fields
                .borrow()
                .iter()
                .map(|(n, x)| Ok((n.clone(), value_to_msg(x, self_state)?)))
                .collect::<Result<_, Fault>>()?,
        },
        Value::Variant { name, fields } => MsgValue::Variant {
            name: name.to_string(),
            fields: fields.iter().map(|x| value_to_msg(x, self_state)).collect::<Result<_, _>>()?,
        },
        Value::Closure(c) => {
            let scopes = flatten_env(&c.env, self_state)?;
            MsgValue::Closure { params: c.params.clone(), body: c.body.clone(), scopes }
        }
        Value::Cap(c) => MsgValue::Cap { kind: c.kind, scope: c.scope.clone() },
        Value::Secret(s) => match s.handle_name() {
            Some(h) => MsgValue::SecretHandle(h.to_string()),
            None => MsgValue::SecretLocal(s.reveal()),
        },
        Value::Root(r) => MsgValue::Root(RootMsg {
            console: r.console,
            fs_read: r.fs_read.clone(),
            fs_write: r.fs_write.clone(),
            net: r.net.clone(),
            clock: r.clock,
            rand: r.rand,
            declassify: r.declassify,
            secrets: r.secrets.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            foreign_load: r.foreign_load,
            python_allowlist: r.python_allowlist.clone(),
            broker_secrets: r.broker_secrets.clone(),
        }),
        Value::ActorRef { id, actor } => MsgValue::Actor { id: *id, actor: actor.to_string() },
        // Foreign machinery is actor-pinned or v0.7-fenced at check time; reaching here
        // means the static fence has a hole — say so.
        Value::Foreign(_) | Value::ForeignPtr(_) => {
            return Err(Fault::new(
                "DL0907",
                "a foreign value reached an actor boundary (checker bug if ever seen in a checked program)",
            ))
        }
        #[cfg(feature = "python")]
        Value::PyObj(_) => {
            return Err(Fault::new(
                "DL0907",
                "a PyObj reached an actor boundary (checker bug — PyObj is statically pinned)",
            ))
        }
    })
}

/// Flatten a closure's capture environment into per-scope name/value lists, outermost
/// first, EXCLUDING the root (module globals — the receiver reattaches its own).
fn flatten_env(env: &Env, self_state: Option<(&Value, ActorId, &str)>) -> Result<Vec<Vec<(String, MsgValue)>>, Fault> {
    let mut chain = Vec::new();
    let mut cur = Some(env.clone());
    while let Some(s) = cur {
        chain.push(s.clone());
        cur = s.parent().cloned();
    }
    chain.pop(); // drop the root globals
    chain.reverse(); // outermost first
    let mut out = Vec::new();
    for scope in chain {
        let mut vars = Vec::new();
        for (n, v) in scope.vars_snapshot() {
            vars.push((n, value_to_msg(&v, self_state)?));
        }
        out.push(vars);
    }
    Ok(out)
}

/// Rebuild a message value into the RECEIVING actor's heap. `globals` is the receiving
/// interpreter's module-global scope (closures reattach to it).
pub fn msg_to_value(m: MsgValue, globals: &Env) -> Value {
    match m {
        MsgValue::Int(i) => Value::Int(i),
        MsgValue::Float(f) => Value::Float(f),
        MsgValue::Bool(b) => Value::Bool(b),
        MsgValue::Str(s) => Value::str(s),
        MsgValue::Unit => Value::Unit,
        MsgValue::List(items) => Value::List(Rc::new(std::cell::RefCell::new(
            items.into_iter().map(|x| msg_to_value(x, globals)).collect(),
        ))),
        MsgValue::Record { name, fields } => Value::Record {
            name: Rc::from(name.as_str()),
            fields: Rc::new(std::cell::RefCell::new(
                fields.into_iter().map(|(n, x)| (n, msg_to_value(x, globals))).collect(),
            )),
        },
        MsgValue::Variant { name, fields } => Value::Variant {
            name: Rc::from(name.as_str()),
            fields: Rc::new(fields.into_iter().map(|x| msg_to_value(x, globals)).collect()),
        },
        MsgValue::Closure { params, body, scopes } => {
            let mut env = globals.clone();
            for scope in scopes {
                let child = Scope::child(&env);
                for (n, v) in scope {
                    child.define(&n, msg_to_value(v, globals));
                }
                env = child;
            }
            Value::Closure(Rc::new(Closure { params, body, env }))
        }
        MsgValue::Cap { kind, scope } => Value::Cap(Rc::new(CapVal { kind, scope })),
        MsgValue::SecretLocal(s) => Value::Secret(Rc::new(SecretVal::new(s))),
        MsgValue::SecretHandle(h) => Value::Secret(Rc::new(SecretVal::handle(h))),
        MsgValue::Root(r) => {
            let root = crate::value::RootVal {
                console: r.console,
                fs_read: r.fs_read,
                fs_write: r.fs_write,
                net: r.net,
                clock: r.clock,
                rand: r.rand,
                declassify: r.declassify,
                secrets: r.secrets.into_iter().collect(),
                foreign_load: r.foreign_load,
                python_allowlist: r.python_allowlist,
                broker_secrets: r.broker_secrets,
            };
            Value::Root(Rc::new(root))
        }
        MsgValue::Actor { id, actor } => Value::ActorRef { id, actor: Rc::from(actor.as_str()) },
    }
}

/// The `--debug-rcaps` uniqueness walk (spec §7.4, phase 7i): an iso MOVE's graph must be
/// unaliased. The value in hand is the environment's plus the evaluator's clone, so the top
/// spine allows 2 strong refs; every NESTED mutable node (List/Record) must be exactly 1.
/// Immutable values (Str, scalars, variants, caps) share freely and are exempt. A violation
/// is compiler-bug class: the STATIC uniqueness proof failed — never a runtime safety net
/// (the rebuild boundary is race-free by construction either way).
pub fn assert_unique_graph(v: &Value) -> Result<(), String> {
    fn walk(v: &Value, top: bool, path: &str) -> Result<(), String> {
        match v {
            Value::List(rc) => {
                let max = if top { 2 } else { 1 };
                let n = Rc::strong_count(rc);
                if n > max {
                    return Err(format!("list at {path} has {n} strong refs (max {max})"));
                }
                for (i, x) in rc.borrow().iter().enumerate() {
                    walk(x, false, &format!("{path}[{i}]"))?;
                }
                Ok(())
            }
            Value::Record { fields, .. } => {
                let max = if top { 2 } else { 1 };
                let n = Rc::strong_count(fields);
                if n > max {
                    return Err(format!("record at {path} has {n} strong refs (max {max})"));
                }
                for (fname, x) in fields.borrow().iter() {
                    walk(x, false, &format!("{path}.{fname}"))?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    walk(v, true, "arg")
}
