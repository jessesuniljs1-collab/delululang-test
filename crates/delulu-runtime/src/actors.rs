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
    /// A `Map` crossing the boundary (P3). Carried as an ORDERED vector of pairs rather than a map,
    /// because the wire form should not depend on the receiver rebuilding the same ordering — it
    /// arrives in the order the sender iterated, which is ascending by key.
    Map(Vec<(crate::value::MapKey, MsgValue)>),
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
    /// PS-B-02: the `net.special=` subset of `net`, carried so a root crossing an actor boundary
    /// keeps exactly the special-use authority it had — never widened, and never silently dropped
    /// either (a drop would fail closed, but a rule that holds on two paths out of three is a rule
    /// nobody can state).
    pub net_special: Vec<String>,
    pub clock: bool,
    pub rand: bool,
    pub declassify: bool,
    pub secrets: Vec<(String, String)>,
    pub foreign_load: bool,
    pub python_allowlist: Vec<String>,
    pub broker_secrets: Vec<String>,
    /// Stage 10 (10e): actuator envelopes crossing an actor boundary — plain data, `Send` by
    /// construction like everything else here; the envelope is authority data the receiving
    /// actor can hold but never widen.
    pub actuators: Vec<crate::value::ActuatorEnvelope>,
    pub sensors: Vec<String>,
    /// Stage 10 (10h): compute envelopes, carried for the same reasons the actuator list is
    /// (`HARDENING_CAMPAIGN.md` C35, ruling D46). `ComputeEnvelope` is plain data and `Send` by
    /// construction, the envelope BOUNDS its holder rather than empowering them, and an actor cannot
    /// widen one any more than it can widen an actuator envelope.
    ///
    /// This was withheld until the owner decided, and the argument that settled it is the asymmetry:
    /// actuators cross, and actuation moves physical machines. Refusing the strictly less
    /// consequential dimension while allowing the more consequential one was an omission from phase
    /// 10h, not a safety position — the old comment here even justified withholding as "fail closed,
    /// like the actuator list", next to the line where the actuator list crosses.
    pub computes: Vec<crate::value::ComputeEnvelope>,
    /// P2 (D-V2-27): the plugin-loading grant, carried for the same reason `foreign_load` beside it
    /// is. Loading foreign C already crosses this boundary, and a `.dpx` plugin is the strictly more
    /// confined of the two — it runs on the WASM engine under a grant-derived import slice, where a
    /// foreign library runs as native code in a worker. Withholding the more contained dimension
    /// while the less contained one crosses would be an omission dressed as a safety position, which
    /// is exactly the mistake the `computes` comment above records.
    ///
    /// Plain data, `Send` by construction, and it BOUNDS its holder: these are the roots the operator
    /// named and the hashes the package pinned. An actor can name a path inside them; it cannot add
    /// one, and every load still runs the whole sequence including the holder check.
    pub plugins: Vec<std::path::PathBuf>,
    pub plugins_allow: Vec<String>,
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
    /// Stage 10 (10d): program-wide cycle-collector accounting, aggregated from the workers.
    cycle_runs: AtomicU64,
    cycle_collected: AtomicU64,
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
    /// Stage 10 (10d): cycle-collector sweeps run, program-wide.
    pub cycle_runs: u64,
    /// Stage 10 (10d): garbage-cycle cells broken and freed, program-wide.
    pub cycle_collected: u64,
}

/// The actor system: worker threads, their mailboxes, and the quiescence machinery.
pub struct ActorSystem {
    shared: Arc<Shared>,
    senders: Vec<mpsc::Sender<Job>>,
    handles: Vec<std::thread::JoinHandle<()>>,
    /// What the workers actually got — see [`ActorSystem::reduced_depth_bound`].
    worker_stack: usize,
    worker_max_depth: u32,
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
            cycle_runs: AtomicU64::new(0),
            cycle_collected: AtomicU64::new(0),
        });
        let (senders, receivers): (Vec<_>, Vec<_>) = (0..threads).map(|_| mpsc::channel::<Job>()).unzip();
        // Settled once, before any worker exists — see `usable_worker_stack`.
        let stack_bytes = usable_worker_stack();
        let mut handles = Vec::new();
        for (wi, rx) in receivers.into_iter().enumerate() {
            let module = module.clone();
            let shared_w = shared.clone();
            let senders_w = senders.clone();
            let trace_w = trace.clone();
            let debug_w = debug_rcaps.clone();
            handles.push(spawn_worker(
                wi,
                rx,
                WorkerSetup {
                    module,
                    shared: shared_w,
                    senders: senders_w,
                    trace: trace_w,
                    debug_rcaps: debug_w,
                    max_depth: crate::interp::max_depth_for_stack(stack_bytes),
                },
                stack_bytes,
            ));
        }
        ActorSystem {
            shared,
            senders,
            handles,
            worker_stack: stack_bytes,
            worker_max_depth: crate::interp::max_depth_for_stack(stack_bytes),
        }
    }

    /// The depth bound the workers are actually running with, **when it is lower than the
    /// language's documented one** — otherwise `None`.
    ///
    /// **A degraded capability has to name itself.** `ref.rule.portability.isolation-labels-are-
    /// honest` already settled this shape for isolation profiles: a profile that is unavailable is
    /// refused or *reported* as a weaker fallback, never silently swapped. The same reasoning
    /// applies here, and it applies to a hole this repair itself opened. Before D67 a worker on a
    /// constrained host crashed; after it, the worker quietly enforces a smaller bound. Quietly is
    /// the part that is wrong: without this, an author whose program recurses 900 deep would see it
    /// work on one machine and report `DL0905` on another, with nothing anywhere explaining why.
    ///
    /// Returns `(stack_bytes, max_depth)` so the caller can state both numbers rather than assert a
    /// conclusion. Reached only when the full reservation was refused — a strict `RLIMIT_STACK`, a
    /// container with an address-space cap, a 32-bit host.
    pub fn reduced_depth_bound(&self) -> Option<(usize, u32)> {
        (self.worker_max_depth < crate::interp::DEFAULT_MAX_DEPTH)
            .then_some((self.worker_stack, self.worker_max_depth))
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
            cycle_runs: self.shared.cycle_runs.load(Ordering::SeqCst),
            cycle_collected: self.shared.cycle_collected.load(Ordering::SeqCst),
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

/// Start one actor worker on a stack big enough for the interpreter's own depth bound to be the
/// limit that fires.
///
/// **An actor behavior runs the same tree-walking interpreter `fn main` does, so it needs the same
/// stack — and it was getting the OS default.** `delulu`'s `main.rs` has reserved
/// [`INTERPRETER_STACK_BYTES`] since D15 precisely so that deep recursion is `DL0905` and never a
/// host abort; that reservation belongs to the `delulu-main` thread and a worker inherits none of
/// it. Measured before this function existed: `down(1000)` printed `1000` from `main` and killed the
/// process from inside a behavior, at depth **43** in debug and **~350** in release against a
/// documented bound of 10,000 (`STAGE10_BUILD_ORDER.md` D67).
///
/// **The (stack, bound) pair is the invariant.** If the reservation cannot be met — a constrained
/// container, a low `RLIMIT_STACK`, a 32-bit host — the worker takes the largest stack it *can* get
/// and [`max_depth_for_stack`] lowers its bound to match, so the guard still fires first. Degrading
/// the bound is a diagnostic; degrading the stack alone would be the crash. The ladder is descending
/// and finite, and its last rung is the OS default with a bound sized for it, so this always
/// returns a worker.
fn spawn_worker(
    wi: usize,
    rx: mpsc::Receiver<Job>,
    setup: WorkerSetup,
    stack_bytes: usize,
) -> std::thread::JoinHandle<()> {
    std::thread::Builder::new()
        .name(format!("delulu-actor-{wi}"))
        .stack_size(stack_bytes)
        .spawn(move || worker_loop(wi, rx, setup))
        .expect("spawn actor worker")
}

/// Everything a worker needs beyond its own mailbox, cloned per worker at spawn.
///
/// These five values always travelled together and were passed one by one; `max_depth` was the sixth
/// and pushed the parameter list past the point where a reader can keep it straight. Bundling them
/// also puts `max_depth` where it belongs — **beside the stack it was computed from**, so the pair
/// that D67 made an invariant is visible in one place instead of at two call sites.
struct WorkerSetup {
    module: Module,
    shared: Arc<Shared>,
    senders: Vec<mpsc::Sender<Job>>,
    trace: Option<Arc<Mutex<Vec<TraceRecord>>>>,
    debug_rcaps: Option<Arc<std::collections::HashSet<delulu_syntax::ast::NodeId>>>,
    /// The depth bound that fits this worker's stack. Never set independently of the reservation —
    /// see [`spawn_worker`].
    max_depth: u32,
}

/// The stack every worker in this system will take, decided **once** by probing.
///
/// A worker's receiver is moved into its closure, so a failed `spawn` cannot be retried with the
/// same job — the size has to be settled before any real work is attached to it. Probing with an
/// empty closure costs one thread create/join per system and answers the only question that
/// matters: what will the OS actually give us. The ladder descends so a constrained host (a
/// container with a low address-space limit, a small `RLIMIT_STACK`) still gets a worker, and
/// [`max_depth_for_stack`] then lowers that worker's bound to match — a smaller bound is a
/// diagnostic, a mismatched pair is the crash.
fn usable_worker_stack() -> usize {
    const LADDER: [usize; 3] =
        [crate::interp::INTERPRETER_STACK_BYTES, 64 * 1024 * 1024, 8 * 1024 * 1024];
    for bytes in LADDER {
        if let Ok(h) = std::thread::Builder::new()
            .name("delulu-actor-probe".into())
            .stack_size(bytes)
            .spawn(|| {})
        {
            let _ = h.join();
            return bytes;
        }
    }
    // Rust's default. Reached only when even 8 MiB is refused, and the bound shrinks with it.
    2 * 1024 * 1024
}

fn worker_loop(wi: usize, rx: mpsc::Receiver<Job>, setup: WorkerSetup) {
    let WorkerSetup { module, shared, senders, trace, debug_rcaps, max_depth } = setup;
    let mut interp = crate::interp::Interp::new(&module)
        .with_max_depth(max_depth)
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
                maybe_collect_cycles(&interp, &cells, &shared);
                shared.done();
            }
        }
    }
}

/// Stage 10 (10d): the between-turns sweep. Roots are EVERY live actor state this worker owns —
/// a worker hosts many actors, and a cell allocated by one turn may lawfully live in another
/// actor's state... no: states never share cells across actors (messages cross as owned
/// `MsgValue`), but the registry is worker-wide, so the root set must be worker-wide too.
/// Runs only under registry pressure (the amortizer), and the counts feed `--trace-memory`.
fn maybe_collect_cycles(interp: &crate::interp::Interp, cells: &HashMap<u64, Cell>, shared: &Shared) {
    if interp.cycle_pressure() < crate::cycles::COLLECT_THRESHOLD {
        return;
    }
    let roots: Vec<Value> = cells.values().filter(|c| !c.dead).map(|c| c.state.clone()).collect();
    let broken = interp.collect_cycles(&roots);
    shared.cycle_runs.fetch_add(1, Ordering::SeqCst);
    shared.cycle_collected.fetch_add(broken, Ordering::SeqCst);
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
        // A `Map` is sendable exactly when its values are, which is the `List` rule; the keys are
        // `Str`/`Int`/`Bool` by construction, so they always are. The same debug-lane aliasing check
        // applies for the same reason.
        Value::Map(m) => {
            debug_assert!(
                Rc::strong_count(m) <= 2,
                "actor-boundary map with {} strong refs — the static uniqueness proof failed",
                Rc::strong_count(m)
            );
            MsgValue::Map(
                m.borrow()
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), value_to_msg(v, self_state)?)))
                    .collect::<Result<_, Fault>>()?,
            )
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
            net_special: r.net_special.clone(),
            clock: r.clock,
            rand: r.rand,
            declassify: r.declassify,
            secrets: r.secrets.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            foreign_load: r.foreign_load,
            python_allowlist: r.python_allowlist.clone(),
            broker_secrets: r.broker_secrets.clone(),
            actuators: r.actuators.clone(),
            sensors: r.sensors.clone(),
            plugins: r.plugins.clone(),
            plugins_allow: r.plugins_allow.clone(),
            computes: r.computes.clone(),
        }),
        Value::ActorRef { id, actor } => MsgValue::Actor { id: *id, actor: actor.to_string() },
        // P2: a plugin handle does NOT cross an actor boundary. The grant that loaded it does
        // (see `MsgRoot::plugins`), so an actor may load its OWN plugin and get its own custody
        // node — but a loaded handle carries an `Rc` to a replayed module and a `GrantId` whose
        // liveness is re-checked per call against the custody this side holds. Sending it would
        // hand a second thread a reference whose revocation it cannot see, which is precisely the
        // authority-swap R-6c exists to prevent. The checker's `Type::Plugin` is opaque and not
        // sendable, so a well-typed program never reaches here.
        Value::Plugin(_) | Value::PluginFn(_) => {
            return Err(Fault::new(
                "DL0907",
                "a plugin handle reached an actor boundary (checker bug if ever seen in a checked \
                 program) — load a plugin inside the actor that uses it",
            ))
        }
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
        MsgValue::Map(pairs) => Value::Map(Rc::new(std::cell::RefCell::new(
            pairs.into_iter().map(|(k, v)| (k, msg_to_value(v, globals))).collect(),
        ))),
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
            fields: crate::value::VariantFields::new(fields.into_iter().map(|x| msg_to_value(x, globals)).collect()),
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
                // Carried, like the actuator list beside it (D46): the envelope is what bounds the
                // holder, and the broker still re-checks every dispatch against the grant.
                computes: r.computes,
                console: r.console,
                fs_read: r.fs_read,
                fs_write: r.fs_write,
                net: r.net,
                net_special: r.net_special,
                clock: r.clock,
                rand: r.rand,
                declassify: r.declassify,
                secrets: r.secrets.into_iter().collect(),
                foreign_load: r.foreign_load,
                python_allowlist: r.python_allowlist,
                broker_secrets: r.broker_secrets,
                actuators: r.actuators,
                sensors: r.sensors,
                plugins: r.plugins,
                plugins_allow: r.plugins_allow,
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

#[cfg(test)]
mod boundary_authority_tests {
    /// Every authority dimension a `Root` carries must either CROSS an actor boundary or be listed
    /// here as deliberately withheld. Nothing may be absent by accident.
    ///
    /// This exists because it already happened (`HARDENING_CAMPAIGN.md` C35). `RootMsg` is a
    /// hand-written enumeration of `RootVal`'s dimensions, and Stage 10 phase 10h added `computes`
    /// without extending it — while phase 10e's `actuators` and `sensors`, identical in shape and one
    /// phase earlier, do cross. An actor holding a Root slice therefore loses compute authority
    /// silently. That direction is fail-closed, so nothing is unsafe; what is wrong is that it was an
    /// omission rather than a decision, and no test could tell the difference.
    ///
    /// So this test reads both struct definitions out of the source and compares them. It is an
    /// unusual shape for a unit test, and it is the right one: Rust has no reflection, the two lists
    /// are written by hand in different files, and the failure mode is silence. `delulu-conform`
    /// already scans compiler source for the same reason.
    ///
    /// **If this test fails, do not "fix" it by adding the field here.** Decide whether the dimension
    /// should cross an actor boundary, implement that decision, and only then update this list.
    /// Empty, and that is the answer rather than an oversight (ruling D46, closing C35): **every**
    /// `RootVal` authority dimension now crosses an actor boundary. `computes` was the last holdout
    /// and it is carried, because the envelope is what bounds its holder and actuators — which move
    /// physical machines — already crossed.
    ///
    /// The gate below is the part that must not be removed just because this list is empty: it still
    /// fails if a NEW dimension is added to `RootVal` and not carried, and it still fails if a name
    /// listed here has since started crossing. An empty list means "nothing is withheld", which is a
    /// claim that has to keep being true.
    const WITHHELD_FROM_ACTORS: &[&str] = &[];

    /// Thread-creation sites that deliberately do **not** reserve an interpreter-sized stack,
    /// each with the reason it is safe.
    ///
    /// **A thread that runs a DeluluLang program must reserve a stack sized for the interpreter's
    /// depth bound, or lower the bound to match** — otherwise the guard cannot fire and the process
    /// dies of a native stack overflow instead of reporting `DL0905`. That rule lived at one site
    /// (`delulu`'s `main.rs`) and the actor scheduler, which runs the very same interpreter, never
    /// learned it (ruling D67). This is the seventh instance of the campaign's first design rule:
    /// *a hand-maintained list of authority- or safety-bearing things falls behind the type that
    /// defines it.* So the list is now checked instead of remembered.
    ///
    /// Keyed by file, because a file's *purpose* is what makes its threads safe. Every entry here
    /// spawns a thread that never enters the interpreter: it sleeps, reads a pipe, or serves a
    /// socket.
    const THREADS_THAT_NEVER_RUN_INTERPRETER_CODE: &[(&str, &str)] = &[
        ("adapter.rs", "reads the adapter subprocess's stdout so an exchange can have a deadline"),
        ("device.rs", "the dead-man heartbeat sweeper: sleeps and checks lease state"),
        ("egress.rs", "the resolver thread: one getaddrinfo call, abandoned at the request's deadline"),
        ("budget.rs", "the budget watchdog: samples the process's memory and processor time"),
        ("run_cmd.rs", "flushes stdout, bounded, while a budget-stopped run exits"),
        ("limits.rs", "the WASM wall-clock watchdog: sleeps, then advances the engine epoch"),
        ("lib.rs", "the registry's HTTP listener: serves connections, never evaluates a program"),
    ];

    /// Source files to sweep for thread creation. The interpreter is reachable from the runtime and
    /// from the CLI; nothing else in the workspace constructs an [`crate::interp::Interp`].
    fn thread_creating_sources() -> Vec<(String, String)> {
        let runtime = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
        let cli = concat!(env!("CARGO_MANIFEST_DIR"), "/../delulu/src");
        let wasm = concat!(env!("CARGO_MANIFEST_DIR"), "/../delulu-wasm/src");
        let registry = concat!(env!("CARGO_MANIFEST_DIR"), "/../delulu-registry/src");
        let mut out = Vec::new();
        for dir in [runtime, cli, wasm, registry] {
            let Ok(entries) = std::fs::read_dir(dir) else { continue };
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "rs") {
                    let name = p.file_name().unwrap().to_string_lossy().into_owned();
                    if let Ok(text) = std::fs::read_to_string(&p) {
                        // Cut at the test module. Test threads run test code, not user programs —
                        // and this scanner lives in one, so without the cut it matches the very
                        // string literals it searches for. (The Survey shipped that same bug once:
                        // its first version walked its own output directory.)
                        let shipped = match text.find("#[cfg(test)]") {
                            Some(i) => text[..i].to_string(),
                            None => text,
                        };
                        out.push((name, shipped));
                    }
                }
            }
        }
        assert!(out.len() > 20, "the source sweep found too few files to be right: {}", out.len());
        out
    }

    #[test]
    fn every_thread_either_sizes_its_stack_or_is_listed_as_never_running_a_program() {
        let mut unclassified: Vec<String> = Vec::new();
        for (name, text) in thread_creating_sources() {
            // Each creation site, with a window big enough to hold the builder chain that follows.
            for (idx, _) in text.match_indices("thread::spawn(").chain(text.match_indices("thread::Builder::new()")) {
                let window = &text[idx..text.len().min(idx + 400)];
                if window.contains(".stack_size(") {
                    continue; // sized — the rule is satisfied at this site
                }
                if THREADS_THAT_NEVER_RUN_INTERPRETER_CODE.iter().any(|(f, _)| *f == name) {
                    continue; // classified, with a reason, above
                }
                let line = text[..idx].lines().count();
                unclassified.push(format!("{name}:{line}"));
            }
        }
        assert!(
            unclassified.is_empty(),
            "these threads neither reserve an interpreter-sized stack nor are listed as never \
             running one: {unclassified:?}\n\
             If the thread can evaluate a DeluluLang program, give it `.stack_size(...)` AND pass \
             `max_depth_for_stack(...)` to its `Interp` — the pair is the invariant, and a big \
             stack alone only moves the crash deeper. If it cannot, add it to \
             THREADS_THAT_NEVER_RUN_INTERPRETER_CODE with the reason. Leaving it unclassified is \
             exactly how the actor scheduler shipped with the OS default stack (D67)."
        );

        // And the list must not rot the other way: a file that has since started sizing its stacks,
        // or lost its threads entirely, should not keep an exemption it no longer needs.
        let sources = thread_creating_sources();
        let stale: Vec<&str> = THREADS_THAT_NEVER_RUN_INTERPRETER_CODE
            .iter()
            .filter(|(f, _)| {
                !sources.iter().any(|(name, text)| {
                    name == f
                        && (text.contains("thread::spawn(") || text.contains("thread::Builder::new()"))
                })
            })
            .map(|(f, _)| *f)
            .collect();
        assert!(stale.is_empty(), "these are exempted but no longer create threads: {stale:?}");
    }

    /// A degraded bound must be *reportable*, and an undegraded one must not cry wolf. The second
    /// half is the one worth asserting: a warning that fires on every ordinary run teaches readers
    /// to ignore it, which is how the real one gets missed.
    #[test]
    fn an_ordinary_host_reports_no_reduced_bound() {
        let checked = delulu_check::check_source(0, "module m\n\nfn main(root: Root) {\n}\n");
        assert!(!checked.has_errors(), "the fixture must check: {:?}", checked.diagnostics);
        let system = super::ActorSystem::start(&checked.module, 1, false);
        let reduced = system.reduced_depth_bound();
        let _ = system.finish();
        assert!(
            reduced.is_none(),
            "a host that granted the full reservation must report nothing, got {reduced:?}"
        );
    }

    /// The (stack, bound) pair is the invariant, so the arithmetic that ties them together gets its
    /// own witness — including the degenerate ends, which is where a divide-and-hope would fail.
    #[test]
    fn a_bound_always_fits_the_stack_it_was_sized_for() {
        use crate::interp::{max_depth_for_stack, DEFAULT_MAX_DEPTH, STACK_BYTES_PER_DEPTH};
        // The contract's own reservation earns the full default bound.
        assert_eq!(max_depth_for_stack(crate::interp::INTERPRETER_STACK_BYTES), DEFAULT_MAX_DEPTH);
        // It is capped there: a larger stack does not raise the language's documented bound.
        assert_eq!(max_depth_for_stack(usize::MAX), DEFAULT_MAX_DEPTH);
        // A small stack yields a small bound, and the bound genuinely fits.
        for bytes in [2 * 1024 * 1024, 8 * 1024 * 1024, 64 * 1024 * 1024] {
            let d = max_depth_for_stack(bytes) as usize;
            assert!(d >= 1, "a bound of zero would refuse every call, not bound one");
            assert!(
                d * STACK_BYTES_PER_DEPTH <= bytes,
                "the bound must fit the stack it was computed for: {d} x {STACK_BYTES_PER_DEPTH} > {bytes}"
            );
        }
        // Degenerate: less stack than the fixed reserve still yields a usable, honest bound.
        assert_eq!(max_depth_for_stack(0), 1);
    }

    fn declared_fields(source: &str, struct_name: &str) -> Vec<String> {
        let start = source
            .find(&format!("pub struct {struct_name} {{"))
            .unwrap_or_else(|| panic!("`{struct_name}` must exist in the source"));
        let end = source[start..].find("\n}").expect("the struct must terminate") + start;
        source[start..end]
            .lines()
            .filter_map(|l| {
                let l = l.trim();
                let rest = l.strip_prefix("pub ")?;
                let name = rest.split(':').next()?.trim();
                (!name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
                    .then(|| name.to_string())
            })
            .collect()
    }

    #[test]
    fn every_root_dimension_either_crosses_an_actor_boundary_or_is_listed_as_withheld() {
        let root = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/value.rs"))
            .expect("value.rs is committed");
        let actors = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/actors.rs"))
            .expect("actors.rs is committed");
        let root_dims = declared_fields(&root, "RootVal");
        let msg_dims = declared_fields(&actors, "RootMsg");
        assert!(root_dims.len() > 10, "the RootVal scan found too few fields to be right: {root_dims:?}");

        let missing: Vec<&String> = root_dims
            .iter()
            .filter(|d| !msg_dims.contains(d) && !WITHHELD_FROM_ACTORS.contains(&d.as_str()))
            .collect();
        assert!(
            missing.is_empty(),
            "these Root authority dimensions silently do not cross an actor boundary: {missing:?}\n\
             Decide whether each SHOULD cross. If yes, carry it in `RootMsg` and in the conversion in \
             `value_to_msg`. If no, add it to WITHHELD_FROM_ACTORS with the reason. Do not leave it \
             absent by accident — that is exactly campaign finding C35."
        );

        // The list must not rot in the other direction either: a dimension that has since started
        // crossing should be removed from the withheld list rather than left as a stale claim.
        let stale: Vec<&&str> = WITHHELD_FROM_ACTORS
            .iter()
            .filter(|w| msg_dims.iter().any(|m| m == *w))
            .collect();
        assert!(stale.is_empty(), "these are listed as withheld but now cross: {stale:?}");
    }

    /// The behavioural half of D46b: `computes` actually survives the round trip, rather than merely
    /// being declared on both structs. A field can be present in `RootMsg` and still be dropped by a
    /// conversion — the source scan above cannot see that, so this drives the real code.
    #[test]
    fn a_compute_envelope_survives_the_crossing_into_an_actor_and_back() {
        use super::{msg_to_value, value_to_msg};
        use crate::value::{ComputeEnvelope, RootVal, Value};
        let env = ComputeEnvelope {
            device: "gpu0".to_string(),
            class: "gpu".to_string(),
            adapter: "cpu-reference".to_string(),
            memory_bytes: 1 << 20,
            queue_depth: 4,
            kernel_ms: (0.0, 50.0),
            power_w: (0.0, 12.5),
            formats: vec!["spirv".to_string()],
            kernels: vec![("k".to_string(), "k.bin".to_string())],
            waived: false,
            attested: true,
        };
        let root = RootVal { computes: vec![env.clone()], ..Default::default() };
        let crossed = value_to_msg(&Value::Root(std::rc::Rc::new(root)), None)
            .expect("a Root is sendable by decision (invariant 36)");
        let back = msg_to_value(crossed, &crate::value::Scope::root());
        let Value::Root(r) = back else { panic!("a Root must come back as a Root") };
        assert_eq!(r.computes.len(), 1, "the compute envelope must cross the boundary (C35/D46b)");
        assert_eq!(r.computes[0].device, "gpu0");
        assert_eq!(r.computes[0].memory_bytes, 1 << 20, "and arrive with its BOUNDS intact — the");
        assert_eq!(r.computes[0].kernel_ms, (0.0, 50.0), "envelope is what limits the holder");
        assert_eq!(r.computes[0].kernels.len(), 1, "including the enumerated kernel list");
    }
}
