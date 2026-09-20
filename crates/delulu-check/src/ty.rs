//! The representation of authority in the type system (spec §6.1).
//!
//! This is the heart of DeluluLang. An effect *row* is a static summary of what a function
//! may do; a capability *value* is its permission. Effects arise ONLY from capability
//! operations (the primitive table), so a function holding no capability performs no effect,
//! and the row is a sound over-approximation of runtime behavior (audit §D).

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};

/// A primitive or user-declared effect label. Primitive effects arise only from capability
/// operations (§6.2 T-CapOp); `Declassify` is carried by `Secret.expose` (audit R-2).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Effect {
    Read,
    Write,
    Net,
    Clock,
    Rand,
    Declassify,
    /// Loading a plugin at runtime (Stage 6). Reserved in Stage 1 §2, active here: `load` carries
    /// `{Load, Read}`, so a program that can bring in code after compile time says so in its row.
    Load,
    /// Concurrency (Stage 7, spec §5): `spawn` and behavior sends carry `Async` — and ONLY an
    /// effect: no futures runtime, no await, no colored functions. Unlike the other core
    /// effects it arises from language constructs (T-Spawn/T-Send), not capability ops; the
    /// extended soundness statement (invariant 35) covers exactly this.
    Async,
    /// Reaching foreign (C/Python) code (Stage 4). An ordinary core effect for every row purpose;
    /// nothing special-cases it except reporting (spec §3).
    ForeignCall,
    /// Commanding a physical device (Stage 10 Track D, Constitution §7). Reserved in Stage 1,
    /// active here: `actuator.command(...)` carries `Actuate` — the most physically consequential
    /// effect in the language, synchronous-class at the broker. Sensor READS are `Read` with
    /// sensor scopes, deliberately not a new effect (spec §5.1).
    Actuate,
    /// A user-declared effect (`effect Name`), identified by its name.
    User(String),
}

impl Effect {
    pub fn core_from_name(name: &str) -> Option<Effect> {
        Some(match name {
            "Read" => Effect::Read,
            "Write" => Effect::Write,
            "Net" => Effect::Net,
            "Clock" => Effect::Clock,
            "Rand" => Effect::Rand,
            "Declassify" => Effect::Declassify,
            "ForeignCall" => Effect::ForeignCall,
            "Load" => Effect::Load,
            "Async" => Effect::Async,
            "Actuate" => Effect::Actuate,
            _ => return None,
        })
    }

    pub fn name(&self) -> &str {
        match self {
            Effect::Read => "Read",
            Effect::Write => "Write",
            Effect::Net => "Net",
            Effect::Clock => "Clock",
            Effect::Rand => "Rand",
            Effect::Declassify => "Declassify",
            Effect::ForeignCall => "ForeignCall",
            Effect::Load => "Load",
            Effect::Async => "Async",
            Effect::Actuate => "Actuate",
            Effect::User(n) => n,
        }
    }
}

impl fmt::Display for Effect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A row variable (`e` in `fn(T) -> U ! e`), resolved through the inference substitution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RowVar(pub u32);

/// An effect row: a set of concrete effects plus an optional polymorphic tail.
/// `{Read, Net}` is closed; `{Read | e}` is open with tail `e`; `{}` is provably pure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Row {
    pub effects: BTreeSet<Effect>,
    pub tail: Option<RowVar>,
}

impl Row {
    pub fn pure() -> Row {
        Row { effects: BTreeSet::new(), tail: None }
    }

    pub fn single(e: Effect) -> Row {
        let mut s = BTreeSet::new();
        s.insert(e);
        Row { effects: s, tail: None }
    }

    pub fn closed(effects: BTreeSet<Effect>) -> Row {
        Row { effects, tail: None }
    }

    pub fn is_pure(&self) -> bool {
        self.effects.is_empty() && self.tail.is_none()
    }

    /// Union of concrete effects (used by T-Call/T-If/T-Match to combine callee rows).
    /// If either side has a tail, the result keeps a tail — callers resolve via unification.
    pub fn union(&self, other: &Row) -> Row {
        let mut effects = self.effects.clone();
        effects.extend(other.effects.iter().cloned());
        let tail = self.tail.or(other.tail);
        Row { effects, tail }
    }

    pub fn add(&mut self, e: Effect) {
        self.effects.insert(e);
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("!{")?;
        let mut first = true;
        for e in &self.effects {
            if !first {
                f.write_str(", ")?;
            }
            first = false;
            write!(f, "{e}")?;
        }
        if let Some(RowVar(v)) = self.tail {
            if !first {
                f.write_str(" | ")?;
            }
            write!(f, "'r{v}")?;
        }
        f.write_str("}")
    }
}

/// A type variable, resolved through the inference substitution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeVar(pub u32);

/// Index of a user-declared record or sum type in the program's type table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TypeDefId(pub u32);

/// The resource a capability designates (§7.3). Capabilities are unforgeable: `Cap[R]` has no
/// literal and no constructor; values originate only from `Root` or by attenuation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ResourceKind {
    FsRead,
    FsWrite,
    Console,
    Http,
    Clock,
    Rand,
    Declassify,
    /// Reserved in Stage 1 so plugin/host signatures are stable (§8.2).
    PluginHost,
    /// Gates embedding CPython (`Cap[Python]`, Stage 4, spec §5).
    Python,
    /// Gates binding a foreign C library (`Cap[ForeignLoad]`, Stage 4, spec §3 T-ForeignBind).
    ForeignLoad,
    /// A physical actuator (Stage 10 Track D, Constitution §7): the most physically consequential
    /// capability. Its SCOPE is its envelope — device identity plus per-dimension numeric bounds —
    /// enforced at runtime on every command (§5.3's kind/scope split, at physical stakes).
    Actuator,
    /// A physical sensor (Stage 10 Track D): reads are `Read` with sensor scopes.
    Sensor,
    /// An accelerator or compute device (Stage 10 Track F, invariant 50). Its SCOPE is its device
    /// envelope — memory ceiling, kernel time budget, queue depth, power. Dispatching a kernel
    /// carries the existing core `ForeignCall` effect and **no new one**: a kernel is foreign code,
    /// and DeluluLang bounds its reachability, resources and provenance — never its computation.
    Compute,
}

impl ResourceKind {
    pub fn from_name(name: &str) -> Option<ResourceKind> {
        Some(match name {
            "FsRead" => ResourceKind::FsRead,
            "FsWrite" => ResourceKind::FsWrite,
            "Console" => ResourceKind::Console,
            "Http" => ResourceKind::Http,
            "Clock" => ResourceKind::Clock,
            "Rand" => ResourceKind::Rand,
            "Declassify" => ResourceKind::Declassify,
            "PluginHost" => ResourceKind::PluginHost,
            "Python" => ResourceKind::Python,
            "ForeignLoad" => ResourceKind::ForeignLoad,
            "Actuator" => ResourceKind::Actuator,
            "Sensor" => ResourceKind::Sensor,
            "Compute" => ResourceKind::Compute,
            _ => return None,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            ResourceKind::FsRead => "FsRead",
            ResourceKind::FsWrite => "FsWrite",
            ResourceKind::Console => "Console",
            ResourceKind::Http => "Http",
            ResourceKind::Clock => "Clock",
            ResourceKind::Rand => "Rand",
            ResourceKind::Declassify => "Declassify",
            ResourceKind::PluginHost => "PluginHost",
            ResourceKind::Python => "Python",
            ResourceKind::ForeignLoad => "ForeignLoad",
            ResourceKind::Actuator => "Actuator",
            ResourceKind::Sensor => "Sensor",
            ResourceKind::Compute => "Compute",
        }
    }
}

/// The trust class of a loaded plugin (Stage 6, spec §2.1). Written as the type argument of
/// `Plugin[C]`, where `Verified` and `Contained` are nullary **marker types** ([`Type::Verified`] /
/// [`Type::Contained`]) — that is what lets `C` be an ordinary inference variable pinned by the
/// binding's annotation (build-order deviation 4), exactly as Stage 4 handles `root.foreign[M]`.
///
/// **The class is never inferred from the artifact** (invariant 29): this is what the *host asked
/// for*, and the loader verifies the artifact's declaration against it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PluginClass {
    Verified,
    Contained,
}

impl PluginClass {
    pub fn as_str(self) -> &'static str {
        match self {
            PluginClass::Verified => "verified",
            PluginClass::Contained => "contained",
        }
    }

    /// From the artifact/manifest spelling (`"verified"` / `"contained"`). Named `from_name` (not
    /// `from_str`) so it is not confused with `std::str::FromStr::from_str`, whose `Result` contract
    /// this `Option`-returning parser does not follow (clippy `should_implement_trait`).
    pub fn from_name(s: &str) -> Option<PluginClass> {
        match s {
            "verified" => Some(PluginClass::Verified),
            "contained" => Some(PluginClass::Contained),
            _ => None,
        }
    }

    /// From the source-level marker type name (`Verified` / `Contained`).
    pub fn from_type_name(s: &str) -> Option<PluginClass> {
        match s {
            "Verified" => Some(PluginClass::Verified),
            "Contained" => Some(PluginClass::Contained),
            _ => None,
        }
    }

    pub fn type_name(self) -> &'static str {
        match self {
            PluginClass::Verified => "Verified",
            PluginClass::Contained => "Contained",
        }
    }
}

/// A DeluluLang type (§6.1). Function types carry their row — rows never erase (invariant 3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    Unit,
    List(Box<Type>),
    /// `Map[K, V]` (P3, ruling D-V2-29). `K` is `Str`, `Int` or `Bool` in 1.x — refused otherwise at
    /// the call site that pins it, because `Float` has no total order and a structural key's
    /// canonical form is a design rather than a detail. Iteration is ascending by key, so `keys()`
    /// and `values()` agree with each other and across runs.
    Map(Box<Type>, Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Record(TypeDefId, Vec<Type>),
    Sum(TypeDefId, Vec<Type>),
    Fn { params: Vec<Type>, ret: Box<Type>, row: Row },
    Cap(ResourceKind),
    Secret(Box<Type>),
    Root,
    /// An opaque C pointer (`ForeignPtr`, Stage 4). Marshallable across the FFI, but R-5 opaque:
    /// no `str`/`==`/serialization.
    ForeignPtr,
    /// An opaque embedded-Python object (`PyObj`, Stage 4). R-5 opaque and **not** marshallable
    /// in a `foreign "c"` signature (spec §3 T-Py).
    PyObj,
    /// The nominal opaque handle type introduced by a `foreign … lib M` block, named `M`
    /// (spec §2). R-5 opaque; not marshallable in a foreign signature.
    Foreign(String),
    /// A loaded plugin handle, `Plugin[C]` (Stage 6, spec §3). The argument is the class marker —
    /// [`Type::Verified`], [`Type::Contained`], or (before the annotation pins it) a `Var`.
    /// R-5 opaque: no `str`, no `==`, no serialization, and never marshallable across a boundary.
    Plugin(Box<Type>),
    /// The `Verified` class marker (only meaningful as `Plugin`'s argument).
    Verified,
    /// The `Contained` class marker (only meaningful as `Plugin`'s argument).
    Contained,
    /// An actor reference type, `A` or `A[T]` (Stage 7, spec §2). As a type it is ALWAYS
    /// `tag` (writing another rcap on it is DL1607): outsiders hold opaque identity, and
    /// messages are the only cross-actor interface (T-SyncMethod). Name-based like
    /// [`Type::Foreign`]; never enters plugin DIR (actors in plugins are fenced, v0.7).
    Actor(String, Vec<Type>),
    Var(TypeVar),
}

impl Type {
    pub fn unit() -> Type {
        Type::Unit
    }

    pub fn list(t: Type) -> Type {
        Type::List(Box::new(t))
    }

    pub fn result(ok: Type, err: Type) -> Type {
        Type::Result(Box::new(ok), Box::new(err))
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }
}

/// Resolves a [`TypeDefId`] back to the name the author wrote.
///
/// [`Type::Record`] and [`Type::Sum`] store an **index** into the declaration table, not a name, so
/// a printer that does not hold the table can only emit `T11`. That was C12: a diagnostic naming
/// the *shape* of a disagreement (`Result[Str, T0]` vs `Result[Str, T1]`) while hiding its
/// *content* (`IoErr` vs `NetErr`) — accurate and useless.
///
/// There is deliberately **no `Display for Type`**. Rendering a type requires supplying the names,
/// so no site can print a nameless one by forgetting to — the compiler asks, rather than a
/// convention that decays. This is the same shape as the campaign's first rule: replace "remember
/// to do it" with something that will not build otherwise.
pub trait TypeNames {
    /// The declared name of `id`, or `None` if this table does not hold it.
    fn type_name(&self, id: TypeDefId) -> Option<&str>;
}

/// A [`TypeNames`] that knows no names, for the few printers with no declaration table in reach —
/// the runtime plugin loader reports on types recovered from a DIR, after the table is gone.
///
/// Nominal types then render as `<type #11>`: visibly unresolved. That is the point. It is not an
/// improvement on `T11` as *information*, it is an improvement as *honesty* — a reader can tell the
/// printer failed instead of reading a plausible name that does not exist. Never use this where a
/// table is available; the compiler cannot tell the difference, so this is the one place in the
/// mechanism that still relies on judgement.
pub struct NoTypeNames;

impl TypeNames for NoTypeNames {
    fn type_name(&self, _id: TypeDefId) -> Option<&str> {
        None
    }
}

/// Type names indexed by [`TypeDefId`], owned rather than borrowed.
///
/// Both whole-program paths (`check_program`, `check_workspace`) build ONE global type registry
/// across every module, so a single flat list can name any type in the program. `Program` carries
/// this because it does not keep the per-module `DeclTable`s the check ran against — and without
/// it `interface.json`, the artifact whose stated purpose is letting an agent introspect a
/// dependency without reading its source, published `fn(T9) -> Float`.
#[derive(Clone, Debug, Default)]
pub struct TypeNameList(pub Vec<String>);

impl TypeNames for TypeNameList {
    fn type_name(&self, id: TypeDefId) -> Option<&str> {
        self.0.get(id.0 as usize).map(String::as_str)
    }
}

/// A [`Type`] bound to the names of its nominal parts. Produced by [`Type::show`].
pub struct Shown<'a> {
    ty: &'a Type,
    names: &'a dyn TypeNames,
}

impl Type {
    /// Render this type for a human, naming records and sums.
    pub fn show<'a>(&'a self, names: &'a dyn TypeNames) -> Shown<'a> {
        Shown { ty: self, names }
    }
}

fn write_args(f: &mut fmt::Formatter<'_>, args: &[Type], names: &dyn TypeNames) -> fmt::Result {
    if args.is_empty() {
        return Ok(());
    }
    f.write_str("[")?;
    for (i, a) in args.iter().enumerate() {
        if i > 0 {
            f.write_str(", ")?;
        }
        write!(f, "{}", a.show(names))?;
    }
    f.write_str("]")
}

impl fmt::Display for Shown<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let n = self.names;
        match self.ty {
            Type::Int => f.write_str("Int"),
            Type::Float => f.write_str("Float"),
            Type::Bool => f.write_str("Bool"),
            Type::Str => f.write_str("Str"),
            Type::Unit => f.write_str("Unit"),
            Type::List(t) => write!(f, "List[{}]", t.show(n)),
            Type::Map(k, v) => write!(f, "Map[{}, {}]", k.show(n), v.show(n)),
            Type::Option(t) => write!(f, "Option[{}]", t.show(n)),
            Type::Result(o, e) => write!(f, "Result[{}, {}]", o.show(n), e.show(n)),
            Type::Record(id, args) | Type::Sum(id, args) => {
                // The fallback is deliberately unmistakable. `T11` was indistinguishable from a
                // type an author could have written, so a printer failure read as an answer;
                // `<type #11>` is not source-level syntax and can only mean this lookup failed.
                match n.type_name(*id) {
                    Some(name) => f.write_str(name)?,
                    None => write!(f, "<type #{}>", id.0)?,
                }
                write_args(f, args, n)
            }
            Type::Fn { params, ret, row } => {
                f.write_str("fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{}", p.show(n))?;
                }
                write!(f, ") -> {}", ret.show(n))?;
                if !row.is_pure() {
                    write!(f, " {row}")?;
                }
                Ok(())
            }
            Type::Cap(r) => write!(f, "Cap[{}]", r.name()),
            Type::Secret(t) => write!(f, "Secret[{}]", t.show(n)),
            Type::Root => f.write_str("Root"),
            Type::ForeignPtr => f.write_str("ForeignPtr"),
            Type::PyObj => f.write_str("PyObj"),
            Type::Foreign(name) => f.write_str(name),
            Type::Plugin(c) => write!(f, "Plugin[{}]", c.show(n)),
            Type::Verified => f.write_str("Verified"),
            Type::Contained => f.write_str("Contained"),
            Type::Actor(name, args) => {
                f.write_str(name)?;
                write_args(f, args, n)
            }
            Type::Var(TypeVar(v)) => write!(f, "'t{v}"),
        }
    }
}
