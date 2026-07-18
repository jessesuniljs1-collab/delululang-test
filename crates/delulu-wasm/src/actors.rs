//! Stage 7 phase 7h — the WASM engine's cooperative single-threaded actor scheduler (spec §6.5).
//!
//! The WASM backend has been a SUBSET engine since Stage 3 (`is_compilable` gates per function,
//! DL1201 falls the rest back to the interpreter). Phase 7h extends that subset with actors,
//! **in keeping** with the subset discipline: actor fields and behaviour/ctor parameters within
//! the fragment the backend already compiles — `Int` (i64) and actor references (i64 slot ids).
//! Anything outside stays honestly DL1201.
//!
//! Semantics (identical to the native runtime `actors.rs`, which is the reference — spec §6.5):
//! - **Turn atomicity** is free: single-threaded, a behaviour runs to completion before the next.
//! - **Per-sender-pair FIFO** falls out of a single global FIFO queue (a global order trivially
//!   satisfies the weaker per-pair guarantee the spec makes).
//! - **Quiescence** = the queue drains empty after `main` returns (spec §6.1 exit condition).
//! - **Poison-on-fault**: a wasm trap during a turn poisons that actor; later sends to it drop
//!   and are counted, reported at exit (spec §6.6). The system stays live by default.
//!
//! This module owns the compile-time [`ActorTable`] (shared by codegen and the host so their
//! indices always agree) and the host-side runtime state. The drain loop itself lives in `host.rs`
//! (it needs Wasmtime's `Store`/`Instance`).

use std::collections::{HashMap, VecDeque};

use delulu_syntax::ast::{Item, Module};

/// The compile-time actor table. Codegen resolves `spawn`/send/field **indices** against it;
/// the host dispatches turns against it. Both derive it from the same module in declaration
/// order (via [`actor_table`]), so the two sides' indices always agree — the single source of
/// truth for the actor ABI.
///
/// Export naming convention (how codegen and the host name the wasm functions):
/// `actor$<Actor>$new` for the constructor, `actor$<Actor>$<behaviour>` for each behaviour.
#[derive(Clone, Debug)]
pub struct ActorTable {
    pub actors: Vec<ActorInfo>,
    pub by_name: HashMap<String, u32>,
}

#[derive(Clone, Debug)]
pub struct ActorInfo {
    pub name: String,
    /// Number of fields (the size of the per-actor host-side slot array).
    pub field_count: usize,
    pub ctor_arity: usize,
    pub ctor_export: String,
    pub behaviors: Vec<BehaviorInfo>,
}

#[derive(Clone, Debug)]
pub struct BehaviorInfo {
    pub name: String,
    pub arity: usize,
    pub export: String,
}

impl ActorTable {
    pub fn is_empty(&self) -> bool {
        self.actors.is_empty()
    }

    /// The index of behaviour `name` within actor `actor_idx`, if it exists.
    pub fn behavior_index(&self, actor_idx: u32, name: &str) -> Option<u32> {
        self.actors
            .get(actor_idx as usize)?
            .behaviors
            .iter()
            .position(|b| b.name == name)
            .map(|i| i as u32)
    }
}

pub(crate) fn ctor_export_name(actor: &str) -> String {
    format!("actor${actor}$new")
}

pub(crate) fn behavior_export_name(actor: &str, beh: &str) -> String {
    format!("actor${actor}${beh}")
}

/// Build the actor table from a checked module, in declaration order (spec §2 — the same order
/// codegen and the host both use, so their `actor_idx`/`behavior_idx` agree).
pub fn actor_table(module: &Module) -> ActorTable {
    let mut actors = Vec::new();
    let mut by_name = HashMap::new();
    for it in &module.items {
        if let Item::Actor(a) = it {
            let idx = actors.len() as u32;
            by_name.insert(a.name.name.clone(), idx);
            let behaviors = a
                .behaviors
                .iter()
                .map(|b| BehaviorInfo {
                    name: b.name.name.clone(),
                    arity: b.params.len(),
                    export: behavior_export_name(&a.name.name, &b.name.name),
                })
                .collect();
            actors.push(ActorInfo {
                name: a.name.name.clone(),
                field_count: a.fields.len(),
                ctor_arity: a.ctor.params.len(),
                ctor_export: ctor_export_name(&a.name.name),
                behaviors,
            });
        }
    }
    ActorTable { actors, by_name }
}

/// What the WASM cooperative scheduler observes at quiescence — the same accounting the native
/// `QuiesceReport` makes (`total_turns`, surviving/dead actor counts, dropped sends), so a
/// deterministic actor program yields **identical** counts on both engines (criterion 6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActorReport {
    pub total_turns: u64,
    pub surviving_actors: i64,
    pub dead_actors: u64,
    pub dropped_sends: u64,
}

/// One queued unit of work. A `Create` runs the ctor as the actor's first turn; a `Send` runs a
/// behaviour. Args are already lowered to i64 (the actor-boundary ABI): `Int` and actor-ref slot
/// ids are i64 directly.
pub(crate) enum ActorJob {
    Create { slot: u64, actor_idx: u32, args: Vec<i64> },
    Send { slot: u64, behavior_idx: u32, args: Vec<i64> },
}

pub(crate) struct ActorSlot {
    pub actor_idx: u32,
    pub fields: Vec<i64>,
    pub dead: bool,
}

/// The host-side actor state for one cooperative run: per-actor field slots, a global FIFO job
/// queue, and the quiescence counters. Single-threaded, so turn atomicity needs no locking.
pub(crate) struct ActorRuntime {
    pub table: ActorTable,
    pub slots: Vec<ActorSlot>,
    pub queue: VecDeque<ActorJob>,
    pub total_turns: u64,
    pub dead_actors: u64,
    pub dropped_sends: u64,
}

impl ActorRuntime {
    pub fn new(table: ActorTable) -> ActorRuntime {
        ActorRuntime {
            table,
            slots: Vec::new(),
            queue: VecDeque::new(),
            total_turns: 0,
            dead_actors: 0,
            dropped_sends: 0,
        }
    }

    /// Allocate a fresh slot (fields zero-seeded; the ctor turn assigns them — the checker's
    /// definite-initialisation rule makes an uninitialised read unreachable in a checked program)
    /// and enqueue its `Create`. Returns the address immediately so `main` can send to it at once;
    /// the ctor runs later as the first turn, exactly like the native runtime.
    pub fn spawn(&mut self, actor_idx: u32, args: Vec<i64>) -> u64 {
        let slot = self.slots.len() as u64;
        let field_count = self.table.actors[actor_idx as usize].field_count;
        self.slots.push(ActorSlot { actor_idx, fields: vec![0; field_count], dead: false });
        self.queue.push_back(ActorJob::Create { slot, actor_idx, args });
        slot
    }

    pub fn send(&mut self, slot: u64, behavior_idx: u32, args: Vec<i64>) {
        self.queue.push_back(ActorJob::Send { slot, behavior_idx, args });
    }

    pub fn report(&self) -> ActorReport {
        ActorReport {
            total_turns: self.total_turns,
            surviving_actors: self.slots.len() as i64 - self.dead_actors as i64,
            dead_actors: self.dead_actors,
            dropped_sends: self.dropped_sends,
        }
    }
}
