//! The per-worker cycle collector (Stage 10 phase 10d, Track B1, spec §3).
//!
//! `Rc` cannot free cycles, and the actor model makes the fix cheap: **between turns, an
//! actor's only live roots are its state records** — locals died with the turn, there are no
//! continuations, and module globals are immutable pure constants (§5.5) that can never come to
//! reference a turn's allocations. So any cell allocated during a turn that is still alive but
//! unreachable from every actor state on this worker is garbage in a cycle, by construction.
//!
//! The mechanism is mark-and-break over a registry of the cells a cycle can pass through:
//! - `List` and `Record` cells (the mutable back-edges),
//! - closure-captured `Scope`s (a `var` in a captured scope can point back at the closure).
//!
//! Registration happens ONLY inside actor turns — the main thread, the REPL, and every
//! non-actor program pay one predictable branch per allocation and nothing else (the Study-C
//! perf gate rides on that). Breaking a garbage cell empties it in place; the drop cascade
//! then collapses the rest of the cycle, and the registry's dead weaks confirm it on the next
//! sweep. Collections are counted and reported (`--trace-memory`), never silent.

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::{Rc, Weak};

use crate::value::{Scope, Value};

/// Sweep when this many registered cells have accumulated on a worker — the amortizer: sweep
/// cost is O(reachable + registered), paid once per that many allocations at worst.
pub const COLLECT_THRESHOLD: usize = 64;

pub enum RegCell {
    List(Weak<RefCell<Vec<Value>>>),
    Rec(Weak<RefCell<Vec<(String, Value)>>>),
    Env(Weak<Scope>),
}

#[derive(Default)]
pub struct Registry {
    cells: Vec<RegCell>,
    pub runs: u64,
    pub collected: u64,
}

impl Registry {
    pub fn pressure(&self) -> usize {
        self.cells.len()
    }

    pub fn note_list(&mut self, c: &Rc<RefCell<Vec<Value>>>) {
        self.cells.push(RegCell::List(Rc::downgrade(c)));
    }
    pub fn note_record(&mut self, c: &Rc<RefCell<Vec<(String, Value)>>>) {
        self.cells.push(RegCell::Rec(Rc::downgrade(c)));
    }
    pub fn note_env(&mut self, s: &Rc<Scope>) {
        self.cells.push(RegCell::Env(Rc::downgrade(s)));
    }
    /// Register a value's own top-level cell (used for primitive results, whose internal
    /// allocations either ARE the top cell or were registered at their own eval sites).
    pub fn note_value(&mut self, v: &Value) {
        match v {
            Value::List(c) => self.note_list(c),
            Value::Record { fields, .. } => self.note_record(fields),
            Value::Closure(c) => self.note_env(&c.env),
            _ => {}
        }
    }

    /// Mark everything reachable from `roots`, then break every registered cell that is alive
    /// and unmarked. Returns how many cells were broken. Worklist-based — no recursion, so a
    /// pathologically deep structure cannot overflow the collector's own stack.
    pub fn collect(&mut self, roots: &[Value]) -> u64 {
        let mut seen: HashSet<usize> = HashSet::new();
        let mut values: Vec<Value> = roots.to_vec();
        let mut scopes: Vec<Rc<Scope>> = Vec::new();
        while !values.is_empty() || !scopes.is_empty() {
            if let Some(v) = values.pop() {
                match &v {
                    Value::List(c) => {
                        if seen.insert(Rc::as_ptr(c) as usize) {
                            values.extend(c.borrow().iter().cloned());
                        }
                    }
                    Value::Record { fields, .. } => {
                        if seen.insert(Rc::as_ptr(fields) as usize) {
                            values.extend(fields.borrow().iter().map(|(_, v)| v.clone()));
                        }
                    }
                    Value::Variant { fields, .. } => {
                        // Immutable and never registered, but its children may be.
                        if seen.insert(Rc::as_ptr(fields) as *const () as usize) {
                            values.extend(fields.iter().cloned());
                        }
                    }
                    Value::Closure(c) => scopes.push(c.env.clone()),
                    _ => {}
                }
                continue;
            }
            if let Some(s) = scopes.pop() {
                if seen.insert(Rc::as_ptr(&s) as *const () as usize) {
                    s.each_value(|v| values.push(v.clone()));
                    if let Some(p) = s.parent() {
                        scopes.push(p.clone());
                    }
                }
            }
        }

        let mut broken: u64 = 0;
        self.cells.retain(|cell| match cell {
            RegCell::List(w) => match w.upgrade() {
                None => false,
                Some(c) if seen.contains(&(Rc::as_ptr(&c) as usize)) => true,
                Some(c) => {
                    c.borrow_mut().clear();
                    broken += 1;
                    false
                }
            },
            RegCell::Rec(w) => match w.upgrade() {
                None => false,
                Some(c) if seen.contains(&(Rc::as_ptr(&c) as usize)) => true,
                Some(c) => {
                    c.borrow_mut().clear();
                    broken += 1;
                    false
                }
            },
            RegCell::Env(w) => match w.upgrade() {
                None => false,
                Some(s) if seen.contains(&(Rc::as_ptr(&s) as *const () as usize)) => true,
                Some(s) => {
                    s.clear_for_collector();
                    broken += 1;
                    false
                }
            },
        });
        self.runs += 1;
        self.collected += broken;
        broken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(fields: Vec<(String, Value)>) -> (Value, Rc<RefCell<Vec<(String, Value)>>>) {
        let cell = Rc::new(RefCell::new(fields));
        (Value::Record { name: Rc::from("Pair"), fields: cell.clone() }, cell)
    }

    /// The defining case: two records pointing at each other, all external strong refs gone.
    /// Rc alone leaks this forever; the sweep breaks it, and the weak handles PROVE the memory
    /// was actually freed — not just uncounted.
    #[test]
    fn an_unreachable_record_cycle_is_broken_and_freed() {
        let mut reg = Registry::default();
        let (a, ca) = record(vec![]);
        let (b, cb) = record(vec![("other".into(), a.clone())]);
        ca.borrow_mut().push(("other".into(), b.clone()));
        reg.note_record(&ca);
        reg.note_record(&cb);
        let (wa, wb) = (Rc::downgrade(&ca), Rc::downgrade(&cb));
        drop((a, b, ca, cb));
        assert!(wa.upgrade().is_some(), "the cycle keeps itself alive — the leak is real");

        let broken = reg.collect(&[]);
        assert!(broken >= 1, "the sweep breaks into the cycle");
        assert!(wa.upgrade().is_none() && wb.upgrade().is_none(), "both cells actually freed");
    }

    /// The reachability half: an identical cycle that IS reachable from a root must survive
    /// untouched — a collector that frees live data is worse than a leak.
    #[test]
    fn a_reachable_cycle_is_never_touched() {
        let mut reg = Registry::default();
        let (a, ca) = record(vec![]);
        let (b, _cb) = record(vec![("other".into(), a.clone())]);
        ca.borrow_mut().push(("other".into(), b.clone()));
        reg.note_record(&ca);
        let root = Value::List(Rc::new(RefCell::new(vec![a.clone()])));
        let broken = reg.collect(&[root]);
        assert_eq!(broken, 0, "reachable data is live data");
        assert!(!ca.borrow().is_empty(), "the record's contents are intact");
    }

    /// A self-referential list — the shape the in-language corpus builds via `push`.
    #[test]
    fn a_self_referential_list_is_collected() {
        let mut reg = Registry::default();
        let cell = Rc::new(RefCell::new(Vec::new()));
        let l = Value::List(cell.clone());
        cell.borrow_mut().push(l.clone());
        reg.note_list(&cell);
        let w = Rc::downgrade(&cell);
        drop((l, cell));
        assert!(w.upgrade().is_some(), "self-reference keeps it alive");
        assert_eq!(reg.collect(&[]), 1);
        assert!(w.upgrade().is_none(), "freed");
    }

    /// The scope cycle: a closure capturing the scope whose `var` holds the closure.
    #[test]
    fn a_closure_scope_cycle_is_collected() {
        use crate::value::Closure;
        let mut reg = Registry::default();
        let env = Scope::root();
        let clo = Value::Closure(Rc::new(Closure {
            params: vec![],
            body: delulu_syntax::ast::Block {
                stmts: vec![],
                id: delulu_syntax::ast::NodeId(0),
                span: delulu_diag::Span::new(0, 0, 0),
            },
            env: env.clone(),
        }));
        env.define("f", clo.clone());
        reg.note_env(&env);
        let w = Rc::downgrade(&env);
        drop((clo, env));
        assert!(w.upgrade().is_some(), "the capture cycle keeps the scope alive");
        assert_eq!(reg.collect(&[]), 1);
        assert!(w.upgrade().is_none(), "the scope was freed");
    }
}
