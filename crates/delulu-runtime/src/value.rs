//! Runtime values, environments, and capability values (spec §7.1).
//!
//! Capabilities are unforgeable at runtime just as in the type system: `CapVal`/`RootVal`/
//! `SecretVal` are constructed only by the broker (`broker.rs`) and by attenuation, never from
//! user data. Secrets never render: `Display`/`Debug` print «secret», and the bytes are
//! best-effort zeroized on drop.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::rc::Rc;

use delulu_check::ResourceKind;
use delulu_diag::Span;
use delulu_syntax::ast::Block;

/// A runtime fault: a defined, non-UB failure that aborts with a DL09xx diagnostic (§7.1).
#[derive(Clone, Debug)]
pub struct Fault {
    pub code: &'static str,
    pub message: String,
    pub span: Option<Span>,
}

impl Fault {
    pub fn new(code: &'static str, message: impl Into<String>) -> Fault {
        Fault { code, message: message.into(), span: None }
    }
    pub fn at(code: &'static str, message: impl Into<String>, span: Span) -> Fault {
        Fault { code, message: message.into(), span: Some(span) }
    }
}

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(Rc<str>),
    Unit,
    List(Rc<RefCell<Vec<Value>>>),
    /// A `Map[K, V]` (P3, ruling D-V2-29), stored as a `BTreeMap` keyed by [`MapKey`].
    ///
    /// A `BTreeMap` rather than a hash map, and that is the whole design: iteration is ascending by
    /// key, so `keys()` and `values()` are in the same order as each other and the same order on
    /// every run, on every platform, under every allocator. The hash-order nondeterminism that makes
    /// other languages' map output untestable simply does not exist here — which matters for a
    /// language whose tests compare printed output and whose plugin DIR must be byte-reproducible.
    Map(Rc<RefCell<std::collections::BTreeMap<MapKey, Value>>>),
    Record { name: Rc<str>, fields: Rc<RefCell<Vec<(String, Value)>>> },
    /// A sum-type value: builtin (`Ok`/`Err`/`Some`/`None`) or a user variant, by name.
    ///
    /// The payload is [`VariantFields`] rather than a bare `Rc<Vec<Value>>` for one reason, and it is
    /// a safety reason: this is the **only** unbounded nesting vector in the value representation, so
    /// it is where the iterative teardown lives (campaign finding INTERP-DROP-1).
    Variant { name: Rc<str>, fields: VariantFields },
    Closure(Rc<Closure>),
    Cap(Rc<CapVal>),
    Secret(Rc<SecretVal>),
    Root(Rc<RootVal>),
    /// A bound foreign-library handle, minted only by `root.foreign(load)` (Stage 4, spec §4). Opaque
    /// (R-5): it never stringifies or compares — the checker's `Type::Foreign` is opaque.
    Foreign(Rc<crate::foreign::ForeignHandle>),
    /// An opaque C pointer (`ForeignPtr`, spec §4.2), stored as an integer address so no raw pointer
    /// leaks into general evaluation. Opaque (R-5): never stringified or compared.
    ForeignPtr(usize),
    /// An opaque embedded-Python object (`PyObj`, Stage 4 phase 4f, spec §5), minted only by the
    /// `std.py` surface. Present only with the `python` feature. Opaque (R-5): the checker's
    /// `Type::PyObj` rejects `str`/`==`/serialize, so a well-typed program never displays or compares
    /// one. The GIL/ownership discipline lives in `python.rs`.
    #[cfg(feature = "python")]
    PyObj(crate::python::PyObjVal),
    /// An actor reference (Stage 7) — `tag`: opaque identity, sendable, no synchronous
    /// access (the checker enforces all three). Carries the process-wide address; the id is
    /// never reused, so a reference to a dead/unloaded actor stays dead forever.
    ActorRef { id: crate::actors::ActorId, actor: Rc<str> },
    /// P2: a loaded plugin, minted only by `load(host, path, grant)` (Stage 6, spec §3.1). Opaque
    /// (R-5) exactly as `Foreign` is — the checker's `Type::Plugin` refuses `str`, `==` and
    /// serialization, so a well-typed program never displays or compares one.
    Plugin(Rc<crate::plugin::LoadedHandle>),
    /// P2: one export of a loaded plugin, as a callable. Minted only by `p.get(name)`, and opaque for
    /// the same reason: it is a function value over code that arrived after compile time.
    PluginFn(Rc<crate::plugin::PluginFn>),
}

/// A `Map` key: `Str`, `Int` or `Bool`, and nothing else (ruling D-V2-29 decision 4).
///
/// A closed enum rather than an `Ord` impl on [`Value`], for the same reason `List.sort` uses its own
/// key type: deriving `Ord` on `Value` would define an order for every variant at once — including
/// `Float`, whose NaN breaks the total-order law a `BTreeMap` depends on, and including the opaque
/// handles, whose identity must never be comparable. The checker refuses every other key type at the
/// call site that supplies it, so [`MapKey::of`] returning `None` is a checker bug and says so.
///
/// `Bool` sorts false-before-true. The variant ORDER here is the key order: booleans, then integers,
/// then strings. It is arbitrary but fixed, and a map has one key type anyway, so the cross-type
/// ordering is unobservable from a well-typed program.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MapKey {
    Bool(bool),
    Int(i64),
    Str(String),
}

impl MapKey {
    /// The key a value denotes, or `None` when it cannot be one.
    pub fn of(v: &Value) -> Option<MapKey> {
        match v {
            Value::Bool(b) => Some(MapKey::Bool(*b)),
            Value::Int(i) => Some(MapKey::Int(*i)),
            Value::Str(s) => Some(MapKey::Str(s.to_string())),
            _ => None,
        }
    }

    /// The key back as a value, for `keys()`.
    pub fn to_value(&self) -> Value {
        match self {
            MapKey::Bool(b) => Value::Bool(*b),
            MapKey::Int(i) => Value::Int(*i),
            MapKey::Str(s) => Value::str(s.clone()),
        }
    }
}

/// The payload of a [`Value::Variant`] — an `Rc<Vec<Value>>` that tears itself down **iteratively**.
///
/// # Campaign finding INTERP-DROP-1
///
/// `main.rs` states `ref.rule.runtime.faults-are-diagnostics`: *a runtime fault must be a diagnostic
/// (`DL0905`), never a host crash*. The 512 MiB `delulu-main` stack exists precisely so the
/// interpreter's `MAX_DEPTH` is the limit that fires. But `MAX_DEPTH` bounds **call** depth, and
/// nothing bounded **data** depth: the derived `Drop` for a nested value recurses once per level, so
/// a 5,000,000-deep `type Chain = Nil | Link(Chain)` aborted the host — on Windows with
/// `0xC00000FD STATUS_STACK_OVERFLOW`, on Linux with `exit 134, fatal runtime error: stack overflow`
/// — *after* the program had printed its output and finished. The crash was in the runtime's own
/// teardown, and a bigger stack only moves the threshold.
///
/// **Why the destructor lives here and not on `Value`.** The recursion runs
/// `Value → Rc<Vec<Value>> → Vec<Value> → Value`, so the payload is the right place to break it.
/// Putting `impl Drop` on `Value` itself would forbid moving out of a `Value` anywhere in the crate
/// (`E0509`, measured: 9 sites in this crate alone before its dependents are reached) — for no gain,
/// since the cycle can be cut at either link.
///
/// **Why `Variant` is the only place this is needed.** Unbounded nesting requires a *recursive type*,
/// and a recursive type requires a sum: a directly recursive record is uninhabited (you would need a
/// value to build the first one), and `List[List[…]]` is a static type whose depth is bounded by the
/// source text. Every unbounded chain therefore passes through a `Variant` — including one that nests
/// through `Option`. The walker below still descends into `List` and `Record` children, so a mixed
/// structure is dismantled whole once the first `Variant` triggers it.
#[derive(Clone)]
pub struct VariantFields(Rc<Vec<Value>>);

impl VariantFields {
    pub fn new(fields: Vec<Value>) -> Self {
        VariantFields(Rc::new(fields))
    }

    /// Identity of the shared payload, for cycle detection (`cycles.rs`). Not the values' identity —
    /// the allocation's.
    pub fn ptr_id(&self) -> usize {
        Rc::as_ptr(&self.0) as *const () as usize
    }
}

/// Reading a variant's fields is what almost every call site does, so they keep working unchanged.
impl std::ops::Deref for VariantFields {
    type Target = Vec<Value>;
    fn deref(&self) -> &Vec<Value> {
        &self.0
    }
}

impl Drop for VariantFields {
    fn drop(&mut self) {
        // Only the LAST owner dismantles; a shared payload is still reachable, so this is just a
        // refcount decrement and the work belongs to whoever holds the final reference.
        let Some(owned) = Rc::get_mut(&mut self.0) else { return };
        // Depth becomes breadth: children are moved onto an explicit worklist instead of being
        // dropped in place, so teardown costs heap, which is bounded by the structure that already
        // fit in memory — never native stack, which is not.
        let mut work: Vec<Value> = std::mem::take(owned);
        while let Some(v) = work.pop() {
            match v {
                Value::Variant { fields, .. } => {
                    let mut fields = fields;
                    if let Some(inner) = Rc::get_mut(&mut fields.0) {
                        work.append(&mut std::mem::take(inner));
                    }
                    // `fields` drops here holding an EMPTY vec (or a still-shared Rc), so this
                    // recursion is one level deep and stops.
                }
                Value::List(rc) => {
                    let mut rc = rc;
                    if let Some(cell) = Rc::get_mut(&mut rc) {
                        work.append(&mut std::mem::take(cell.get_mut()));
                    }
                }
                Value::Record { fields, .. } => {
                    let mut rc = fields;
                    if let Some(cell) = Rc::get_mut(&mut rc) {
                        work.extend(std::mem::take(cell.get_mut()).into_iter().map(|(_, v)| v));
                    }
                }
                // A `Map` owns its values, so it joins the iterative walk for the same reason a
                // `List` does — INTERP-DROP-1 was a stack overflow in `Drop`, and a map of maps is
                // the same shape as a list of lists.
                Value::Map(m) => {
                    let mut rc = m;
                    if let Some(cell) = Rc::get_mut(&mut rc) {
                        work.extend(std::mem::take(cell.get_mut()).into_values());
                    }
                }
                // Every other variant owns no `Value` children, so dropping it is O(1).
                _ => {}
            }
        }
    }
}

impl Value {
    pub fn str(s: impl Into<String>) -> Value {
        Value::Str(Rc::from(s.into().as_str()))
    }
    pub fn variant(name: &str, fields: Vec<Value>) -> Value {
        Value::Variant { name: Rc::from(name), fields: VariantFields::new(fields) }
    }
    pub fn ok(v: Value) -> Value {
        Value::variant("Ok", vec![v])
    }
    pub fn err(v: Value) -> Value {
        Value::variant("Err", vec![v])
    }

    /// Human display (used by `println`, `str`, and the REPL). Opaque values never reveal.
    pub fn display(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Float(x) => {
                if x.fract() == 0.0 && x.is_finite() {
                    format!("{x:.1}")
                } else {
                    x.to_string()
                }
            }
            Value::Bool(b) => b.to_string(),
            Value::Str(s) => s.to_string(),
            Value::Unit => "()".to_string(),
            // Ascending by key, which is the same order `keys()` and `values()` give — so what a
            // test compares and what a program iterates cannot disagree.
            Value::Map(m) => {
                let inner: Vec<String> =
                    m.borrow().iter().map(|(k, v)| format!("{}: {}", k.to_value().display(), v.display())).collect();
                format!("{{{}}}", inner.join(", "))
            }
            Value::List(items) => {
                let inner: Vec<String> = items.borrow().iter().map(|v| v.display()).collect();
                format!("[{}]", inner.join(", "))
            }
            Value::Record { name, fields } => {
                let inner: Vec<String> =
                    fields.borrow().iter().map(|(n, v)| format!("{n}: {}", v.display())).collect();
                format!("{name} {{ {} }}", inner.join(", "))
            }
            Value::Variant { name, fields } => {
                if fields.is_empty() {
                    name.to_string()
                } else {
                    let inner: Vec<String> = fields.iter().map(|v| v.display()).collect();
                    format!("{name}({})", inner.join(", "))
                }
            }
            Value::Closure(_) => "<fn>".to_string(),
            Value::Cap(c) => format!("<cap {}>", c.kind.name()),
            Value::Secret(_) => "«secret»".to_string(),
            Value::Root(_) => "<root>".to_string(),
            // Opaque (R-5): the checker rejects `str`/`==`/serialize on these, so a well-typed
            // program never displays one; the runtime still refuses to reveal anything useful.
            Value::Foreign(h) => format!("<foreign lib {}>", h.name),
            Value::ForeignPtr(_) => "<foreign ptr>".to_string(),
            #[cfg(feature = "python")]
            Value::PyObj(_) => "<py obj>".to_string(),
            // Opaque (tag): identity never renders — the checker rejects `str`/`==` anyway.
            Value::ActorRef { actor, .. } => format!("<actor {actor}>"),
            // Opaque (R-5), like `Foreign` above: a plugin is code that arrived after compile time,
            // and neither its identity nor its exports reveal anything through `str`.
            Value::Plugin(p) => format!("<plugin {}>", p.name),
            Value::PluginFn(f) => format!("<plugin fn {}>", f.reference.export),
        }
    }

    /// Structural equality for the value types `==` accepts (opaque types are rejected by the
    /// checker with DL0605, so they never reach here in a well-typed program).
    ///
    /// Deliberately **not** `PartialEq`. This is DeluluLang's `==`, not Rust's: it is partial by
    /// design — the checker refuses the cases it does not handle — and implementing `PartialEq`
    /// would advertise a total, reflexive, symmetric relation that this is not obliged to be
    /// (`Float` alone breaks reflexivity on NaN). Clippy's warning is that the name can be confused
    /// for the trait method; the answer is that the confusion runs the other way, and a trait impl
    /// would let Rust code call it in contexts the language's own rules never sanctioned.
    #[allow(clippy::should_implement_trait)]
    pub fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Variant { name: n1, fields: f1 }, Value::Variant { name: n2, fields: f2 }) => {
                n1 == n2 && f1.len() == f2.len() && f1.iter().zip(f2.iter()).all(|(a, b)| a.eq(b))
            }
            (Value::List(a), Value::List(b)) => {
                let (a, b) = (a.borrow(), b.borrow());
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.eq(y))
            }
            _ => false,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Secret(_) => f.write_str("«secret»"),
            other => f.write_str(&other.display()),
        }
    }
}

/// A user or lambda closure: parameter names, the AST body, and the captured environment.
pub struct Closure {
    pub params: Vec<String>,
    pub body: Block,
    pub env: Env,
}

/// A compute device's envelope (Stage 10 Track F, spec §7.1): the SCOPE of a `Cap[Compute]`.
///
/// Vendor neutrality (invariant 49) is why `class`, `adapter` and `formats` are plain data the
/// interface carries rather than switches anything inspects: **no adapter may have semantics the
/// interface cannot express.** The day one vendor needs a privileged hook is the day the authority
/// model has a second class of citizen. Nothing in the dispatch path branches on `adapter`.
#[derive(Clone, Debug)]
pub struct ComputeEnvelope {
    pub device: String,
    /// Descriptive taxonomy (`cpu`, `gpu`, `tpu`) — recorded and displayed, never a decision input.
    pub class: String,
    /// Which adapter backs this device. `cpu-reference` is the only one in-tree.
    pub adapter: String,
    /// The buffer ceiling, in bytes. A dispatch whose buffer exceeds it is refused (DL1907).
    pub memory_bytes: u64,
    /// Kernel wall-clock budget, milliseconds, inclusive: a kernel that overruns is refused.
    pub kernel_ms: (f64, f64),
    /// How many dispatches may be in flight. Carried and enforced as a per-run counter.
    pub queue_depth: u32,
    /// Power envelope, watts. Carried; the reference adapter draws no measurable power and
    /// **does not pretend to enforce it** — see `ComputeBroker::dispatch`.
    pub power_w: (f64, f64),
    /// The kernel artifact formats this device accepts (`ptx-8`, `spirv-1.6`, `refkernel-1`).
    pub formats: Vec<String>,
    /// Kernel names this grant carries, each with the PATH of its artifact. A dispatch naming
    /// anything else is `UnknownKernel` — kernels are DATA, enumerated at grant time (§7.1).
    /// The path is grant data, exactly like `foreign.c=LIB:PATH`: the manifest says which kernels
    /// a package may reach for, the human says which bytes those names resolve to.
    pub kernels: Vec<(String, String)>,
    /// Whether the adapter attested independent below-adapter envelope enforcement, and — if not —
    /// whether a human explicitly waived that requirement. See DL1911.
    pub attested: bool,
    pub waived: bool,
}

impl ComputeEnvelope {
    /// Parse the grant form
    /// `DEVICE:memory_bytes=N,kernel_ms=lo..hi,queue_depth=N,power_w=lo..hi,adapter=A,
    ///  format=F[,format=F2],kernel=NAME:PATH[,kernel=N2:P2][,class=C][,waive-attestation]`.
    ///
    /// Fail-closed, and every bounding term is MANDATORY and refused **by name** when absent. The
    /// rule is 10f's, applied to silicon: a resource the grant forgot to bound is a resource
    /// nothing bounds, and defaulting one would be the runtime quietly making a decision that
    /// belongs to whoever is paying for the hardware.
    ///
    /// `class` is the one optional term, because it is the one term nothing branches on — it is
    /// display-only taxonomy (invariant 49). Defaulting a *descriptive* field is fine; defaulting
    /// a *bounding* field is the sin.
    ///
    /// Note what cannot be written here at all: an attestation. A grant may `waive-attestation`,
    /// never claim one — see `compute::check_grant`.
    pub fn parse(spec: &str) -> Result<ComputeEnvelope, String> {
        let (device, rest) =
            spec.split_once(':').ok_or("missing `:` (use DEVICE:memory_bytes=N,...)")?;
        let device = device.trim();
        if device.is_empty() {
            return Err("empty device name".into());
        }
        let (mut memory_bytes, mut queue_depth) = (None, None);
        let (mut kernel_ms, mut power_w) = (None, None);
        let (mut adapter, mut class) = (None, None);
        let (mut formats, mut kernels) = (Vec::new(), Vec::new());
        let mut waived = false;

        let range = |k: &str, v: &str| -> Result<(f64, f64), String> {
            let (lo, hi) = v.split_once("..").ok_or_else(|| format!("bad `{k}` range `{v}` (use lo..hi)"))?;
            let lo: f64 = lo.trim().parse().map_err(|_| format!("bad `{k}` bound `{lo}`"))?;
            let hi: f64 = hi.trim().parse().map_err(|_| format!("bad `{k}` bound `{hi}`"))?;
            // Same refusal, same reason as `ActuatorEnvelope::parse` (C41), and here the term's own
            // mandatory-ness made the gap sharper: `kernel_ms` is required because "a kernel with no
            // time budget can occupy the device forever", and `kernel_ms=0..inf` satisfied the
            // requirement while being exactly the unbounded budget the requirement exists to prevent.
            if !lo.is_finite() || !hi.is_finite() {
                return Err(format!(
                    "non-finite bound in `{k}={v}` — an envelope must be a real interval, and an \
                     infinite one bounds nothing while looking like a bound"
                ));
            }
            if lo > hi {
                return Err(format!("inverted range `{k}={lo}..{hi}`"));
            }
            Ok((lo, hi))
        };

        for part in rest.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            // The one valueless term. Spelled as a word rather than `waive=true` so it reads like
            // what it is in a shell history someone audits later.
            if part == "waive-attestation" {
                waived = true;
                continue;
            }
            let (k, v) = part.split_once('=').ok_or_else(|| format!("bad compute grant part `{part}`"))?;
            let (k, v) = (k.trim(), v.trim());
            match k {
                "memory_bytes" => {
                    let n: u64 = v.parse().map_err(|_| format!("bad memory_bytes `{v}`"))?;
                    if n == 0 {
                        return Err("memory_bytes=0 would refuse every dispatch, including the empty one".into());
                    }
                    memory_bytes = Some(n);
                }
                "queue_depth" => {
                    let n: u32 = v.parse().map_err(|_| format!("bad queue_depth `{v}`"))?;
                    if n == 0 {
                        return Err("queue_depth=0 would refuse every dispatch".into());
                    }
                    queue_depth = Some(n);
                }
                "kernel_ms" => kernel_ms = Some(range("kernel_ms", v)?),
                "power_w" => power_w = Some(range("power_w", v)?),
                "adapter" => adapter = Some(v.to_string()),
                "class" => class = Some(v.to_string()),
                "format" => formats.push(v.to_string()),
                "kernel" => {
                    // `NAME:PATH` — the provenance half of "kernels are data", and deliberately the
                    // same shape as `foreign.c=LIB:PATH`. Split at the FIRST `:` so a Windows path
                    // (`C:\k\reduce.refkernel`) survives intact on the right.
                    let (name, path) = v.split_once(':').ok_or_else(|| {
                        format!("bad kernel `{v}` — use kernel=NAME:PATH naming the artifact file")
                    })?;
                    let (name, path) = (name.trim(), path.trim());
                    if name.is_empty() || path.is_empty() {
                        return Err(format!("bad kernel `{v}` — both the name and the artifact path must be present"));
                    }
                    kernels.push((name.to_string(), path.to_string()));
                }
                _ => return Err(format!("unknown compute grant term `{k}`")),
            }
        }

        // Each refusal names the term the operator left out. An operator who forgot one should be
        // told which, not handed a syntax summary to diff by eye (10f's rule, D11a).
        let memory_bytes = memory_bytes.ok_or(
            "no `memory_bytes` — a device with no memory ceiling is a device with no ceiling",
        )?;
        let kernel_ms = kernel_ms.ok_or(
            "no `kernel_ms=lo..hi` — a kernel with no time budget can occupy the device forever",
        )?;
        let queue_depth = queue_depth
            .ok_or("no `queue_depth` — unbounded in-flight work is unbounded resource use")?;
        let power_w = power_w.ok_or(
            "no `power_w=lo..hi` — power is a real envelope dimension even where this build \
             cannot measure it (see the honesty note on enforcement)",
        )?;
        let adapter = adapter.ok_or(
            "no `adapter=` — which adapter backs this device is not something the runtime may pick \
             for you",
        )?;
        if formats.is_empty() {
            return Err(
                "no `format=` — a device that accepts any kernel format accepts kernels nobody \
                 vetted"
                    .into(),
            );
        }
        if kernels.is_empty() {
            return Err(
                "no `kernel=NAME:PATH` — a compute grant enumerates the kernels it carries, each \
                 pointing at the artifact file it resolves to; a grant that named none would \
                 either dispatch nothing or dispatch anything, and the second is how a kernel \
                 nobody signed gets to run"
                    .into(),
            );
        }
        Ok(ComputeEnvelope {
            device: device.to_string(),
            class: class.unwrap_or_else(|| "unspecified".to_string()),
            adapter,
            memory_bytes,
            kernel_ms,
            queue_depth,
            power_w,
            formats,
            kernels,
            // Set by the pre-flight from the ADAPTER's own declaration, never from this string.
            attested: false,
            waived,
        })
    }
}

/// An actuator's envelope (Stage 10 Track D, spec §5.1): the SCOPE of a `Cap[Actuator]`,
/// constructed by the human at grant time and enforced on every command. Dimensions are named
/// data, not hardcoded fields — vendor-neutral by construction (invariant 49): a joint bounds
/// `torque_nm`/`angle_deg`, a battery bounds `charge_a`/`soc_pct`, and the mechanism is one.
#[derive(Clone, Debug)]
pub struct ActuatorEnvelope {
    pub device: String,
    /// `(dimension, lo, hi)` — inclusive bounds. A command field naming a dimension not listed
    /// here is REFUSED (fail-closed): the envelope cannot vouch for what it never bounded.
    pub dims: Vec<(String, f64, f64)>,
    pub rate_hz: Option<u32>,
    /// Stage 10 (10f, spec §5.2): the dead-man terms. All three are MANDATORY — a device grant
    /// that does not say how fast the holder must prove it is alive, when the loan ends, and what
    /// the machine does when either fails is not a grant this runtime will issue (build-order
    /// D11a). Defaulting them would be the runtime making an operator's safety decision quietly.
    pub heartbeat_ms: u64,
    pub ttl_ms: u64,
    pub fail_state: crate::device::FailState,
}

impl ActuatorEnvelope {
    /// Parse the grant form
    /// `DEVICE:dim=lo..hi[,dim=lo..hi...][,rate_hz=N],heartbeat_ms=N,ttl_ms=N,fail=STATE`.
    /// Fail-closed: any part that does not parse is an error, never a silently-unbounded
    /// dimension and never a defaulted dead-man.
    pub fn parse(spec: &str) -> Result<ActuatorEnvelope, String> {
        let (device, rest) = spec.split_once(':').ok_or("missing `:` (use DEVICE:dim=lo..hi,...)")?;
        let device = device.trim();
        if device.is_empty() {
            return Err("empty device name".into());
        }
        let mut dims = Vec::new();
        let mut rate_hz = None;
        let mut heartbeat_ms = None;
        let mut ttl_ms = None;
        let mut fail_state = None;
        // Every term seen so far. A repeated term is refused rather than resolved — the broker's
        // `device_scope::parse` states the full reasoning, and the two must agree because a grant and
        // the capability value minted from it are the same envelope read twice (C40). This side kept
        // the FIRST occurrence (`dims` is a `Vec` and `envelope_check` stops at the first match) while
        // the broker kept the LAST, so appending a tighter bound tightened the record and not the
        // machine.
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for part in rest.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let (k, v) = part.split_once('=').ok_or_else(|| format!("bad envelope part `{part}`"))?;
            let (k, v) = (k.trim(), v.trim());
            if seen.contains(k) {
                return Err(format!(
                    "`{k}` appears twice in this envelope — a term stated twice is an ambiguity, and \
                     an ambiguity about a physical bound is refused rather than resolved (state \
                     `{k}` once)"
                ));
            }
            seen.insert(k.to_string());
            match k {
                "rate_hz" => {
                    rate_hz = Some(v.parse::<u32>().map_err(|_| format!("bad rate_hz `{v}`"))?);
                    continue;
                }
                "heartbeat_ms" => {
                    let n = v.parse::<u64>().map_err(|_| format!("bad heartbeat_ms `{v}`"))?;
                    if n == 0 {
                        return Err("heartbeat_ms=0 would revoke the lease before the first command".into());
                    }
                    heartbeat_ms = Some(n);
                    continue;
                }
                "ttl_ms" => {
                    let n = v.parse::<u64>().map_err(|_| format!("bad ttl_ms `{v}`"))?;
                    if n == 0 {
                        return Err("ttl_ms=0 would revoke the lease before the first command".into());
                    }
                    ttl_ms = Some(n);
                    continue;
                }
                "fail" => {
                    fail_state = Some(crate::device::FailState::parse(v).ok_or_else(|| {
                        format!("unknown fail-state `{v}` (use hold, coast, or safe-park)")
                    })?);
                    continue;
                }
                _ => {}
            }
            let (lo, hi) = v.split_once("..").ok_or_else(|| format!("bad range `{v}` (use lo..hi)"))?;
            let lo: f64 = lo.trim().parse().map_err(|_| format!("bad bound `{lo}`"))?;
            let hi: f64 = hi.trim().parse().map_err(|_| format!("bad bound `{hi}`"))?;
            // `"inf".parse::<f64>()` and `"NaN".parse::<f64>()` both succeed, so this branch is
            // reachable from any grant string. The broker has refused non-finite bounds since D12e
            // and documents the refusal as load-bearing; this side did not, which made
            // `angle_deg=-inf..inf` an envelope that passed every mandatory-term check while bounding
            // nothing (C41). An envelope that bounds nothing is not an envelope, and the machine is
            // moved from THIS side of the grammar.
            if !lo.is_finite() || !hi.is_finite() {
                return Err(format!(
                    "non-finite bound in `{k}={v}` — an envelope must be a real interval, and an \
                     infinite one bounds nothing while looking like a bound"
                ));
            }
            if lo > hi {
                return Err(format!("inverted range `{k}={lo}..{hi}`"));
            }
            dims.push((k.to_string(), lo, hi));
        }
        if dims.is_empty() {
            return Err("an envelope with no bounded dimension bounds nothing".into());
        }
        // The three dead-man terms are named individually in their refusals: an operator who
        // forgot one should be told which one, not handed a syntax summary to diff by eye.
        let heartbeat_ms = heartbeat_ms.ok_or(
            "no `heartbeat_ms` — a device grant with no dead-man is not a grant (spec §5.2)",
        )?;
        let ttl_ms = ttl_ms
            .ok_or("no `ttl_ms` — a lease with no end is a transfer of the device, not a loan")?;
        let fail_state = fail_state.ok_or(
            "no `fail=hold|coast|safe-park` — what this machine does when the software stops is \
             not a decision the runtime may make for you",
        )?;
        if ttl_ms < heartbeat_ms {
            return Err(format!(
                "ttl_ms={ttl_ms} is shorter than heartbeat_ms={heartbeat_ms} — the lease would \
                 expire before its first beat was ever due"
            ));
        }
        Ok(ActuatorEnvelope {
            device: device.to_string(),
            dims,
            rate_hz,
            heartbeat_ms,
            ttl_ms,
            fail_state,
        })
    }
}

/// The runtime capability scope (§7.1) — enforced host-side on every use.
#[derive(Clone, Debug)]
pub enum CapScope {
    Fs { root: PathBuf, write: bool },
    /// `allow` is the program's host list, each entry granted exactly. `special` is the subset of it
    /// the operator granted with `net.special=` (PS-B-02) — the only hosts the egress client will
    /// connect to in a special-use range. It travels WITH the capability, so an actor handed this
    /// capability holds exactly the special-use authority its minter held, never more.
    Net { allow: Vec<String>, special: Vec<String> },
    Console,
    Clock,
    Rand,
    Declassify { names: Vec<String> },
    /// Gates binding a foreign C library (`Cap[ForeignLoad]`, spec §3 T-ForeignBind). Carries no
    /// scope of its own — the per-lib authority decision is the `foreign.c` grant checked at bind.
    ForeignLoad,
    /// Stage 10 (10e): the actuator's scope IS its envelope (spec §5.1).
    Actuator(ActuatorEnvelope),
    /// Stage 10 (10e): the sensor's scope is its device identity; reads are `Read` under it.
    Sensor { device: String },
    /// Stage 10 (10h): the compute device's scope IS its envelope (spec §7.1, invariant 50).
    Compute(ComputeEnvelope),
    /// Gates embedded CPython (`Cap[Python]`, spec §5). Carries the granted import allowlist patterns
    /// (`foreign.python`); `py.import` is checked against them at runtime (DL1305).
    Python { allowlist: Vec<String> },
    /// P2 (D-V2-27): gates loading a plugin artifact at run time (`Cap[PluginHost]`).
    ///
    /// The scope IS the two halves of the loading decision, and both are the OPERATOR's: `roots` are
    /// the resolved directories or files `--grant plugin=` named, and `allow_hashes` is the package
    /// manifest's `[plugins] allow` list. A program holding this capability can name a path; it cannot
    /// name a root it was not given, and it cannot add a hash to the list. Empty `allow_hashes` means
    /// the package pinned nothing, and then the roots alone decide — pinning is the honest form, not a
    /// mandatory one.
    PluginHost { roots: Vec<PathBuf>, allow_hashes: Vec<String> },
    /// PS-A-02: a capability the HOST holds; this process has only the number the host minted for
    /// it. A guest's capabilities are all of this shape, which is what "the guest performs no
    /// effects" means concretely: there is no path, host or socket here to act on.
    ///
    /// The same shape as a daemon-mode `Secret` handle, and for the same reason. The primitive
    /// table refuses it (DL1401): an in-process performer cannot perform a host-held capability, so
    /// a handle that reaches the local path is a failure, never a silent no-op.
    Handle(u64),
}

/// An unforgeable capability value: a resource kind plus its scope.
pub struct CapVal {
    pub kind: ResourceKind,
    pub scope: CapScope,
}

/// A secret value. In EMBEDDED mode the bytes live here and only here until a lawful `expose`. In
/// DAEMON mode (Stage 5 phase 5g) a `Secret` is an opaque **handle** — the bytes live only in the
/// broker process (invariant 23) and cross into this process for the first time on `expose`, which
/// routes through `Custody::expose` (a broker round-trip). A handle carries NO secret bytes, so a
/// program-process memory scan finds nothing pre-`expose` (criterion 6).
pub struct SecretVal {
    inner: SecretInner,
}

enum SecretInner {
    /// Embedded: the bytes are in-process (Stage 1–4 behavior).
    Local(RefCell<String>),
    /// Daemon: only the broker secret NAME — an opaque reference; the bytes are broker-resident.
    Handle(String),
}

impl SecretVal {
    /// An in-process (embedded-mode) secret carrying its bytes.
    pub fn new(v: String) -> SecretVal {
        SecretVal { inner: SecretInner::Local(RefCell::new(v)) }
    }

    /// A daemon-mode handle: an opaque reference to a broker-held secret `name` — NO bytes here.
    pub fn handle(name: impl Into<String>) -> SecretVal {
        SecretVal { inner: SecretInner::Handle(name.into()) }
    }

    /// The broker secret name, if this is a daemon-mode handle (else `None`). Used by the
    /// interpreter to route `expose` through `Custody::expose`.
    pub fn handle_name(&self) -> Option<&str> {
        match &self.inner {
            SecretInner::Handle(name) => Some(name),
            SecretInner::Local(_) => None,
        }
    }

    /// The only in-process reader — reached solely through `Secret.expose(Cap[Declassify])`
    /// (§6.4/R-2) in EMBEDDED mode. A daemon handle carries no bytes (they never entered this
    /// process), so `reveal` yields the empty string for a handle; the interpreter routes a handle's
    /// `expose` through `Custody::expose` instead of ever calling this.
    pub fn reveal(&self) -> String {
        match &self.inner {
            SecretInner::Local(v) => v.borrow().clone(),
            SecretInner::Handle(_) => String::new(),
        }
    }

    /// Constant-time-ish comparison for `Secret.verify` (no early return on mismatch). Two daemon
    /// handles (no local bytes) never compare equal in v0.5 — broker-side `verify` is post-chunk-3
    /// (flagged); embedded comparison is unchanged.
    pub fn verify(&self, other: &SecretVal) -> bool {
        let (SecretInner::Local(a), SecretInner::Local(b)) = (&self.inner, &other.inner) else {
            return false;
        };
        let a = a.borrow();
        let b = b.borrow();
        if a.len() != b.len() {
            return false;
        }
        let mut acc = 0u8;
        for (x, y) in a.bytes().zip(b.bytes()) {
            acc |= x ^ y;
        }
        acc == 0
    }
}

impl Drop for SecretVal {
    fn drop(&mut self) {
        // Zeroization of any in-process bytes. A daemon handle holds none — the broker owns the
        // bytes (invariant 23).
        //
        // P17-F: this loop used to be a plain `*b = 0`, which the compiler is entitled to DELETE.
        // A non-volatile store to memory that is never read again is a dead store, and the
        // allocation is freed on the next line — so LLVM may remove the whole loop and the secret
        // stays in the freed heap block. "Best-effort" understated it: the effort could be zero,
        // and nothing in the build would say so. This is exactly why the `zeroize` crate exists.
        //
        // `write_volatile` may not be elided, and the fence stops the write being sunk past the
        // deallocation. `std` alone is enough here, so this adds no dependency (`zeroize` is
        // already in the tree via ml-dsa/ml-kem, but only for THEIR key material).
        //
        // Still honest about the limit: this zeroes the CURRENT allocation only. Any earlier
        // buffer left behind by a `String` reallocation, and any copy made by `reveal`, is not
        // reachable from here and is not zeroed. Zeroization bounds exposure; it does not
        // eliminate it.
        if let SecretInner::Local(v) = &self.inner {
            let mut v = v.borrow_mut();
            // SAFETY: writing 0 preserves the UTF-8 invariant (NUL is valid UTF-8), and the
            // `String` is dropped immediately afterwards regardless.
            let bytes = unsafe { v.as_bytes_mut() };
            for b in bytes.iter_mut() {
                // SAFETY: `b` is a valid, aligned, uniquely-borrowed `u8` from the slice above.
                unsafe { std::ptr::write_volatile(b, 0) };
            }
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
        }
    }
}

/// Root authority: exactly the slice the human/broker granted (§7.2). Constructed only by the
/// broker; `main` is the sole place it enters a program.
#[derive(Default)]
pub struct RootVal {
    pub console: bool,
    pub fs_read: Vec<PathBuf>,
    pub fs_write: Vec<PathBuf>,
    pub net: Vec<String>,
    /// PS-B-02: the subset of `net` granted with the explicit spelling `net.special=` (see
    /// `Grants::net_special`). A capability minted for one of these hosts may connect into a
    /// special-use range; one minted for any other host may not.
    pub net_special: Vec<String>,
    pub clock: bool,
    pub rand: bool,
    pub declassify: bool,
    pub secrets: HashMap<String, String>,
    /// Whether `root.foreign_load()` may mint a `Cap[ForeignLoad]` — true iff any `foreign.c` lib or
    /// `foreign.python` pattern was granted (spec §4.1/§5.1). The per-lib gate is enforced separately
    /// when `root.foreign(load)` binds; the per-import gate is the allowlist checked at `py.import`.
    pub foreign_load: bool,
    /// Granted `foreign.python` import allowlist patterns (spec §5.1). Non-empty iff Python is
    /// granted; `root.python(load)` mints `Cap[Python]` carrying these, and refuses `NotGranted`
    /// (DL1303) when empty.
    pub python_allowlist: Vec<String>,
    /// DAEMON mode (Stage 5 phase 5g): the NAMES of broker-held secrets available to this program.
    /// `root.secret(name)` for a name in this set returns an opaque **handle** (no bytes) — the bytes
    /// stay in the broker until `expose`. Empty in embedded mode, where `secrets` carries the bytes.
    pub broker_secrets: Vec<String>,
    /// Stage 10 (10e): granted actuator envelopes; `root.actuator(device)` mints the matching cap.
    pub actuators: Vec<ActuatorEnvelope>,
    /// Stage 10 (10e): granted sensor devices; `root.sensor(device)` mints the matching cap.
    pub sensors: Vec<String>,
    /// Stage 10 (10h): granted compute devices; `root.compute(device)` mints the matching cap.
    pub computes: Vec<ComputeEnvelope>,
    /// P2 (D-V2-27): resolved roots a plugin artifact may be loaded from (`--grant plugin=`).
    /// `root.plugin_host()` mints `Cap[PluginHost]` iff this is non-empty.
    pub plugins: Vec<PathBuf>,
    /// P2 (D-V2-27): the package manifest's `[plugins] allow` hash ceiling, carried into the
    /// capability. A ceiling, never a source of authority.
    pub plugins_allow: Vec<String>,
}

// ----- environments --------------------------------------------------------

pub type Env = Rc<Scope>;

pub struct Scope {
    vars: RefCell<HashMap<String, Value>>,
    parent: Option<Env>,
}

impl Scope {
    pub fn root() -> Env {
        Rc::new(Scope { vars: RefCell::new(HashMap::new()), parent: None })
    }
    pub fn child(parent: &Env) -> Env {
        Rc::new(Scope { vars: RefCell::new(HashMap::new()), parent: Some(parent.clone()) })
    }
    pub fn define(&self, name: &str, value: Value) {
        self.vars.borrow_mut().insert(name.to_string(), value);
    }
    /// Visit every binding's value (Stage 10 cycle collector, `cycles.rs`).
    pub fn each_value(&self, mut f: impl FnMut(&Value)) {
        for v in self.vars.borrow().values() {
            f(v);
        }
    }
    /// Empty this scope's bindings — called by the cycle collector ONLY on scopes it has
    /// proven unreachable from every root; the drop cascade collapses the rest of the cycle.
    pub fn clear_for_collector(&self) {
        self.vars.borrow_mut().clear();
    }
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.vars.borrow().get(name) {
            return Some(v.clone());
        }
        self.parent.as_ref().and_then(|p| p.get(name))
    }
    /// The parent scope, if any (Stage 7: the actor boundary flattens capture chains).
    pub fn parent(&self) -> Option<&Env> {
        self.parent.as_ref()
    }

    /// A snapshot of this scope's own bindings (Stage 7: closure conversion at the actor
    /// boundary — sendable closures capture only immutable values, so a snapshot is exact).
    pub fn vars_snapshot(&self) -> Vec<(String, Value)> {
        self.vars.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Assign to an existing binding in the nearest enclosing scope. Returns false if unbound.
    pub fn assign(&self, name: &str, value: Value) -> bool {
        if self.vars.borrow().contains_key(name) {
            self.vars.borrow_mut().insert(name.to_string(), value);
            true
        } else if let Some(p) = &self.parent {
            p.assign(name, value)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod teardown_tests {
    use super::*;

    /// **INTERP-DROP-1 regression lock.** A deeply nested value must tear down without recursing the
    /// native stack.
    ///
    /// Before the fix this aborted the whole test binary — a stack overflow is not a catchable
    /// panic, so the falsification signal is the process dying, not an assertion failing. That is
    /// exactly the failure mode the rule `ref.rule.runtime.faults-are-diagnostics` forbids: a runtime
    /// fault must be a diagnostic (`DL0905`), never a host crash. `MAX_DEPTH` bounds *call* depth;
    /// this bounds nothing — it makes teardown cost heap instead of stack, so the only limit is the
    /// memory the structure already occupies.
    ///
    /// Depth is chosen to be decisive on a *test* thread's stack (far smaller than the 512 MiB
    /// `delulu-main` reserves), so the lock fires here without needing the CLI's reservation.
    #[test]
    fn a_deeply_nested_value_tears_down_without_recursing_the_native_stack() {
        let mut v = Value::variant("Nil", vec![]);
        for _ in 0..500_000 {
            v = Value::variant("Link", vec![v]);
        }
        drop(v); // the operation under test
    }

    /// The same for a chain that alternates variant and list levels, because the walker has to
    /// descend through `List`/`Record` children too — a structure that nests through a list between
    /// every variant must not find a recursive path back.
    #[test]
    fn a_mixed_variant_list_record_nesting_also_tears_down_iteratively() {
        let mut v = Value::variant("Nil", vec![]);
        for i in 0..200_000 {
            v = Value::List(Rc::new(RefCell::new(vec![v])));
            v = Value::Record { name: Rc::from("Box"), fields: Rc::new(RefCell::new(vec![(format!("f{i}"), v)])) };
            v = Value::variant("Link", vec![v]);
        }
        drop(v);
    }

    /// Teardown must not disturb a payload someone else still holds: the last owner does the work,
    /// and an earlier drop is only a refcount decrement.
    #[test]
    fn a_shared_variant_payload_survives_until_its_last_owner_drops() {
        let inner = Value::variant("Leaf", vec![Value::Int(7)]);
        let a = Value::variant("Holder", vec![inner.clone()]);
        let b = a.clone(); // shares the same Rc payload
        drop(a);
        // `b` must still be intact and readable after its sibling went away.
        match &b {
            Value::Variant { name, fields } => {
                assert_eq!(&**name, "Holder");
                assert_eq!(fields.len(), 1, "the shared payload must survive the first drop");
                assert!(matches!(&fields[0], Value::Variant { .. }), "and its child with it");
            }
            _ => panic!("expected a variant"),
        }
    }
}
