//! The Stage-1 tree-walking interpreter (spec §7). It runs the *checked* AST, so it assumes
//! well-typedness and focuses on faithful evaluation and host-side capability enforcement.
//! Runtime failures are defined faults (DL09xx) that abort cleanly — never UB.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use delulu_check::ResourceKind;
use delulu_syntax::ast::*;

use crate::custody::{Custody, CustodyDecision, EmbeddedCustody, Op as CustodyOp};
use crate::foreign::{self, FKind, FVal, ForeignBinder, ForeignHandle, ForeignSig, InProcBinder};
use crate::prim;
use crate::trace::{self, TraceRecord, TraceSink};
use crate::value::{ActuatorEnvelope, CapScope, CapVal, Closure, Env, Fault, Scope, SecretVal, Value};

/// The default logical call-depth bound, and the native stack it assumes.
///
/// **This number is a contract with the host, not a free parameter** (`HARDENING_CAMPAIGN.md` C21,
/// ruling D51). The interpreter is a tree-walker: one DeluluLang call costs several native frames,
/// and a debug build's frames are large — 10,000 of them need **more than 16 MiB** of native stack,
/// measured. `delulu`'s own `main.rs` therefore runs everything on a thread with 512 MiB reserved,
/// so `MAX_DEPTH` is the limit that actually fires and deep recursion is DL0905 rather than a crash.
///
/// An **embedder** gets no such thread for free. Rust's default is about 2 MiB, which this bound
/// overruns long before it triggers — so the guard never fires and the process dies of
/// `STATUS_STACK_OVERFLOW`/SIGSEGV instead. That is the host-crash class D15 fixed for the CLI,
/// resurfacing for anyone using `delulu-runtime` as a library.
///
/// The fix is to make the bound an explicit part of the API rather than an undocumented requirement:
/// see [`Interp::with_max_depth`]. The default is unchanged, so the CLI behaves exactly as before.
pub const DEFAULT_MAX_DEPTH: u32 = 10_000;

/// Native stack to reserve **per unit of call depth**, for embedders sizing a thread.
///
/// **80 KiB, and the number is measured rather than derived.** Bracketed by running the guard on a
/// deliberately small thread and moving the bound until it broke (ruling D51):
///
/// | thread stack | bound | bytes/depth | result |
/// |---|---|---|---|
/// | 8 MiB | 500 | 16 KiB | `STATUS_STACK_OVERFLOW` |
/// | 8 MiB | 200 | 40 KiB | `STATUS_STACK_OVERFLOW` |
/// | 8 MiB | 100 | **80 KiB** | **DL0905, clean** |
///
/// Those figures are a **debug** build, which is what an embedder's tests run and therefore the case
/// that must not crash. A release build is far cheaper: `delulu`'s own `main.rs` reserves 512 MiB for
/// [`DEFAULT_MAX_DEPTH`] — about 52 KiB per unit — and deep recursion on the release CLI reports
/// DL0905 cleanly at 100,000 calls, so release fits inside that. Budget for debug and release is
/// covered automatically.
///
/// **The figure C21 recorded — "10,000 frames need more than 16 MiB" — is true but reads as if 16 MiB
/// were nearly enough.** It is a lower bound roughly an order of magnitude below the real cost, and a
/// first draft of this constant took it literally and would have advised an embedder into exactly the
/// crash this contract exists to prevent.
pub const STACK_BYTES_PER_DEPTH: usize = 80 * 1024;

/// The native stack **every thread that runs interpreter code must reserve**, so that
/// [`DEFAULT_MAX_DEPTH`] is the limit that fires and deep recursion is DL0905 rather than an abort.
///
/// **This constant lives here, next to the bound it pays for, because it had drifted.** It was a
/// private constant in `delulu`'s `main.rs`, and the rule it encodes — *a thread that runs a
/// DeluluLang program reserves a stack sized for the depth bound* — was therefore maintained at
/// exactly one site. The actor scheduler in `actors.rs` spawns worker threads that run the very same
/// interpreter, and it never set a stack size at all. The consequence was measured, not theorised:
///
/// | where the recursion runs | thread | stack | `down(1000)` |
/// |---|---|---|---|
/// | `fn main` | `delulu-main` | 512 MiB (explicit) | prints `1000` |
/// | an actor behavior | `delulu-actor-N` | OS default | **`STATUS_STACK_OVERFLOW`** |
///
/// Same function, same depth, same process. On Windows the actor path aborted above depth **43**
/// (debug) and **~350** (release) — against a documented bound of 10,000 — which made
/// `ref.rule.runtime.faults-are-diagnostics` false on the concurrency path while the CLI witness for
/// that rule kept passing, because the witness recurses in `main`.
///
/// **It is derived from the two numbers above rather than picked, and that closed a contradiction
/// the tree had been carrying.** `main.rs` reserved 512 MiB, while [`STACK_BYTES_PER_DEPTH`]
/// published 80 KiB for a [`DEFAULT_MAX_DEPTH`] of 10,000 — which is 800 MiB. The reservation and
/// the published budget disagreed by 264 MiB, in the direction where the *advice* was safer than
/// what the toolchain gave itself. 512 MiB is sufficient for the *measured* per-frame cost, so
/// nothing crashed and nothing surfaced it; the first thing to actually compare them was
/// [`max_depth_for_stack`], which computed a bound of 6,550 for the CLI's own thread.
///
/// A contract that reserves less than it advises is not a contract, so the reservation now follows
/// the advice. The reservation is virtual — pages are committed only as they are touched — so a
/// program that never recurses pays nothing for the difference.
///
/// A thread that cannot get this much must lower its bound to match — see [`max_depth_for_stack`].
pub const INTERPRETER_STACK_BYTES: usize =
    DEFAULT_MAX_DEPTH as usize * STACK_BYTES_PER_DEPTH + NON_RECURSIVE_RESERVE;

/// Stack that is *not* interpreter recursion: the frames already on the stack when the first
/// DeluluLang call happens (the worker loop, the actor turn machinery, the host's own callers) plus
/// headroom for the diagnostic path, which has to run *after* the bound trips.
const NON_RECURSIVE_RESERVE: usize = 256 * 1024;

/// The largest call-depth bound that fits inside `stack_bytes`, for a thread that could not reserve
/// [`INTERPRETER_STACK_BYTES`].
///
/// **The pair (stack, bound) is the invariant; neither half is meaningful alone.** A thread that
/// silently keeps [`DEFAULT_MAX_DEPTH`] on a small stack is the crash this whole contract exists to
/// prevent, so a caller that settles for less stack must pass the result of this function to
/// [`Interp::with_max_depth`] in the same breath.
///
/// Budgeted at the conservative [`STACK_BYTES_PER_DEPTH`], less a fixed reserve for the frames that
/// are not interpreter recursion (the worker loop, the actor turn machinery, and whatever the host
/// had on the stack already). The result is clamped to at least 1: a bound of zero would refuse
/// every call rather than bound the depth of one.
/// Run `f` on a thread sized for [`DEFAULT_MAX_DEPTH`], so the guard is what fires.
///
/// **This exists because a contract nobody calls is a contract nobody is keeping.** C21 was filed as
/// a library-embedding hazard and D51 closed it by making the depth bound explicit
/// ([`Interp::with_max_depth`], [`STACK_BYTES_PER_DEPTH`]). That was the right mechanism — and for
/// the whole of its life nothing in this repository called it, including the one caller that most
/// needed to (the actor scheduler, C70/D67). The lesson generalizes past that one bug: an embedder
/// who has to *remember* to size a thread is an embedder who will one day not, and the failure is
/// not a diagnostic but a `SIGSEGV`. So the safe path is now the short one:
///
/// ```no_run
/// # use delulu_runtime::{on_interpreter_thread, Interp};
/// let outcome = on_interpreter_thread(move || {
///     // build and run an `Interp` here; `DEFAULT_MAX_DEPTH` is reachable and DL0905 fires
///     42
/// })?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// **It reports rather than degrades, and that asymmetry is deliberate.** The actor scheduler, which
/// owns its own threads and must keep running, walks a ladder down and lowers its bound to match. A
/// library cannot make that trade on an application's behalf — an embedder may prefer to fail, to
/// retry smaller, or to run shallow work anyway — so a refused reservation comes back as an `Err`
/// with the OS's reason, and the caller decides. Deciding for them would be the silent fallback
/// `ref.rule.portability.isolation-labels-are-honest` exists to forbid.
///
/// A panic inside `f` is reported as an error rather than resumed, because the payload is not
/// `Send`-recoverable in a useful form here; the panic itself has already been printed by the
/// runtime.
pub fn on_interpreter_thread<T, F>(f: F) -> std::io::Result<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    std::thread::Builder::new()
        .name("delulu-interpreter".into())
        .stack_size(INTERPRETER_STACK_BYTES)
        .spawn(f)?
        .join()
        .map_err(|_| std::io::Error::other("the interpreter thread panicked"))
}

pub const fn max_depth_for_stack(stack_bytes: usize) -> u32 {
    let usable = stack_bytes.saturating_sub(NON_RECURSIVE_RESERVE);
    let depth = usable / STACK_BYTES_PER_DEPTH;
    if depth == 0 {
        1
    } else if depth > DEFAULT_MAX_DEPTH as usize {
        DEFAULT_MAX_DEPTH
    } else {
        depth as u32
    }
}

/// Default best-effort step budget for a plugin run when `Limits::fuel == 0` (spec §4/§5.4). A
/// bound, never "unlimited" — but generous, because the interpreter is a *courtesy* path, not the
/// enforcement path: a Contained plugin always runs on the WASM engine by construction.
pub const DEFAULT_INTERP_STEPS: u64 = 200_000_000;
/// Default best-effort memory-accounting budget (bytes) when `Limits::mem_mb == 0`.
pub const DEFAULT_INTERP_MEM_BYTES: u64 = 256 * 1024 * 1024;

/// Best-effort interpreter execution limits (spec §5.4, invariant 32).
///
/// **"Best-effort" is a load-bearing label, not a hedge.** The tree-walking interpreter is *not*
/// the enforcement-grade sandbox for hostile code — a Contained plugin ALWAYS runs on the WASM
/// engine (spec §5.1/§5.4), and a Verified plugin was re-proved safe *by type* at load. These
/// limits only bound *accidental* runaway (an infinite loop, a runaway allocation) in a Verified
/// plugin a host chose to run here. `fuel` maps to a **step counter**; `mem_mb` maps to
/// **allocator accounting** at the interpreter's own value-construction sites — which cannot see
/// every byte the Rust allocator touches, so it under-counts, and the diagnostic says so. We never
/// claim the interpreter *contains* hostile code.
///
/// Interior-mutable (`Cell`) because the interpreter evaluates through `&self`; [`Budget::reset`]
/// gives each call its own fresh budget so one call cannot starve a later one.
struct Budget {
    fuel: u64,
    mem_bytes: u64,
    steps_left: Cell<u64>,
    bytes_left: Cell<u64>,
}

impl Budget {
    fn new(fuel: u64, mem_bytes: u64) -> Budget {
        Budget { fuel, mem_bytes, steps_left: Cell::new(fuel), bytes_left: Cell::new(mem_bytes) }
    }
    /// Restore the full budget — called before each export invocation.
    fn reset(&self) {
        self.steps_left.set(self.fuel);
        self.bytes_left.set(self.mem_bytes);
    }
    /// Charge one evaluation step (fuel). `Err("fuel")` once the budget is spent.
    fn step(&self) -> Result<(), &'static str> {
        let s = self.steps_left.get();
        if s == 0 {
            return Err("fuel");
        }
        self.steps_left.set(s - 1);
        Ok(())
    }
    /// Charge `n` bytes of allocation (mem). `Err("mem_mb")` once the budget is spent.
    fn alloc(&self, n: u64) -> Result<(), &'static str> {
        let b = self.bytes_left.get();
        if n > b {
            self.bytes_left.set(0);
            return Err("mem_mb");
        }
        self.bytes_left.set(b - n);
        Ok(())
    }
}

/// The DL1506 message for a best-effort interpreter limit. It **LABELS** the mechanism (spec §5.4)
/// so no reader mistakes the interpreter for the enforcement-grade path: a Contained plugin runs on
/// the WASM engine, where the same limit is *enforced*, not merely *accounted*.
fn best_effort_limit_msg(limit: &str) -> String {
    format!(
        "best-effort interpreter {limit} budget exhausted — the interpreter is not the enforcement-grade sandbox (spec §5.4); a Contained plugin runs on the WASM engine, where this limit is enforced rather than merely accounted"
    )
}

/// Non-local control flow. `Fault` is a real runtime error; the others are ordinary control.
enum Escape {
    Return(Value),
    Propagate(Value), // `?` propagating an `Err` as the function's result
    Break,            // Tier 1: `break` unwinds to the nearest enclosing loop, which catches it
    Continue,         // Tier 1: `continue` unwinds to the nearest enclosing loop's next iteration
    Fault(Fault),
}

type R<T> = Result<T, Escape>;

pub struct Interp {
    funcs: HashMap<String, FnDecl>,
    /// `test` bodies by name (Stage 8, phase 8g) — read ONLY by [`Interp::run_test`],
    /// never by any normal execution path (invariant 41).
    tests: HashMap<String, delulu_syntax::ast::Block>,
    consts: Vec<(String, Expr)>,
    globals: Env,
    depth: Cell<u32>,
    /// The logical call-depth bound this interpreter enforces (DL0905). Defaults to
    /// [`DEFAULT_MAX_DEPTH`]; an embedder on a small native stack lowers it via
    /// [`Interp::with_max_depth`] so the guard fires before the stack runs out (C21/D51).
    max_depth: u32,
    /// Stage 10 (10d): true while an actor turn executes — the ONLY time allocations are
    /// registered with the cycle collector. Main-thread programs never set it, so the Study-C
    /// perf surface pays one predictable branch per allocation and nothing else.
    in_turn: Cell<bool>,
    /// Stage 10 (10d): this worker's cycle-collector registry (`cycles.rs`).
    cycle: std::cell::RefCell<crate::cycles::Registry>,
    /// Effect tracing (spec §6.1): absent by default, attached via `with_trace`. Additive — does
    /// not change `Interp::new`'s signature or behavior.
    trace: Option<TraceSink>,
    trace_seq: Cell<u64>,
    /// PS-A-01: the ONE seam every capability operation leaves the interpreter through. Defaults to
    /// [`crate::sink::LocalSink`], today's in-process path, byte for byte. PS-A's guest swaps in a
    /// channel sink and holds no OS authority of its own.
    effects: Rc<dyn crate::sink::EffectSink>,
    /// Foreign blocks by lib name (Stage 4): each declared function's marshalling signature, lowered
    /// from the module. Empty for a program with no `foreign` blocks — so nothing changes for it.
    foreign_blocks: HashMap<String, Vec<ForeignSig>>,
    /// Which lib each `root.foreign(load)` call node binds (from the checker's `foreign_binds`), the
    /// granted binary path per lib (from `--grant`), and the return-size ceiling. All inert unless
    /// `with_foreign` is called.
    foreign_binds: HashMap<NodeId, String>,
    foreign_grants: HashMap<String, String>,
    foreign_max_ret: usize,
    /// How a bound foreign lib executes (Stage 5 phase 5h). Default [`InProcBinder`] — Stage-4
    /// in-process load, so `--foreign-isolation inproc` (and every program that never opts in) is
    /// byte-identical to Stage 4. `--foreign-isolation process` injects the CLI's worker-spawning
    /// binder via [`Interp::with_foreign_binder`], running each granted C lib in an isolated
    /// subprocess. `Rc` so the binder is shared cheaply (bind may happen at several call sites).
    foreign_binder: Rc<dyn ForeignBinder>,
    /// P2: the plugin container reader, injected for the same reason the foreign binder is —
    /// `delulu-wasm` implements the `PluginEngine` trait this crate DEFINES, so this crate cannot
    /// reach it. `None` means nothing wired it, which is a wiring bug rather than a policy: the CLI
    /// always supplies one, and `load` says so rather than refusing as though the artifact were bad.
    plugin_engine: Option<Rc<dyn crate::plugin::PluginEngine>>,
    /// Where authority lives (Stage 5 phase 5f). Default: [`EmbeddedCustody`] — a pass-through, so a
    /// program built with `Interp::new` behaves EXACTLY as in Stages 1–4 (criterion 11). `--broker
    /// daemon` swaps in a `BrokerClientCustody` (IPC) via [`Interp::with_custody`]. Interior
    /// mutability because `eval_*` take `&self` and custody `check`/`expose` mutate the epoch cache.
    custody: RefCell<Box<dyn Custody>>,
    /// Best-effort execution limits (spec §5.4, Stage 6 phase 6e.5). **`None` for every Stage-1..5
    /// entry point** — attached only for a Verified plugin run via [`Interp::with_plugin_budget`], so
    /// a program built with `Interp::new` is byte-identical to Stages 1–4 (criterion 11). When
    /// absent, every budget check is a single always-false branch.
    budget: Option<Budget>,
    /// Actor declarations by name (Stage 7) — used for self-dispatch (sync fns and
    /// self-sends inside member bodies). Empty for a program with no actors.
    actors: HashMap<String, delulu_syntax::ast::ActorDecl>,
    /// The actor system's per-thread handle (Stage 7). `None` unless attached via
    /// [`Interp::with_actors`] — a program with no actors is byte-identical to v0.6.
    actors_host: Option<crate::actors::ActorHost>,
    /// The turn context while a behavior/ctor runs on THIS interpreter: (state record,
    /// address, actor name) — how `self` learns its own address for reply-to sends.
    current_self: RefCell<Option<(Value, crate::actors::ActorId, String, String)>>,
    /// Consts evaluated into `globals` (idempotence guard — Stage 7 evaluates per turn).
    consts_ready: Cell<bool>,
    /// `--debug-rcaps` (Stage 7 phase 7i, spec §6.4): the checker's iso-move send-argument
    /// node ids. When set, each such argument's graph is verified unaliased at the boundary
    /// (DL1610 on violation — a compiler-bug detector, never the guarantee).
    debug_rcaps: Option<std::sync::Arc<std::collections::HashSet<NodeId>>>,
    /// Stage 10 (10f, spec §5.2/§5.4): the device broker — dead-man leases plus whichever adapter
    /// the run's `--broker-profile` selected. `None` is 10e's null-adapter behavior exactly: the
    /// envelope still refuses, sensors still read `NoDevice`, no lease exists to lose.
    devices: Option<std::sync::Arc<crate::device::DeviceBroker>>,
    /// Stage 10 (10h, spec §7.1): the compute broker — the granted device envelopes plus the
    /// adapter backing them. `None` means no adapter is bound, and a dispatch answers `NoAdapter`
    /// rather than a fabricated number (invariant 50, at silicon).
    computes: Option<std::sync::Arc<crate::compute::ComputeBroker>>,
}

impl Interp {
    pub fn new(module: &Module) -> Interp {
        let mut funcs = HashMap::new();
        let mut consts = Vec::new();
        let mut foreign_blocks = HashMap::new();
        let mut actors = HashMap::new();
        let mut tests = HashMap::new();
        for item in &module.items {
            match item {
                Item::Fn(f) => {
                    funcs.insert(f.name.name.clone(), f.clone());
                }
                Item::Const(c) => consts.push((c.name.name.clone(), c.value.clone())),
                Item::Foreign(fd) => {
                    let sigs = fd.fns.iter().map(lower_foreign_sig).collect();
                    foreign_blocks.insert(fd.name.name.clone(), sigs);
                }
                Item::Actor(a) => {
                    actors.insert(a.name.name.clone(), a.clone());
                }
                // Stage 8 (phase 8g): test bodies are held for the RUNNER's entry point
                // only — no normal execution path reads this map, so tests stay compiled
                // out of `delulu run` (invariant 41; the 8a witnesses pin it).
                Item::Test(t) => {
                    tests.insert(t.name.clone(), t.body.clone());
                }
                _ => {}
            }
        }
        // std.actors (Stage 7 phase 7j): Promise injects wherever actors can run — the
        // same canonical source the checker registered, so verify == run.
        {
            let (std_mod, _) = delulu_syntax::parse_file(u32::MAX, delulu_check::STD_ACTORS_SRC);
            for item in std_mod.items {
                if let Item::Actor(a) = item {
                    actors.entry(a.name.name.clone()).or_insert(a);
                }
            }
        }
        Interp {
            funcs,
            tests,
            consts,
            globals: Scope::root(),
            depth: Cell::new(0),
            max_depth: DEFAULT_MAX_DEPTH,
            in_turn: Cell::new(false),
            cycle: std::cell::RefCell::new(crate::cycles::Registry::default()),
            trace: None,
            effects: Rc::new(crate::sink::LocalSink),
            trace_seq: Cell::new(0),
            foreign_blocks,
            foreign_binds: HashMap::new(),
            foreign_grants: HashMap::new(),
            foreign_max_ret: foreign::DEFAULT_MAX_RET,
            foreign_binder: Rc::new(InProcBinder),
            plugin_engine: None,
            custody: RefCell::new(Box::new(EmbeddedCustody::new())),
            budget: None,
            actors,
            actors_host: None,
            current_self: RefCell::new(None),
            consts_ready: Cell::new(false),
            debug_rcaps: None,
            devices: None,
            computes: None,
        }
    }

    /// Attach the checker's iso-move node set for `--debug-rcaps` verification (phase 7i).
    pub fn with_debug_rcaps(
        mut self,
        moves: std::sync::Arc<std::collections::HashSet<NodeId>>,
    ) -> Interp {
        self.debug_rcaps = Some(moves);
        self
    }

    /// The §7.4 debug check at a boundary: if this argument is a statically-proven iso MOVE
    /// and `--debug-rcaps` is on, its graph must be unaliased — else DL1610.
    fn debug_check_iso(&self, arg: &Expr, v: &Value) -> Result<(), Fault> {
        if let Some(moves) = &self.debug_rcaps {
            if moves.contains(&arg.id()) {
                if let Err(why) = crate::actors::assert_unique_graph(v) {
                    return Err(Fault::at(
                        "DL1610",
                        format!(
                            "debug race-checker violation: an iso move's graph is aliased ({why}) — the static uniqueness proof failed; please file a compiler bug"
                        ),
                        arg.span(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Attach the actor system's host handle (Stage 7 phase 7g). Additive: a program that
    /// never spawns behaves identically without one; a spawn WITHOUT one is an honest fault.
    pub fn with_actors(mut self, host: crate::actors::ActorHost) -> Interp {
        self.actors_host = Some(host);
        self
    }

    /// Run one actor TURN (a constructor or behavior body) to completion on this
    /// interpreter — run-to-completion atomicity is the caller's scheduling guarantee
    /// (invariant 34); this just evaluates. `state` is the actor's field record, bound as
    /// `self`; the turn context makes `self`-sends resolve to `self_ref`'s address.
    pub fn run_actor_turn(
        &self,
        body: &Block,
        member: &str,
        params: &[String],
        args: Vec<Value>,
        state: Value,
        self_ref: Value,
    ) -> Result<(), Fault> {
        let (id, name) = match &self_ref {
            Value::ActorRef { id, actor } => (*id, actor.to_string()),
            _ => return Err(Fault::new("DL0907", "actor turn without an actor address")),
        };
        if let Err(Escape::Fault(f)) = self.eval_consts() {
            return Err(f);
        }
        let env = Scope::child(&self.globals);
        env.define("self", state.clone());
        for (p, v) in params.iter().zip(args) {
            env.define(p, v);
        }
        let prev = self.current_self.replace(Some((state, id, name, member.to_string())));
        // Stage 10 (10d): allocations made inside the turn register with the cycle collector;
        // the sweep itself runs between turns, from the worker loop, with every actor state on
        // this worker as a root.
        self.in_turn.set(true);
        let result = self.exec_block_value(body, &env);
        self.in_turn.set(false);
        self.current_self.replace(prev);
        match result {
            // A behavior yields Unit at the send site; `return`/`?` inside it are ordinary
            // turn completion.
            Ok(_) | Err(Escape::Return(_)) | Err(Escape::Propagate(_)) => Ok(()),
            // `break`/`continue` are caught by their enclosing loop; reaching here means the checker
            // let one through outside any loop (DL0412), which is a bug, so surface it loudly.
            Err(Escape::Break) | Err(Escape::Continue) => {
                Err(Fault::new("DL0907", "internal: `break`/`continue` escaped its loop"))
            }
            Err(Escape::Fault(f)) => Err(f),
        }
    }

    /// The declaration of an actor visible to this interpreter (module actors plus the
    /// injected `std.actors` prelude) — the worker loop's single source of truth.
    pub fn actor_decl(&self, name: &str) -> Option<&delulu_syntax::ast::ActorDecl> {
        self.actors.get(name)
    }

    /// Stage 10 (10d): register an in-turn allocation with the cycle collector. A no-op
    /// outside turns — non-actor programs pay exactly this branch.
    fn note_alloc(&self, v: &Value) {
        if self.in_turn.get() {
            self.cycle.borrow_mut().note_value(v);
        }
    }

    /// How many registered cells await a sweep (the worker loop's collect trigger).
    pub fn cycle_pressure(&self) -> usize {
        self.cycle.borrow().pressure()
    }

    /// Run the cycle collector with `roots` (every actor state this worker owns). Returns how
    /// many garbage-cycle cells were broken.
    pub fn collect_cycles(&self, roots: &[Value]) -> u64 {
        self.cycle.borrow_mut().collect(roots)
    }

    /// (runs, cells collected) so far — the `--trace-memory` numbers.
    pub fn cycle_stats(&self) -> (u64, u64) {
        let c = self.cycle.borrow();
        (c.runs, c.collected)
    }

    /// Rebuild an actor-boundary message into THIS interpreter's heap (closures reattach to
    /// these globals).
    pub fn msg_to_value(&self, m: crate::actors::MsgValue) -> Value {
        crate::actors::msg_to_value(m, &self.globals)
    }

    /// Convert a value FOR an actor boundary, with this turn's self-identity attached (so
    /// `self` in an argument position becomes the actor's own address — the reply-to
    /// pattern; `ref self <: tag` makes it statically legal).
    fn value_to_msg_here(&self, v: &Value) -> Result<crate::actors::MsgValue, Fault> {
        let ctx = self.current_self.borrow();
        let self_state = ctx.as_ref().map(|(s, id, name, _)| (s, *id, name.as_str()));
        crate::actors::value_to_msg(v, self_state)
    }

    /// A record template carrying this turn's actor attribution (spec §6.3). Main-line
    /// records get all-`None`, so a program without actors traces byte-identically to v0.6.
    /// `turn`/`cause` are the WORKER's knowledge, stamped when it drains the sink.
    fn trace_attrib_record(&self) -> TraceRecord {
        match &*self.current_self.borrow() {
            Some((_, id, name, member)) => TraceRecord {
                actor: Some(format!("{name}#{}", id.id)),
                member: Some(format!("{name}.{member}")),
                ..Default::default()
            },
            None => TraceRecord::default(),
        }
    }

    /// The causal identity of THIS execution context for an outgoing send: (sender label,
    /// sender member). Main line: ("main#0", None).
    pub fn sender_identity(&self) -> (String, Option<String>) {
        match &*self.current_self.borrow() {
            Some((_, id, name, member)) => (format!("{name}#{}", id.id), Some(format!("{name}.{member}"))),
            None => ("main#0".to_string(), None),
        }
    }

    /// Attach best-effort execution limits for a Verified plugin run (spec §5.4, phase 6e.5). Builder
    /// style, additive: `fuel` becomes a step budget and `mem_bytes` an allocation-accounting budget.
    /// Every existing entry point leaves this `None` — so the interpreter's behavior for a host
    /// program is unchanged (criterion 11). The budget is **best-effort and labeled so**: the
    /// interpreter is not the enforcement-grade sandbox (that is the WASM engine, spec §5.1/§5.4).
    pub fn with_plugin_budget(mut self, fuel: u64, mem_bytes: u64) -> Interp {
        self.budget = Some(Budget::new(fuel, mem_bytes));
        self
    }

    /// Restore the full best-effort budget (no-op if none is attached). Called before each Verified
    /// export invocation so a plugin cannot starve a later call with an earlier one's spending.
    pub fn reset_plugin_budget(&self) {
        if let Some(b) = &self.budget {
            b.reset();
        }
    }

    /// Charge one evaluation step against the best-effort budget (no-op when absent). A spent budget
    /// surfaces as DL1506 (`LimitExceeded`) with the best-effort label (spec §5.4).
    fn charge_step(&self) -> R<()> {
        if let Some(b) = &self.budget {
            if let Err(limit) = b.step() {
                return Err(Escape::Fault(Fault::new("DL1506", best_effort_limit_msg(limit))));
            }
        }
        Ok(())
    }

    /// Charge `n` bytes of allocation against the best-effort budget (no-op when absent).
    fn charge_alloc(&self, n: u64) -> R<()> {
        if let Some(b) = &self.budget {
            if let Err(limit) = b.alloc(n) {
                return Err(Escape::Fault(Fault::new("DL1506", best_effort_limit_msg(limit))));
            }
        }
        Ok(())
    }

    /// Route foreign binds through a custom [`ForeignBinder`] (Stage 5 phase 5h). The CLI attaches a
    /// worker-spawning binder for `--foreign-isolation process`; the default is [`InProcBinder`], so
    /// every existing entry point loads foreign code in-process exactly as Stage 4 (criterion 11).
    /// Builder style; additive.
    pub fn with_foreign_binder(mut self, binder: Rc<dyn ForeignBinder>) -> Interp {
        self.foreign_binder = binder;
        self
    }

    /// P2: give this run the plugin container reader (`delulu-wasm`'s `WasmPluginEngine`).
    pub fn with_plugin_engine(mut self, engine: Rc<dyn crate::plugin::PluginEngine>) -> Interp {
        self.plugin_engine = Some(engine);
        self
    }

    /// Set the logical call-depth bound that raises DL0905 (C21, ruling D51).
    ///
    /// **Embedders on a small native stack must call this.** The interpreter is a tree-walker, so one
    /// DeluluLang call costs several native frames; [`DEFAULT_MAX_DEPTH`] assumes the 512 MiB thread
    /// `delulu`'s own `main.rs` creates. On Rust's ~2 MiB default thread stack that bound is never
    /// reached — the process dies of a native stack overflow first, which is a host crash rather than
    /// the diagnostic this guard exists to produce.
    ///
    /// Budget roughly [`STACK_BYTES_PER_DEPTH`] of stack per unit of depth, and more in a debug
    /// build. A 2 MiB thread should use a bound around 1,000; the default suits 32 MiB and up.
    ///
    /// Builder style, additive, and the default is unchanged — every existing entry point behaves
    /// exactly as before.
    pub fn with_max_depth(mut self, max_depth: u32) -> Interp {
        self.max_depth = max_depth.max(1);
        self
    }

    /// Route authority decisions through a custom [`Custody`] (Stage 5 phase 5f). The CLI attaches a
    /// `BrokerClientCustody` here for `--broker daemon`; the default is [`EmbeddedCustody`], so every
    /// existing entry point is unchanged (criterion 11). Builder style; additive.
    pub fn with_custody(mut self, custody: Box<dyn Custody>) -> Interp {
        self.custody = RefCell::new(custody);
        self
    }

    /// Attach the Stage-4 foreign runtime data (builder style): the checker's `root.foreign(load)`
    /// bind-site → lib map, the `--grant foreign.c=lib:path` binary paths, and the `--foreign-max-ret`
    /// return-size ceiling. Additive; a program with no foreign blocks is unaffected.
    pub fn with_foreign(
        mut self,
        binds: HashMap<NodeId, String>,
        grants: HashMap<String, String>,
        max_ret: usize,
    ) -> Interp {
        self.foreign_binds = binds;
        self.foreign_grants = grants;
        self.foreign_max_ret = max_ret;
        self
    }

    /// Attach a trace sink (builder style, spec §6.1): every EFFECTFUL primitive operation
    /// dispatched by `eval_method` appends a `TraceRecord` here as it executes. Pure operations
    /// (attenuation, `verify`, `Str`/`List` methods, `Root` capability minting) are never traced.
    pub fn with_trace(mut self, sink: TraceSink) -> Interp {
        self.trace = Some(sink);
        self
    }

    /// Send capability operations somewhere other than this process (PS-A-01). Without this the
    /// interpreter uses [`crate::sink::LocalSink`] and behaves exactly as it always has.
    pub fn with_effect_sink(mut self, sink: Rc<dyn crate::sink::EffectSink>) -> Interp {
        self.effects = sink;
        self
    }

    /// Attach the device broker (Stage 10 phase 10f). Without it the physical surface behaves
    /// exactly as 10e shipped it, which is the fallback a missing adapter deserves: refuse
    /// commands the envelope cannot vouch for, and answer sensor reads with absence.
    pub fn with_devices(mut self, broker: std::sync::Arc<crate::device::DeviceBroker>) -> Interp {
        self.devices = Some(broker);
        self
    }

    /// Attach this run's compute broker (10h). Absent = no adapter bound, and a dispatch answers
    /// `NoAdapter` rather than inventing a result.
    pub fn with_computes(mut self, broker: std::sync::Arc<crate::compute::ComputeBroker>) -> Interp {
        self.computes = Some(broker);
        self
    }

    fn next_trace_seq(&self) -> u64 {
        let s = self.trace_seq.get();
        self.trace_seq.set(s + 1);
        s
    }

    /// Run `main(root)`. Returns the runtime value or a fault.
    pub fn run_main(&self, root: Value) -> Result<Value, Fault> {
        self.eval_consts().map_err(unwrap_fault)?;
        self.call_fn("main", vec![root]).map_err(unwrap_fault)
    }

    /// Run one `test` block body (Stage 8, phase 8g): `test_root` bound, `Unit` result.
    /// The ONLY entry point that executes a test — `run_main` and every other path never
    /// touch the tests map (invariant 41).
    pub fn run_test(&self, name: &str, test_root: Value) -> Result<Value, Fault> {
        self.eval_consts().map_err(unwrap_fault)?;
        let Some(body) = self.tests.get(name) else {
            return Err(Fault::new("DL0907", format!("unknown test \"{name}\"")));
        };
        let run = || -> R<Value> {
            self.enter()?;
            let env = Scope::child(&self.globals);
            env.define("test_root", test_root);
            let result = self.exec_block_value(body, &env);
            self.leave();
            self.finish_call(result)
        };
        run().map_err(unwrap_fault)
    }

    /// The names of this module's `test` blocks, in declaration order-independent form.
    pub fn test_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.tests.keys().cloned().collect();
        names.sort();
        names
    }

    /// Call a pure function with Int arguments and return its Int (or Bool-as-Int) result. Used by
    /// the WASM backend's two-engine parity harness — the interpreter is the reference engine
    /// (spec §9). Additive; does not change existing entry points.
    pub fn call_int_fn(&self, name: &str, args: &[i64]) -> Result<i64, Fault> {
        self.eval_consts().map_err(unwrap_fault)?;
        let argvals: Vec<Value> = args.iter().map(|&a| Value::Int(a)).collect();
        match self.call_fn(name, argvals) {
            Ok(Value::Int(n)) => Ok(n),
            Ok(Value::Bool(b)) => Ok(if b { 1 } else { 0 }),
            Ok(other) => Err(Fault::new("DL0907", format!("expected an Int/Bool result, got `{}`", other.display()))),
            Err(e) => Err(unwrap_fault(e)),
        }
    }

    /// Call a function with arbitrary argument values (e.g. a `Cap[Console]`), for the WASM
    /// backend's effect-parity harness. Additive; the interpreter is the reference engine.
    pub fn call_with(&self, name: &str, args: Vec<Value>) -> Result<Value, Fault> {
        self.eval_consts().map_err(unwrap_fault)?;
        self.call_fn(name, args).map_err(unwrap_fault)
    }

    /// Evaluate one top-level expression (for the REPL), with an optional root binding.
    pub fn eval_toplevel(&self, e: &Expr, root: Option<Value>) -> Result<Value, Fault> {
        self.eval_consts().map_err(unwrap_fault)?;
        let env = Scope::child(&self.globals);
        if let Some(r) = root {
            env.define("root", r);
        }
        self.eval_expr(e, &env).map_err(unwrap_fault)
    }

    fn eval_consts(&self) -> R<()> {
        // Idempotent: consts are pure (checker invariant), and Stage 7 calls this per actor
        // turn — evaluate once, then it is a load.
        if self.consts_ready.get() {
            return Ok(());
        }
        for (name, expr) in &self.consts {
            let v = self.eval_expr(expr, &self.globals)?;
            self.globals.define(name, v);
        }
        self.consts_ready.set(true);
        Ok(())
    }

    // ----- calls ----------------------------------------------------------

    fn enter(&self) -> R<()> {
        let d = self.depth.get() + 1;
        if d > self.max_depth {
            return Err(Escape::Fault(Fault::new("DL0905", "recursion depth exceeded")));
        }
        self.depth.set(d);
        Ok(())
    }
    fn leave(&self) {
        self.depth.set(self.depth.get().saturating_sub(1));
    }

    fn call_fn(&self, name: &str, args: Vec<Value>) -> R<Value> {
        let f = match self.funcs.get(name) {
            Some(f) => f,
            None => return Err(Escape::Fault(Fault::new("DL0907", format!("unknown function `{name}`")))),
        };
        self.enter()?;
        let env = Scope::child(&self.globals);
        for (p, v) in f.params.iter().zip(args) {
            env.define(&p.name.name, v);
        }
        let result = self.exec_block_value(&f.body, &env);
        self.leave();
        self.finish_call(result)
    }

    fn call_closure(&self, clo: &Rc<Closure>, args: Vec<Value>) -> R<Value> {
        self.enter()?;
        let env = Scope::child(&clo.env);
        for (p, v) in clo.params.iter().zip(args) {
            env.define(p, v);
        }
        let result = self.exec_block_value(&clo.body, &env);
        self.leave();
        self.finish_call(result)
    }

    /// Convert a body result into a call result: `return`/`?`-propagation become the value;
    /// faults keep propagating.
    fn finish_call(&self, result: R<Value>) -> R<Value> {
        match result {
            Ok(v) => Ok(v),
            Err(Escape::Return(v)) => Ok(v),
            Err(Escape::Propagate(v)) => Ok(v),
            // A loop always catches its own `break`/`continue`; one reaching a call boundary means
            // the checker admitted it outside a loop, so fail loudly rather than swallow it.
            Err(Escape::Break) | Err(Escape::Continue) => {
                Err(Escape::Fault(Fault::new("DL0907", "internal: `break`/`continue` escaped its loop")))
            }
            Err(Escape::Fault(f)) => Err(Escape::Fault(f)),
        }
    }

    // ----- statements and blocks ------------------------------------------

    fn exec_block_value(&self, block: &Block, parent: &Env) -> R<Value> {
        let env = Scope::child(parent);
        let mut value = Value::Unit;
        let n = block.stmts.len();
        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_last = i + 1 == n;
            value = self.exec_stmt(stmt, &env, is_last)?;
        }
        Ok(value)
    }

    fn exec_stmt(&self, stmt: &Stmt, env: &Env, is_last: bool) -> R<Value> {
        match stmt {
            Stmt::Let { name, value, .. } => {
                let v = self.eval_expr(value, env)?;
                env.define(&name.name, v);
                Ok(Value::Unit)
            }
            Stmt::Assign { target, value, span } => {
                let v = self.eval_expr(value, env)?;
                self.assign(target, v, env, *span)?;
                Ok(Value::Unit)
            }
            Stmt::While { cond, body, .. } => {
                // `while let Value::Bool(true) = …` reads as though a non-`Bool` condition simply
                // ends the loop, which is not what is meant: the checker guarantees the condition is
                // `Bool`, and anything else arriving here is a bug in the checker, not a false
                // condition. Written this way the `_ => break` is visibly the impossible branch
                // rather than part of the loop's contract.
                #[allow(clippy::while_let_loop)]
                loop {
                    match self.eval_expr(cond, env)? {
                        Value::Bool(true) => match self.exec_block_value(body, env) {
                            Ok(_) | Err(Escape::Continue) => {} // continue: re-evaluate the condition
                            Err(Escape::Break) => break,
                            Err(other) => return Err(other), // Return / Propagate / Fault escape the loop
                        },
                        _ => break,
                    }
                }
                Ok(Value::Unit)
            }
            Stmt::For { var, iter, body, .. } => {
                let iterable = self.eval_expr(iter, env)?;
                let Value::List(items) = iterable else {
                    // The checker (DL0411) guarantees a `List` here; anything else is a checker bug.
                    return Err(Escape::Fault(Fault::new("DL0907", "internal: `for` iterated a non-list value")));
                };
                // Iterate a SNAPSHOT so mutating the underlying list inside the body cannot make the
                // loop skip, repeat, or run forever — the length is fixed the moment the loop starts.
                let snapshot: Vec<Value> = items.borrow().clone();
                for item in snapshot {
                    let loop_env = Scope::child(env);
                    loop_env.define(&var.name, item);
                    match self.exec_block_value(body, &loop_env) {
                        Ok(_) | Err(Escape::Continue) => {} // continue: move to the next element
                        Err(Escape::Break) => break,
                        Err(other) => return Err(other),
                    }
                }
                Ok(Value::Unit)
            }
            Stmt::Break { .. } => Err(Escape::Break),
            Stmt::Continue { .. } => Err(Escape::Continue),
            Stmt::Return { value, .. } => {
                let v = match value {
                    Some(e) => self.eval_expr(e, env)?,
                    None => Value::Unit,
                };
                Err(Escape::Return(v))
            }
            Stmt::Expr(e) => {
                let v = self.eval_expr(e, env)?;
                Ok(if is_last { v } else { Value::Unit })
            }
        }
    }

    fn assign(&self, target: &LValue, value: Value, env: &Env, span: delulu_diag::Span) -> R<()> {
        match target {
            LValue::Var(name) => {
                if !env.assign(&name.name, value) {
                    return Err(Escape::Fault(Fault::at("DL0907", format!("assignment to unbound `{}`", name.name), span)));
                }
                Ok(())
            }
            LValue::Field(base, field) => {
                let recv = self.eval_lvalue(base, env)?;
                if let Value::Record { fields, .. } = recv {
                    for (n, slot) in fields.borrow_mut().iter_mut() {
                        if *n == field.name {
                            *slot = value;
                            return Ok(());
                        }
                    }
                }
                Err(Escape::Fault(Fault::at("DL0907", "field assignment on non-record", span)))
            }
            LValue::Index(base, idx) => {
                let recv = self.eval_lvalue(base, env)?;
                let i = self.eval_expr(idx, env)?;
                if let (Value::List(l), Value::Int(n)) = (recv, i) {
                    let mut b = l.borrow_mut();
                    let n = n as usize;
                    if n < b.len() {
                        b[n] = value;
                        return Ok(());
                    }
                    return Err(Escape::Fault(Fault::at("DL0903", "index out of bounds", span)));
                }
                Err(Escape::Fault(Fault::at("DL0907", "index assignment on non-list", span)))
            }
        }
    }

    fn eval_lvalue(&self, lv: &LValue, env: &Env) -> R<Value> {
        match lv {
            LValue::Var(name) => env
                .get(&name.name)
                .ok_or_else(|| Escape::Fault(Fault::at("DL0907", format!("unbound `{}`", name.name), name.span))),
            LValue::Field(base, field) => {
                let b = self.eval_lvalue(base, env)?;
                self.field(b, &field.name, field.span)
            }
            LValue::Index(base, idx) => {
                let b = self.eval_lvalue(base, env)?;
                let i = self.eval_expr(idx, env)?;
                self.index(b, i, base.span())
            }
        }
    }

    // ----- expressions ----------------------------------------------------

    fn eval_expr(&self, e: &Expr, env: &Env) -> R<Value> {
        // Best-effort fuel (spec §5.4): one step per expression node. A no-op unless a plugin budget
        // is attached, so every host-program entry point is unaffected (criterion 11).
        self.charge_step()?;
        match e {
            Expr::Lit { kind, .. } => Ok(self.lit(kind)),
            Expr::Var { path, span, .. } => self.eval_var(path, *span, env),
            Expr::List { items, .. } => {
                let mut vs = Vec::with_capacity(items.len());
                for it in items {
                    vs.push(self.eval_expr(it, env)?);
                }
                // Best-effort memory accounting (spec §5.4): ~one word per slot plus the header.
                self.charge_alloc((items.len() as u64).saturating_mul(16).saturating_add(16))?;
                let v = Value::List(Rc::new(std::cell::RefCell::new(vs)));
                self.note_alloc(&v); // 10d: a mutable cell a cycle can pass through
                Ok(v)
            }
            Expr::Record { path, fields, .. } => {
                let name: Rc<str> = Rc::from(path.segs.last().unwrap().name.as_str());
                let mut fs = Vec::new();
                for (fname, fexpr) in fields {
                    fs.push((fname.name.clone(), self.eval_expr(fexpr, env)?));
                }
                self.charge_alloc((fields.len() as u64).saturating_mul(16).saturating_add(16))?;
                let v = Value::Record { name, fields: Rc::new(std::cell::RefCell::new(fs)) };
                self.note_alloc(&v); // 10d
                Ok(v)
            }
            Expr::Call { callee, args, span, .. } => self.eval_call(callee, args, *span, env),
            Expr::Method { recv, name, args, span, id } => self.eval_method(recv, name, args, *span, *id, env),
            Expr::Field { recv, name, .. } => {
                let r = self.eval_expr(recv, env)?;
                self.field(r, &name.name, name.span)
            }
            Expr::Index { recv, index, .. } => {
                let r = self.eval_expr(recv, env)?;
                let i = self.eval_expr(index, env)?;
                self.index(r, i, recv.span())
            }
            Expr::Unary { op, operand, span, .. } => {
                let v = self.eval_expr(operand, env)?;
                self.unary(*op, v, *span)
            }
            Expr::Binary { op, lhs, rhs, span, .. } => self.binary(*op, lhs, rhs, *span, env),
            Expr::If { cond, then_, else_, .. } => {
                match self.eval_expr(cond, env)? {
                    Value::Bool(true) => self.exec_block_value(then_, env),
                    _ => match else_ {
                        Some(e) => self.eval_expr(e, env),
                        None => Ok(Value::Unit),
                    },
                }
            }
            Expr::Match { scrutinee, arms, span, .. } => self.eval_match(scrutinee, arms, *span, env),
            Expr::Lambda { params, body, .. } => {
                let v = Value::Closure(Rc::new(Closure {
                    params: params.iter().map(|p| p.name.name.clone()).collect(),
                    body: body.clone(),
                    env: env.clone(),
                }));
                // 10d: the captured scope is where a `var`-holds-its-own-closure cycle lives.
                self.note_alloc(&v);
                Ok(v)
            }
            Expr::Try { inner, span, .. } => {
                let v = self.eval_expr(inner, env)?;
                match v {
                    Value::Variant { ref name, ref fields } if &**name == "Ok" => {
                        Ok(fields.first().cloned().unwrap_or(Value::Unit))
                    }
                    Value::Variant { ref name, .. } if &**name == "Err" => Err(Escape::Propagate(v.clone())),
                    _ => Err(Escape::Fault(Fault::at("DL0907", "`?` on a non-Result value", *span))),
                }
            }
            Expr::Block(b) => self.exec_block_value(b, env),
            // Stage 7: `consume`/`recover` are static disciplines — at runtime `consume x`
            // is x's value (the binding-kill is the checker's job) and a recover block just
            // evaluates (the environment restriction is the checker's job).
            Expr::Consume { name, span, .. } => {
                self.eval_var(&Path { segs: vec![name.clone()] }, *span, env)
            }
            Expr::Recover { body, .. } => self.exec_block_value(body, env),
            // T-Spawn at runtime (7g): mint an address, enqueue the Create (the constructor
            // runs as the new actor's first turn on ITS worker), yield the tag reference.
            Expr::Spawn { actor, args, span, .. } => {
                let Some(host) = &self.actors_host else {
                    return Err(Escape::Fault(Fault::at(
                        "DL0907",
                        "`spawn` without an actor system attached (this entry point does not run actors)",
                        *span,
                    )));
                };
                let name = &actor.segs.last().expect("path has segments").name;
                let mut msgs = Vec::with_capacity(args.len());
                for a in args {
                    let v = self.eval_expr(a, env)?;
                    self.debug_check_iso(a, &v).map_err(Escape::Fault)?;
                    msgs.push(self.value_to_msg_here(&v).map_err(Escape::Fault)?);
                }
                let (sender, sender_member) = self.sender_identity();
                let cause = crate::actors::SendCause {
                    sender,
                    sender_member,
                    send_span: Some((span.file, span.start, span.end)),
                };
                // Stage 10 (10c): the declaration's `(mailbox = N)` rides along; the host
                // resolves it against the manifest default (decl wins; nothing = unbounded).
                let decl_bound = self.actors.get(name.as_str()).and_then(|d| d.mailbox);
                let id = host.spawn(name, msgs, cause, decl_bound);
                Ok(Value::ActorRef { id, actor: Rc::from(name.as_str()) })
            }
        }
    }

    fn lit(&self, kind: &LitKind) -> Value {
        match kind {
            LitKind::Int(i) => Value::Int(*i),
            LitKind::Float(f) => Value::Float(*f),
            LitKind::Str(s) => Value::str(s.clone()),
            LitKind::Bool(b) => Value::Bool(*b),
        }
    }

    fn eval_var(&self, path: &Path, span: delulu_diag::Span, env: &Env) -> R<Value> {
        let name = &path.segs[0].name;
        if let Some(v) = env.get(name) {
            return Ok(v);
        }
        // A nullary variant constructor as a value (`None`, a user enum's `Red`, prelude `NotFound`).
        // The checker has already resolved it, so a capitalized unbound name is a nullary variant.
        if name.chars().next().is_some_and(char::is_uppercase) {
            return Ok(Value::variant(name, vec![]));
        }
        // A top-level function used AS A VALUE — `apply(double, 21)`, `xs.map(double)`.
        //
        // The checker types this fine (that is the whole point of row polymorphism), and a
        // lambda in the same position has always worked; only a *named* function was missing
        // here, so every such program checked clean and then faulted at runtime with "unbound
        // name". The reference sample for row polymorphism is one of these, and it went
        // unnoticed because the Book's samples were checked and never run.
        //
        // A named function closes over the globals — exactly the environment `call_fn` builds
        // for a direct call — so calling it through this value and calling it by name are the
        // same computation. See `HARDENING_CAMPAIGN.md` C13.
        if let Some(f) = self.funcs.get(name) {
            let v = Value::Closure(Rc::new(Closure {
                params: f.params.iter().map(|p| p.name.name.clone()).collect(),
                body: f.body.clone(),
                env: Scope::child(&self.globals),
            }));
            self.note_alloc(&v);
            return Ok(v);
        }
        Err(Escape::Fault(Fault::at("DL0907", format!("unbound name `{name}`"), span)))
    }

    fn eval_call(&self, callee: &Expr, args: &[Expr], span: delulu_diag::Span, env: &Env) -> R<Value> {
        // Evaluate arguments left to right.
        let mut argvals = Vec::with_capacity(args.len());
        for a in args {
            argvals.push(self.eval_expr(a, env)?);
        }
        if let Expr::Var { path, .. } = callee {
            if path.segs.len() == 1 {
                let name = &path.segs[0].name;
                // A locally-bound closure value takes precedence (e.g. `f` inside `apply`).
                if let Some(Value::Closure(clo)) = env.get(name) {
                    return self.call_closure(&clo, argvals);
                }
                // P2 — `load(host, path, grant)` is handled HERE rather than in `prim`, for the
                // reason `root.foreign` is: it mints a custody node and reads a container through the
                // injected engine, and `prim` has neither.
                if name == "load" && env.get(name).is_none() && !self.funcs.contains_key(name) {
                    let v = self.load_plugin(&argvals, span).map_err(Escape::Fault)?;
                    self.note_alloc(&v);
                    return Ok(v);
                }
                // Prelude builtins/constructors.
                if let Some(res) = prim::call_builtin(name, &argvals, span) {
                    if let Ok(v) = &res {
                        self.note_alloc(v); // 10d
                    }
                    return res.map_err(Escape::Fault);
                }
                // A user function.
                if self.funcs.contains_key(name) {
                    return self.call_fn(name, argvals);
                }
                // A variant constructor with fields (`Say(x)`, `Other(m)`, …) — capitalized, and not
                // a builtin/closure/fn. The checker has already validated it.
                if name.chars().next().is_some_and(char::is_uppercase) {
                    return Ok(Value::variant(name, argvals));
                }
            }
        }
        // Otherwise the callee must evaluate to a closure.
        match self.eval_expr(callee, env)? {
            Value::Closure(clo) => self.call_closure(&clo, argvals),
            // P2: calling a plugin export. The liveness re-check happens on every call (R-6c) rather
            // than once at `get`, which is the whole point: an `unload` or a `grants revoke` kills
            // every retained callable without the holder having to notice.
            Value::PluginFn(f) => self.call_plugin_export(&f, argvals, span),
            other => Err(Escape::Fault(Fault::at("DL0907", format!("value `{}` is not callable", other.display()), span))),
        }
    }

    fn eval_method(&self, recv: &Expr, name: &Ident, args: &[Expr], span: delulu_diag::Span, node_id: NodeId, env: &Env) -> R<Value> {
        let recvv = self.eval_expr(recv, env)?;

        // P3 (D-V2-29): the other closure-taking List methods. They are here, beside `map`, for the
        // same reason `map` is — they call back into evaluation, so they cannot live in `prim.rs`
        // beside the first-order methods.
        //
        // Each one SNAPSHOTS the list before iterating, exactly as `map` does. A callback that
        // pushes onto the list it is filtering would otherwise be iterating a `RefCell` it is also
        // mutating: either a panic on the borrow, or a loop whose length changes underneath it.
        // Snapshotting makes the answer "the list as it was when the call began", which is a
        // definition a reader can rely on, and it is the same definition `map` already has.
        if matches!(name.name.as_str(), "filter" | "find" | "fold") {
            if let Value::List(items) = &recvv {
                let cb_at = if name.name == "fold" { 1 } else { 0 };
                let f = self.eval_expr(&args[cb_at], env)?;
                let Value::Closure(clo) = f else {
                    return Err(Escape::Fault(Fault::at(
                        "DL0907",
                        format!("List.{} expects a function (checker bug)", name.name),
                        span,
                    )));
                };
                let snapshot: Vec<Value> = items.borrow().clone();
                match name.name.as_str() {
                    "filter" => {
                        let mut out = Vec::new();
                        for it in snapshot {
                            match self.call_closure(&clo, vec![it.clone()])? {
                                Value::Bool(true) => out.push(it),
                                Value::Bool(false) => {}
                                other => {
                                    return Err(Escape::Fault(Fault::at(
                                        "DL0907",
                                        format!("List.filter needs a Bool, got `{}` (checker bug)", other.display()),
                                        span,
                                    )))
                                }
                            }
                        }
                        let v = Value::List(Rc::new(std::cell::RefCell::new(out)));
                        self.note_alloc(&v);
                        return Ok(v);
                    }
                    // Short-circuits on the first match, which is observable through the callback's
                    // effects: `find` over a printing predicate prints once per element examined and
                    // then stops. That is the point of `find` over `filter`, so it is a promise the
                    // reference makes rather than an implementation detail.
                    "find" => {
                        for it in snapshot {
                            match self.call_closure(&clo, vec![it.clone()])? {
                                Value::Bool(true) => return Ok(Value::variant("Some", vec![it])),
                                Value::Bool(false) => {}
                                other => {
                                    return Err(Escape::Fault(Fault::at(
                                        "DL0907",
                                        format!("List.find needs a Bool, got `{}` (checker bug)", other.display()),
                                        span,
                                    )))
                                }
                            }
                        }
                        return Ok(Value::variant("None", vec![]));
                    }
                    // `fold(init, f)` — left fold, `f(acc, x)`. The argument order is the reason
                    // `higher_order_callback_arg` returns a POSITION: the callback is argument 1.
                    _ => {
                        let mut acc = self.eval_expr(&args[0], env)?;
                        for it in snapshot {
                            acc = self.call_closure(&clo, vec![acc, it])?;
                        }
                        self.note_alloc(&acc);
                        return Ok(acc);
                    }
                }
            }
        }

        // Closure-taking methods are handled here (they call back into evaluation).
        if name.name == "map" {
            match &recvv {
                Value::List(items) => {
                    let f = self.eval_expr(&args[0], env)?;
                    let Value::Closure(clo) = f else {
                        return Err(Escape::Fault(Fault::at("DL0907", "map expects a function", span)));
                    };
                    let mut out = Vec::new();
                    let snapshot: Vec<Value> = items.borrow().clone();
                    for it in snapshot {
                        out.push(self.call_closure(&clo, vec![it])?);
                    }
                    let v = Value::List(Rc::new(std::cell::RefCell::new(out)));
                    self.note_alloc(&v); // 10d
                    return Ok(v);
                }
                Value::Secret(s) => {
                    let f = self.eval_expr(&args[0], env)?;
                    let Value::Closure(clo) = f else {
                        return Err(Escape::Fault(Fault::at("DL0907", "Secret.map expects a function", span)));
                    };
                    let inner = Value::str(s.reveal());
                    let mapped = self.call_closure(&clo, vec![inner])?;
                    // Stage 1 supports Secret[Str].map(fn(Str) -> Str).
                    return match mapped {
                        Value::Str(x) => Ok(Value::Secret(Rc::new(crate::value::SecretVal::new(x.to_string())))),
                        _ => Err(Escape::Fault(Fault::at("DL0907", "Stage-1 Secret.map supports Str results only", span))),
                    };
                }
                _ => {}
            }
        }

        let mut argvals = Vec::with_capacity(args.len());
        for a in args {
            argvals.push(self.eval_expr(a, env)?);
        }
        self.trace_dispatch(&recvv, name, &argvals, span);
        // Custody gate (Stage 5 phase 5f): authorize the effectful op through the trait BEFORE
        // performing it. Embedded custody always allows (the in-process `prim` scope check remains the
        // enforcement — criterion 11); daemon custody round-trips synchronous ops to the broker and
        // validates epoch ops against the cached snapshot, so a revoked/expired lease faults here
        // (DL1403/DL1402) and an unreachable broker faults DL1401 (fail closed, invariant 27).
        if let Some((op, arg)) = custody_op_for(&recvv, &name.name, &argvals) {
            if let CustodyDecision::Deny(d) = self.custody.borrow_mut().check(op, arg.as_deref()) {
                // `Actuate` is the one custody refusal that is a VALUE rather than a fault, and the
                // asymmetry is deliberate (10e's law, carried up a layer). Every other denial here
                // means the program asked for something it never held; this one can also mean an
                // operator hit e-stop a millisecond ago. A supervisor holding four arms must lose
                // the revoked one and keep parking the other three — killing the process would
                // strand them at whatever they were doing, which is the opposite of an e-stop.
                if op == CustodyOp::Actuate {
                    let device = arg.as_deref().unwrap_or("<device>");
                    self.trace_actuate_refusal(device, "command.revoked", &d.message, span);
                    return Ok(Value::err(Value::variant("LeaseRevoked", vec![Value::str(d.message)])));
                }
                return Err(Escape::Fault(Fault::at(d.code, d.message, span)));
            }
        }
        // T-Send at runtime (7g): any method on an actor REFERENCE is a behavior send (sync
        // fns on references are checker-refused), and a behavior named on the SELF record is
        // a self-send; a sync fn named on the self record evaluates synchronously in-turn.
        match &recvv {
            Value::ActorRef { id, actor } => {
                let Some(host) = &self.actors_host else {
                    return Err(Escape::Fault(Fault::at(
                        "DL0907",
                        "behavior send without an actor system attached",
                        span,
                    )));
                };
                let mut msgs = Vec::with_capacity(argvals.len());
                for (a, v) in args.iter().zip(&argvals) {
                    self.debug_check_iso(a, v).map_err(Escape::Fault)?;
                    msgs.push(self.value_to_msg_here(v).map_err(Escape::Fault)?);
                }
                let (sender, sender_member) = self.sender_identity();
                let cause = crate::actors::SendCause {
                    sender,
                    sender_member,
                    send_span: Some((span.file, span.start, span.end)),
                };
                // Stage 10 (10c): an overflow drop under `drop-new` is DL1902 in abort mode —
                // telemetry-class otherwise (counted, reported at exit), per spec §10.
                if host.send(*id, &name.name, msgs, cause) == crate::actors::SendOutcome::DroppedOverflow
                    && host.abort_on_death()
                {
                    return Err(Escape::Fault(Fault::at(
                        "DL1902",
                        "mailbox overflow: message dropped (`drop-new` policy, abort mode)",
                        span,
                    )));
                }
                let _ = actor;
                return Ok(Value::Unit);
            }
            Value::Record { name: rname, fields } => {
                if let Some(decl) = self.actors.get(&**rname).cloned() {
                    let is_self = self
                        .current_self
                        .borrow()
                        .as_ref()
                        .is_some_and(|(s, _, _, _)| matches!(s, Value::Record { fields: sf, .. } if Rc::ptr_eq(sf, fields)));
                    if is_self {
                        if let Some(f) = decl.fns.iter().find(|f| f.name.name == name.name) {
                            // T-SyncMethod: runs within the actor's own turn.
                            let fenv = Scope::child(&self.globals);
                            fenv.define("self", recvv.clone());
                            for (p, v) in f.params.iter().zip(argvals) {
                                fenv.define(&p.name.name, v);
                            }
                            self.enter()?;
                            let r = self.finish_call(self.exec_block_value(&f.body, &fenv));
                            self.leave();
                            return r;
                        }
                        if decl.behaviors.iter().any(|b| b.name.name == name.name) {
                            // A self-send: enqueue to our own mailbox — a later turn, never
                            // reentrant (behaviors are atomic, invariant 34).
                            let Some(host) = &self.actors_host else {
                                return Err(Escape::Fault(Fault::at(
                                    "DL0907",
                                    "self-send without an actor system attached",
                                    span,
                                )));
                            };
                            let self_id = self.current_self.borrow().as_ref().map(|(_, id, _, _)| *id);
                            let id = self_id.expect("turn context has an address");
                            let mut msgs = Vec::with_capacity(argvals.len());
                            for (a, v) in args.iter().zip(&argvals) {
                                self.debug_check_iso(a, v).map_err(Escape::Fault)?;
                                msgs.push(self.value_to_msg_here(v).map_err(Escape::Fault)?);
                            }
                            let (sender, sender_member) = self.sender_identity();
                            let cause = crate::actors::SendCause {
                                sender,
                                sender_member,
                                send_span: Some((span.file, span.start, span.end)),
                            };
                            if host.send(id, &name.name, msgs, cause)
                                == crate::actors::SendOutcome::DroppedOverflow
                                && host.abort_on_death()
                            {
                                return Err(Escape::Fault(Fault::at(
                                    "DL1902",
                                    "mailbox overflow: message dropped (`drop-new` policy, abort mode)",
                                    span,
                                )));
                            }
                            return Ok(Value::Unit);
                        }
                    }
                }
            }
            _ => {}
        }
        let result = match &recvv {
            // T-ForeignBind (spec §4): binding a lib is handled here, not in `prim`, because it needs
            // the checker's bind-site → lib map, the grant paths, and the native loader.
            Value::Root(_) if name.name == "foreign" => self.bind_foreign(node_id, span),
            // PS-A-03: minting is on the seam too — a guest that could mint its own capabilities
            // would be granting itself authority. `LocalSink` calls the primitive table as before.
            Value::Root(r) => self.effects.root_method(r, &name.name, &argvals, span),
            // `Secret.expose` (Declassify): in daemon mode the receiver is an opaque broker HANDLE and
            // the bytes cross for the first time here, via `Custody::expose` (audited with the span).
            // In embedded mode the secret is local and reveals in-process (Stage 1–4 behavior).
            Value::Secret(s) if name.name == "expose" => self.expose_secret(s, span),
            // T-ForeignCall (spec §4): a method on a bound lib handle marshals + calls foreign code.
            Value::Foreign(h) => self.call_foreign(h, &name.name, &argvals, span),
            // T-Py (spec §5): a `Cap[Python]` operation (import/of_*/list/to_*) runs the embedded
            // interpreter under the GIL; handled here (not `prim`) to trace `ForeignCall` and thread
            // the `--foreign-max-ret` message bound.
            Value::Cap(c) if c.kind == ResourceKind::Python => self.call_python(c, &name.name, &argvals, span),
            // Stage 10 (10e): `Cap[Actuator]` is handled here, not in `prim`, because an envelope
            // refusal must append its DL1904 record to the trace sink (the DL1305 pattern).
            Value::Cap(c) if c.kind == ResourceKind::Actuator => self.call_actuator(c, &name.name, &argvals, span),
            // Stage 10 (10f): likewise routed here rather than through `prim`, because the reading
            // comes from the run's bound adapter and `prim` has no way to reach it.
            Value::Cap(c) if c.kind == ResourceKind::Sensor => self.call_sensor(c, &name.name, span),
            // Stage 10 (10h): compute dispatch, same reason — the adapter lives on the run.
            Value::Cap(c) if c.kind == ResourceKind::Compute => {
                self.call_compute(c, &name.name, &argvals, span)
            }
            // PS-A-01: the seam. `LocalSink` calls `prim::call_cap_method` — the same call this line
            // made before — so the local path is unchanged; a guest sends it to the host instead.
            Value::Cap(c) => self.effects.cap_method(c, &name.name, &argvals, span),
            // T-Py (spec §5): a `PyObj` operation (attr/call/call_method/index). Present only with the
            // `python` feature — with it off no `Cap[Python]` exists, so no `PyObj` value is ever made.
            #[cfg(feature = "python")]
            Value::PyObj(o) => self.call_pyobj(o, &name.name, &argvals, span),
            // P2: the loaded plugin's two methods (spec §3.3). Here rather than in `prim` because
            // both touch custody: `get` re-checks the node's liveness, `unload` revokes it.
            Value::Plugin(p) => match name.name.as_str() {
                "get" => self.plugin_get(p, &argvals, span),
                "unload" => self.plugin_unload(p),
                other => Err(Fault::at("DL0907", format!("a plugin has no method `{other}`"), span)),
            },
            Value::Secret(s) => prim::call_secret_method(s, &name.name, &argvals, span),
            Value::Str(s) => prim::call_str_method(s, &name.name, &argvals, span),
            Value::List(l) => prim::call_list_method(l, &name.name, &argvals, span),
            Value::Map(m) => prim::call_map_method(m, &name.name, &argvals, span),
            other => Err(Fault::at("DL0907", format!("type has no method `{}` on `{}`", name.name, other.display()), span)),
        };
        // 10d: a primitive may mint a fresh mutable cell (`split`, `slice`, …) — register its
        // top-level cell. Anything nested was either built at eval sites (already registered)
        // or is a clone of an existing registered cell.
        if let Ok(v) = &result {
            self.note_alloc(v);
        }
        result.map_err(Escape::Fault)
    }

    /// Append a `TraceRecord` at the dispatch point (spec §6.1) if tracing is enabled and this
    /// (receiver kind, method) pair is effectful. Secret redaction: if the receiver or any
    /// argument is a `Secret`, `detail` is always `trace::OPAQUE` — the raw value never reaches
    /// the trace, regardless of what a human-useful detail would otherwise show.
    fn trace_dispatch(&self, recvv: &Value, name: &Ident, argvals: &[Value], span: delulu_diag::Span) {
        let Some(sink) = &self.trace else { return };
        let cap_kind = match recvv {
            Value::Cap(c) => Some(c.kind.name()),
            Value::Secret(_) => Some("Secret"),
            _ => None,
        };
        let Some(kind) = cap_kind else { return };
        let Some(effect) = trace::effect_for(kind, &name.name) else { return };
        let involves_secret =
            matches!(recvv, Value::Secret(_)) || argvals.iter().any(|v| matches!(v, Value::Secret(_)));
        let detail = if involves_secret { Some(trace::OPAQUE.to_string()) } else { trace_detail(recvv, kind, &name.name, argvals) };
        // REMAINING_WORK 6.13: a filesystem record also says where the path pointed — the same
        // resolution the custody gate authorizes (`fs_scope_arg`) — and the scope it was resolved
        // in. Never for a secret-involving call (redaction outranks explanation).
        let (resolved_path, scope_root) = match recvv {
            Value::Cap(c) if !involves_secret && matches!(c.kind, ResourceKind::FsRead | ResourceKind::FsWrite) => match &c.scope {
                CapScope::Fs { root, .. } => {
                    (fs_scope_arg(&c.scope, argvals), Some(root.to_string_lossy().to_string()))
                }
                _ => (None, None),
            },
            _ => (None, None),
        };
        sink.push(TraceRecord {
            seq: self.next_trace_seq(),
            effect: effect.to_string(),
            op: name.name.clone(),
            cap_kind: kind.to_string(),
            detail,
            span: Some((span.file, span.start, span.end)),
            resolved_path,
            scope_root,
            ..self.trace_attrib_record()
        });
    }

    // ----- foreign C FFI (Stage 4, spec §4) -------------------------------

    /// `root.foreign(load)` — bind the lib this call site names (checker-resolved), returning
    /// `Result[M, ForeignErr]`. All declared symbols resolve now (fail-fast, spec §4.2 / trap 4): a
    /// missing symbol is `ForeignErr::SymbolMissing` (DL1304), a lib that will not load is
    /// `ForeignErr::Unavailable`, and — belt-and-suspenders behind the CLI grant pre-flight — an
    /// ungranted lib is `ForeignErr::NotGranted` (DL1303).
    fn bind_foreign(&self, node_id: NodeId, span: delulu_diag::Span) -> Result<Value, Fault> {
        let Some(lib_name) = self.foreign_binds.get(&node_id).cloned() else {
            return Err(Fault::at("DL0907", "foreign bind site with no resolved lib (checker/wiring bug)", span));
        };
        // Custody gate (Stage 5 phase 5f): `ForeignBind` is synchronous-class — authorize the bind
        // through the broker (daemon) before loading any native code. Embedded custody always allows.
        if let CustodyDecision::Deny(d) = self.custody.borrow_mut().check(CustodyOp::ForeignBind, Some(&lib_name)) {
            return Err(Fault::at(d.code, d.message, span));
        }
        // The path is grant data (spec §4.1). No grant → NotGranted; the CLI normally refuses at
        // startup (criterion 4) so a program that reaches here has already been granted.
        let Some(path) = self.foreign_grants.get(&lib_name).cloned() else {
            return Ok(Value::err(Value::variant("NotGranted", vec![])));
        };
        let sigs = self.foreign_blocks.get(&lib_name).cloned().unwrap_or_default();
        // Stage 5 phase 5h: the binder decides in-process vs. isolated worker. A worker that fails to
        // spawn/load surfaces as a catchable `ForeignErr` bind result (e.g. `Unavailable`) — exactly
        // like an in-process load failure — never a host-process crash.
        match self.foreign_binder.bind(&lib_name, &path, sigs, self.foreign_max_ret) {
            Ok(exec) => Ok(Value::ok(Value::Foreign(Rc::new(ForeignHandle { name: lib_name, exec })))),
            Err(e) => Ok(Value::err(foreign_err_value(&e))),
        }
    }

    /// P2 — `load(host, path, grant)`: the Stage-6 load sequence, driven from a running program.
    ///
    /// Handled here rather than in `prim` for the reason `bind_foreign` is: it needs custody (step 4
    /// mints a child node) and the engine, and `prim` has neither.
    ///
    /// The order is the normative one, and the two checks that come BEFORE any of it are the ones this
    /// phase adds:
    ///
    /// 1. **Where.** The path is resolved inside the capability's granted roots by the same lexical
    ///    containment `fs.*` uses. A path outside them is refused before a byte is read — so `..`, a
    ///    symlink spelling or a case difference cannot reach an artifact the operator did not permit.
    /// 2. **Which.** When the package pinned hashes (`[plugins] allow`), the blake3 of the bytes IN
    ///    HAND must be on the list. On the bytes, not on the path: an artifact swapped between
    ///    `plugin verify` and this load is the TOCTOU case, and a path is not a name for bytes.
    ///
    /// Then `load_verified` runs steps 1–6 exactly as `plugin verify` does, with the same functions,
    /// so `verify` and `load` cannot reach different verdicts on the same artifact.
    ///
    /// Every refusal reaches the program as a `PluginErr` VALUE, never a fault: the point of the load
    /// sequence is that a host can decide what to do when a plugin will not load.
    fn load_plugin(&self, args: &[Value], span: delulu_diag::Span) -> Result<Value, Fault> {
        use crate::plugin::{LoadedHandle, PluginErr};

        let Some(Value::Cap(host)) = args.first() else {
            return Err(Fault::at("DL0907", "`load` expects a Cap[PluginHost] as its first argument", span));
        };
        let crate::value::CapScope::PluginHost { roots, allow_hashes } = &host.scope else {
            // A host-held handle (a sandbox guest) or any other scope: the channel carries no plugin
            // request kind, and a guest loading code is exactly what the sandbox refuses.
            return Err(Fault::at(
                "DL1401",
                "this `Cap[PluginHost]` carries no granted roots — a sandboxed guest cannot load code, \
                 and the run is refused before it starts rather than reaching here",
                span,
            ));
        };
        let Some(Value::Str(path)) = args.get(1) else {
            return Err(Fault::at("DL0907", "`load` expects a Str path as its second argument", span));
        };
        let grant = match args.get(2) {
            Some(v) => grant_from_value(v),
            None => return Err(Fault::at("DL0907", "`load` expects a Grant as its third argument", span)),
        };

        // ----- 1. where ---------------------------------------------------------------------------
        // The same rule a `Cap[FsRead]` operation obeys: resolve inside the granted root, lexically,
        // and refuse anything that escapes. `hostile_path` first, because a spelling the primitive
        // table refuses from a program is refused here too (D-NE-29) — this is the path that decides
        // which code runs.
        if let Some(why) = crate::prim::hostile_path(path, false) {
            return Ok(Value::err(plugin_err_value(&PluginErr::NotGranted(format!(
                "the path `{path}` is refused: {why}"
            )))));
        }
        let mut found: Option<std::path::PathBuf> = None;
        for root in roots {
            let candidate = crate::prim::resolve_norm(root, path);
            // Inside the root, or the root itself when the grant named a single file.
            if candidate.starts_with(root) || &candidate == root {
                found = Some(candidate);
                break;
            }
        }
        let Some(file) = found else {
            return Ok(Value::err(plugin_err_value(&PluginErr::NotGranted(format!(
                "`{path}` is outside every granted plugin root ({}) — pass `--grant plugin=<dir>` for \
                 the directory the artifact is in",
                roots.iter().map(|r| r.display().to_string()).collect::<Vec<_>>().join(", ")
            )))));
        };
        let bytes = match std::fs::read(&file) {
            Ok(b) => b,
            Err(e) => {
                return Ok(Value::err(plugin_err_value(&PluginErr::BadArtifact(format!(
                    "cannot read `{}`: {e}",
                    file.display()
                )))))
            }
        };

        // ----- 2. which ---------------------------------------------------------------------------
        // The hash of the bytes in hand. A path names a location; only a hash names bytes, and the
        // bytes are what will run.
        if !allow_hashes.is_empty() {
            // The same hashing function the audit chain and the compute artifact registry use, so a
            // hash an operator copies out of one place matches in the other.
            let digest = delulu_broker::content_hash(&bytes);
            if !allow_hashes.iter().any(|h| h == &digest) {
                return Ok(Value::err(plugin_err_value(&PluginErr::NotGranted(format!(
                    "the artifact at `{path}` hashes to {digest}, which this package's `[plugins] allow` \
                     does not list — the manifest pins artifacts by their bytes, so a swapped file is \
                     refused even at a permitted path"
                )))));
            }
        }

        // ----- the sequence -----------------------------------------------------------------------
        let Some(engine) = self.plugin_engine.clone() else {
            return Err(Fault::at(
                "DL0907",
                "no plugin engine is wired into this run (a wiring bug, not a refusal — the CLI always                  supplies one, and saying `bad artifact` here would blame the artifact for the host)",
                span,
            ));
        };
        let art = match engine.read_artifact(&bytes) {
            Ok(a) => a,
            Err(r) => return Ok(Value::err(plugin_err_value(&r.to_plugin_err()))),
        };
        let exports = art.exports();
        let name = art.name().to_string();
        // P2-05: resource limits are REFUSED rather than ignored.
        //
        // `Grant.limits` (fuel, memory, wall clock) are the WASM engine's instruments, and a Verified
        // plugin does not run on the WASM engine — it runs its DIR on the interpreter, which has no
        // fuel meter and no preemption. A grant asking for a 100 ms wall limit that nothing enforces is
        // a promise nobody keeps, and the one outcome this project refuses is a boundary that quietly
        // did not apply. So a non-zero limit refuses, and says which instrument is missing.
        //
        // Zero means "no limit requested", which is what the flagship example passes, so an ordinary
        // load is unaffected. The Contained class is where these limits belong and where
        // `kill_on_limit` already lives; it arrives with the WASM execution path.
        let l = &grant.limits;
        if l.fuel != 0 || l.mem_mb != 0 || l.wall_ms != 0 {
            return Ok(Value::err(plugin_err_value(&PluginErr::NotGranted(format!(
                "this grant asks for resource limits (fuel {}, memory {} MiB, wall {} ms) and this build                  cannot enforce them: a Verified plugin executes its DIR on the interpreter, which has                  no fuel meter and no preemption. They are refused rather than ignored — a limit nothing                  enforces is worse than no limit, because it reads as one. Pass zeros, or keep the work                  in the host.",
                l.fuel, l.mem_mb, l.wall_ms
            )))));
        }
        // The two dimensions whose enforcement lives in CUSTODY rather than in the primitive table.
        // A plugin export runs in its own interpreter (see `call_plugin_export`), and custody cannot
        // be shared between two of them — so a grant carrying either of these would have its broker
        // decision made by a fresh embedded custody that always allows. Refused here, before the node
        // is minted, with the reason: this is a limitation of the plugin execution model, not of the
        // artifact, and pretending otherwise would be the silent downgrade this project refuses.
        for dim in ["Declassify", "ForeignCall"] {
            if grant.effects.iter().any(|e| e == dim) {
                return Ok(Value::err(plugin_err_value(&PluginErr::NotGranted(format!(
                    "a plugin grant cannot carry `{dim}` in this build: its enforcement lives in                      custody, and a plugin export runs in its own interpreter which cannot share the                      host's. Remove it from the grant, or keep that work in the host."
                )))));
            }
        }
        let mut custody = self.custody.borrow_mut();
        match crate::plugin::load_verified(&art, &grant, &mut **custody) {
            Ok(crate::plugin::LoadedPlugin::Verified { grant_id, authority, verified, signer }) => {
                // Drop the custody borrow before tracing: `trace_load` takes `&self`, and holding a
                // `RefCell` borrow across an unrelated call is how a re-entrant path panics later.
                drop(custody);
                self.trace_load(&name, grant_id.as_str(), span);
                Ok(Value::ok(Value::Plugin(Rc::new(LoadedHandle {
                    grant_id,
                    authority,
                    verified: Rc::from(*verified),
                    exports,
                    name,
                    signer,
                }))))
            }
            Err(r) => Ok(Value::err(plugin_err_value(&r.to_plugin_err()))),
        }
    }

    /// `p.get(name)` — one export as a callable, or a `PluginErr`.
    ///
    /// The liveness re-check is here as well as at the call (R-6c): a `get` on an already-unloaded
    /// plugin must not hand back a callable that looks usable.
    fn plugin_get(&self, p: &Rc<crate::plugin::LoadedHandle>, args: &[Value], span: delulu_diag::Span) -> Result<Value, Fault> {
        use crate::plugin::{PluginErr, PluginFn, PluginRef};
        let Some(Value::Str(export)) = args.first() else {
            return Err(Fault::at("DL0907", "`get` expects a Str export name", span));
        };
        let reference = PluginRef::new(p.grant_id.clone(), export.to_string());
        if let Err(e) = reference.check_call(&**self.custody.borrow()) {
            return Ok(Value::err(plugin_err_value(&e)));
        }
        if !p.exports.contains_key(export.as_ref()) {
            return Ok(Value::err(plugin_err_value(&PluginErr::NotGranted(format!(
                "plugin `{}` exports no `{export}` (it exports: {})",
                p.name,
                p.exports.keys().cloned().collect::<Vec<_>>().join(", ")
            )))));
        }
        Ok(Value::ok(Value::PluginFn(Rc::new(PluginFn { reference, plugin: p.clone() }))))
    }

    /// P2 — call one export of a loaded plugin.
    ///
    /// **Per call, not per `get`.** The liveness re-check is here (R-6c), so an `unload` or a
    /// `grants revoke` kills every retained callable without the holder having to notice. A reference
    /// binds the load-time `GrantId`, so a reload mints a fresh node and an old callable stays dead
    /// for ever rather than being silently re-bound to different authority.
    ///
    /// **Why a nested interpreter over the plugin's own module.** The plugin's functions must resolve
    /// against the PLUGIN's items — its own helpers, its own consts. Injecting them into the host's
    /// function table would make a bare name inside the plugin resolve against the HOST's functions,
    /// which is a plugin calling code it was never given. So the plugin's module gets its own
    /// interpreter, and what crosses is only what should: the host's trace sink, so the plugin's
    /// effects appear in the host's trace and `--assert-trace` sees them; the host's effect sink, so a
    /// sandboxed run still performs them host-side; and the host's depth bound.
    ///
    /// What does NOT cross is custody, because `Box<dyn Custody>` cannot be shared between two
    /// interpreters — and that is exactly why `load` refuses any grant carrying `Declassify` or
    /// `ForeignCall`. Those are the two dimensions whose enforcement lives in custody rather than in
    /// the primitive table; letting them through here would move a broker decision into a fresh
    /// embedded custody that always allows. Every other dimension is enforced by the capability the
    /// HOST hands in as an argument, checked by the same primitive table on the same resolved paths.
    fn call_plugin_export(
        &self,
        f: &Rc<crate::plugin::PluginFn>,
        args: Vec<Value>,
        span: delulu_diag::Span,
    ) -> R<Value> {
        if let Err(e) = f.reference.check_call(&**self.custody.borrow()) {
            let code = crate::plugin::plugin_err_code(&e);
            return Err(Escape::Fault(Fault::at(
                code,
                format!(
                    "the plugin export `{}` is no longer callable: {e:?} — a reference binds the \
                     load-time grant node, so it stays dead after an unload or a revoke",
                    f.reference.export
                ),
                span,
            )));
        }
        let module = &f.plugin.verified.dir.module;
        let mut sub = Interp::new(module).with_max_depth(self.max_depth);
        if let Some(t) = &self.trace {
            sub = sub.with_trace(t.clone());
        }
        sub = sub.with_effect_sink(self.effects.clone());
        sub.call_exported(&f.reference.export, args, span)
    }

    /// Call a `pub fn` by name from THIS interpreter's module — the entry point a plugin export is
    /// reached through. Separate from `call_fn` because it is the only place an outside caller names a
    /// function, and the refusal for a missing one must say that rather than "unknown function".
    fn call_exported(&self, name: &str, args: Vec<Value>, span: delulu_diag::Span) -> R<Value> {
        if !self.funcs.contains_key(name) {
            return Err(Escape::Fault(Fault::at(
                "DL1508",
                format!("this plugin's module has no function `{name}` (the manifest and the code disagree)"),
                span,
            )));
        }
        self.call_fn(name, args)
    }

    /// `p.unload()` — revoke the plugin's custody node. Every retained callable dies with it, because
    /// each one re-checks the node's liveness on every call rather than trusting the handle it holds.
    fn plugin_unload(&self, p: &Rc<crate::plugin::LoadedHandle>) -> Result<Value, Fault> {
        let mut custody = self.custody.borrow_mut();
        let _ = crate::plugin::unload(&mut **custody, &p.grant_id);
        Ok(Value::Unit)
    }

    /// `m.method(args)` on a bound lib handle — marshal per spec §4.2, call via `libffi`, validate the
    /// return (invariant 21), and record the `ForeignCall` trace event (criterion 1). A return that
    /// fails shape validation surfaces as a defined DL1306 fault: a bare `-> Str`/`-> Float` foreign
    /// signature has no `Result` channel for the program to catch, so the honest outcome is a clean
    /// abort — never UB, never a panic, never a silent truncation (criterion 8).
    fn call_foreign(&self, handle: &Rc<ForeignHandle>, method: &str, args: &[Value], span: delulu_diag::Span) -> Result<Value, Fault> {
        let Some(sig) = handle.exec.sig(method) else {
            return Err(Fault::at("DL0907", format!("unknown foreign method `{method}` (checker bug)"), span));
        };
        let fargs: Vec<FVal> = sig
            .params
            .iter()
            .zip(args)
            .map(|(k, v)| value_to_fval(*k, v))
            .collect();
        self.trace_foreign(&handle.name, method, span);
        match handle.exec.call(method, &fargs, self.foreign_max_ret) {
            Ok(fv) => Ok(fval_to_value(fv)),
            Err(foreign::ForeignErr::BadReturn(reason)) => Err(Fault::at(
                "DL1306",
                format!("foreign return validation failed: {reason} (ForeignErr::BadReturn)"),
                span,
            )),
            // Stage 5 phase 5h headline (spec §5, criterion 7): the isolated worker died mid-call (a C
            // segfault / hard crash). The host process SURVIVED — it turned the worker's death into a
            // clean DL1409 fault instead of dying alongside it.
            Err(foreign::ForeignErr::WorkerDied(reason)) => Err(Fault::at(
                "DL1409",
                format!(
                    "foreign worker died: {reason} (ForeignErr::WorkerDied) — the isolated worker \
                     process crashed; the host survived and reported this cleanly instead of aborting"
                ),
                span,
            )),
            Err(other) => Err(Fault::at("DL1306", format!("foreign call failed: {other:?}"), span)),
        }
    }

    /// Append the `ForeignCall` trace record (criterion 1's "with `ForeignCall` in the trace"). Like
    /// every effect, it goes through the shared seq counter; `cap_kind` names the lib.
    fn trace_foreign(&self, lib: &str, method: &str, span: delulu_diag::Span) {
        let Some(sink) = &self.trace else { return };
        sink.push(TraceRecord {
            seq: self.next_trace_seq(),
            effect: "ForeignCall".to_string(),
            op: method.to_string(),
            cap_kind: lib.to_string(),
            detail: Some(format!("{lib}.{method}")),
            span: Some((span.file, span.start, span.end)),
            ..self.trace_attrib_record()
        });
    }

    /// P2-07: the `Load` trace record — code arrived after compile time, and the trace says so.
    ///
    /// `Load` is deliberately absent from `trace::effect_for`, which maps (capability kind, method)
    /// pairs: a load is not a method on a capability, it is a free call handled in the interpreter, so
    /// it appends its own record exactly as `ForeignCall` does. The `detail` carries the loaded NODE
    /// ID, because that is what a reader needs in order to follow the plugin through the audit chain
    /// and to revoke it — a record saying only "a plugin loaded" answers nothing.
    ///
    /// Appended AFTER the sequence succeeds, and only then: a refused load performed nothing, and a
    /// trace record for it would say code arrived when none did. The refusal is already a value the
    /// program can see and a `channel-violation`-style fact the audit chain carries.
    fn trace_load(&self, plugin: &str, node: &str, span: delulu_diag::Span) {
        let Some(sink) = &self.trace else { return };
        sink.push(TraceRecord {
            seq: self.next_trace_seq(),
            effect: "Load".to_string(),
            op: "load".to_string(),
            cap_kind: "PluginHost".to_string(),
            detail: Some(format!("{plugin} @ {node}")),
            span: Some((span.file, span.start, span.end)),
            ..self.trace_attrib_record()
        });
    }

    // ----- embedded CPython (Stage 4 phase 4f, spec §5) -------------------

    /// A `Cap[Python]` operation (`py.import`/`of_*`/`list`/`to_*`). The `ForeignCall` trace record is
    /// appended BEFORE the call, so a DENIED `py.import` (DL1305) is still visible in the trace
    /// (criterion 5). The op is recorded as `py.<method>`; `import`/`attr`/`call_method` carry the
    /// name argument as `detail`.
    fn call_python(&self, cap: &Rc<CapVal>, method: &str, args: &[Value], span: delulu_diag::Span) -> Result<Value, Fault> {
        self.trace_python(method, python_detail(method, args), span);
        crate::python::call_python_cap(cap, method, args, self.foreign_max_ret, span)
    }

    /// A `PyObj` operation (`attr`/`call`/`call_method`/`index`). Present only with the `python`
    /// feature (a `PyObj` value can exist only when the feature is on).
    #[cfg(feature = "python")]
    fn call_pyobj(&self, obj: &crate::python::PyObjVal, method: &str, args: &[Value], span: delulu_diag::Span) -> Result<Value, Fault> {
        self.trace_python(method, python_detail(method, args), span);
        crate::python::call_pyobj(obj, method, args, self.foreign_max_ret, span)
    }

    /// Append a `ForeignCall` trace record for a `std.py` op (every §5.2 row is `!{ForeignCall}`).
    /// `cap_kind` is the literal `"Python"`; `op` is `py.<method>`.
    fn trace_python(&self, method: &str, detail: Option<String>, span: delulu_diag::Span) {
        let Some(sink) = &self.trace else { return };
        sink.push(TraceRecord {
            seq: self.next_trace_seq(),
            effect: "ForeignCall".to_string(),
            op: format!("py.{method}"),
            cap_kind: "Python".to_string(),
            detail,
            span: Some((span.file, span.start, span.end)),
            ..self.trace_attrib_record()
        });
    }

    /// `Cap[Actuator].command(shape)` — the physical boundary (Stage 10 phase 10e, spec §5.1).
    /// The envelope is enforced HERE, on every command, fail-closed; a refusal is
    /// `Err(Envelope(reason))` — the COMMAND dies, never the process — and appends a DL1904
    /// telemetry record so the refusal is visible evidence, not a silent swallow (the DL1305
    /// denied-attempt pattern). An in-envelope command reaches the null adapter (`Ok(Unit)`)
    /// until the 10f reference simulator gives it somewhere real to go.
    fn call_actuator(&self, cap: &Rc<CapVal>, method: &str, args: &[Value], span: delulu_diag::Span) -> Result<Value, Fault> {
        if method != "command" {
            return Err(Fault::at("DL0907", format!("unknown Actuator method `{method}` (checker bug)"), span));
        }
        let CapScope::Actuator(env) = &cap.scope else {
            return Err(Fault::at("DL0907", "Actuator capability without an envelope scope (wiring bug)", span));
        };
        // The runtime half of the check (10e), against the capability's own scope.
        if let Err(reason) = envelope_check(env, args.first()) {
            // A refused command is still an INTERACTION with the device, and the broker has to be
            // told so even though nothing is being dispatched (C39). Under a stepped clock the
            // simulated time a real controller would have burned here is burned here too, so a
            // program whose every command is refused loses its device on the same schedule the wall
            // clock would enforce — instead of freezing simulated time and holding the machine
            // forever. If that sweep is what killed the lease, the lease is the more important fact:
            // "you no longer hold this device" outranks "your setpoint was out of range", the same
            // ordering `DeviceBroker::command` documents for the accepted path.
            if let Some(broker) = &self.devices {
                if let Some(revoked) = broker.note_refused_attempt(&env.device) {
                    self.trace_actuate_refusal(&env.device, "command.revoked", &revoked, span);
                    return Ok(Value::err(Value::variant("LeaseRevoked", vec![Value::str(revoked)])));
                }
            }
            self.trace_actuate_refusal(&env.device, "command.refused", &reason, span);
            return Ok(Value::err(Value::variant("Envelope", vec![Value::str(reason)])));
        }
        // The broker half (10f): the lease, the rate, and the envelope as the GRANT recorded it.
        // A command that passed the check above can still die here, and that ordering is the
        // point — the capability value is a copy of the authority, never the authority itself.
        let Some(broker) = &self.devices else { return Ok(Value::ok(Value::Unit)) };
        let fields = numeric_fields(args.first());
        match broker.command(&env.device, &fields) {
            Ok(()) => Ok(Value::ok(Value::Unit)),
            Err(crate::device::CommandRefusal::Envelope(reason)) => {
                self.trace_actuate_refusal(&env.device, "command.refused", &reason, span);
                Ok(Value::err(Value::variant("Envelope", vec![Value::str(reason)])))
            }
            Err(crate::device::CommandRefusal::Revoked(reason)) => {
                self.trace_actuate_refusal(&env.device, "command.revoked", &reason, span);
                Ok(Value::err(Value::variant("LeaseRevoked", vec![Value::str(reason)])))
            }
        }
    }

    /// `Cap[Compute].dispatch(kernel, buffer)` (10h, spec §7.1). The refusal channel is a VALUE,
    /// like every other device surface: an over-envelope kernel kills the dispatch, not the host.
    fn call_compute(
        &self,
        cap: &Rc<CapVal>,
        method: &str,
        args: &[Value],
        span: delulu_diag::Span,
    ) -> Result<Value, Fault> {
        if method != "dispatch" {
            return Err(Fault::at("DL0907", format!("unknown Compute method `{method}` (checker bug)"), span));
        }
        let CapScope::Compute(env) = &cap.scope else {
            return Err(Fault::at("DL0907", "Compute capability without an envelope scope (wiring bug)", span));
        };
        let Some(Value::Str(kernel)) = args.first() else {
            return Err(Fault::at("DL0907", "compute.dispatch without a kernel name (checker bug)", span));
        };
        let buffer: Vec<f64> = match args.get(1) {
            Some(Value::List(items)) => items
                .borrow()
                .iter()
                .map(|v| match v {
                    Value::Float(x) => *x,
                    Value::Int(i) => *i as f64,
                    _ => f64::NAN,
                })
                .collect(),
            _ => return Err(Fault::at("DL0907", "compute.dispatch without a buffer (checker bug)", span)),
        };
        // No adapter bound: absence, never a fabricated result. This is invariant 50's rule at
        // silicon — the same reason an unbound sensor reads `NoDevice` instead of a plausible float.
        let Some(broker) = &self.computes else {
            return Ok(Value::err(Value::variant("NoAdapter", vec![])));
        };
        match broker.dispatch(&env.device, kernel, &buffer) {
            Ok(v) => Ok(Value::ok(Value::Float(v))),
            Err(crate::compute::DispatchRefusal::Envelope(reason)) => {
                self.trace_compute_refusal(&env.device, kernel, &reason, span);
                Ok(Value::err(Value::variant("KernelEnvelope", vec![Value::str(reason)])))
            }
            Err(crate::compute::DispatchRefusal::UnknownKernel(reason)) => {
                Ok(Value::err(Value::variant("UnknownKernel", vec![Value::str(reason)])))
            }
        }
    }

    /// The DL1907 telemetry record: an auditor reading the trace sees the dispatch attempt and its
    /// refusal, in order, with the envelope term that caught it.
    fn trace_compute_refusal(&self, device: &str, kernel: &str, reason: &str, span: delulu_diag::Span) {
        let Some(sink) = &self.trace else { return };
        sink.push(TraceRecord {
            seq: self.next_trace_seq(),
            effect: "ForeignCall".to_string(),
            op: "dispatch.refused".to_string(),
            cap_kind: "Compute".to_string(),
            detail: Some(format!("DL1907 {device}/{kernel}: {reason}")),
            span: Some((span.file, span.start, span.end)),
            ..Default::default()
        });
    }

    /// `Cap[Sensor].read()` (10e's shape, 10f's adapter). Absence still reads as absence — the
    /// simulator answers only for devices it actually models, and everything else is `NoDevice`.
    fn call_sensor(&self, cap: &Rc<CapVal>, method: &str, span: delulu_diag::Span) -> Result<Value, Fault> {
        if method != "read" {
            return Err(Fault::at("DL0907", format!("unknown Sensor method `{method}` (checker bug)"), span));
        }
        let CapScope::Sensor { device } = &cap.scope else {
            return Err(Fault::at("DL0907", "Sensor capability without a device scope (wiring bug)", span));
        };
        match self.devices.as_ref().and_then(|b| b.read(device)) {
            Some(x) => Ok(Value::ok(Value::Float(x))),
            None => Ok(Value::err(Value::variant("NoDevice", vec![]))),
        }
    }

    /// Append the DL1904 refusal record. Separate from the ordinary `Actuate` dispatch record
    /// (which `trace_dispatch` already appended): an auditor reading the trace sees BOTH the
    /// attempt and its refusal, in order, with the reason in `detail`.
    fn trace_actuate_refusal(&self, device: &str, op: &str, reason: &str, span: delulu_diag::Span) {
        let Some(sink) = &self.trace else { return };
        sink.push(TraceRecord {
            seq: self.next_trace_seq(),
            effect: "Actuate".to_string(),
            op: op.to_string(),
            cap_kind: "Actuator".to_string(),
            // DL1904 is the ENVELOPE refusal's code. A dead lease is a different event and must
            // not borrow it: an auditor counting DL1904s is counting commands the envelope caught,
            // not devices the operator lost.
            detail: Some(match op {
                "command.refused" => format!("DL1904 {device}: {reason}"),
                _ => format!("{device}: {reason}"),
            }),
            span: Some((span.file, span.start, span.end)),
            ..self.trace_attrib_record()
        });
    }

    /// `Secret.expose(Cap[Declassify])`. Embedded: reveal the local bytes (Stage 1–4). Daemon: the
    /// receiver is an opaque broker handle — fetch the bytes through `Custody::expose` (a synchronous
    /// broker round-trip, audited with the calling span). This is the ONLY place daemon secret bytes
    /// enter the program process (invariant 23 / spec §4.4).
    fn expose_secret(&self, s: &Rc<SecretVal>, span: delulu_diag::Span) -> Result<Value, Fault> {
        match s.handle_name() {
            Some(name) => {
                let span_str = format!("{}:{}:{}", span.file, span.start, span.end);
                match self.custody.borrow_mut().expose(name, Some(&span_str)) {
                    Ok(bytes) => Ok(Value::str(bytes)),
                    Err(d) => Err(Fault::at(d.code, d.message, span)),
                }
            }
            None => prim::call_secret_method(s, "expose", &[], span),
        }
    }

    fn eval_match(&self, scrutinee: &Expr, arms: &[Arm], span: delulu_diag::Span, env: &Env) -> R<Value> {
        let value = self.eval_expr(scrutinee, env)?;
        for arm in arms {
            let arm_env = Scope::child(env);
            if self.match_pattern(&arm.pattern, &value, &arm_env) {
                return self.eval_expr(&arm.body, &arm_env);
            }
        }
        Err(Escape::Fault(Fault::at("DL0907", "no match arm applied (checker guarantees exhaustiveness)", span)))
    }

    fn match_pattern(&self, pat: &Pattern, value: &Value, env: &Env) -> bool {
        match pat {
            Pattern::Wildcard(_) => true,
            Pattern::Bind(name) => {
                env.define(&name.name, value.clone());
                true
            }
            Pattern::Lit(kind, _) => self.lit(kind).eq(value),
            Pattern::Variant { path, fields, .. } => {
                let vname = &path.segs.last().unwrap().name;
                if let Value::Variant { name, fields: vfields } = value {
                    if &**name == vname.as_str() && vfields.len() == fields.len() {
                        return fields.iter().zip(vfields.iter()).all(|(p, v)| self.match_pattern(p, v, env));
                    }
                }
                false
            }
        }
    }

    fn field(&self, recv: Value, field: &str, span: delulu_diag::Span) -> R<Value> {
        if let Value::Record { fields, .. } = &recv {
            for (n, v) in fields.borrow().iter() {
                if n == field {
                    return Ok(v.clone());
                }
            }
        }
        Err(Escape::Fault(Fault::at("DL0907", format!("no field `{field}`"), span)))
    }

    fn index(&self, recv: Value, idx: Value, span: delulu_diag::Span) -> R<Value> {
        match (recv, idx) {
            (Value::List(l), Value::Int(n)) => {
                let b = l.borrow();
                if n < 0 || n as usize >= b.len() {
                    Err(Escape::Fault(Fault::at("DL0903", format!("index {n} out of bounds (len {})", b.len()), span)))
                } else {
                    Ok(b[n as usize].clone())
                }
            }
            _ => Err(Escape::Fault(Fault::at("DL0907", "cannot index this value", span))),
        }
    }

    fn unary(&self, op: UnOp, v: Value, span: delulu_diag::Span) -> R<Value> {
        match (op, v) {
            (UnOp::Neg, Value::Int(i)) => i
                .checked_neg()
                .map(Value::Int)
                .ok_or_else(|| Escape::Fault(Fault::at("DL0901", "integer overflow negating Int::MIN", span))),
            (UnOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
            (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            _ => Err(Escape::Fault(Fault::at("DL0907", "bad operand for unary operator", span))),
        }
    }

    fn binary(&self, op: BinOp, lhs: &Expr, rhs: &Expr, span: delulu_diag::Span, env: &Env) -> R<Value> {
        // Short-circuit logical operators.
        if matches!(op, BinOp::And | BinOp::Or) {
            let l = self.eval_expr(lhs, env)?;
            let lb = matches!(l, Value::Bool(true));
            return match op {
                BinOp::And if !lb => Ok(Value::Bool(false)),
                BinOp::Or if lb => Ok(Value::Bool(true)),
                _ => Ok(self.eval_expr(rhs, env)?),
            };
        }
        let l = self.eval_expr(lhs, env)?;
        let r = self.eval_expr(rhs, env)?;
        use BinOp::*;
        match op {
            Eq => Ok(Value::Bool(l.eq(&r))),
            Ne => Ok(Value::Bool(!l.eq(&r))),
            Add if matches!(l, Value::Str(_)) => {
                // Charge the projected result size (best-effort memory, spec §5.4) BEFORE allocating
                // it, so a runaway doubling concatenation trips the byte budget instead of the heap.
                let ls = l.display();
                let rs = r.display();
                self.charge_alloc((ls.len() + rs.len()) as u64)?;
                Ok(Value::str(format!("{ls}{rs}")))
            }
            Add | Sub | Mul | Div | Rem => self.arith(op, l, r, span),
            Lt | Le | Gt | Ge => self.compare(op, l, r, span),
            // Unreachable because the `matches!(op, And | Or)` branch at the top of this function
            // returns unconditionally for exactly these two operators. That guard is twenty-five
            // lines away, which is far enough for the argument to rot, so it is written down here:
            // if that early return is ever narrowed — made conditional on the operand type, say —
            // this arm becomes live and panics inside the evaluator on ordinary user code. A
            // panicking arm in the interpreter is a crash, and a crash is not a refusal.
            //
            // It is NOT converted to a fault. A fault would let a genuine dispatch bug reach the
            // user as an ordinary program error and stay hidden; this is an internal invariant, and
            // the only inputs that could violate it come from this file, not from any program.
            And | Or => unreachable!("And/Or are short-circuited above and never reach here"),
        }
    }

    fn arith(&self, op: BinOp, l: Value, r: Value, span: delulu_diag::Span) -> R<Value> {
        let overflow = || Escape::Fault(Fault::at("DL0901", "integer overflow", span));
        let divzero = || Escape::Fault(Fault::at("DL0902", "division by zero", span));
        match (l, r) {
            (Value::Int(a), Value::Int(b)) => {
                let v = match op {
                    BinOp::Add => a.checked_add(b).ok_or_else(overflow)?,
                    BinOp::Sub => a.checked_sub(b).ok_or_else(overflow)?,
                    BinOp::Mul => a.checked_mul(b).ok_or_else(overflow)?,
                    BinOp::Div => {
                        if b == 0 {
                            return Err(divzero());
                        }
                        a.checked_div(b).ok_or_else(overflow)?
                    }
                    BinOp::Rem => {
                        if b == 0 {
                            return Err(divzero());
                        }
                        a.checked_rem(b).ok_or_else(overflow)?
                    }
                    _ => unreachable!(),
                };
                Ok(Value::Int(v))
            }
            (Value::Float(a), Value::Float(b)) => {
                let v = match op {
                    BinOp::Add => a + b,
                    BinOp::Sub => a - b,
                    BinOp::Mul => a * b,
                    BinOp::Div => a / b,
                    BinOp::Rem => a % b,
                    _ => unreachable!(),
                };
                Ok(Value::Float(v))
            }
            _ => Err(Escape::Fault(Fault::at("DL0907", "arithmetic on non-numeric values", span))),
        }
    }

    fn compare(&self, op: BinOp, l: Value, r: Value, span: delulu_diag::Span) -> R<Value> {
        let ord = match (&l, &r) {
            (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),
            (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
            _ => return Err(Escape::Fault(Fault::at("DL0907", "comparison on non-numeric values", span))),
        };
        let Some(ord) = ord else {
            return Ok(Value::Bool(false));
        };
        use std::cmp::Ordering::*;
        let b = match op {
            BinOp::Lt => ord == Less,
            BinOp::Le => ord != Greater,
            BinOp::Gt => ord == Greater,
            BinOp::Ge => ord != Less,
            _ => unreachable!(),
        };
        Ok(Value::Bool(b))
    }
}

/// A human-useful summary of the operation's primary argument for the trace `detail` field
/// (spec §6.1: "the path for `read_text`, host for `get`"). Only ever called once the caller
/// (`Interp::trace_dispatch`) has established that no secret is involved — this function trusts
/// that and never itself redacts.
/// The envelope law (Stage 10 phase 10e, spec §5.1), fail-closed on every branch: a command must
/// be a record; every field must be numeric (`Int` or `Float`), must name a dimension the
/// envelope bounds, and must sit inside the inclusive `lo..hi`. The skip branch — "the checker
/// couldn't tell what this field means" (non-record command, non-numeric field, unlisted
/// dimension) — REFUSES: the envelope cannot vouch for what it never bounded. A `NaN` fails both
/// range comparisons, so it is refused too, not waved through. `rate_hz` is carried by the
/// envelope but deliberately NOT enforced here: rate limiting needs a clock, and actuation time
/// belongs to the 10f dead-man lease machinery — an honest, documented gap, not a silent one.
fn envelope_check(env: &ActuatorEnvelope, cmd: Option<&Value>) -> Result<(), String> {
    let Some(Value::Record { fields, .. }) = cmd else {
        return Err("command must be a record of named dimensions".into());
    };
    for (fname, fval) in fields.borrow().iter() {
        let x = match fval {
            Value::Int(i) => *i as f64,
            Value::Float(f) => *f,
            _ => return Err(format!("field `{fname}` is not numeric — the envelope cannot bound it")),
        };
        let Some((_, lo, hi)) = env.dims.iter().find(|(d, _, _)| d == fname) else {
            return Err(format!("dimension `{fname}` is not bounded by the envelope for `{}`", env.device));
        };
        if !(x >= *lo && x <= *hi) {
            return Err(format!("`{fname}` = {x} is outside the envelope [{lo}, {hi}]"));
        }
    }
    Ok(())
}

/// Flatten a command record to `(dimension, magnitude)` pairs for the broker. Only reached once
/// `envelope_check` has already established that the command IS a record of numbers, so a
/// non-numeric field here is impossible rather than dropped — but the `_ => {}` arm still refuses
/// to invent a value for one, because a silently-omitted dimension is a dimension the broker
/// would never check.
fn numeric_fields(cmd: Option<&Value>) -> Vec<(String, f64)> {
    let Some(Value::Record { fields, .. }) = cmd else { return Vec::new() };
    let mut out = Vec::new();
    for (fname, fval) in fields.borrow().iter() {
        match fval {
            Value::Int(i) => out.push((fname.clone(), *i as f64)),
            Value::Float(f) => out.push((fname.clone(), *f)),
            _ => {}
        }
    }
    out
}

fn trace_detail(recv: &Value, cap_kind: &str, method: &str, args: &[Value]) -> Option<String> {
    match (cap_kind, method) {
        ("Console", "println") | ("Console", "print") => args.first().map(|v| v.display()),
        ("FsRead", "read_text") | ("FsRead", "list_dir") => args.first().map(|v| v.display()),
        ("FsWrite", "write_text") | ("FsWrite", "append_text") => args.first().map(|v| v.display()),
        ("Http", "get") => args.first().map(|v| v.display()),
        // Stage 10 (10e): the device-scoped effects name their DEVICE, taken from the capability's
        // scope rather than its arguments — an audit of `Actuate`, the most physically
        // consequential effect in the language, that cannot say which actuator moved is not an
        // audit. The scope is where the truth lives: the argument is the command, and the command
        // is meaningless without the thing it was sent to.
        ("Actuator", "command") | ("Sensor", "read") => match recv {
            Value::Cap(c) => match &c.scope {
                CapScope::Actuator(e) => Some(e.device.clone()),
                CapScope::Sensor { device } => Some(device.clone()),
                // Unreachable while minting is the only source of these caps; if a future path
                // ever produces one without a device scope, the record says so out loud rather
                // than quietly claiming an unnamed device moved.
                _ => Some("<unscoped device>".to_string()),
            },
            _ => None,
        },
        _ => None,
    }
}

/// Map an effectful `(receiver-cap, method)` dispatch to the broker [`CustodyOp`] + scope argument
/// the custody trait authorizes (Stage 5 phase 5f). Returns `None` for pure/non-effectful calls
/// (attenuation like `fs.narrow`, `Str`/`List` methods, `Root` minting) — those never gate. The fs
/// argument is the resolved absolute path (the exact string the daemon node's fs scope was granted
/// against); the net argument is the URL host.
fn custody_op_for(recvv: &Value, method: &str, argvals: &[Value]) -> Option<(CustodyOp, Option<String>)> {
    let Value::Cap(c) = recvv else { return None };
    match (c.kind, method) {
        (ResourceKind::Console, "println") | (ResourceKind::Console, "print") => Some((CustodyOp::Console, None)),
        (ResourceKind::FsRead, "read_text") | (ResourceKind::FsRead, "list_dir") => {
            Some((CustodyOp::FsRead, fs_scope_arg(&c.scope, argvals)))
        }
        (ResourceKind::FsWrite, "write_text") | (ResourceKind::FsWrite, "append_text") => {
            Some((CustodyOp::FsWrite, fs_scope_arg(&c.scope, argvals)))
        }
        (ResourceKind::Http, "get") => {
            let host = match argvals.first() {
                Some(Value::Str(s)) => Some(host_of(s)),
                _ => None,
            };
            Some((CustodyOp::Net, host))
        }
        (ResourceKind::Clock, "now_ms") => Some((CustodyOp::Clock, None)),
        (ResourceKind::Rand, "int") | (ResourceKind::Rand, "float") => Some((CustodyOp::Rand, None)),
        // 10g: every actuator command round-trips to the grant tree, per command, carrying the
        // device name (spec §5.1's synchronous class; addendum §2.5 states the model). This is the
        // round-trip an operator e-stop arrives through. Sensor `read` is NOT routed: it is a
        // `Read` with a sensor scope, and the sensor scope lives in the device broker.
        (ResourceKind::Actuator, "command") => Some((CustodyOp::Actuate, actuator_device(&c.scope))),
        _ => None,
    }
}

/// The device an actuator capability commands — the `arg` the broker's `Actuate` check is made
/// against, so a revocation can be reasoned about per device rather than per process.
fn actuator_device(scope: &CapScope) -> Option<String> {
    match scope {
        CapScope::Actuator(env) => Some(env.device.clone()),
        _ => None,
    }
}

/// The resolved absolute path a filesystem op reaches (cap-scope root joined with the relative arg,
/// lexically normalized the SAME way the interpreter and broker normalize — no filesystem access).
fn fs_scope_arg(scope: &CapScope, argvals: &[Value]) -> Option<String> {
    let CapScope::Fs { root, .. } = scope else { return None };
    let rel = match argvals.first() {
        Some(Value::Str(s)) => s.to_string(),
        _ => return None,
    };
    Some(prim::resolve_norm(root, &rel).to_string_lossy().to_string())
}

/// Extract the host from an `https://host[:port][/path]` URL, so the broker's exact-set `net` check
/// sees the same host string the capability check used.
///
/// **Delegates** rather than re-implementing. This was a second copy of the parse, kept in step with
/// `prim`'s by hand and by a comment — so when the parse turned out to mis-handle userinfo (C86),
/// the authorization decision and the audit record were wrong in exactly the same way, in two
/// places. Design rule 1: one function referenced by both sides.
fn host_of(url: &str) -> String {
    prim::host_of(url).to_string()
}

fn unwrap_fault(e: Escape) -> Fault {
    match e {
        Escape::Fault(f) => f,
        Escape::Return(_) | Escape::Propagate(_) | Escape::Break | Escape::Continue => {
            Fault::new("DL0907", "control-flow escaped the top level (checker bug)")
        }
    }
}

/// The human-useful `detail` for a traced `std.py` op: the name argument for `import` (the module
/// being reached for — the DENIED name too, criterion 5), `attr`, and `call_method`. These arguments
/// are ordinary `Str` method names, never secrets (a secret cannot cross to PyObj-land).
fn python_detail(method: &str, args: &[Value]) -> Option<String> {
    match method {
        "import" | "attr" | "call_method" => match args.first() {
            Some(Value::Str(s)) => Some(s.to_string()),
            _ => None,
        },
        _ => None,
    }
}

// ----- foreign marshalling helpers (Stage 4, spec §4.2) --------------------

/// Lower one declared `foreign_fn` to its runtime marshalling signature. A parameter/return type
/// that does not marshal is impossible in a checked program (T-ForeignSig / DL1301); we default it
/// to `Unit` rather than panic, keeping the interpreter total on unchecked input. **Public so the
/// WASM engine's host (`delulu-wasm`) lowers foreign blocks the SAME way — verify≡run, one path.**
pub fn lower_foreign_sig(f: &ForeignFn) -> ForeignSig {
    let kind_of = |te: &TypeExpr| -> FKind {
        match te {
            TypeExpr::Named { path, .. } => {
                FKind::from_type_name(&path.segs.last().unwrap().name).unwrap_or(FKind::Unit)
            }
            TypeExpr::Fn { .. } => FKind::Unit, // fenced by DL1302 at check time
            TypeExpr::Rcap { .. } => FKind::Unit, // fenced by DL1301 at check time (not marshallable)
        }
    };
    let params = f.params.iter().map(|p| kind_of(&p.ty)).collect();
    let ret = f.ret.as_ref().map(kind_of).unwrap_or(FKind::Unit);
    ForeignSig { name: f.name.name.clone(), params, ret }
}

/// Map a runtime `Value` to the marshalled `FVal` its foreign parameter expects. In a well-typed
/// program the pair always matches; a mismatch (only reachable on unchecked input) marshals a benign
/// default rather than panicking.
fn value_to_fval(kind: FKind, v: &Value) -> FVal {
    match (kind, v) {
        (FKind::Int, Value::Int(i)) => FVal::Int(*i),
        (FKind::Float, Value::Float(f)) => FVal::Float(*f),
        (FKind::Bool, Value::Bool(b)) => FVal::Bool(*b),
        (FKind::Str, Value::Str(s)) => FVal::Str(s.clone()),
        (FKind::Ptr, Value::ForeignPtr(p)) => FVal::Ptr(*p),
        (FKind::Unit, _) => FVal::Unit,
        (FKind::Int, _) => FVal::Int(0),
        (FKind::Float, _) => FVal::Float(0.0),
        (FKind::Bool, _) => FVal::Bool(false),
        (FKind::Str, _) => FVal::Str(std::rc::Rc::from("")),
        (FKind::Ptr, _) => FVal::Ptr(0),
    }
}

/// Map a validated foreign return `FVal` back to a runtime `Value`.
fn fval_to_value(fv: FVal) -> Value {
    match fv {
        FVal::Int(i) => Value::Int(i),
        FVal::Float(f) => Value::Float(f),
        FVal::Bool(b) => Value::Bool(b),
        FVal::Str(s) => Value::Str(s),
        FVal::Unit => Value::Unit,
        FVal::Ptr(p) => Value::ForeignPtr(p),
    }
}

/// Map a [`foreign::ForeignErr`] to its `std.foreign.ForeignErr` sum value (spec §8).
/// P2: a `PluginErr` as the in-language sum a program catches (spec §4).
///
/// Every load refusal reaches the program as one of these, and the DL code the CLI renders comes
/// from `plugin::plugin_err_code` on the same value — one mapping, so the code a human reads and the
/// variant a program matches on cannot disagree.
fn plugin_err_value(e: &crate::plugin::PluginErr) -> Value {
    use crate::plugin::PluginErr::*;
    match e {
        NotGranted(s) => Value::variant("NotGranted", vec![Value::str(s.clone())]),
        VerifyFailed(s) => Value::variant("VerifyFailed", vec![Value::str(s.clone())]),
        BadArtifact(s) => Value::variant("BadArtifact", vec![Value::str(s.clone())]),
        Revoked(seq) => Value::variant("Revoked", vec![Value::Int(*seq)]),
        LimitExceeded(s) => Value::variant("LimitExceeded", vec![Value::str(s.clone())]),
        ApiMismatch(s) => Value::variant("ApiMismatch", vec![Value::str(s.clone())]),
    }
}

/// P2: read a program's `Grant` record into the runtime's `Grant`.
///
/// A `Grant` is an ORDINARY RECORD (spec §4): it *describes* authority and confers none. Conferral is
/// step 3's ceiling intersection and step 4's holder check, both at `load`. So this function is a
/// plain projection with no checking in it — a missing or mistyped field reads as its empty value
/// rather than erroring, because the checker already typed the record and a runtime disagreement here
/// would be a checker bug, not a program's mistake. What the projection must never do is invent
/// authority: every field that is absent must project to *less*, never more.
fn grant_from_value(v: &Value) -> crate::plugin::Grant {
    use crate::plugin::{Grant, Limits};
    let field = |name: &str| -> Option<Value> {
        if let Value::Record { fields, .. } = v {
            fields.borrow().iter().find(|(n, _)| n == name).map(|(_, val)| val.clone())
        } else {
            None
        }
    };
    let strings = |name: &str| -> Vec<String> {
        match field(name) {
            Some(Value::List(items)) => items.borrow().iter().map(|x| x.display()).collect(),
            _ => Vec::new(),
        }
    };
    let int_in = |rec: &Option<Value>, name: &str| -> i64 {
        match rec {
            Some(Value::Record { fields, .. }) => fields
                .borrow()
                .iter()
                .find(|(n, _)| n == name)
                .and_then(|(_, v)| if let Value::Int(i) = v { Some(*i) } else { None })
                .unwrap_or(0),
            _ => 0,
        }
    };
    let limits_rec = field("limits");
    Grant {
        effects: strings("effects"),
        fs_read: strings("fs_read"),
        fs_write: strings("fs_write"),
        net: strings("net"),
        secrets: strings("secrets"),
        declassify: strings("declassify"),
        limits: Limits {
            fuel: int_in(&limits_rec, "fuel"),
            mem_mb: int_in(&limits_rec, "mem_mb"),
            wall_ms: int_in(&limits_rec, "wall_ms"),
        },
        // Absent reads as `false`, which is the LESS demanding value — so a malformed record can
        // never accidentally turn the signature requirement on and make a good artifact refuse. The
        // opposite default would be the safer-sounding one and the wrong one: a grant that demands a
        // signature nobody asked for is a refusal the operator did not write.
        require_signed: matches!(field("require_signed"), Some(Value::Bool(true))),
    }
}

fn foreign_err_value(e: &foreign::ForeignErr) -> Value {
    use foreign::ForeignErr::*;
    match e {
        NotGranted => Value::variant("NotGranted", vec![]),
        SymbolMissing(s) => Value::variant("SymbolMissing", vec![Value::str(s.clone())]),
        BadReturn(s) => Value::variant("BadReturn", vec![Value::str(s.clone())]),
        Unavailable(s) => Value::variant("Unavailable", vec![Value::str(s.clone())]),
        // `WorkerDied` is a call-time failure surfaced as a DL1409 fault, never a bind result — but if
        // a worker dies during the bind handshake it maps to the language's `Unavailable` variant (a
        // catchable "the library could not be made available"), keeping the language sum unchanged.
        WorkerDied(s) => Value::variant("Unavailable", vec![Value::str(format!("foreign worker died: {s}"))]),
    }
}

#[cfg(test)]
mod depth_contract_tests {
    use super::*;

    /// **The depth bound is an explicit contract, not an undocumented stack requirement** (C21/D51).
    ///
    /// The interpreter is a tree-walker, so one DeluluLang call costs several native frames and
    /// `DEFAULT_MAX_DEPTH` needs more than 16 MiB of native stack. `delulu`'s `main.rs` provides 512
    /// MiB, so on the CLI the guard is what fires and deep recursion is DL0905. An EMBEDDER gets no
    /// such thread: on Rust's ~2 MiB default the bound is never reached and the process dies of a
    /// native stack overflow instead — the host-crash class D15 fixed for the CLI, resurfacing for
    /// anyone using this crate as a library.
    ///
    /// This test runs on a deliberately SMALL thread (1 MiB) with a bound chosen to fit it, and
    /// proves the guard fires there. If the bound were still fixed at 10,000, this thread would abort
    /// rather than fail — which is exactly why the knob exists.
    #[test]
    fn a_small_bound_fires_on_a_small_stack_instead_of_overflowing_it() {
        let handle = std::thread::Builder::new()
            .name("embedder-small-stack".into())
            .stack_size(8 * 1024 * 1024) // 8 MiB
            .spawn(|| {
                let src = "module m\n\nfn down(n: Int) -> Int {\n    if n <= 0 { 0 } else { down(n - 1) + 1 }\n}\n\nfn main(root: Root) ! {} {\n    let x = down(100000)\n}\n";
                let checked = delulu_check::check_source(0, src);
                assert!(!checked.has_errors(), "the fixture must check: {:?}", checked.diagnostics);
                // 200 is comfortably inside 1 MiB; the default 10,000 would not be.
                let interp = Interp::new(&checked.module).with_max_depth(100);
                let root = Value::Root(std::rc::Rc::new(crate::value::RootVal::default()));
                let fault = interp.run_main(root).expect_err(
                    "unbounded recursion must be refused, not run",
                );
                assert_eq!(fault.code, "DL0905", "the guard fires as a diagnostic: {fault:?}");
            })
            .expect("spawn");
        handle.join().expect("the thread must FAIL CLEANLY, never abort on a stack overflow");
    }

    /// The embedder's short path: `on_interpreter_thread` supplies the stack, so the DEFAULT bound
    /// is reachable and deep recursion is `DL0905` — without the caller having to know that a
    /// tree-walker costs several native frames per call, or that Rust's default thread is ~2 MiB.
    ///
    /// The test above proves the knob works for someone who knows to reach for it. This one proves
    /// they no longer have to: **C21's residual was never that the mechanism was missing, it was
    /// that using it correctly required knowing a number.**
    #[test]
    fn an_embedder_gets_the_full_bound_without_sizing_anything_itself() {
        let code = on_interpreter_thread(|| {
            let src = "module m\n\nfn down(n: Int) -> Int {\n    if n <= 0 { 0 } else { down(n - 1) + 1 }\n}\n\nfn main(root: Root) ! {} {\n    let x = down(100000)\n}\n";
            let checked = delulu_check::check_source(0, src);
            assert!(!checked.has_errors(), "the fixture must check: {:?}", checked.diagnostics);
            // No `with_max_depth`: the point is that the DEFAULT is safe here.
            let interp = Interp::new(&checked.module);
            let root = Value::Root(std::rc::Rc::new(crate::value::RootVal::default()));
            let fault = interp.run_main(root).expect_err("unbounded recursion must be refused");
            fault.code
        })
        .expect("the interpreter thread must start and return, never abort");
        assert_eq!(code, "DL0905", "the guard fires as a diagnostic on the supplied stack");
    }

    /// A value comes back out, so the helper is usable for real work rather than only for its
    /// side effects.
    #[test]
    fn the_thread_helper_returns_its_closures_value() {
        assert_eq!(on_interpreter_thread(|| 7 * 6).expect("spawn"), 42);
    }

    #[test]
    fn the_default_is_unchanged_so_the_cli_behaves_exactly_as_before() {
        assert_eq!(DEFAULT_MAX_DEPTH, 10_000);
        // The published budget must cover what the CLI actually reserves for the default bound —
        // 512 MiB — or an embedder following this crate's own advice would under-provision.
        assert!(
            STACK_BYTES_PER_DEPTH * DEFAULT_MAX_DEPTH as usize >= 512 * 1024 * 1024,
            "the per-depth budget must cover the 512 MiB `main.rs` reserves for the default bound"
        );
    }
}
