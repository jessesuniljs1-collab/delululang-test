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
    Record { name: Rc<str>, fields: Rc<RefCell<Vec<(String, Value)>>> },
    /// A sum-type value: builtin (`Ok`/`Err`/`Some`/`None`) or a user variant, by name.
    Variant { name: Rc<str>, fields: Rc<Vec<Value>> },
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
}

impl Value {
    pub fn str(s: impl Into<String>) -> Value {
        Value::Str(Rc::from(s.into().as_str()))
    }
    pub fn variant(name: &str, fields: Vec<Value>) -> Value {
        Value::Variant { name: Rc::from(name), fields: Rc::new(fields) }
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
        }
    }

    /// Structural equality for the value types `==` accepts (opaque types are rejected by the
    /// checker with DL0605, so they never reach here in a well-typed program).
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

/// The runtime capability scope (§7.1) — enforced host-side on every use.
#[derive(Clone, Debug)]
pub enum CapScope {
    Fs { root: PathBuf, write: bool },
    Net { allow: Vec<String> },
    Console,
    Clock,
    Rand,
    Declassify { names: Vec<String> },
    /// Gates binding a foreign C library (`Cap[ForeignLoad]`, spec §3 T-ForeignBind). Carries no
    /// scope of its own — the per-lib authority decision is the `foreign.c` grant checked at bind.
    ForeignLoad,
    /// Gates embedded CPython (`Cap[Python]`, spec §5). Carries the granted import allowlist patterns
    /// (`foreign.python`); `py.import` is checked against them at runtime (DL1305).
    Python { allowlist: Vec<String> },
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
        // Best-effort zeroization of any in-process bytes. A daemon handle holds none — the broker
        // owns the bytes (invariant 23).
        if let SecretInner::Local(v) = &self.inner {
            let mut v = v.borrow_mut();
            unsafe {
                for b in v.as_bytes_mut() {
                    *b = 0;
                }
            }
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
