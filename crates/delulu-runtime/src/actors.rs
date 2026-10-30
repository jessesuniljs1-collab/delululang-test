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

use std::collections::{HashMap, HashSet};
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
    List(MsgParts<MsgValue>),
    /// A `Map` crossing the boundary (P3). Carried as an ORDERED vector of pairs rather than a map,
    /// because the wire form should not depend on the receiver rebuilding the same ordering — it
    /// arrives in the order the sender iterated, which is ascending by key.
    Map(MsgParts<(crate::value::MapKey, MsgValue)>),
    Record {
        name: String,
        fields: MsgParts<(String, MsgValue)>,
    },
    Variant {
        name: String,
        fields: MsgParts<MsgValue>,
    },
    /// A sendable (`val`-inferred) closure: body AST plus the converted capture scopes,
    /// outermost first (the module-global scope is NOT carried — the receiver reattaches
    /// its own identical globals).
    Closure {
        params: Vec<String>,
        body: Block,
        scopes: MsgParts<Vec<(String, MsgValue)>>,
    },
    Cap {
        kind: ResourceKind,
        scope: CapScope,
    },
    /// A local secret's bytes and, P9-04 (c), its grant name and audience — owned, so the message stays `Send`.
    SecretLocal(String, Option<(String, Vec<crate::egress::Audience>)>),
    /// A broker-held secret's name and, P9-04 (d), its audience — a handle sent to another actor goes where it may go.
    SecretHandle(String, Option<(String, Vec<crate::egress::Audience>)>),
    /// Boxed: a root slice is many times the size of every other message value, and every part of a message would pay
    /// for it.
    Root(Box<RootMsg>),
    Actor {
        id: ActorId,
        actor: String,
    },
    /// A `List`, `Map` or `Record` cell this message already carries: the `n`th met, counting each cell where it is first
    /// met, before what it holds (MSG-CYCLE-1). A moved graph that holds itself, or holds one cell twice, arrives as the
    /// same graph — not as a walk that never ends, and not as copies.
    Back(u32),
}

/// The parts a message value holds — a list's items, a map's pairs, a record's fields, a variant's payload, a closure's
/// captured scopes — dismantled without recursion when a message is dropped unread (DATA-DEPTH-2), as `VariantFields`
/// dismantles a `Value`: the derived drop of a chain of 300,000 links recursed once per link and overflowed the stack.
/// The wrapper, not `MsgValue`, holds the `Drop`, so a message can still be taken apart by value.
pub struct MsgParts<T: HoldsMsgs>(pub Vec<T>);

/// A part of a message value, and the message values it holds.
pub trait HoldsMsgs {
    fn into_msgs(self, out: &mut Vec<MsgValue>);
}

impl HoldsMsgs for MsgValue {
    fn into_msgs(self, out: &mut Vec<MsgValue>) {
        out.push(self);
    }
}

impl<K> HoldsMsgs for (K, MsgValue) {
    fn into_msgs(self, out: &mut Vec<MsgValue>) {
        out.push(self.1);
    }
}

impl HoldsMsgs for Vec<(String, MsgValue)> {
    fn into_msgs(self, out: &mut Vec<MsgValue>) {
        out.extend(self.into_iter().map(|(_, m)| m));
    }
}

impl<T: HoldsMsgs> MsgParts<T> {
    /// The parts, leaving none behind.
    pub fn take(&mut self) -> Vec<T> {
        std::mem::take(&mut self.0)
    }
}

impl<T: HoldsMsgs> Drop for MsgParts<T> {
    fn drop(&mut self) {
        let mut work: Vec<MsgValue> = Vec::new();
        for part in self.take() {
            part.into_msgs(&mut work);
        }
        // Depth becomes breadth: each value's parts are moved onto the worklist before it drops, so its own drop finds
        // none and this recursion is one level deep.
        while let Some(m) = work.pop() {
            match m {
                MsgValue::List(mut p) | MsgValue::Variant { fields: mut p, .. } => work.append(&mut p.0),
                MsgValue::Map(mut p) => p.take().into_iter().for_each(|x| x.into_msgs(&mut work)),
                MsgValue::Record { fields: mut p, .. } => p.take().into_iter().for_each(|x| x.into_msgs(&mut work)),
                MsgValue::Closure { scopes: mut p, .. } => p.take().into_iter().for_each(|x| x.into_msgs(&mut work)),
                _ => {}
            }
        }
    }
}

impl<T: HoldsMsgs + std::fmt::Debug> std::fmt::Debug for MsgParts<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
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
    /// P9-04 (c): each secret's audience, carried with it — a root that crossed and lost it would send its
    /// secrets nowhere (closed), but a rule that holds on one side of a boundary is a rule nobody can state.
    pub secret_audiences: Vec<(String, Vec<crate::egress::Audience>)>,
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
    /// RW 6.15: the lease the root's authority came from, carried so an actor's refusal names the same remedy `main`'s
    /// would. Not authority.
    pub lease: Option<String>,
}

enum Job {
    /// `key`: the spawner's [`crate::prim::next_spawn_key`], which the actor's own `Cap[Rand]` stream starts from.
    Create {
        id: u64,
        actor: String,
        args: Vec<MsgValue>,
        cause: SendCause,
        key: u64,
    },
    Send {
        id: u64,
        behavior: String,
        args: Vec<MsgValue>,
        cause: SendCause,
    },
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
    pub fn spawn(&self, actor: &str, args: Vec<MsgValue>, cause: SendCause, decl_bound: Option<u64>) -> ActorId {
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
        // Drawn on the spawner's thread, from its own stream (D-V2-192).
        let key = crate::prim::next_spawn_key();
        let _ = self.senders[worker].send(Job::Create { id, actor: actor.to_string(), args, cause, key });
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
                    if (st.depth.load(Ordering::SeqCst).max(0) as usize) < st.bound || self.shared.died.load(Ordering::SeqCst) {
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
        let _ = self.senders[to.worker].send(Job::Send { id: to.id, behavior: behavior.to_string(), args, cause });
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

/// What a worker needs to perform an actor's effects under the RUN's authority — the custody and the
/// brokers `main`'s interpreter holds (ACTOR-CUSTODY-1, RW 4.47).
///
/// **An actor's effects are the run's effects.** Until routine run 16 a worker built its interpreter
/// with none of these, so under `--broker daemon` an actor's uses never asked the daemon (a revoke,
/// the e-stop, did not reach them) and were never recorded; and in every mode its actuator commands
/// were envelope-checked and answered `Ok` by nobody — no lease, no rate bound, no device. A capability
/// handed to an actor is the same authority it was in `main`, so it answers to the same custody.
///
/// `Default` is what [`ActorSystem::start`] passes: embedded pass-through custody and no brokers,
/// which is exactly `main`'s own interpreter when nothing is attached to it.
#[derive(Clone, Default)]
pub struct WorkerEffects {
    /// Builds each worker's custody ON the worker's thread. A custody client caches its node's epoch
    /// and is not shared between threads, so each worker holds its own client for the same node.
    pub custody: Option<Arc<dyn Fn() -> Box<dyn crate::custody::Custody> + Send + Sync>>,
    /// The run's device broker — shared, as it is between `main` and the host channel: one set of
    /// leases, one rate bound and one dead-man per device, whoever commands it.
    pub devices: Option<Arc<crate::device::DeviceBroker>>,
    /// The run's compute broker — without it an actor's dispatch answers `NoAdapter` beside a bound
    /// adapter.
    pub computes: Option<Arc<crate::compute::ComputeBroker>>,
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
        ActorSystem::start_governed(
            module,
            threads,
            abort_on_death,
            trace,
            debug_rcaps,
            default_mailbox,
            overflow_drop_new,
            WorkerEffects::default(),
        )
    }

    /// [`ActorSystem::start_with`] whose workers perform their effects under the run's custody and
    /// brokers ([`WorkerEffects`]) — what `delulu run` starts (ACTOR-CUSTODY-1).
    #[allow(clippy::too_many_arguments)]
    pub fn start_governed(
        module: &Module,
        threads: usize,
        abort_on_death: bool,
        trace: Option<Arc<Mutex<Vec<TraceRecord>>>>,
        debug_rcaps: Option<Arc<std::collections::HashSet<delulu_syntax::ast::NodeId>>>,
        default_mailbox: Option<usize>,
        overflow_drop_new: bool,
        effects: WorkerEffects,
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
        let fixed_clock = crate::prim::fixed_clock_ms();
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
                    effects: effects.clone(),
                    fixed_clock,
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
        (self.worker_max_depth < crate::interp::DEFAULT_MAX_DEPTH).then_some((self.worker_stack, self.worker_max_depth))
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
                .wait_while(guard, |_| self.shared.pending.load(Ordering::SeqCst) > 0 && !self.shared.died.load(Ordering::SeqCst))
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
    /// The actor's own `Cap[Rand]` stream, installed for each of its turns (ACTOR-DETERMINISM-1, D-V2-192).
    rand: crate::prim::RandStream,
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
fn spawn_worker(wi: usize, rx: mpsc::Receiver<Job>, setup: WorkerSetup, stack_bytes: usize) -> std::thread::JoinHandle<()> {
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
    /// The run's custody and brokers, attached to this worker's interpreter (ACTOR-CUSTODY-1).
    effects: WorkerEffects,
    /// The run's `--clock fixed:` — a thread-local the main thread set, which no worker saw (ACTOR-DETERMINISM-1).
    fixed_clock: Option<i64>,
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
    const LADDER: [usize; 3] = [crate::interp::INTERPRETER_STACK_BYTES, 64 * 1024 * 1024, 8 * 1024 * 1024];
    for bytes in LADDER {
        if let Ok(h) = std::thread::Builder::new().name("delulu-actor-probe".into()).stack_size(bytes).spawn(|| {}) {
            let _ = h.join();
            return bytes;
        }
    }
    // Rust's default. Reached only when even 8 MiB is refused, and the bound shrinks with it.
    2 * 1024 * 1024
}

fn worker_loop(wi: usize, rx: mpsc::Receiver<Job>, setup: WorkerSetup) {
    let WorkerSetup { module, shared, senders, trace, debug_rcaps, max_depth, effects, fixed_clock } = setup;
    crate::prim::set_fixed_clock_ms(fixed_clock);
    let mut interp = crate::interp::Interp::new(&module).with_max_depth(max_depth).with_actors(ActorHost {
        shared: shared.clone(),
        senders,
        me: Some(wi),
    });
    // ACTOR-CUSTODY-1: an actor's uses go through the run's custody and its devices through the run's
    // brokers, exactly as `main`'s do — never a pass-through that answers for a device nobody holds.
    if let Some(make) = &effects.custody {
        interp = interp.with_custody(make());
    }
    if let Some(devices) = &effects.devices {
        interp = interp.with_devices(devices.clone());
    }
    if let Some(computes) = &effects.computes {
        interp = interp.with_computes(computes.clone());
    }
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
            let c =
                Cause { sender: cause.sender.clone(), sender_member: cause.sender_member.clone(), send_span: cause.send_span };
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
            Job::Create { id, actor, args, cause, key } => {
                let Some(decl) = interp.actor_decl(&actor).cloned() else {
                    shared.dead_sends.fetch_add(1, Ordering::SeqCst);
                    shared.done();
                    continue;
                };
                // Fields pre-seed Unit; the checker's definite-initialization rule makes an
                // uninitialized read unreachable in a checked program.
                let fields: Vec<(String, Value)> = decl.fields.iter().map(|f| (f.name.name.clone(), Value::Unit)).collect();
                let state = Value::Record { name: Rc::from(actor.as_str()), fields: Rc::new(std::cell::RefCell::new(fields)) };
                let self_ref = Value::ActorRef { id: ActorId { worker: wi, id }, actor: Rc::from(actor.as_str()) };
                let params: Vec<String> = decl.ctor.params.iter().map(|p| p.name.name.clone()).collect();
                let argvals: Vec<Value> = args.into_iter().map(|m| interp.msg_to_value(m)).collect();
                let turn_id = shared.total_turns.fetch_add(1, Ordering::SeqCst);
                let outer = crate::prim::swap_rand_stream(crate::prim::RandStream::from_key(key));
                let dead = match interp.run_actor_turn(&decl.ctor.body, "new", &params, argvals, state.clone(), self_ref) {
                    Ok(()) => false,
                    Err(fault) => {
                        actor_died(&shared, &actor, "new", &fault);
                        true
                    }
                };
                let rand = crate::prim::swap_rand_stream(outer);
                ship(turn_id, &cause);
                cells.insert(id, Cell { actor, state, dead, rand });
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
                let (actor_name, state, stream) = {
                    let c = &cells[&id];
                    (c.actor.clone(), c.state.clone(), c.rand)
                };
                let decl = interp.actor_decl(&actor_name).cloned().expect("cell's actor is declared");
                let Some(beh) = decl.behaviors.iter().find(|b| b.name.name == behavior) else {
                    shared.dead_sends.fetch_add(1, Ordering::SeqCst);
                    shared.done();
                    continue;
                };
                let self_ref = Value::ActorRef { id: ActorId { worker: wi, id }, actor: Rc::from(actor_name.as_str()) };
                let params: Vec<String> = beh.params.iter().map(|p| p.name.name.clone()).collect();
                let argvals: Vec<Value> = args.into_iter().map(|m| interp.msg_to_value(m)).collect();
                let turn_id = shared.total_turns.fetch_add(1, Ordering::SeqCst);
                let outer = crate::prim::swap_rand_stream(stream);
                let ran = interp.run_actor_turn(&beh.body, &behavior, &params, argvals, state, self_ref);
                let stream = crate::prim::swap_rand_stream(outer);
                if let Some(c) = cells.get_mut(&id) {
                    c.rand = stream;
                }
                if let Err(fault) = ran {
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
    // TERMINAL-TEXT-1: a fault can quote the program's own strings (an assertion's values); escaped, as a run's fault is
    // (RW 6.40: this line printed them raw, so a behaviour's assertion wrote control sequences onto the terminal).
    eprintln!(
        "actor {actor} died in `{member}`: {} [{}] (its later messages will be dropped and counted)",
        fault.message,
        fault.code
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
    // DATA-DEPTH-2: depth becomes breadth. Converted by recursion, a value nested past the native stack — a chain of
    // 300,000 links built in a loop and sent to an actor — overflowed it and killed the process. The values still to
    // convert wait on a worklist, each converted value on `done` until the cell holding it is finished.
    let mut cells: HashMap<usize, u32> = HashMap::new();
    let mut work: Vec<ToMsg> = Vec::new();
    let mut done: Vec<MsgValue> = Vec::new();
    to_msg_step(v, self_state, &mut cells, &mut work, &mut done)?;
    while let Some(step) = work.pop() {
        match step {
            ToMsg::Part(cell, i) => {
                if i + 1 < cell.len() {
                    work.push(ToMsg::Part(cell.clone(), i + 1));
                }
                cell.with_part(i, |part| to_msg_step(part, self_state, &mut cells, &mut work, &mut done))?;
            }
            ToMsg::List(n) => {
                let items = take_last(&mut done, n);
                done.push(MsgValue::List(MsgParts(items)));
            }
            ToMsg::Map(keys) => {
                let held = take_last(&mut done, keys.len());
                done.push(MsgValue::Map(MsgParts(keys.into_iter().zip(held).collect())));
            }
            ToMsg::Record(name, names) => {
                let held = take_last(&mut done, names.len());
                done.push(MsgValue::Record { name, fields: MsgParts(names.into_iter().zip(held).collect()) });
            }
            ToMsg::Variant(name, n) => {
                let fields = take_last(&mut done, n);
                done.push(MsgValue::Variant { name, fields: MsgParts(fields) });
            }
            ToMsg::Closure(params, body, shapes) => {
                let total = shapes.iter().map(Vec::len).sum();
                let mut held = take_last(&mut done, total).into_iter();
                let scopes = shapes.into_iter().map(|names| names.into_iter().zip(held.by_ref()).collect()).collect();
                done.push(MsgValue::Closure { params, body, scopes: MsgParts(scopes) });
            }
        }
    }
    Ok(done.pop().expect("the value converted is the one left"))
}

/// A step of `value_to_msg`: the `i`th part of a cell to convert, or a cell to finish from the values its parts left on
/// `done`.
enum ToMsg {
    Part(Holder, usize),
    List(usize),
    Map(Vec<crate::value::MapKey>),
    Record(String, Vec<String>),
    Variant(String, usize),
    /// A closure's parameters, body and each captured scope's names, outermost first.
    Closure(Vec<String>, Block, Vec<Vec<String>>),
}

/// A cell whose parts `value_to_msg` is converting. A part is read through the cell that holds it, never cloned onto the
/// worklist, so the debug lane's reference counts see what the recursive walk saw.
#[derive(Clone)]
enum Holder {
    List(Rc<std::cell::RefCell<Vec<Value>>>),
    /// A map, and its keys in order: its `i`th part is the value under the `i`th key.
    Map(Rc<std::cell::RefCell<std::collections::BTreeMap<crate::value::MapKey, Value>>>, Rc<[crate::value::MapKey]>),
    Record(Rc<std::cell::RefCell<Vec<(String, Value)>>>),
    Variant(crate::value::VariantFields),
    /// A closure's captured values, every scope's in order, outermost first.
    Captured(Rc<Vec<Value>>),
}

impl Holder {
    fn len(&self) -> usize {
        match self {
            Holder::List(c) => c.borrow().len(),
            Holder::Map(_, keys) => keys.len(),
            Holder::Record(c) => c.borrow().len(),
            Holder::Variant(f) => f.len(),
            Holder::Captured(v) => v.len(),
        }
    }

    /// The step from this cell to its `i`th part in a path the debug lane reports: `[i]` into a list, `[key]` into a map,
    /// `.field` into a record, `.i` into a variant's payload.
    fn step(&self, i: usize) -> String {
        match self {
            Holder::List(_) => format!("[{i}]"),
            Holder::Map(_, keys) => format!("[{}]", keys[i].to_value().display()),
            Holder::Record(c) => format!(".{}", c.borrow()[i].0),
            Holder::Variant(_) | Holder::Captured(_) => format!(".{i}"),
        }
    }

    /// Run `f` on the `i`th part, where it lies.
    fn with_part<R>(&self, i: usize, f: impl FnOnce(&Value) -> R) -> R {
        match self {
            Holder::List(c) => f(&c.borrow()[i]),
            Holder::Map(c, keys) => f(c.borrow().get(&keys[i]).expect("a map is not changed while it is sent")),
            Holder::Record(c) => f(&c.borrow()[i].1),
            Holder::Variant(fields) => f(&fields[i]),
            Holder::Captured(v) => f(&v[i]),
        }
    }
}

/// Put the step that finishes a cell on `work`, and then the first of its parts, if it has any.
fn open_cell(work: &mut Vec<ToMsg>, finish: ToMsg, parts: Holder) {
    work.push(finish);
    if parts.len() > 0 {
        work.push(ToMsg::Part(parts, 0));
    }
}

/// The last `n` of `done`, in order.
fn take_last<T>(done: &mut Vec<T>, n: usize) -> Vec<T> {
    let at = done.len().checked_sub(n).expect("a cell's parts are converted before it is finished");
    done.split_off(at)
}

/// The address of a mutable cell (`List`, `Map`, `Record`) — what tells one cell from another in a moved graph.
fn cell_addr<T>(cell: &Rc<std::cell::RefCell<T>>) -> usize {
    Rc::as_ptr(cell) as *const () as usize
}

/// Convert `v`: a value holding no other, onto `done`; a value that holds others, the step that finishes it onto `work`
/// under its parts, so they convert first and in order. MSG-CYCLE-1: a cell met again is a reference back to where it was
/// first carried; a cell met first is numbered before what it holds, as `msg_to_value` rebuilds it before what it holds.
fn to_msg_step(
    v: &Value,
    self_state: Option<(&Value, ActorId, &str)>,
    cells: &mut HashMap<usize, u32>,
    work: &mut Vec<ToMsg>,
    done: &mut Vec<MsgValue>,
) -> Result<(), Fault> {
    let first = |addr: usize, cells: &mut HashMap<usize, u32>| -> Option<MsgValue> {
        if let Some(n) = cells.get(&addr) {
            return Some(MsgValue::Back(*n));
        }
        let n = u32::try_from(cells.len()).unwrap_or(u32::MAX);
        cells.insert(addr, n);
        None
    };
    // `self` crossing a boundary is the actor handing out its own ADDRESS (ref <: tag) —
    // the classic reply-to pattern. Detected by heap identity with the current turn's state.
    if let (Value::Record { fields, .. }, Some((Value::Record { fields: sf, .. }, id, name))) = (v, self_state) {
        if Rc::ptr_eq(fields, sf) {
            done.push(MsgValue::Actor { id, actor: name.to_string() });
            return Ok(());
        }
    }
    let leaf = match v {
        Value::Int(i) => MsgValue::Int(*i),
        Value::Float(f) => MsgValue::Float(*f),
        Value::Bool(b) => MsgValue::Bool(*b),
        Value::Str(s) => MsgValue::Str(s.to_string()),
        Value::Unit => MsgValue::Unit,
        Value::List(items) => {
            if let Some(back) = first(cell_addr(items), cells) {
                done.push(back);
                return Ok(());
            }
            // Debug lane (§7.4 / deviation 7): an iso move's graph must be unaliased. At
            // this layer we can only see the spine; `--debug-rcaps` (7i) deepens this.
            debug_assert!(
                Rc::strong_count(items) <= 2, // the binding + this borrow's clone path
                "actor-boundary list with {} strong refs — the static uniqueness proof failed",
                Rc::strong_count(items)
            );
            let n = items.borrow().len();
            open_cell(work, ToMsg::List(n), Holder::List(items.clone()));
            return Ok(());
        }
        // A `Map` is sendable exactly when its values are, which is the `List` rule; the keys are
        // `Str`/`Int`/`Bool` by construction, so they always are. The same debug-lane aliasing check
        // applies for the same reason.
        Value::Map(m) => {
            if let Some(back) = first(cell_addr(m), cells) {
                done.push(back);
                return Ok(());
            }
            debug_assert!(
                Rc::strong_count(m) <= 2,
                "actor-boundary map with {} strong refs — the static uniqueness proof failed",
                Rc::strong_count(m)
            );
            let keys: Vec<crate::value::MapKey> = m.borrow().keys().cloned().collect();
            open_cell(work, ToMsg::Map(keys.clone()), Holder::Map(m.clone(), keys.into()));
            return Ok(());
        }
        Value::Record { name, fields } => {
            if let Some(back) = first(cell_addr(fields), cells) {
                done.push(back);
                return Ok(());
            }
            let names = fields.borrow().iter().map(|(n, _)| n.clone()).collect();
            open_cell(work, ToMsg::Record(name.to_string(), names), Holder::Record(fields.clone()));
            return Ok(());
        }
        Value::Variant { name, fields } => {
            open_cell(work, ToMsg::Variant(name.to_string(), fields.len()), Holder::Variant(fields.clone()));
            return Ok(());
        }
        Value::Closure(c) => {
            let scopes = captured_scopes(c);
            let shapes = scopes.iter().map(|vars| vars.iter().map(|(n, _)| n.clone()).collect()).collect();
            let values: Vec<Value> = scopes.into_iter().flatten().map(|(_, x)| x).collect();
            open_cell(work, ToMsg::Closure(c.params.clone(), c.body.clone(), shapes), Holder::Captured(Rc::new(values)));
            return Ok(());
        }
        Value::Cap(c) => MsgValue::Cap { kind: c.kind, scope: c.scope.clone() },
        Value::Secret(s) => match s.handle_name() {
            Some(h) => MsgValue::SecretHandle(h.to_string(), s.audience().map(|a| (a.name.clone(), a.to.clone()))),
            None => MsgValue::SecretLocal(s.reveal(), s.audience().map(|a| (a.name.clone(), a.to.clone()))),
        },
        Value::Root(r) => MsgValue::Root(Box::new(RootMsg {
            console: r.console,
            fs_read: r.fs_read.clone(),
            fs_write: r.fs_write.clone(),
            net: r.net.clone(),
            net_special: r.net_special.clone(),
            clock: r.clock,
            rand: r.rand,
            declassify: r.declassify,
            secrets: r.secrets.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            secret_audiences: r.secret_audiences.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            foreign_load: r.foreign_load,
            python_allowlist: r.python_allowlist.clone(),
            broker_secrets: r.broker_secrets.clone(),
            actuators: r.actuators.clone(),
            sensors: r.sensors.clone(),
            plugins: r.plugins.clone(),
            plugins_allow: r.plugins_allow.clone(),
            computes: r.computes.clone(),
            lease: r.lease.clone(),
        })),
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
            return Err(Fault::new("DL0907", "a PyObj reached an actor boundary (checker bug — PyObj is statically pinned)"))
        }
    };
    done.push(leaf);
    Ok(())
}

/// What a closure carries across an actor boundary: each name its body reads from where it was made
/// (`delulu_check::rcap_check::closure_captures`, the set the checker judged sendable), with the value the innermost
/// scope below the module's globals binds to it — the receiver reattaches its own globals — as one scope.
///
/// CLOSURE-SEND-1: every binding of every enclosing scope crossed. A closure bound by `let` is one of those bindings, so
/// sending it walked into itself without end — a stack overflow before DATA-DEPTH-2, the memory budget after — and a
/// closure carried values it never reads, an `iso` value held beside it included.
fn captured_scopes(c: &Closure) -> Vec<Vec<(String, Value)>> {
    let mut chain = Vec::new();
    let mut cur = Some(c.env.clone());
    while let Some(s) = cur {
        chain.push(s.clone());
        cur = s.parent().cloned();
    }
    chain.pop(); // the root globals
    let vars = delulu_check::rcap_check::closure_captures(&c.params, &c.body)
        .into_iter()
        .filter_map(|n| chain.iter().find_map(|scope| scope.get_here(&n)).map(|v| (n, v)))
        .collect();
    vec![vars]
}

/// Rebuild a message value into the RECEIVING actor's heap. `globals` is the receiving
/// interpreter's module-global scope (closures reattach to it).
///
/// Each `List`, `Map` and `Record` cell is made empty and recorded in `cells` before what it holds is rebuilt into it, in
/// the order `value_to_msg` numbered them, so a `Back` inside it finds it. As `value_to_msg` does, the walk keeps a
/// worklist (DATA-DEPTH-2): a cell's step that fills it waits under the rebuilding of its parts.
pub fn msg_to_value(m: MsgValue, globals: &Env) -> Value {
    let mut cells: Vec<Value> = Vec::new();
    let mut work: Vec<FromMsg> = vec![FromMsg::Msg(m)];
    let mut done: Vec<Value> = Vec::new();
    while let Some(step) = work.pop() {
        match step {
            FromMsg::Msg(m) => from_msg_step(m, &mut cells, &mut work, &mut done),
            FromMsg::List(cell, n) => {
                let held = take_last(&mut done, n);
                *cell.borrow_mut() = held;
                done.push(Value::List(cell));
            }
            FromMsg::Map(cell, keys) => {
                let held = take_last(&mut done, keys.len());
                *cell.borrow_mut() = keys.into_iter().zip(held).collect();
                done.push(Value::Map(cell));
            }
            FromMsg::Record(cell, name, names) => {
                let held = take_last(&mut done, names.len());
                *cell.borrow_mut() = names.into_iter().zip(held).collect();
                done.push(Value::Record { name, fields: cell });
            }
            FromMsg::Variant(name, n) => {
                let fields = take_last(&mut done, n);
                done.push(Value::Variant { name: Rc::from(name.as_str()), fields: crate::value::VariantFields::new(fields) });
            }
            FromMsg::Closure(params, body, shapes) => {
                let total = shapes.iter().map(Vec::len).sum();
                let mut held = take_last(&mut done, total).into_iter();
                let mut env = globals.clone();
                for names in shapes {
                    let child = Scope::child(&env);
                    for (n, v) in names.into_iter().zip(held.by_ref()) {
                        child.define(&n, v);
                    }
                    env = child;
                }
                done.push(Value::Closure(Rc::new(Closure { params, body, env, home: None })));
            }
        }
    }
    done.pop().expect("the value rebuilt is the one left")
}

/// A step of `msg_to_value`: a message value to rebuild, or a cell to fill from the values its parts left on `done`.
enum FromMsg {
    Msg(MsgValue),
    /// A list's cell and the number of its items.
    List(Rc<std::cell::RefCell<Vec<Value>>>, usize),
    Map(Rc<std::cell::RefCell<std::collections::BTreeMap<crate::value::MapKey, Value>>>, Vec<crate::value::MapKey>),
    Record(Rc<std::cell::RefCell<Vec<(String, Value)>>>, Rc<str>, Vec<String>),
    Variant(String, usize),
    Closure(Vec<String>, Block, Vec<Vec<String>>),
}

/// Rebuild `m`: a value holding no other, onto `done`; one that holds others, its cell made and recorded now, the step
/// that fills it onto `work` under its parts, so they are rebuilt first and in order.
fn from_msg_step(m: MsgValue, cells: &mut Vec<Value>, work: &mut Vec<FromMsg>, done: &mut Vec<Value>) {
    let leaf = match m {
        MsgValue::Int(i) => Value::Int(i),
        MsgValue::Float(f) => Value::Float(f),
        MsgValue::Bool(b) => Value::Bool(b),
        MsgValue::Str(s) => Value::str(s),
        MsgValue::Unit => Value::Unit,
        MsgValue::Map(mut pairs) => {
            let cell = Rc::new(std::cell::RefCell::new(std::collections::BTreeMap::new()));
            cells.push(Value::Map(cell.clone()));
            let (keys, held): (Vec<_>, Vec<_>) = pairs.take().into_iter().unzip();
            work.push(FromMsg::Map(cell, keys));
            work.extend(held.into_iter().rev().map(FromMsg::Msg));
            return;
        }
        MsgValue::List(mut items) => {
            let items = items.take();
            let cell = Rc::new(std::cell::RefCell::new(Vec::with_capacity(items.len())));
            cells.push(Value::List(cell.clone()));
            work.push(FromMsg::List(cell, items.len()));
            work.extend(items.into_iter().rev().map(FromMsg::Msg));
            return;
        }
        MsgValue::Record { name, mut fields } => {
            let cell = Rc::new(std::cell::RefCell::new(Vec::new()));
            let name: Rc<str> = Rc::from(name.as_str());
            cells.push(Value::Record { name: name.clone(), fields: cell.clone() });
            let (names, held): (Vec<_>, Vec<_>) = fields.take().into_iter().unzip();
            work.push(FromMsg::Record(cell, name, names));
            work.extend(held.into_iter().rev().map(FromMsg::Msg));
            return;
        }
        MsgValue::Back(n) => cells.get(n as usize).cloned().expect(
            "a back-reference names a cell rebuilt before it (`value_to_msg` numbers cells in the order they are rebuilt)",
        ),
        MsgValue::Variant { name, mut fields } => {
            let fields = fields.take();
            work.push(FromMsg::Variant(name, fields.len()));
            work.extend(fields.into_iter().rev().map(FromMsg::Msg));
            return;
        }
        MsgValue::Closure { params, body, mut scopes } => {
            let scopes = scopes.take();
            let mut names = Vec::with_capacity(scopes.len());
            let mut held = Vec::new();
            for scope in scopes {
                let (n, h): (Vec<_>, Vec<_>) = scope.into_iter().unzip();
                names.push(n);
                held.extend(h);
            }
            work.push(FromMsg::Closure(params, body, names));
            work.extend(held.into_iter().rev().map(FromMsg::Msg));
            return;
        }
        MsgValue::Cap { kind, scope } => Value::Cap(Rc::new(CapVal { kind, scope })),
        MsgValue::SecretLocal(s, aud) => {
            Value::Secret(Rc::new(SecretVal::bound(s, aud.map(|(name, to)| Rc::new(crate::value::SecretAudience { name, to })))))
        }
        MsgValue::SecretHandle(h, aud) => Value::Secret(Rc::new(SecretVal::handle_bound(
            h,
            aud.map(|(name, to)| Rc::new(crate::value::SecretAudience { name, to })),
        ))),
        MsgValue::Root(r) => {
            let r = *r;
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
                secret_audiences: r.secret_audiences.into_iter().collect(),
                foreign_load: r.foreign_load,
                python_allowlist: r.python_allowlist,
                broker_secrets: r.broker_secrets,
                actuators: r.actuators,
                sensors: r.sensors,
                plugins: r.plugins,
                plugins_allow: r.plugins_allow,
                lease: r.lease,
            };
            Value::Root(Rc::new(root))
        }
        MsgValue::Actor { id, actor } => Value::ActorRef { id, actor: Rc::from(actor.as_str()) },
    };
    done.push(leaf);
}

/// The `--debug-rcaps` uniqueness walk (spec §6.2/§6.4, phase 7i): an iso MOVE's graph must have no alias OUTSIDE it.
/// Every strong reference to a mutable node (`List`, `Map`, `Record`) must come from inside the graph being moved — the
/// nodes and variant payloads it holds — except the top node's two (the binding and the evaluator's clone). One more is a
/// reference from outside: the static uniqueness proof failed, a compiler-bug class report (DL1610). Immutable values
/// (`Str`, scalars, caps) share freely; a variant is walked through, each payload once.
///
/// D-V2-165: this counted every node below the top as needing exactly one reference, so a graph that `recover` lifts
/// legally — one that holds itself (`k.next = Some(k)`) or holds one list twice (`[a, a]`) — was reported as a compiler
/// bug, while an alias reached through a variant or a map was never looked at. It now counts the graph's own references.
pub fn assert_unique_graph(v: &Value) -> Result<(), String> {
    /// Each mutable node met, in the order first met: its kind and the path it was met at, its strong count, and how many
    /// references to it the graph holds.
    struct Node {
        kind: &'static str,
        path: usize,
        strong: usize,
        inside: usize,
    }
    /// Meet `v` at path `at`: a mutable node is counted, or numbered and its parts queued; a variant's payload is walked
    /// through, each payload once. A node's strong count is read before anything of this walk holds a reference to it.
    fn meet(
        v: &Value,
        at: usize,
        nodes: &mut Vec<Node>,
        index: &mut HashMap<usize, usize>,
        payloads: &mut HashSet<usize>,
        work: &mut Vec<(Holder, usize, usize)>,
    ) {
        let (addr, kind, strong, parts) = match v {
            Value::List(rc) => (cell_addr(rc), "list", Rc::strong_count(rc), Holder::List(rc.clone())),
            Value::Map(rc) => {
                let keys: Vec<crate::value::MapKey> = rc.borrow().keys().cloned().collect();
                (cell_addr(rc), "map", Rc::strong_count(rc), Holder::Map(rc.clone(), keys.into()))
            }
            Value::Record { fields, .. } => {
                (cell_addr(fields), "record", Rc::strong_count(fields), Holder::Record(fields.clone()))
            }
            Value::Variant { fields, .. } => {
                if payloads.insert(fields.ptr_id()) && !fields.is_empty() {
                    work.push((Holder::Variant(fields.clone()), 0, at));
                }
                return;
            }
            _ => return,
        };
        if let Some(&i) = index.get(&addr) {
            nodes[i].inside += 1;
            return;
        }
        index.insert(addr, nodes.len());
        nodes.push(Node { kind, path: at, strong, inside: 1 });
        if parts.len() > 0 {
            work.push((parts, 0, at));
        }
    }
    // DATA-DEPTH-2: the walk recursed once per level and built each node's whole path as it went, so a moved graph
    // 100,000 deep spent the memory budget under `--debug-rcaps` where the move itself ran. The parts still to meet wait on
    // a worklist, a part read through the cell holding it; a path is kept as its parent and its last step, and spelled out
    // only for a node reported.
    let mut paths: Vec<(usize, String)> = vec![(usize::MAX, "arg".to_string())];
    let (mut nodes, mut index, mut payloads, mut work) = (Vec::new(), HashMap::new(), HashSet::new(), Vec::new());
    meet(v, 0, &mut nodes, &mut index, &mut payloads, &mut work);
    while let Some((cell, i, at)) = work.pop() {
        if i + 1 < cell.len() {
            work.push((cell.clone(), i + 1, at));
        }
        paths.push((at, cell.step(i)));
        let here = paths.len() - 1;
        cell.with_part(i, |part| meet(part, here, &mut nodes, &mut index, &mut payloads, &mut work));
    }
    let spell = |mut p: usize| {
        let mut steps = Vec::new();
        while p != usize::MAX {
            steps.push(paths[p].1.as_str());
            p = paths[p].0;
        }
        steps.iter().rev().copied().collect::<String>()
    };
    for (i, n) in nodes.iter().enumerate() {
        // The walk's first step into the top node is the one reference from outside it may have, held twice (the binding
        // and the evaluator's clone); every other node's references must all be the graph's own.
        let max = if i == 0 { n.inside + 1 } else { n.inside };
        if n.strong > max {
            return Err(format!("{} at {} has {} strong refs (max {max})", n.kind, spell(n.path), n.strong));
        }
    }
    Ok(())
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
        ("pipe_channel.rs", "the sandbox guest's channel watchdog (sleeps, and ends a guest whose host fell silent) and the host's reader of an external launcher's output (PS-D-01) and its writer of the launcher's input (F5)"),
        ("guest.rs", "the sandbox host's watchdog (SANDBOX-STOP-1): sleeps until `--limits wall=`, reads the guest's processor time (Windows) and peak memory (macOS, D-V2-90), and ends the guest"),
        ("brokerd.rs", "the broker daemon's accept thread and its per-connection readers (RW 4.40): each reads one request frame; custody is handled on the serve loop, which runs no program"),
        ("mcp.rs", "drains a tool subprocess's stdout and stderr, so a large answer cannot stall it"),
        ("broker_client.rs", "a run's beater (RW 4.57, D-V2-157): sleeps, and beats the dead-man on the run's own node — the program runs on the run's own threads"),
        ("monitor_cmd.rs", "the monitor's beater (DEADMAN-BUSY-1): sleeps, and beats the broker while the watch waits on it — the program it watches runs elsewhere"),
        ("microvm.rs", "relays a microVM's console, bounded, and the VM's wall-clock watchdog: neither evaluates a program — the program runs in the guest"),
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
                !sources
                    .iter()
                    .any(|(name, text)| name == f && (text.contains("thread::spawn(") || text.contains("thread::Builder::new()")))
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
        assert!(reduced.is_none(), "a host that granted the full reservation must report nothing, got {reduced:?}");
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
                (!name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')).then(|| name.to_string())
            })
            .collect()
    }

    #[test]
    fn every_root_dimension_either_crosses_an_actor_boundary_or_is_listed_as_withheld() {
        let root = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/value.rs")).expect("value.rs is committed");
        let actors =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/actors.rs")).expect("actors.rs is committed");
        let root_dims = declared_fields(&root, "RootVal");
        let msg_dims = declared_fields(&actors, "RootMsg");
        assert!(root_dims.len() > 10, "the RootVal scan found too few fields to be right: {root_dims:?}");

        let missing: Vec<&String> =
            root_dims.iter().filter(|d| !msg_dims.contains(d) && !WITHHELD_FROM_ACTORS.contains(&d.as_str())).collect();
        assert!(
            missing.is_empty(),
            "these Root authority dimensions silently do not cross an actor boundary: {missing:?}\n\
             Decide whether each SHOULD cross. If yes, carry it in `RootMsg` and in the conversion in \
             `value_to_msg`. If no, add it to WITHHELD_FROM_ACTORS with the reason. Do not leave it \
             absent by accident — that is exactly campaign finding C35."
        );

        // The list must not rot in the other direction either: a dimension that has since started
        // crossing should be removed from the withheld list rather than left as a stale claim.
        let stale: Vec<&&str> = WITHHELD_FROM_ACTORS.iter().filter(|w| msg_dims.iter().any(|m| m == *w)).collect();
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
        let crossed =
            value_to_msg(&Value::Root(std::rc::Rc::new(root)), None).expect("a Root is sendable by decision (invariant 36)");
        let back = msg_to_value(crossed, &crate::value::Scope::root());
        let Value::Root(r) = back else { panic!("a Root must come back as a Root") };
        assert_eq!(r.computes.len(), 1, "the compute envelope must cross the boundary (C35/D46b)");
        assert_eq!(r.computes[0].device, "gpu0");
        assert_eq!(r.computes[0].memory_bytes, 1 << 20, "and arrive with its BOUNDS intact — the");
        assert_eq!(r.computes[0].kernel_ms, (0.0, 50.0), "envelope is what limits the holder");
        assert_eq!(r.computes[0].kernels.len(), 1, "including the enumerated kernel list");
    }

    /// RW 6.15: the lease a root's authority came from crosses with it, so an actor's DL0703 names the delegation as
    /// `main`'s does — not a `--grant` the lease run refuses.
    #[test]
    fn a_roots_lease_survives_the_crossing_into_an_actor_and_back() {
        use super::{msg_to_value, value_to_msg};
        use crate::value::{RootVal, Value};
        let root = RootVal { lease: Some("g_feed".to_string()), ..Default::default() };
        let back = msg_to_value(value_to_msg(&Value::Root(std::rc::Rc::new(root)), None).unwrap(), &crate::value::Scope::root());
        let Value::Root(r) = back else { panic!("a Root must come back as a Root") };
        assert_eq!(r.lease.as_deref(), Some("g_feed"));
    }

    /// P9-04 (c), D-V2-131: a secret's audience crosses an actor boundary with it — on a `Root` (the audience of each
    /// secret it may mint) and on a minted `Secret` (where its bytes may go). A crossing that dropped it would fail
    /// closed (sent nowhere); one that kept the bytes and lost where they may go would be a rule that holds on one
    /// side of a boundary only. Driven through the real conversions, both directions.
    #[test]
    fn a_secrets_audience_survives_the_crossing_into_an_actor_and_back() {
        use super::{msg_to_value, value_to_msg};
        use crate::egress::Audience;
        use crate::value::{RootVal, SecretAudience, SecretVal, Value};
        use std::rc::Rc;
        let to = vec![Audience { host: "api.example.com".to_string(), port: 443 }];
        let mut root = RootVal::default();
        root.secrets.insert("K".to_string(), "v".to_string());
        root.secret_audiences.insert("K".to_string(), to.clone());
        let back = msg_to_value(value_to_msg(&Value::Root(Rc::new(root)), None).unwrap(), &crate::value::Scope::root());
        let Value::Root(r) = back else { panic!("a Root must come back as a Root") };
        assert_eq!(r.secret_audiences.get("K"), Some(&to), "a root's secret audiences cross");
        let secret = SecretVal::bound("v".to_string(), Some(Rc::new(SecretAudience { name: "K".to_string(), to: to.clone() })));
        let back = msg_to_value(value_to_msg(&Value::Secret(Rc::new(secret)), None).unwrap(), &crate::value::Scope::root());
        let Value::Secret(s) = back else { panic!("a Secret must come back as a Secret") };
        let a = s.audience().expect("a minted secret's audience crosses");
        assert_eq!((a.name.as_str(), &a.to), ("K", &to));
        let plain = msg_to_value(
            value_to_msg(&Value::Secret(Rc::new(SecretVal::new("v".to_string()))), None).unwrap(),
            &crate::value::Scope::root(),
        );
        let Value::Secret(s) = plain else { panic!() };
        assert!(s.audience().is_none(), "and none is not invented for a secret granted without one");
        // P9-04 (d), D-V2-134: a broker HANDLE keeps its audience too — and stays a handle, with no bytes.
        let handle = SecretVal::handle_bound("K", Some(Rc::new(SecretAudience { name: "K".to_string(), to: to.clone() })));
        let back = msg_to_value(value_to_msg(&Value::Secret(Rc::new(handle)), None).unwrap(), &crate::value::Scope::root());
        let Value::Secret(s) = back else { panic!() };
        assert_eq!(s.handle_name(), Some("K"));
        assert_eq!(s.audience().map(|a| a.to.clone()), Some(to), "a handle's audience crosses");
    }
}

#[cfg(test)]
mod move_graph_tests {
    use super::*;
    use std::cell::RefCell;

    /// MSG-CYCLE-1 (D-V2-165): a moved graph arrives as the same graph. A list held twice arrives as one list held twice,
    /// not two copies (a push through one would otherwise not be seen through the other, as it is on the sending side), and
    /// a record holding itself arrives holding itself, where the walk used to overflow the stack.
    #[test]
    fn a_moved_graph_keeps_its_shape() {
        let a = Value::List(Rc::new(RefCell::new(vec![Value::Int(1)])));
        let twice = Value::List(Rc::new(RefCell::new(vec![a.clone(), a])));
        let back = msg_to_value(value_to_msg(&twice, None).expect("sendable"), &Scope::root());
        let Value::List(outer) = back else { panic!("a list arrives") };
        let outer = outer.borrow();
        let (Value::List(x), Value::List(y)) = (&outer[0], &outer[1]) else { panic!("two lists") };
        assert!(Rc::ptr_eq(x, y), "one list, held twice");

        let fields = Rc::new(RefCell::new(vec![("v".to_string(), Value::Int(7))]));
        let k = Value::Record { name: Rc::from("Node"), fields: fields.clone() };
        fields.borrow_mut().push(("next".to_string(), Value::variant("Some", vec![k.clone()])));
        let back = msg_to_value(value_to_msg(&k, None).expect("sendable"), &Scope::root());
        assert_eq!(back.display(), "Node { v: 7, next: Some(Node { ... }) }");
        let Value::Record { fields: got, .. } = &back else { panic!("a record arrives") };
        let next = got.borrow()[1].1.clone();
        let Value::Variant { fields: some, .. } = next else { panic!("Some") };
        let Value::Record { fields: inner, .. } = &some[0] else { panic!("a record inside") };
        assert!(Rc::ptr_eq(got, inner), "it holds itself");
        fields.borrow_mut().clear(); // break the sender's cycle, so the test does not leak it
        got.borrow_mut().clear();
    }

    /// DATA-DEPTH-2: a value nested past the native stack — 200,000 levels through a variant, a list, a record and a map in
    /// turn, what a loop builds — crosses and arrives whole, and a message of it dropped unread is taken apart without
    /// recursion. Each walk recursed once per level, and a chain of 300,000 links sent to an actor killed the process. As
    /// with the teardown locks in `value.rs`, the falsification signal is the test binary dying.
    #[test]
    fn a_value_nested_past_the_stack_crosses_whole() {
        let deep = |leaf: i64| {
            let mut v = Value::Int(leaf);
            for i in 0..200_000 {
                v = match i % 4 {
                    0 => Value::variant("Link", vec![v]),
                    1 => Value::List(Rc::new(RefCell::new(vec![v]))),
                    2 => Value::Record { name: Rc::from("Box"), fields: Rc::new(RefCell::new(vec![("f".to_string(), v)])) },
                    _ => Value::Map(Rc::new(RefCell::new(std::collections::BTreeMap::from([(crate::value::MapKey::Int(1), v)])))),
                };
            }
            v
        };
        let sent = deep(7);
        let back = msg_to_value(value_to_msg(&sent, None).expect("sendable"), &Scope::root());
        assert!(back.eq(&sent), "it arrives as it was sent");
        assert!(!back.eq(&deep(8)), "down to its deepest value");
        drop(value_to_msg(&sent, None).expect("sendable")); // a message dropped unread
    }
}
