//! The Stage-1 tree-walking interpreter (spec §7). It runs the *checked* AST, so it assumes
//! well-typedness and focuses on faithful evaluation and host-side capability enforcement.
//! Runtime failures are defined faults (DL09xx) that abort cleanly — never UB.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use delulu_check::ResourceKind;
use delulu_syntax::ast::*;

use crate::foreign::{self, FKind, FVal, ForeignHandle, ForeignSig};
use crate::prim;
use crate::trace::{self, TraceRecord, TraceSink};
use crate::value::{CapVal, Closure, Env, Fault, Scope, Value};

const MAX_DEPTH: u32 = 10_000;

/// Non-local control flow. `Fault` is a real runtime error; the others are ordinary control.
enum Escape {
    Return(Value),
    Propagate(Value), // `?` propagating an `Err` as the function's result
    Fault(Fault),
}

type R<T> = Result<T, Escape>;

pub struct Interp {
    funcs: HashMap<String, FnDecl>,
    consts: Vec<(String, Expr)>,
    globals: Env,
    depth: Cell<u32>,
    /// Effect tracing (spec §6.1): absent by default, attached via `with_trace`. Additive — does
    /// not change `Interp::new`'s signature or behavior.
    trace: Option<TraceSink>,
    trace_seq: Cell<u64>,
    /// Foreign blocks by lib name (Stage 4): each declared function's marshalling signature, lowered
    /// from the module. Empty for a program with no `foreign` blocks — so nothing changes for it.
    foreign_blocks: HashMap<String, Vec<ForeignSig>>,
    /// Which lib each `root.foreign(load)` call node binds (from the checker's `foreign_binds`), the
    /// granted binary path per lib (from `--grant`), and the return-size ceiling. All inert unless
    /// `with_foreign` is called.
    foreign_binds: HashMap<NodeId, String>,
    foreign_grants: HashMap<String, String>,
    foreign_max_ret: usize,
}

impl Interp {
    pub fn new(module: &Module) -> Interp {
        let mut funcs = HashMap::new();
        let mut consts = Vec::new();
        let mut foreign_blocks = HashMap::new();
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
                _ => {}
            }
        }
        Interp {
            funcs,
            consts,
            globals: Scope::root(),
            depth: Cell::new(0),
            trace: None,
            trace_seq: Cell::new(0),
            foreign_blocks,
            foreign_binds: HashMap::new(),
            foreign_grants: HashMap::new(),
            foreign_max_ret: foreign::DEFAULT_MAX_RET,
        }
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
        for (name, expr) in &self.consts {
            let v = self.eval_expr(expr, &self.globals)?;
            self.globals.define(name, v);
        }
        Ok(())
    }

    // ----- calls ----------------------------------------------------------

    fn enter(&self) -> R<()> {
        let d = self.depth.get() + 1;
        if d > MAX_DEPTH {
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
                loop {
                    match self.eval_expr(cond, env)? {
                        Value::Bool(true) => {
                            self.exec_block_value(body, env)?;
                        }
                        _ => break,
                    }
                }
                Ok(Value::Unit)
            }
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
        match e {
            Expr::Lit { kind, .. } => Ok(self.lit(kind)),
            Expr::Var { path, span, .. } => self.eval_var(path, *span, env),
            Expr::List { items, .. } => {
                let mut vs = Vec::with_capacity(items.len());
                for it in items {
                    vs.push(self.eval_expr(it, env)?);
                }
                Ok(Value::List(Rc::new(std::cell::RefCell::new(vs))))
            }
            Expr::Record { path, fields, .. } => {
                let name: Rc<str> = Rc::from(path.segs.last().unwrap().name.as_str());
                let mut fs = Vec::new();
                for (fname, fexpr) in fields {
                    fs.push((fname.name.clone(), self.eval_expr(fexpr, env)?));
                }
                Ok(Value::Record { name, fields: Rc::new(std::cell::RefCell::new(fs)) })
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
            Expr::Lambda { params, body, .. } => Ok(Value::Closure(Rc::new(Closure {
                params: params.iter().map(|p| p.name.name.clone()).collect(),
                body: body.clone(),
                env: env.clone(),
            }))),
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
                // Prelude builtins/constructors.
                if let Some(res) = prim::call_builtin(name, &argvals, span) {
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
            other => Err(Escape::Fault(Fault::at("DL0907", format!("value `{}` is not callable", other.display()), span))),
        }
    }

    fn eval_method(&self, recv: &Expr, name: &Ident, args: &[Expr], span: delulu_diag::Span, node_id: NodeId, env: &Env) -> R<Value> {
        let recvv = self.eval_expr(recv, env)?;

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
                    return Ok(Value::List(Rc::new(std::cell::RefCell::new(out))));
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
        let result = match &recvv {
            // T-ForeignBind (spec §4): binding a lib is handled here, not in `prim`, because it needs
            // the checker's bind-site → lib map, the grant paths, and the native loader.
            Value::Root(_) if name.name == "foreign" => self.bind_foreign(node_id, span),
            Value::Root(r) => prim::call_root_method(r, &name.name, &argvals, span),
            // T-ForeignCall (spec §4): a method on a bound lib handle marshals + calls foreign code.
            Value::Foreign(h) => self.call_foreign(h, &name.name, &argvals, span),
            // T-Py (spec §5): a `Cap[Python]` operation (import/of_*/list/to_*) runs the embedded
            // interpreter under the GIL; handled here (not `prim`) to trace `ForeignCall` and thread
            // the `--foreign-max-ret` message bound.
            Value::Cap(c) if c.kind == ResourceKind::Python => self.call_python(c, &name.name, &argvals, span),
            Value::Cap(c) => prim::call_cap_method(c, &name.name, &argvals, span),
            // T-Py (spec §5): a `PyObj` operation (attr/call/call_method/index). Present only with the
            // `python` feature — with it off no `Cap[Python]` exists, so no `PyObj` value is ever made.
            #[cfg(feature = "python")]
            Value::PyObj(o) => self.call_pyobj(o, &name.name, &argvals, span),
            Value::Secret(s) => prim::call_secret_method(s, &name.name, &argvals, span),
            Value::Str(s) => prim::call_str_method(s, &name.name, &argvals, span),
            Value::List(l) => prim::call_list_method(l, &name.name, &argvals, span),
            other => Err(Fault::at("DL0907", format!("type has no method `{}` on `{}`", name.name, other.display()), span)),
        };
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
        let detail = if involves_secret { Some(trace::OPAQUE.to_string()) } else { trace_detail(kind, &name.name, argvals) };
        sink.push(TraceRecord {
            seq: self.next_trace_seq(),
            effect: effect.to_string(),
            op: name.name.clone(),
            cap_kind: kind.to_string(),
            detail,
            span: Some((span.file, span.start, span.end)),
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
        // The path is grant data (spec §4.1). No grant → NotGranted; the CLI normally refuses at
        // startup (criterion 4) so a program that reaches here has already been granted.
        let Some(path) = self.foreign_grants.get(&lib_name).cloned() else {
            return Ok(Value::err(Value::variant("NotGranted", vec![])));
        };
        let sigs = self.foreign_blocks.get(&lib_name).cloned().unwrap_or_default();
        match foreign::load_and_resolve(&path, sigs) {
            Ok(lib) => Ok(Value::ok(Value::Foreign(Rc::new(ForeignHandle { name: lib_name, lib })))),
            Err(e) => Ok(Value::err(foreign_err_value(&e))),
        }
    }

    /// `m.method(args)` on a bound lib handle — marshal per spec §4.2, call via `libffi`, validate the
    /// return (invariant 21), and record the `ForeignCall` trace event (criterion 1). A return that
    /// fails shape validation surfaces as a defined DL1306 fault: a bare `-> Str`/`-> Float` foreign
    /// signature has no `Result` channel for the program to catch, so the honest outcome is a clean
    /// abort — never UB, never a panic, never a silent truncation (criterion 8).
    fn call_foreign(&self, handle: &Rc<ForeignHandle>, method: &str, args: &[Value], span: delulu_diag::Span) -> Result<Value, Fault> {
        let Some(sig) = handle.lib.sig(method).cloned() else {
            return Err(Fault::at("DL0907", format!("unknown foreign method `{method}` (checker bug)"), span));
        };
        let fargs: Vec<FVal> = sig
            .params
            .iter()
            .zip(args)
            .map(|(k, v)| value_to_fval(*k, v))
            .collect();
        self.trace_foreign(&handle.name, method, span);
        match foreign::call(&handle.lib, method, &fargs, self.foreign_max_ret) {
            Ok(fv) => Ok(fval_to_value(fv)),
            Err(foreign::ForeignErr::BadReturn(reason)) => Err(Fault::at(
                "DL1306",
                format!("foreign return validation failed: {reason} (ForeignErr::BadReturn)"),
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
        });
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
                Ok(Value::str(format!("{}{}", l.display(), r.display())))
            }
            Add | Sub | Mul | Div | Rem => self.arith(op, l, r, span),
            Lt | Le | Gt | Ge => self.compare(op, l, r, span),
            And | Or => unreachable!(),
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
fn trace_detail(cap_kind: &str, method: &str, args: &[Value]) -> Option<String> {
    match (cap_kind, method) {
        ("Console", "println") | ("Console", "print") => args.first().map(|v| v.display()),
        ("FsRead", "read_text") | ("FsRead", "list_dir") => args.first().map(|v| v.display()),
        ("FsWrite", "write_text") | ("FsWrite", "append_text") => args.first().map(|v| v.display()),
        ("Http", "get") => args.first().map(|v| v.display()),
        _ => None,
    }
}

fn unwrap_fault(e: Escape) -> Fault {
    match e {
        Escape::Fault(f) => f,
        Escape::Return(_) | Escape::Propagate(_) => Fault::new("DL0907", "control-flow escaped the top level (checker bug)"),
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
/// to `Unit` rather than panic, keeping the interpreter total on unchecked input.
fn lower_foreign_sig(f: &ForeignFn) -> ForeignSig {
    let kind_of = |te: &TypeExpr| -> FKind {
        match te {
            TypeExpr::Named { path, .. } => {
                FKind::from_type_name(&path.segs.last().unwrap().name).unwrap_or(FKind::Unit)
            }
            TypeExpr::Fn { .. } => FKind::Unit, // fenced by DL1302 at check time
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
fn foreign_err_value(e: &foreign::ForeignErr) -> Value {
    use foreign::ForeignErr::*;
    match e {
        NotGranted => Value::variant("NotGranted", vec![]),
        SymbolMissing(s) => Value::variant("SymbolMissing", vec![Value::str(s.clone())]),
        BadReturn(s) => Value::variant("BadReturn", vec![Value::str(s.clone())]),
        Unavailable(s) => Value::variant("Unavailable", vec![Value::str(s.clone())]),
    }
}
