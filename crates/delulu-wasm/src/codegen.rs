//! Phase 3a/3b code generation: DeluluLang → core WebAssembly.
//!
//! Phase 3a (pure): `Int` (i64), `Bool` (i32), arithmetic, comparisons, `&&`/`||`, unary `-`/`!`,
//! `if`/`else`, `let`, calls, recursion.
//! Phase 3b (the `delulu:cap` floor, incrementally): `Str` values (string LITERALS live in the
//! module's linear memory, length-prefixed; a `Str` is an i32 pointer to `[len:u32-le][bytes]`),
//! `Cap[Console]` values (i32 handles into a host cap table), `Unit`, and `Cap[Console].println`
//! compiled to an imported host function `delulu:cap.console_println(cap, str_ptr)`. The host
//! performs the effect and its scope check (§4); the guest never touches an OS handle.
//! Phase 3i (`Str` concatenation): `Str + Str` compiles to a synthetic `__concat` helper that
//! bump-allocates a fresh `[len:u32-le][bytes]` buffer in guest linear memory (a mutable global is
//! the bump pointer, starting just past the interned literals) and returns its pointer.
//! Phase 3j (`str(Int)`): the `str` builtin on an `Int` compiles to a synthetic `__int_to_str`
//! helper that formats the integer as decimal into the bump heap (matching `i64::to_string`,
//! `i64::MIN` included); `str` on a `Str` is the identity.
//! Phase 3k (`Cap[Clock]`): `root.clock()` and `clk.now_ms()` compile to the `delulu:cap` host
//! imports `root_clock`/`clock_now_ms`; the clock is read host-side (fixed under `--clock fixed:MS`
//! for deterministic replay, else the wall clock), so a fixed clock gives byte-identical output.
//! Phase 3l (`Cap[Rand]`): `root.rand()` and `r.int(lo, hi)` compile to `root_rand`/`rand_int`; the
//! host runs the interpreter's exact xorshift64 PRNG, seeded via `--seed`, so seeded random
//! sequences match byte-for-byte across engines.
//!
//! Phase 3o (sum types): `Result[T,E]`/`Option[T]` with scalar payloads compile to heap `[tag][field]`
//! cells; `Ok`/`Err`/`Some`/`None` construct them (expected-type-directed, since the backend does no
//! inference), and `match` lowers to a tag test with typed field binding.
//! Phase 3p (filesystem cap): `root.fs_read(path)` and `fs.read_text(rel)` compile to the `delulu:cap`
//! imports `root_fs_read`/`fs_read_text`; the host performs the scoped read and CONSTRUCTS the
//! `Result[Str, IoErr]` cells in guest memory (the guest heap `__heap` global is exported for it).
//!
//! Stage 4 phase 4g (foreign C FFI): `root.foreign_load()`, `root.foreign(load)`, and method calls
//! on a bound lib handle compile to the `delulu:foreign` host imports (`foreign_load`/`foreign_bind`/
//! `foreign_call`/`float_to_str`); ALL FFI (spec §4.1–4.2 — grants, bind-time symbol resolution,
//! marshalling, return validation) runs host-side and the guest never touches a raw pointer. Effect
//! host fns now carry a `(file,start,end)` span so the host records `TraceRecord`s identical to the
//! interpreter's (criterion 6). `Float` literals, `str(Float)`, and `str(Bool)` compile in support.
//!
//! RW 3.4 (D-V2-163): `while`, `break`, `continue` and assignment to a local; `Str.len()` (`__str_len`, Unicode scalar
//! values, as the interpreter counts); every allocation through `__alloc`, which grows the memory (the heap had been a
//! fixed 1 MiB). An inlined body — a generic function's, a lambda's — sees the names of where it was written and no
//! others (WASM-HYGIENE-1: a lambda's captured name had resolved to the generic function's own local).
//!
//! Constructs outside this fragment (`for` and every `List`, embedded Python (`root.python`/`py.*` — head-chef
//! ruling: the DL1201 interpreter fallback, since `py.list` needs guest `List` values), GC types,
//! user enums with non-scalar payloads) are `CompileError::Unsupported` (DL1201) and stay on the
//! interpreter, which remains the reference engine. Secret-handling constructs (`root.secret(...)`,
//! `Secret.expose(...)`) are refused as `CompileError::SecretInGuest` (DL1205, Phase 3n) so secret
//! bytes never enter guest linear memory.

use std::cell::RefCell;
use std::collections::HashMap;

use delulu_syntax::ast::*;

use crate::actors::{actor_table, behavior_export_name, ctor_export_name, ActorTable};
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, DataSection, EntityType, ExportKind, ExportSection, Function, FunctionSection,
    GlobalSection, GlobalType, ImportSection, Instruction, MemArg, MemorySection, MemoryType, Module as WasmModule, TypeSection,
    ValType,
};

#[derive(Clone, Debug)]
pub enum CompileError {
    /// A construct outside the WASM fragment — DL1201, falls back to the interpreter.
    Unsupported(String),
    /// A construct that would place SECRET contents in guest linear memory — DL1205. Refused so the
    /// program runs on the interpreter, where secret bytes never cross into a guest (§4.4, spec §9.8).
    SecretInGuest(String),
}

impl CompileError {
    pub fn message(&self) -> String {
        match self {
            CompileError::Unsupported(what) => format!("WASM codegen does not support {what}"),
            CompileError::SecretInGuest(what) => {
                format!("secret contents cannot enter WASM guest memory ({what}); it runs on the interpreter")
            }
        }
    }

    /// The diagnostic code the CLI reports for this refusal.
    pub fn code(&self) -> &'static str {
        match self {
            CompileError::Unsupported(_) => "DL1201",
            CompileError::SecretInGuest(_) => "DL1205",
        }
    }
}

/// The value types the backend handles. `Str` and `Cap` are both i32 (a memory pointer / a host
/// handle); they are kept distinct so the type checker in codegen refuses nonsense like `str + str`
/// or arithmetic on a capability. `Unit` occupies zero stack slots.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ty {
    I64,     // Int
    I32,     // Bool
    F64,     // Float (Stage 4 phase 4g — used for `foreign "c"` scalar marshalling)
    Str,     // i32 pointer into linear memory
    Cap,     // i32 host handle (Console)
    Clock,   // i32 host handle (Clock, Phase 3k)
    Rand,    // i32 host handle (Rand, Phase 3l)
    FsRead,  // i32 host handle (Cap[FsRead], Phase 3p)
    FsWrite, // i32 host handle (Cap[FsWrite], RW 3.4)
    Http,    // i32 host handle (Cap[Http], RW 3.4)
    Root,    // i32 host handle to the root authority (Phase 3e)
    // Stage 4 phase 4g: `Cap[ForeignLoad]` (i32 host handle) and a bound `foreign` lib handle
    // `Foreign(lib_id)` (i32 host handle; `lib_id` indexes the module's foreign blocks so a method
    // call knows its marshalling signature). `ForeignPtr` is the opaque `void*` handle (i32 index
    // into the host's foreign-pointer table — never a raw address in the guest).
    ForeignLoad,
    Foreign(u32),
    ForeignPtr,
    Unit, // no value
    // Phase 3o: sum types. A variant value is an i32 pointer to `[tag:i32][field…]` in the heap.
    // `Result`/`Option` carry their payload scalars inline; user/prelude enums (Phase 3o Checkpoint 3)
    // are `Enum(id)` where `id` indexes the module's `EnumEnv` (so nested enums work — e.g. an
    // `IoErr` in `Result[Str, IoErr]` is `Scalar::Enum(io_err_id)`).
    Result(Scalar, Scalar), // Ok(T), Err(E)
    Option(Scalar),         // None, Some(T)
    Enum(u32),              // a named sum type (index into EnumEnv)
    // Stage 7 phase 7h: an actor reference (`tag A`) is an i64 host-side slot id; `u32` is the
    // actor's index into the module's `ActorTable` (so a send resolves the behaviour index).
    ActorRef(u32),
    // RW 3.4: a `List[T]` of scalars, an i32 pointer to its header `[len:i32][cap:i32][data:i32]`; `data` holds `cap`
    // 8-byte slots. Lists are shared, as the interpreter's are: `let ys = xs` names the same header, so a `push`
    // through either is seen by both. A list of lists is outside the fragment (a `Scalar` holds no list).
    List(Scalar),
    // RW 3.4: a record of a declared, non-generic `type R { … }`, an i32 pointer to its cell — one 8-byte slot per field,
    // in declaration order (`RecordDesc`). Shared, as the interpreter's `Rc<RefCell<…>>` is: `let q = p` names one cell,
    // so `q.x = 1` is seen through `p`.
    Record(u32),
}

/// The list header's fields (`Ty::List`): its length, its capacity in slots, and a pointer to its slots.
const LIST_LEN: u64 = 0;
const LIST_CAP: u64 = 4;
const LIST_DATA: u64 = 8;
const LIST_HEADER_BYTES: i32 = 12;

/// The payload types a variant field may hold (a `Copy` subset of `Ty`, so `Ty` stays `Copy`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Scalar {
    I64,
    I32,
    F64,
    Str,
    Unit,
    Enum(u32),
    Foreign(u32), // a bound `foreign` lib handle payload — e.g. the `Ok(m)` of `Result[M, ForeignErr]`
    ForeignPtr,
    Record(u32), // a record's cell pointer (RW 3.4)
}

impl Scalar {
    fn to_ty(self) -> Ty {
        match self {
            Scalar::I64 => Ty::I64,
            Scalar::I32 => Ty::I32,
            Scalar::F64 => Ty::F64,
            Scalar::Str => Ty::Str,
            Scalar::Unit => Ty::Unit,
            Scalar::Enum(i) => Ty::Enum(i),
            Scalar::Foreign(i) => Ty::Foreign(i),
            Scalar::ForeignPtr => Ty::ForeignPtr,
            Scalar::Record(i) => Ty::Record(i),
        }
    }
}

/// A payload-capable `Ty` narrows to a `Scalar`; caps and `Result`/`Option` cannot be payloads yet.
fn ty_to_scalar(t: Ty) -> Option<Scalar> {
    match t {
        Ty::I64 => Some(Scalar::I64),
        Ty::I32 => Some(Scalar::I32),
        Ty::F64 => Some(Scalar::F64),
        Ty::Str => Some(Scalar::Str),
        Ty::Unit => Some(Scalar::Unit),
        Ty::Enum(i) => Some(Scalar::Enum(i)),
        Ty::Foreign(i) => Some(Scalar::Foreign(i)),
        Ty::ForeignPtr => Some(Scalar::ForeignPtr),
        Ty::Record(i) => Some(Scalar::Record(i)),
        _ => None,
    }
}

/// Emit the store instruction for a variant field slot of the given scalar payload kind.
fn store_scalar(cx: &mut Cx, s: Scalar) {
    match s {
        Scalar::I64 => cx.emit(Instruction::I64Store(i64_at())),
        Scalar::F64 => cx.emit(Instruction::F64Store(i64_at())),
        Scalar::I32 | Scalar::Str | Scalar::Enum(_) | Scalar::Foreign(_) | Scalar::ForeignPtr | Scalar::Record(_) => {
            cx.emit(Instruction::I32Store(u32_at()))
        }
        Scalar::Unit => {}
    }
}

/// Emit the load instruction for a variant field slot of the given scalar payload kind.
fn load_scalar(cx: &mut Cx, s: Scalar) {
    match s {
        Scalar::I64 => cx.emit(Instruction::I64Load(i64_at())),
        Scalar::F64 => cx.emit(Instruction::F64Load(i64_at())),
        Scalar::I32 | Scalar::Str | Scalar::Enum(_) | Scalar::Foreign(_) | Scalar::ForeignPtr | Scalar::Record(_) => {
            cx.emit(Instruction::I32Load(u32_at()))
        }
        Scalar::Unit => {}
    }
}

/// A named sum type's constructors, in declaration order (the tag is the index).
#[derive(Clone)]
struct EnumDesc {
    ctors: Vec<EnumCtorDesc>,
}

#[derive(Clone)]
struct EnumCtorDesc {
    name: String,
    payloads: Vec<Scalar>,
}

/// One `foreign "c" lib M { … }` block, lowered for codegen (Stage 4 phase 4g): the logical lib
/// name and each declared method's marshalling `Ty`s. Kept parallel to (not shared with) the
/// runtime's `foreign::ForeignSig` — this side drives guest codegen, that side drives the host FFI.
#[derive(Clone)]
struct ForeignLibInfo {
    name: String,
    methods: Vec<ForeignMethodInfo>,
}

#[derive(Clone)]
struct ForeignMethodInfo {
    name: String,
    params: Vec<Ty>,
    ret: Ty,
}

/// The module's named sum types (prelude `IoErr`/`NetErr`/`ForeignErr` + user `enum` declarations),
/// resolved to stable indices so `Ty::Enum(id)`/`Scalar::Enum(id)` can name them, plus the `foreign`
/// lib blocks (`Ty::Foreign(id)` — Stage 4 phase 4g).
struct EnumEnv {
    descs: Vec<EnumDesc>,
    by_name: HashMap<String, u32>,
    foreign_libs: Vec<ForeignLibInfo>,
    foreign_by_name: HashMap<String, u32>,
    /// Actor names → index into the module's `ActorTable` (Stage 7 phase 7h). Lets `wasm_ty`
    /// lower an actor-typed field/param (`from: Ping`) to `Ty::ActorRef(idx)`. Same declaration
    /// order as [`actor_table`], so the indices agree with the host's dispatch table.
    actors_by_name: HashMap<String, u32>,
    /// RW 3.4: the module's non-generic record types, by declaration order, and their names.
    records: Vec<RecordDesc>,
    records_by_name: HashMap<String, u32>,
}

/// A record type the backend lays out: its fields in declaration order, each a slot's type (`field_ty`) — or `None` when
/// a field holds a type the backend does not (the record is then refused where it is built or read, never miscompiled).
struct RecordDesc {
    name: String,
    fields: Option<Vec<(String, Ty)>>,
}

/// The types a record field may hold: a scalar, or an `Option`, `Result` or `List` of scalars — each one value in its
/// 8-byte slot, the three compound ones a pointer to a cell shared as the interpreter shares it (`n.next`, `t.kids`).
fn field_ty(t: Ty) -> Option<Ty> {
    (ty_to_scalar(t).is_some() || matches!(t, Ty::Option(_) | Ty::Result(..) | Ty::List(_))).then_some(t)
}

/// Store the value on the stack into the slot whose address is under it, by the width of `t` (nothing for `Unit`).
fn store_ty(cx: &mut Cx, t: Ty) {
    match wasm_valtype(t) {
        Some(ValType::I64) => cx.emit(Instruction::I64Store(i64_at())),
        Some(ValType::F64) => cx.emit(Instruction::F64Store(i64_at())),
        Some(_) => cx.emit(Instruction::I32Store(u32_at())),
        None => {}
    }
}

/// The type descriptors (`type_desc`) the module's bodies name, laid out after the interned literals as a string is —
/// `[len:u32][bytes]` — from `base` on, each once.
struct Descs {
    base: u32,
    bytes: Vec<u8>,
    at: HashMap<String, u32>,
}

/// The address of `t`'s descriptor in the module's memory, laid out on first use.
fn desc_ptr(cx: &Cx, t: Ty) -> Result<i32, CompileError> {
    let text = type_desc(cx.env, t)?;
    let mut d = cx.descs.borrow_mut();
    if let Some(&at) = d.at.get(&text) {
        return Ok(at as i32);
    }
    let at = d.base + d.bytes.len() as u32;
    let len = u32::try_from(text.len()).map_err(|_| CompileError::Unsupported("a type too large to describe".into()))?;
    d.bytes.extend_from_slice(&len.to_le_bytes());
    d.bytes.extend_from_slice(text.as_bytes());
    d.at.insert(text, at);
    i32::try_from(at).map_err(|_| CompileError::Unsupported("a module whose data passes 2 GiB".into()))
}

/// The descriptor the host rebuilds a value of type `t` by (`reify.rs`): the type, then each record and sum it reaches,
/// each after a `|`. A type is `i` `b` `f` `s` `u` (`Int`, `Bool`, `Float`, `Str`, `Unit`), `o`T, `r`TE, `l`T, `R`n`.` or
/// `E`n`.` (record or sum `n`); a record is `R`n`=`Name`{`field`:`T`,`…`}`, a sum `E`n`={`Ctor`(`T…`)`,`…`}`, its
/// constructors in tag order. A type holding a foreign handle has none: it is opaque, and the checker refuses `str` and
/// `==` of it.
fn type_desc(env: &EnumEnv, t: Ty) -> Result<String, CompileError> {
    fn ty(t: Ty, out: &mut String, todo: &mut Vec<Ty>) -> Result<(), CompileError> {
        match t {
            Ty::I64 => out.push('i'),
            Ty::I32 => out.push('b'),
            Ty::F64 => out.push('f'),
            Ty::Str => out.push('s'),
            Ty::Unit => out.push('u'),
            Ty::Option(s) => {
                out.push('o');
                ty(s.to_ty(), out, todo)?;
            }
            Ty::Result(o, e) => {
                out.push('r');
                ty(o.to_ty(), out, todo)?;
                ty(e.to_ty(), out, todo)?;
            }
            Ty::List(s) => {
                out.push('l');
                ty(s.to_ty(), out, todo)?;
            }
            Ty::Record(n) => {
                out.push_str(&format!("R{n}."));
                todo.push(t);
            }
            Ty::Enum(n) => {
                out.push_str(&format!("E{n}."));
                todo.push(t);
            }
            other => return Err(CompileError::Unsupported(format!("`str` or `==` of a value holding a {other:?}"))),
        }
        Ok(())
    }
    let mut out = String::new();
    let mut todo = Vec::new();
    ty(t, &mut out, &mut todo)?;
    let mut done: Vec<Ty> = Vec::new();
    while let Some(named) = todo.pop() {
        if done.contains(&named) {
            continue;
        }
        done.push(named);
        match named {
            Ty::Record(n) => {
                let name = env.record_name(n);
                out.push_str(&format!("|R{n}={name}{{"));
                for (k, (f, ft)) in env.record_fields(n)?.iter().enumerate() {
                    if k > 0 {
                        out.push(',');
                    }
                    out.push_str(f);
                    out.push(':');
                    ty(*ft, &mut out, &mut todo)?;
                }
                out.push('}');
            }
            Ty::Enum(n) => {
                out.push_str(&format!("|E{n}={{"));
                for (k, c) in env.desc(n).ctors.iter().enumerate() {
                    if k > 0 {
                        out.push(',');
                    }
                    out.push_str(&c.name);
                    out.push('(');
                    for p in &c.payloads {
                        ty(p.to_ty(), &mut out, &mut todo)?;
                    }
                    out.push(')');
                }
                out.push('}');
            }
            _ => {}
        }
    }
    Ok(out)
}

/// Load a value of type `t` from the slot whose address is on the stack (nothing for `Unit`).
fn load_ty(cx: &mut Cx, t: Ty) {
    match wasm_valtype(t) {
        Some(ValType::I64) => cx.emit(Instruction::I64Load(i64_at())),
        Some(ValType::F64) => cx.emit(Instruction::F64Load(i64_at())),
        Some(_) => cx.emit(Instruction::I32Load(u32_at())),
        None => {}
    }
}

impl EnumEnv {
    /// The record id for a name introduced by a non-generic `type R { … }` (RW 3.4).
    fn record_id(&self, name: &str) -> Option<u32> {
        self.records_by_name.get(name).copied()
    }
    /// Record `id`'s name.
    fn record_name(&self, id: u32) -> &str {
        &self.records[id as usize].name
    }
    /// Record `id`'s fields, if the backend can lay them out.
    fn record_fields(&self, id: u32) -> Result<&[(String, Ty)], CompileError> {
        self.records[id as usize]
            .fields
            .as_deref()
            .ok_or_else(|| CompileError::Unsupported("a record with a field of a type outside the WASM fragment".into()))
    }
    fn id(&self, name: &str) -> Option<u32> {
        self.by_name.get(name).copied()
    }
    /// The named sum type a constructor builds, when exactly one declares it — the checker's rule (`variant_ctor`): a
    /// name two sums share (`IoErr`'s and `NetErr`'s `Other`) names neither.
    fn enum_of_ctor(&self, ctor: &str) -> Option<u32> {
        let mut owners = (0..self.descs.len() as u32).filter(|&i| self.desc(i).ctors.iter().any(|c| c.name == ctor));
        let first = owners.next()?;
        owners.next().is_none().then_some(first)
    }
    /// The actor index for a name introduced by an `actor A { … }` declaration (Stage 7 phase 7h).
    fn actor_id(&self, name: &str) -> Option<u32> {
        self.actors_by_name.get(name).copied()
    }
    fn desc(&self, id: u32) -> &EnumDesc {
        &self.descs[id as usize]
    }
    /// The foreign-lib id for a name introduced by a `foreign … lib M` block (Stage 4 phase 4g).
    fn foreign_id(&self, name: &str) -> Option<u32> {
        self.foreign_by_name.get(name).copied()
    }
    fn foreign_method(&self, id: u32, method: &str) -> Option<&ForeignMethodInfo> {
        self.foreign_libs[id as usize].methods.iter().find(|m| m.name == method)
    }
    /// The constructors of a variant type (built-in `Result`/`Option` or a named enum).
    fn ctors_of(&self, ty: Ty) -> Option<Vec<(String, Vec<Scalar>)>> {
        match ty {
            Ty::Result(t, e) => Some(vec![("Ok".into(), vec![t]), ("Err".into(), vec![e])]),
            Ty::Option(t) => Some(vec![("None".into(), vec![]), ("Some".into(), vec![t])]),
            Ty::Enum(i) => Some(self.desc(i).ctors.iter().map(|c| (c.name.clone(), c.payloads.clone())).collect()),
            _ => None,
        }
    }
}

/// Build the enum environment: the fixed prelude sum types plus every monomorphic `enum` declared in
/// the module. Two passes so ctor payloads can reference other (and their own) enum names.
fn build_enum_env(module: &Module) -> EnumEnv {
    let mut by_name: HashMap<String, u32> = HashMap::new();
    // Names first (prelude, then user), so payload resolution below can see every enum. `ForeignErr`
    // is a Stage-4 prelude sum (spec §8) so `Result[M, ForeignErr]` binds/matches on the WASM engine.
    let prelude: [&str; 3] = ["IoErr", "NetErr", "ForeignErr"];
    for name in prelude {
        let id = by_name.len() as u32;
        by_name.insert(name.to_string(), id);
    }
    for it in &module.items {
        if let Item::Type(td) = it {
            if td.generics.is_empty() && matches!(td.kind, TypeDeclKind::Sum(_)) && !by_name.contains_key(&td.name.name) {
                let id = by_name.len() as u32;
                by_name.insert(td.name.name.clone(), id);
            }
        }
    }

    // Foreign-lib blocks introduce nominal opaque handle types (`Ty::Foreign(id)`). Register their
    // names first so a foreign method's own signature (only ever Int/Float/Bool/Str/Unit/ForeignPtr
    // per the T-ForeignSig fence) never needs them, but a `Result[M, ForeignErr]` annotation resolves.
    let mut foreign_by_name: HashMap<String, u32> = HashMap::new();
    for it in &module.items {
        if let Item::Foreign(fd) = it {
            if !foreign_by_name.contains_key(&fd.name.name) {
                let id = foreign_by_name.len() as u32;
                foreign_by_name.insert(fd.name.name.clone(), id);
            }
        }
    }

    // Actor names → declaration-order index (Stage 7 phase 7h), matching `actor_table`.
    let mut actors_by_name: HashMap<String, u32> = HashMap::new();
    for it in &module.items {
        if let Item::Actor(a) = it {
            if !actors_by_name.contains_key(&a.name.name) {
                let id = actors_by_name.len() as u32;
                actors_by_name.insert(a.name.name.clone(), id);
            }
        }
    }

    // Record names → declaration-order index (RW 3.4); their fields are resolved once every name is known.
    let mut records_by_name: HashMap<String, u32> = HashMap::new();
    for it in &module.items {
        if let Item::Type(td) = it {
            if td.generics.is_empty()
                && matches!(td.kind, TypeDeclKind::Record(_))
                && !records_by_name.contains_key(&td.name.name)
            {
                let id = records_by_name.len() as u32;
                records_by_name.insert(td.name.name.clone(), id);
            }
        }
    }

    let mut env = EnumEnv {
        descs: vec![EnumDesc { ctors: Vec::new() }; by_name.len()],
        by_name,
        foreign_libs: Vec::new(),
        foreign_by_name,
        actors_by_name,
        records: Vec::new(),
        records_by_name,
    };
    // Prelude descriptors.
    let str_payload = vec![Scalar::Str];
    set_ctors(&mut env, "IoErr", &[("NotFound", vec![]), ("Denied", vec![]), ("Other", str_payload.clone())]);
    set_ctors(&mut env, "NetErr", &[("Refused", vec![]), ("Timeout", vec![]), ("Other", str_payload.clone())]);
    // `ForeignErr = NotGranted | SymbolMissing(Str) | BadReturn(Str) | Unavailable(Str)` (spec §8) —
    // the tag order MUST match the host's `foreign_bind` cell construction and the interpreter's sum.
    set_ctors(
        &mut env,
        "ForeignErr",
        &[
            ("NotGranted", vec![]),
            ("SymbolMissing", str_payload.clone()),
            ("BadReturn", str_payload.clone()),
            ("Unavailable", str_payload),
        ],
    );
    // Foreign lib method signatures (params/return lowered to `Ty`). Ordered to match `foreign_by_name`.
    let mut foreign_libs: Vec<ForeignLibInfo> =
        vec![ForeignLibInfo { name: String::new(), methods: Vec::new() }; env.foreign_by_name.len()];
    for it in &module.items {
        if let Item::Foreign(fd) = it {
            if let Some(&id) = env.foreign_by_name.get(&fd.name.name) {
                let methods = fd
                    .fns
                    .iter()
                    .map(|f| ForeignMethodInfo {
                        name: f.name.name.clone(),
                        params: f.params.iter().map(|p| wasm_ty(&env, &p.ty).unwrap_or(Ty::Unit)).collect(),
                        ret: f.ret.as_ref().and_then(|t| wasm_ty(&env, t)).unwrap_or(Ty::Unit),
                    })
                    .collect();
                foreign_libs[id as usize] = ForeignLibInfo { name: fd.name.name.clone(), methods };
            }
        }
    }
    env.foreign_libs = foreign_libs;
    // User enum descriptors (payloads resolved against the now-populated name table).
    for it in &module.items {
        if let Item::Type(td) = it {
            if let TypeDeclKind::Sum(variants) = &td.kind {
                if td.generics.is_empty() {
                    if let Some(&id) = env.by_name.get(&td.name.name) {
                        let mut ctors = Vec::new();
                        for v in variants {
                            let mut payloads = Vec::new();
                            let mut ok = true;
                            for ft in &v.fields {
                                match wasm_ty(&env, ft).and_then(ty_to_scalar) {
                                    Some(s) => payloads.push(s),
                                    None => {
                                        ok = false;
                                        break;
                                    }
                                }
                            }
                            // A ctor with a non-scalar payload makes the enum non-compilable; leave it
                            // with no ctors so `ctors_of` yields a shape `match` will reject (DL1201).
                            if !ok {
                                ctors.clear();
                                break;
                            }
                            ctors.push(EnumCtorDesc { name: v.name.name.clone(), payloads });
                        }
                        env.descs[id as usize].ctors = ctors;
                    }
                }
            }
        }
    }
    // Record layouts (RW 3.4), resolved against the populated names: a field the backend cannot hold leaves the record
    // without a layout, refused where it is built or read.
    let mut records: Vec<RecordDesc> =
        (0..env.records_by_name.len()).map(|_| RecordDesc { name: String::new(), fields: None }).collect();
    for it in &module.items {
        if let Item::Type(td) = it {
            if let (TypeDeclKind::Record(fields), true) = (&td.kind, td.generics.is_empty()) {
                if let Some(&id) = env.records_by_name.get(&td.name.name) {
                    records[id as usize].name = td.name.name.clone();
                    records[id as usize].fields = fields
                        .iter()
                        .map(|f| wasm_ty(&env, &f.ty).and_then(field_ty).map(|t| (f.name.name.clone(), t)))
                        .collect();
                }
            }
        }
    }
    env.records = records;
    env
}

fn set_ctors(env: &mut EnumEnv, name: &str, ctors: &[(&str, Vec<Scalar>)]) {
    if let Some(&id) = env.by_name.get(name) {
        env.descs[id as usize].ctors =
            ctors.iter().map(|(n, p)| EnumCtorDesc { name: (*n).to_string(), payloads: p.clone() }).collect();
    }
}

/// Host import function indices for the module (only the ones the module actually imports are
/// meaningful; codegen only reads an index after detecting the corresponding construct).
#[derive(Clone, Copy)]
struct Imports {
    root_console: u32,
    console_println: u32,
    root_clock: u32,
    clock_now_ms: u32,
    root_rand: u32,
    rand_int: u32,
    root_fs_read: u32,
    fs_read_text: u32,
    /// `delulu:cap.fs_narrow` — `Cap[FsRead].narrow(sub)`, a narrower handle minted host-side by the interpreter's own
    /// `resolve_in_scope`.
    fs_narrow: u32,
    /// `delulu:cap.root_fs_write` and `fs_write_text` — `root.fs_write(p)` and `write_text`/`append_text`, performed
    /// host-side in the interpreter's order (RW 3.4).
    root_fs_write: u32,
    fs_write_text: u32,
    /// `delulu:cap.root_http` and `http_get` — `root.http(hosts)`, minted host-side by the interpreter's own rule, and
    /// `Cap[Http].get(url)`, performed host-side in the interpreter's order by its own egress client (RW 3.4).
    root_http: u32,
    http_get: u32,
    // Stage 4 phase 4g — the `delulu:foreign` host interface.
    foreign_load: u32,
    foreign_bind: u32,
    foreign_call: u32,
    float_to_str: u32,
    // RW 3.4, the WASM `List`: the `delulu:list` imports — `Str.split`, computed by the interpreter's own rule host-side,
    // and the out-of-range index's fault, in the interpreter's words — and the two list helpers, which follow the user
    // functions and the actor members. `u32::MAX` when the module uses no list.
    str_split: u32,
    index_fault: u32,
    list_new: u32,
    list_slot: u32,
    /// `__str_eq` (`Str == Str`), after the list helpers; `u32::MAX` when the module compares nothing.
    str_eq: u32,
    /// `delulu:str.str_method` — a `Str` method run host-side by the interpreter's own `call_str_method`.
    str_method: u32,
    /// `delulu:str.parse_int` — the builtin `parse_int`, run host-side by the interpreter's own `call_builtin`.
    parse_int: u32,
    /// `delulu:str.parse_float` — the builtin `parse_float`, the same way.
    parse_float: u32,
    /// `delulu:float.rem` — `Float % Float`, which WebAssembly has no instruction for, computed by the host as the
    /// interpreter computes it.
    float_rem: u32,
    /// `delulu:value.display` and `delulu:value.eq` — `str` and `==` of a list, an option, a result, a sum or a record,
    /// answered by the interpreter's own `Value::display` and `Value::eq` over the value rebuilt host-side by its type's
    /// descriptor (`type_desc`, `reify.rs`).
    value_display: u32,
    value_eq: u32,
}

/// Host import indices for the `delulu:actors` interface (Stage 7 phase 7h). `u32::MAX` sentinels
/// mean the module declared no actors and never reads them.
#[derive(Clone, Copy)]
struct ActorImports {
    spawn: u32,
    send: u32,
    field_get: u32,
    field_set: u32,
}

impl ActorImports {
    fn none() -> ActorImports {
        ActorImports { spawn: u32::MAX, send: u32::MAX, field_get: u32::MAX, field_set: u32::MAX }
    }
}

/// The per-body actor context (Stage 7 phase 7h): present only while compiling an actor member
/// (ctor or behaviour). `self_slot` is the local holding the actor's slot id (the implicit first
/// param); `fields` maps a field name to its slot index + lowered type; `actor_idx` is which
/// actor `self` is (so `self.beh(…)` resolves the behaviour index).
struct ActorCtx {
    self_slot: u32,
    actor_idx: u32,
    fields: HashMap<String, (u32, Ty)>,
}

fn wasm_valtype(t: Ty) -> Option<ValType> {
    match t {
        Ty::I64 => Some(ValType::I64),
        Ty::F64 => Some(ValType::F64),
        // An actor reference is an i64 host-side slot id (Stage 7 phase 7h).
        Ty::ActorRef(_) => Some(ValType::I64),
        // Str/Cap/variant handles are all i32 (a pointer or a host handle).
        Ty::I32
        | Ty::Str
        | Ty::Cap
        | Ty::Clock
        | Ty::Rand
        | Ty::FsRead
        | Ty::FsWrite
        | Ty::Http
        | Ty::Root
        | Ty::ForeignLoad
        | Ty::Foreign(_)
        | Ty::ForeignPtr
        | Ty::Result(..)
        | Ty::Option(..)
        | Ty::Enum(_)
        | Ty::List(_)
        | Ty::Record(_) => Some(ValType::I32),
        Ty::Unit => None,
    }
}

/// The block type of an `if` or `block` whose value has type `t` (none for `Unit`).
fn block_type(t: Ty) -> BlockType {
    match wasm_valtype(t) {
        Some(v) => BlockType::Result(v),
        None => BlockType::Empty,
    }
}

/// The element types a `List` may hold on the WASM engine: the scalars a slot can carry and a `match` can bind.
fn list_element(t: Ty) -> Option<Scalar> {
    match ty_to_scalar(t)? {
        s @ (Scalar::I64 | Scalar::I32 | Scalar::F64 | Scalar::Str | Scalar::Enum(_) | Scalar::Record(_)) => Some(s),
        _ => None,
    }
}

fn wasm_ty(env: &EnumEnv, t: &TypeExpr) -> Option<Ty> {
    // A reference capability (`ref`, `val`, `iso`, …) is the checker's aliasing discipline; the value it qualifies is
    // represented the same either way — on the interpreter as here (RW 3.4: `let xs: ref List[Int] = []`).
    if let TypeExpr::Rcap { inner, .. } = t {
        return wasm_ty(env, inner);
    }
    if let TypeExpr::Named { path, args, .. } = t {
        if path.segs.len() == 1 {
            let name = path.segs[0].name.as_str();
            if name == "Cap" {
                if let [TypeExpr::Named { path: rp, args: ra, .. }] = &args[..] {
                    if ra.is_empty() && rp.segs.len() == 1 {
                        return match rp.segs[0].name.as_str() {
                            "Console" => Some(Ty::Cap),
                            "Clock" => Some(Ty::Clock),
                            "Rand" => Some(Ty::Rand),
                            "FsRead" => Some(Ty::FsRead),
                            "FsWrite" => Some(Ty::FsWrite),
                            "Http" => Some(Ty::Http),
                            "ForeignLoad" => Some(Ty::ForeignLoad),
                            _ => None,
                        };
                    }
                }
                return None; // other capability kinds (e.g. Python) fall back to the interpreter
            }
            // Phase 3o: Result[T, E] and Option[T] (payloads may be scalars or named enums).
            if name == "Result" {
                if let [t, e] = &args[..] {
                    let ts = ty_to_scalar(wasm_ty(env, t)?)?;
                    let es = ty_to_scalar(wasm_ty(env, e)?)?;
                    return Some(Ty::Result(ts, es));
                }
                return None;
            }
            if name == "Option" {
                if let [t] = &args[..] {
                    return Some(Ty::Option(ty_to_scalar(wasm_ty(env, t)?)?));
                }
                return None;
            }
            if name == "List" {
                if let [t] = &args[..] {
                    return Some(Ty::List(list_element(wasm_ty(env, t)?)?));
                }
                return None;
            }
            if args.is_empty() {
                return match name {
                    "Int" => Some(Ty::I64),
                    "Bool" => Some(Ty::I32),
                    "Float" => Some(Ty::F64),
                    "Str" => Some(Ty::Str),
                    "Root" => Some(Ty::Root),
                    "Unit" => Some(Ty::Unit),
                    "ForeignPtr" => Some(Ty::ForeignPtr),
                    // A named sum type (prelude or user `enum`), else a `foreign … lib M` handle
                    // type, else an actor type used as a reference (`tag A` — Stage 7 phase 7h), else a record (RW 3.4).
                    other => env
                        .id(other)
                        .map(Ty::Enum)
                        .or_else(|| env.foreign_id(other).map(Ty::Foreign))
                        .or_else(|| env.actor_id(other).map(Ty::ActorRef))
                        .or_else(|| env.record_id(other).map(Ty::Record)),
                };
            }
        }
    }
    None
}

fn ret_ty(env: &EnumEnv, f: &FnDecl) -> Option<Ty> {
    match &f.ret {
        None => Some(Ty::Unit),
        Some(t) => wasm_ty(env, t),
    }
}

fn is_compilable(env: &EnumEnv, f: &FnDecl) -> bool {
    f.generics.is_empty()
        && f.params.iter().all(|p| matches!(wasm_ty(env, &p.ty), Some(t) if t != Ty::Unit))
        && ret_ty(env, f).is_some()
}

/// Is `f`'s body compiled into the module? A compilable function's is, and so is a generic one's, inlined into each
/// function that calls it (Phase 3r) — so what a generic body needs, an import, a helper or a string literal, the module
/// needs too (RW 3.4: a generic body's `.trim()`, `==` on `Str`s or list was refused for want of what it needed).
fn body_is_compiled(env: &EnumEnv, f: &FnDecl) -> bool {
    !f.generics.is_empty() || is_compilable(env, f)
}

/// Does any compilable function call a method whose name is in `names`? Used to decide which host
/// imports the module needs (`console`/`println` → the console imports; `clock`/`now_ms` → clock).
/// The walk is `module_any`'s, which names every expression form: this one's own walk passed over a record literal's
/// fields, so `Stamp { at: root.clock().now_ms() }` compiled a call to an import the module did not have (WASM-SCAN-1).
fn module_calls_method(env: &EnumEnv, module: &Module, names: &[&str]) -> bool {
    module_any(env, module, &|e| matches!(e, Expr::Method { name, .. } if names.contains(&name.name.as_str())), false)
}

/// Does any compiled body take `str` of, or compare with `==`/`!=`, a value the checker typed as a list, an option, a
/// result, a sum or a record — or as a type variable, which an inlined generic body may make one? Then the module carries
/// `delulu:value`, and every other module is unchanged. Without the checker's types (`compile_module`) none is known, and
/// such a `str` or `==` is refused where it is compiled.
fn module_shows_values(env: &EnumEnv, module: &Module, node_types: &HashMap<NodeId, delulu_check::Type>) -> bool {
    use delulu_check::Type;
    let compound = |e: &Expr| {
        matches!(
            node_types.get(&e.id()),
            Some(Type::List(_) | Type::Option(_) | Type::Result(..) | Type::Record(..) | Type::Sum(..) | Type::Var(_))
        )
    };
    let hit = |e: &Expr| match e {
        Expr::Call { callee, args, .. } => matches!((&**callee, &args[..]),
            (Expr::Var { path, .. }, [a]) if path.segs.len() == 1 && path.segs[0].name == "str" && compound(a)),
        Expr::Binary { op: BinOp::Eq | BinOp::Ne, lhs, .. } => compound(lhs),
        _ => false,
    };
    module_any(env, module, &hit, false)
}

/// Does any compilable function make or index a `List`, walk one with `for`, or call `split`, `push` or `map`? Then the
/// module carries the list helpers (`__list_new`, `__list_slot`, after the user functions) and the `delulu:list` imports,
/// and every other module is byte-identical to before (the WASM `List`, RW 3.4).
fn module_uses_lists(env: &EnumEnv, module: &Module) -> bool {
    let hit = |e: &Expr| match e {
        Expr::List { .. } | Expr::Index { .. } => true,
        Expr::Method { name, .. } => matches!(name.name.as_str(), "split" | "push" | "map"),
        _ => false,
    };
    module_any(env, module, &hit, true)
}

/// Does any compilable function call one of the `Str` methods the host runs (`STR_METHODS`)?
fn module_calls_str_methods(env: &EnumEnv, module: &Module) -> bool {
    module_any(env, module, &|e| matches!(e, Expr::Method { name, .. } if str_method_sig(&name.name).is_some()), false)
}

/// Does any function whose body is compiled call the builtin `name` (`parse_int`)? A builtin's name cannot be redefined
/// (DL0302), so a call by that name is the builtin's.
fn module_calls_builtin(env: &EnumEnv, module: &Module, name: &str) -> bool {
    let hit = |e: &Expr| matches!(e, Expr::Call { callee, .. } if matches!(&**callee, Expr::Var { path, .. } if path.segs.len() == 1 && path.segs[0].name == name));
    module_any(env, module, &hit, false)
}

/// Does any function whose body is compiled make or take a `Float` — a literal, `float(…)`, `parse_float(…)`, or a
/// `Float` written in its signature? Only these bring a `Float` into a compiled body, so only such a module carries the
/// `delulu:float` interface, and every other module is unchanged.
fn module_uses_floats(env: &EnumEnv, module: &Module) -> bool {
    let hit = |e: &Expr| match e {
        Expr::Lit { kind: LitKind::Float(_), .. } => true,
        Expr::Call { callee, .. } => matches!(&**callee, Expr::Var { path, .. }
            if path.segs.len() == 1 && matches!(path.segs[0].name.as_str(), "float" | "parse_float")),
        _ => false,
    };
    let signed = |f: &FnDecl| f.params.iter().any(|p| mentions_float(&p.ty)) || f.ret.as_ref().is_some_and(mentions_float);
    module.items.iter().any(|it| matches!(it, Item::Fn(f) if body_is_compiled(env, f) && signed(f)))
        || module_any(env, module, &hit, false)
}

/// Does a written type name `Float` anywhere in it — `Float`, `Option[Float]`, `List[Float]`, a function type over one?
fn mentions_float(t: &TypeExpr) -> bool {
    match t {
        TypeExpr::Named { path, args, .. } => {
            path.segs.last().is_some_and(|s| s.name == "Float") || args.iter().any(mentions_float)
        }
        TypeExpr::Fn { params, ret, .. } => params.iter().any(mentions_float) || ret.as_deref().is_some_and(mentions_float),
        TypeExpr::Rcap { inner, .. } => mentions_float(inner),
    }
}

/// The `Str` methods the WASM engine runs host-side, through the interpreter's own `call_str_method` — so each answers
/// exactly what the interpreter answers, Unicode and all (RW 3.4). A method's id is its index here; `host.rs` reads it
/// back. `len` and `split` have their own paths (`__str_len`, `delulu:list.str_split`).
pub(crate) const STR_METHODS: [&str; 8] =
    ["trim", "to_upper", "to_lower", "chars", "contains", "starts_with", "replace", "slice"];

/// A `Str` method's id, its arguments' types and its result's type.
fn str_method_sig(name: &str) -> Option<(u32, &'static [Ty], Ty)> {
    let id = STR_METHODS.iter().position(|m| *m == name)? as u32;
    let (args, ret): (&'static [Ty], Ty) = match name {
        "trim" | "to_upper" | "to_lower" => (&[], Ty::Str),
        "chars" => (&[], Ty::List(Scalar::Str)),
        "contains" | "starts_with" => (&[Ty::Str], Ty::I32),
        "replace" => (&[Ty::Str, Ty::Str], Ty::Str),
        "slice" => (&[Ty::I64, Ty::I64], Ty::Str),
        _ => return None,
    };
    Some((id, args, ret))
}

/// What `host.rs` hands `call_str_method` for the method numbered `id`: its name, and how many `Str` arguments (the
/// import's `a` and `b`) and `Int` arguments (`lo` and `hi`) it takes — read from `str_method_sig`, so the two sides
/// cannot disagree on a slot.
pub(crate) fn str_method_arity(id: i32) -> Option<(&'static str, usize, usize)> {
    let name = *STR_METHODS.get(usize::try_from(id).ok()?)?;
    let (_, params, _) = str_method_sig(name)?;
    let strs = params.iter().filter(|t| **t == Ty::Str).count();
    Some((name, strs, params.len() - strs))
}

/// Does any compilable function compare with `==` or `!=`? Then the module carries `__str_eq` (after the list helpers),
/// which a comparison of two `Str`s calls.
fn module_compares(env: &EnumEnv, module: &Module) -> bool {
    module_any(env, module, &|e| matches!(e, Expr::Binary { op: BinOp::Eq | BinOp::Ne, .. }), false)
}

/// Does any compilable function hold an expression for which `hit` holds — or, with `fors`, a `for`? Every expression
/// form is named: a walk that passes over one misses what it holds (WASM-INTERN-1).
fn module_any(env: &EnumEnv, module: &Module, hit: &dyn Fn(&Expr) -> bool, fors: bool) -> bool {
    fn block(b: &Block, hit: &dyn Fn(&Expr) -> bool, fors: bool) -> bool {
        b.stmts.iter().any(|s| match s {
            Stmt::Let { value, .. } | Stmt::Assign { value, .. } => expr(value, hit, fors),
            Stmt::While { cond, body, .. } => expr(cond, hit, fors) || block(body, hit, fors),
            Stmt::For { iter, body, .. } => fors || expr(iter, hit, fors) || block(body, hit, fors),
            Stmt::Return { value: Some(e), .. } | Stmt::Expr(e) => expr(e, hit, fors),
            Stmt::Break { .. } | Stmt::Continue { .. } | Stmt::Return { value: None, .. } => false,
        })
    }
    fn expr(e: &Expr, hit: &dyn Fn(&Expr) -> bool, fors: bool) -> bool {
        if hit(e) {
            return true;
        }
        let ex = |x: &Expr| expr(x, hit, fors);
        match e {
            Expr::Lit { .. } | Expr::Var { .. } | Expr::Consume { .. } => false,
            Expr::List { items, .. } => items.iter().any(ex),
            Expr::Record { fields, .. } => fields.iter().any(|(_, x)| ex(x)),
            Expr::Call { callee, args, .. } => ex(callee) || args.iter().any(ex),
            Expr::Method { recv, args, .. } => ex(recv) || args.iter().any(ex),
            Expr::Field { recv, .. } => ex(recv),
            Expr::Index { recv, index, .. } => ex(recv) || ex(index),
            Expr::Unary { operand, .. } => ex(operand),
            Expr::Binary { lhs, rhs, .. } => ex(lhs) || ex(rhs),
            Expr::If { cond, then_, else_, .. } => ex(cond) || block(then_, hit, fors) || else_.as_deref().is_some_and(ex),
            Expr::Match { scrutinee, arms, .. } => ex(scrutinee) || arms.iter().any(|a| ex(&a.body)),
            Expr::Try { inner, .. } => ex(inner),
            Expr::Block(b) | Expr::Lambda { body: b, .. } | Expr::Recover { body: b, .. } => block(b, hit, fors),
            Expr::Spawn { args, .. } => args.iter().any(ex),
        }
    }
    module.items.iter().any(|it| matches!(it, Item::Fn(f) if body_is_compiled(env, f) && block(&f.body, hit, fors)))
}

/// Does the module perform console output (so it needs the console host imports)? Public helper;
/// builds its own enum environment for callers that don't have one.
pub fn uses_console(module: &Module) -> bool {
    module_calls_method(&build_enum_env(module), module, &["console", "println"])
}

/// Compile a checked module's compilable functions to a WASM module exporting each by name. A
/// program with no `foreign` blocks needs no bind map — this is the Stage-3 entry point unchanged.
pub fn compile_module(module: &Module) -> Result<Vec<u8>, CompileError> {
    compile_module_with(module, &HashMap::new())
}

/// Compile with the checker's `root.foreign(load)` bind-site → lib-name map (Stage 4 phase 4g). The
/// grammar has no method type-argument syntax, so — exactly as the interpreter does — codegen recovers
/// which lib each bind site resolves to from `foreign_binds` (keyed by the call's `NodeId`).
pub fn compile_module_with(module: &Module, foreign_binds: &HashMap<NodeId, String>) -> Result<Vec<u8>, CompileError> {
    compile_module_typed(module, foreign_binds, &HashMap::new())
}

/// Compile with everything the checker knows that the syntax does not say: the foreign bind sites, and the type it
/// inferred for each node — so an empty list whose element type no `let` writes (`let out = []`, filled by
/// `out.push(i)` later) has the type the checker gave it (RW 3.4). What the command line compiles with.
pub fn compile_checked(module: &Module, checked: &delulu_check::CheckResult) -> Result<Vec<u8>, CompileError> {
    compile_module_typed(module, &checked.foreign_binds, &checked.node_types)
}

fn compile_module_typed(
    module: &Module,
    foreign_binds: &HashMap<NodeId, String>,
    node_types: &HashMap<NodeId, delulu_check::Type>,
) -> Result<Vec<u8>, CompileError> {
    // Resolve the module's named sum types (prelude + user `enum`s) once, up front (Phase 3o).
    let env = build_enum_env(module);

    // Generic user functions are not compiled as top-level exports; they are inlined (monomorphised)
    // at each call site (Phase 3r). Collect them by name for the inliner.
    let generics: HashMap<String, FnDecl> = module
        .items
        .iter()
        .filter_map(|it| match it {
            Item::Fn(f) if !f.generics.is_empty() => Some((f.name.name.clone(), f.clone())),
            _ => None,
        })
        .collect();

    let fns: Vec<&FnDecl> = module
        .items
        .iter()
        .filter_map(|it| if let Item::Fn(f) = it { Some(f) } else { None })
        .filter(|f| is_compilable(&env, f))
        .collect();
    // RW 3.4: each function a value may name — a compilable one, or a generic one.
    let named: HashMap<String, Callable> =
        fns.iter().copied().chain(generics.values()).map(|f| (f.name.name.clone(), named_value(f))).collect();

    // Stage 7 phase 7h: the module's actor table (declaration order, shared with the host) and the
    // ordered plan of members to compile as exports (ctor then behaviours, per actor).
    let table = actor_table(module);
    let needs_actors = !table.is_empty();
    let mut member_plans: Vec<(u32, &ActorDecl, &[Param], &Block, String)> = Vec::new();
    for it in &module.items {
        if let Item::Actor(a) = it {
            let aidx = *table.by_name.get(&a.name.name).expect("actor is in the table");
            member_plans.push((aidx, a, a.ctor.params.as_slice(), &a.ctor.body, ctor_export_name(&a.name.name)));
            for b in &a.behaviors {
                member_plans.push((aidx, a, b.params.as_slice(), &b.body, behavior_export_name(&a.name.name, &b.name.name)));
            }
        }
    }

    let needs_console = module_calls_method(&env, module, &["console", "println"]);
    let needs_clock = module_calls_method(&env, module, &["clock", "now_ms"]);
    let needs_rand = module_calls_method(&env, module, &["rand", "int"]);
    let needs_fs = module_calls_method(&env, module, &["fs_read", "read_text", "narrow"]);
    // Stage 4 phase 4g: a program needs the `delulu:foreign` imports when it binds/loads a foreign lib.
    let needs_foreign = module_calls_method(&env, module, &["foreign", "foreign_load"]);

    // Assign imported-function indices in a fixed order (console, clock, rand, fs pairs, then the
    // four foreign fns). Only the present ones consume indices; the rest are left as sentinels
    // codegen never reads.
    let mut n_imports = 0u32;
    let mut imp = Imports {
        root_console: u32::MAX,
        console_println: u32::MAX,
        root_clock: u32::MAX,
        clock_now_ms: u32::MAX,
        root_rand: u32::MAX,
        rand_int: u32::MAX,
        root_fs_read: u32::MAX,
        fs_read_text: u32::MAX,
        fs_narrow: u32::MAX,
        root_fs_write: u32::MAX,
        fs_write_text: u32::MAX,
        root_http: u32::MAX,
        http_get: u32::MAX,
        foreign_load: u32::MAX,
        foreign_bind: u32::MAX,
        foreign_call: u32::MAX,
        float_to_str: u32::MAX,
        str_split: u32::MAX,
        index_fault: u32::MAX,
        list_new: u32::MAX,
        list_slot: u32::MAX,
        str_eq: u32::MAX,
        str_method: u32::MAX,
        parse_int: u32::MAX,
        parse_float: u32::MAX,
        float_rem: u32::MAX,
        value_display: u32::MAX,
        value_eq: u32::MAX,
    };
    if needs_console {
        imp.root_console = n_imports;
        imp.console_println = n_imports + 1;
        n_imports += 2;
    }
    if needs_clock {
        imp.root_clock = n_imports;
        imp.clock_now_ms = n_imports + 1;
        n_imports += 2;
    }
    if needs_rand {
        imp.root_rand = n_imports;
        imp.rand_int = n_imports + 1;
        n_imports += 2;
    }
    if needs_fs {
        imp.root_fs_read = n_imports;
        imp.fs_read_text = n_imports + 1;
        imp.fs_narrow = n_imports + 2;
        n_imports += 3;
    }
    let needs_fs_write = module_calls_method(&env, module, &["fs_write", "write_text", "append_text"]);
    if needs_fs_write {
        imp.root_fs_write = n_imports;
        imp.fs_write_text = n_imports + 1;
        n_imports += 2;
    }
    // By the mint alone: a `Cap[Http]` comes from nowhere else, and `get` is also a list's method, so a module whose
    // only `get` is on a list imports no network interface.
    let needs_http = module_calls_method(&env, module, &["http"]);
    if needs_http {
        imp.root_http = n_imports;
        imp.http_get = n_imports + 1;
        n_imports += 2;
    }
    if needs_foreign {
        imp.foreign_load = n_imports;
        imp.foreign_bind = n_imports + 1;
        imp.foreign_call = n_imports + 2;
        imp.float_to_str = n_imports + 3;
        n_imports += 4;
    }
    // Stage 7 phase 7h: the four `delulu:actors` imports follow the foreign ones (so a non-actor
    // program's import indices are untouched — byte-identical to v0.6).
    let mut actor_imp = ActorImports::none();
    if needs_actors {
        actor_imp.spawn = n_imports;
        actor_imp.send = n_imports + 1;
        actor_imp.field_get = n_imports + 2;
        actor_imp.field_set = n_imports + 3;
        n_imports += 4;
    }
    // RW 3.4: the `delulu:list` imports follow the actors' (so a program without a list is unchanged).
    let needs_lists = module_uses_lists(&env, module);
    let needs_str_eq = module_compares(&env, module);
    if needs_lists {
        imp.str_split = n_imports;
        imp.index_fault = n_imports + 1;
        n_imports += 2;
    }
    let needs_str_methods = module_calls_str_methods(&env, module);
    if needs_str_methods {
        imp.str_method = n_imports;
        n_imports += 1;
    }
    let needs_parse_int = module_calls_builtin(&env, module, "parse_int");
    if needs_parse_int {
        imp.parse_int = n_imports;
        n_imports += 1;
    }
    let needs_parse_float = module_calls_builtin(&env, module, "parse_float");
    if needs_parse_float {
        imp.parse_float = n_imports;
        n_imports += 1;
    }
    // A module that makes or takes a `Float` carries `delulu:float`: `str` of one (unless the foreign interface already
    // brought `float_to_str`), and `%`.
    let needs_floats = module_uses_floats(&env, module);
    let needs_float_str = needs_floats && !needs_foreign;
    if needs_float_str {
        imp.float_to_str = n_imports;
        n_imports += 1;
    }
    if needs_floats {
        imp.float_rem = n_imports;
        n_imports += 1;
    }
    let needs_values = module_shows_values(&env, module, node_types);
    if needs_values {
        imp.value_display = n_imports;
        imp.value_eq = n_imports + 1;
        n_imports += 2;
    }
    let arith_base = n_imports; // the 4 checked-arithmetic helpers occupy [n_imports, n_imports+4)
                                // `__concat`, `__int_to_str`, `__alloc` and `__str_len` follow; user functions after them.
    let user_base = n_imports + N_ARITH_HELPERS + N_STR_HELPERS;

    // Collect string literals into a length-prefixed linear-memory image.
    let mut str_off: HashMap<String, u32> = HashMap::new();
    let mut data: Vec<u8> = Vec::new();
    for f in &fns {
        collect_strings_block(&f.body, &mut str_off, &mut data);
    }
    // A generic function's literals too, after the others' (so a module without one is unchanged): its body is inlined.
    for it in &module.items {
        if let Item::Fn(f) = it {
            if !f.generics.is_empty() {
                collect_strings_block(&f.body, &mut str_off, &mut data);
            }
        }
    }
    // Stage 7 phase 7h: actor member bodies may hold string literals too (Int-only witnesses do
    // not, but intern any so the fragment composes if a future body prints).
    for (_, _, _, body, _) in &member_plans {
        collect_strings_block(body, &mut str_off, &mut data);
    }
    // Stage 4 phase 4g: the foreign lib names (for `foreign_bind`) and method names (for
    // `foreign_call`) cross to the host as interned guest strings — intern every one up front.
    if needs_foreign {
        for lib in &env.foreign_libs {
            intern_string(&lib.name, &mut str_off, &mut data);
            for m in &lib.methods {
                intern_string(&m.name, &mut str_off, &mut data);
            }
        }
    }
    // `str(Bool)` selects one of these — any `Bool`, a foreign return's included (spec §4.2 marshalling matrix). Interned
    // for every module since RW 3.4: a loop's flag is a `Bool` too, and `str` of it was DL1201 without the foreign interface.
    intern_string("true", &mut str_off, &mut data);
    intern_string("false", &mut str_off, &mut data);
    let descs = RefCell::new(Descs { base: data.len() as u32, bytes: Vec::new(), at: HashMap::new() });

    // Function index map: name -> (absolute wasm function index, return type).
    let mut index: HashMap<String, (u32, Vec<Ty>, Ty)> = HashMap::new();
    for (i, f) in fns.iter().enumerate() {
        let ptys: Vec<Ty> = f.params.iter().map(|p| wasm_ty(&env, &p.ty).unwrap()).collect();
        index.insert(f.name.name.clone(), (user_base + i as u32, ptys, ret_ty(&env, f).unwrap()));
    }

    // Types: [import types...], the shared helper type (i64,i64)->i64, the string helpers, each user
    // fn's. Import types are emitted in the same fixed order the indices were assigned above.
    let mut types = TypeSection::new();
    let mut next_type = 0u32;
    // Effect host fns carry a `(file, start, end)` span triple (Stage 4 phase 4g): the host records
    // the effect's `TraceRecord` with that span, so `--engine wasm --trace-effects` produces a trace
    // byte-identical to the interpreter's (criterion 6). The span args are ignored when no sink is set.
    let mut console_root_ty = 0;
    let mut console_println_ty = 0;
    if needs_console {
        console_root_ty = next_type;
        types.ty().function([ValType::I32], [ValType::I32]); // root_console(root) -> cap
        console_println_ty = next_type + 1;
        types.ty().function([ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32], []); // console_println(cap, ptr, file, start, end)
        next_type += 2;
    }
    let mut clock_root_ty = 0;
    let mut clock_now_ty = 0;
    if needs_clock {
        clock_root_ty = next_type;
        types.ty().function([ValType::I32], [ValType::I32]); // root_clock(root) -> cap
        clock_now_ty = next_type + 1;
        types.ty().function([ValType::I32, ValType::I32, ValType::I32, ValType::I32], [ValType::I64]); // clock_now_ms(cap, file, start, end) -> i64
        next_type += 2;
    }
    let mut rand_root_ty = 0;
    let mut rand_int_ty = 0;
    if needs_rand {
        rand_root_ty = next_type;
        types.ty().function([ValType::I32], [ValType::I32]); // root_rand(root) -> cap
        rand_int_ty = next_type + 1;
        types.ty().function([ValType::I32, ValType::I64, ValType::I64, ValType::I32, ValType::I32, ValType::I32], [ValType::I64]); // rand_int(cap, lo, hi, file, start, end) -> i64
        next_type += 2;
    }
    let mut fs_root_ty = 0;
    let mut fs_read_ty = 0;
    if needs_fs {
        fs_root_ty = next_type;
        types.ty().function([ValType::I32, ValType::I32], [ValType::I32]); // root_fs_read(root, path) -> cap
        fs_read_ty = next_type + 1;
        types.ty().function([ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32], [ValType::I32]); // fs_read_text(cap, path, file, start, end) -> result_ptr
        next_type += 2;
    }
    let (mut fs_write_root_ty, mut fs_write_ty) = (0, 0);
    if needs_fs_write {
        fs_write_root_ty = next_type;
        types.ty().function([ValType::I32, ValType::I32], [ValType::I32]); // root_fs_write(root, path) -> cap
        fs_write_ty = next_type + 1;
        // fs_write_text(cap, path, body, append, file, start, end) -> result_ptr
        types.ty().function([ValType::I32; 7], [ValType::I32]);
        next_type += 2;
    }
    let (mut http_root_ty, mut http_get_ty) = (0, 0);
    if needs_http {
        http_root_ty = next_type;
        types.ty().function([ValType::I32; 3], [ValType::I32]); // root_http(root, hosts, desc) -> cap
        http_get_ty = next_type + 1;
        types.ty().function([ValType::I32; 5], [ValType::I32]); // http_get(cap, url, file, start, end) -> result_ptr
        next_type += 2;
    }
    let mut foreign_load_ty = 0;
    let mut foreign_bind_ty = 0;
    let mut foreign_call_ty = 0;
    let mut float_to_str_ty = 0;
    if needs_foreign {
        foreign_load_ty = next_type;
        types.ty().function([ValType::I32], [ValType::I32]); // foreign_load(root) -> Cap[ForeignLoad]
        foreign_bind_ty = next_type + 1;
        types.ty().function([ValType::I32, ValType::I32], [ValType::I32]); // foreign_bind(load, name_ptr) -> Result[M,ForeignErr] cell ptr
        foreign_call_ty = next_type + 2;
        // foreign_call(lib, method_ptr, args_ptr, argc, file, start, end) -> i64 (marshalled return)
        types.ty().function(
            [ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32],
            [ValType::I64],
        );
        float_to_str_ty = next_type + 3;
        types.ty().function([ValType::F64], [ValType::I32]); // float_to_str(x) -> str ptr (matches Value::Float display)
        next_type += 4;
    }
    // Stage 7 phase 7h: the `delulu:actors` import signatures (all actor-boundary values are i64).
    let mut actor_spawn_ty = 0;
    let mut actor_send_ty = 0;
    let mut actor_field_get_ty = 0;
    let mut actor_field_set_ty = 0;
    if needs_actors {
        actor_spawn_ty = next_type;
        types.ty().function([ValType::I32, ValType::I32, ValType::I32], [ValType::I64]); // actor_spawn(actor_idx, args_ptr, argc) -> slot
        actor_send_ty = next_type + 1;
        types.ty().function([ValType::I64, ValType::I32, ValType::I32, ValType::I32], []); // actor_send(slot, behavior_idx, args_ptr, argc)
        actor_field_get_ty = next_type + 2;
        types.ty().function([ValType::I64, ValType::I32], [ValType::I64]); // actor_field_get(slot, field_idx) -> value
        actor_field_set_ty = next_type + 3;
        types.ty().function([ValType::I64, ValType::I32, ValType::I64], []); // actor_field_set(slot, field_idx, value)
        next_type += 4;
    }
    let mut str_split_ty = 0;
    let mut index_fault_ty = 0;
    if needs_lists {
        str_split_ty = next_type;
        types.ty().function([ValType::I32, ValType::I32], [ValType::I32]); // str_split(s, sep) -> List[Str]
        index_fault_ty = next_type + 1;
        types.ty().function([ValType::I64, ValType::I32], []); // index_fault(index, len): records DL0903, then the guest traps
        next_type += 2;
    }
    let mut str_method_ty = 0;
    if needs_str_methods {
        str_method_ty = next_type;
        // str_method(s, method, a, b, lo, hi) -> a Str, a List[Str] or a Bool
        types.ty().function([ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I64, ValType::I64], [ValType::I32]);
        next_type += 1;
    }
    let mut parse_int_ty = 0;
    if needs_parse_int {
        parse_int_ty = next_type;
        // parse_int(s) -> (parsed, value): 1 and the number, or 0 and 0
        types.ty().function([ValType::I32], [ValType::I32, ValType::I64]);
        next_type += 1;
    }
    let mut parse_float_ty = 0;
    if needs_parse_float {
        parse_float_ty = next_type;
        // parse_float(s) -> (parsed, value)
        types.ty().function([ValType::I32], [ValType::I32, ValType::F64]);
        next_type += 1;
    }
    let mut float_str_ty = 0;
    if needs_float_str {
        float_str_ty = next_type;
        types.ty().function([ValType::F64], [ValType::I32]); // to_str(x) -> str ptr (`Value::Float`'s display)
        next_type += 1;
    }
    let mut float_rem_ty = 0;
    if needs_floats {
        float_rem_ty = next_type;
        types.ty().function([ValType::F64, ValType::F64], [ValType::F64]); // rem(a, b) -> a % b
        next_type += 1;
    }
    let (mut value_display_ty, mut value_eq_ty) = (0, 0);
    if needs_values {
        value_display_ty = next_type;
        types.ty().function([ValType::I32, ValType::I32], [ValType::I32]); // display(v, desc) -> str ptr
        value_eq_ty = next_type + 1;
        types.ty().function([ValType::I32, ValType::I32, ValType::I32], [ValType::I32]); // eq(a, b, desc) -> Bool
        next_type += 2;
    }
    let helper_type = next_type;
    types.ty().function([ValType::I64, ValType::I64], [ValType::I64]);
    next_type += 1;
    let concat_type = next_type; // __concat(a_ptr, b_ptr) -> new_ptr
    types.ty().function([ValType::I32, ValType::I32], [ValType::I32]);
    next_type += 1;
    let int_to_str_type = next_type; // __int_to_str(n) -> ptr
    types.ty().function([ValType::I64], [ValType::I32]);
    next_type += 1;
    let alloc_type = next_type; // __alloc(n) -> ptr
    types.ty().function([ValType::I32], [ValType::I32]);
    next_type += 1;
    let str_len_type = next_type; // __str_len(s) -> n
    types.ty().function([ValType::I32], [ValType::I64]);
    next_type += 1;
    let mut user_types = Vec::new();
    for f in &fns {
        let params: Vec<ValType> = f.params.iter().map(|p| wasm_valtype(wasm_ty(&env, &p.ty).unwrap()).unwrap()).collect();
        let results = result_valtypes(ret_ty(&env, f).unwrap());
        types.ty().function(params, results);
        user_types.push(next_type);
        next_type += 1;
    }
    // Stage 7 phase 7h: the subset gate runs EARLY — one out-of-subset actor field or
    // member parameter refuses the WHOLE module (honest DL1201 interpreter fallback)
    // before any function body compiles, so no partial artifact and no late panic path.
    for (_, decl, params, _, _) in &member_plans {
        for fld in &decl.fields {
            if actor_boundary_ty(&env, &fld.ty).is_none() {
                return Err(CompileError::Unsupported(format!(
                    "actor field `{}.{}` is outside the Int/actor-reference subset",
                    decl.name.name, fld.name.name
                )));
            }
        }
        for p in params.iter() {
            if actor_boundary_ty(&env, &p.ty).is_none() {
                return Err(CompileError::Unsupported(format!(
                    "actor member parameter `{}` is outside the Int/actor-reference subset",
                    p.name.name
                )));
            }
        }
    }

    // Stage 7 phase 7h: one type per actor member — `(self_slot, params…) -> ()`, all i64.
    let mut actor_member_types = Vec::new();
    for (_, _, params, _, _) in &member_plans {
        let ptys: Vec<ValType> = std::iter::once(ValType::I64).chain(params.iter().map(|_| ValType::I64)).collect();
        types.ty().function(ptys, []);
        actor_member_types.push(next_type);
        next_type += 1;
    }

    let mut imports = ImportSection::new();
    if needs_console {
        imports.import("delulu:cap", "root_console", EntityType::Function(console_root_ty));
        imports.import("delulu:cap", "console_println", EntityType::Function(console_println_ty));
    }
    if needs_clock {
        imports.import("delulu:cap", "root_clock", EntityType::Function(clock_root_ty));
        imports.import("delulu:cap", "clock_now_ms", EntityType::Function(clock_now_ty));
    }
    if needs_rand {
        imports.import("delulu:cap", "root_rand", EntityType::Function(rand_root_ty));
        imports.import("delulu:cap", "rand_int", EntityType::Function(rand_int_ty));
    }
    if needs_fs {
        imports.import("delulu:cap", "root_fs_read", EntityType::Function(fs_root_ty));
        imports.import("delulu:cap", "fs_read_text", EntityType::Function(fs_read_ty));
        // fs_narrow(cap, path) -> cap
        imports.import("delulu:cap", "fs_narrow", EntityType::Function(fs_root_ty));
    }
    if needs_fs_write {
        imports.import("delulu:cap", "root_fs_write", EntityType::Function(fs_write_root_ty));
        imports.import("delulu:cap", "fs_write_text", EntityType::Function(fs_write_ty));
    }
    if needs_http {
        imports.import("delulu:cap", "root_http", EntityType::Function(http_root_ty));
        imports.import("delulu:cap", "http_get", EntityType::Function(http_get_ty));
    }
    if needs_foreign {
        imports.import("delulu:foreign", "foreign_load", EntityType::Function(foreign_load_ty));
        imports.import("delulu:foreign", "foreign_bind", EntityType::Function(foreign_bind_ty));
        imports.import("delulu:foreign", "foreign_call", EntityType::Function(foreign_call_ty));
        imports.import("delulu:foreign", "float_to_str", EntityType::Function(float_to_str_ty));
    }
    if needs_actors {
        imports.import("delulu:actors", "actor_spawn", EntityType::Function(actor_spawn_ty));
        imports.import("delulu:actors", "actor_send", EntityType::Function(actor_send_ty));
        imports.import("delulu:actors", "actor_field_get", EntityType::Function(actor_field_get_ty));
        imports.import("delulu:actors", "actor_field_set", EntityType::Function(actor_field_set_ty));
    }
    if needs_lists {
        imports.import("delulu:list", "str_split", EntityType::Function(str_split_ty));
        imports.import("delulu:list", "index_fault", EntityType::Function(index_fault_ty));
    }
    if needs_str_methods {
        imports.import("delulu:str", "str_method", EntityType::Function(str_method_ty));
    }
    if needs_parse_int {
        imports.import("delulu:str", "parse_int", EntityType::Function(parse_int_ty));
    }
    if needs_parse_float {
        imports.import("delulu:str", "parse_float", EntityType::Function(parse_float_ty));
    }
    if needs_float_str {
        imports.import("delulu:float", "to_str", EntityType::Function(float_str_ty));
    }
    if needs_floats {
        imports.import("delulu:float", "rem", EntityType::Function(float_rem_ty));
    }
    if needs_values {
        imports.import("delulu:value", "display", EntityType::Function(value_display_ty));
        imports.import("delulu:value", "eq", EntityType::Function(value_eq_ty));
    }

    // Functions (in code order): the 4 arithmetic helpers, `__concat`, `__int_to_str`, `__alloc`, `__str_len`, users,
    // then the actor members (Stage 7 phase 7h).
    let mut funcsec = FunctionSection::new();
    for _ in 0..N_ARITH_HELPERS {
        funcsec.function(helper_type);
    }
    funcsec.function(concat_type);
    funcsec.function(int_to_str_type);
    funcsec.function(alloc_type);
    funcsec.function(str_len_type);
    for t in &user_types {
        funcsec.function(*t);
    }
    for t in &actor_member_types {
        funcsec.function(*t);
    }
    if needs_lists {
        funcsec.function(alloc_type); // __list_new(len) -> list
        funcsec.function(alloc_type); // __list_slot(list) -> the address of a new last slot
    }
    if needs_str_eq {
        funcsec.function(concat_type); // __str_eq(a, b) -> Bool
    }

    let mut exports = ExportSection::new();
    exports.export("memory", ExportKind::Memory, 0);
    // Export the bump-heap pointer so the host can allocate in guest memory (the filesystem cap
    // writes the file content + `Result[Str, IoErr]` cells into the guest heap — Phase 3p).
    exports.export("__heap", ExportKind::Global, HEAP_GLOBAL);
    for (i, f) in fns.iter().enumerate() {
        exports.export(&f.name.name, ExportKind::Func, user_base + i as u32);
    }
    // Stage 7 phase 7h: export each actor member under the `actor$<Actor>$<member>` convention, at
    // its absolute function index (after the user fns). The host dispatches turns by these names.
    let actor_base = user_base + fns.len() as u32;
    for (j, (_, _, _, _, export)) in member_plans.iter().enumerate() {
        exports.export(export, ExportKind::Func, actor_base + j as u32);
    }
    // RW 3.21: each actor's field count, an immutable global after the heap pointer, so the module carries its own actor
    // table — what a `.dwx` of an actor program is run by (`actors::table_from_module`).
    for (i, a) in table.actors.iter().enumerate() {
        exports.export(&crate::actors::fields_export_name(&a.name), ExportKind::Global, HEAP_GLOBAL + 1 + i as u32);
    }
    // The late helpers follow the actor members, each only where the module uses it.
    let mut late = actor_base + member_plans.len() as u32;
    if needs_lists {
        imp.list_new = late;
        imp.list_slot = late + 1;
        late += 2;
    }
    if needs_str_eq {
        imp.str_eq = late;
    }

    let mut code = CodeSection::new();
    code.function(&ovf_add_fn());
    code.function(&ovf_sub_fn());
    code.function(&ovf_mul_fn());
    code.function(&chk_rem_fn());
    let alloc = arith_base + N_ARITH_HELPERS + 2;
    code.function(&concat_fn(alloc));
    code.function(&int_to_str_fn(alloc));
    code.function(&alloc_fn(HEAP_GLOBAL));
    code.function(&str_len_fn());
    for f in &fns {
        code.function(&compile_fn(
            f,
            &env,
            &generics,
            &named,
            &index,
            &str_off,
            arith_base,
            imp,
            actor_imp,
            &table,
            foreign_binds,
            node_types,
            &descs,
        )?);
    }
    for (actor_idx, decl, params, body, _) in &member_plans {
        code.function(&compile_actor_member(
            *actor_idx,
            decl,
            params,
            body,
            &env,
            &generics,
            &named,
            &index,
            &str_off,
            arith_base,
            imp,
            actor_imp,
            &table,
            foreign_binds,
            node_types,
            &descs,
        )?);
    }
    if needs_lists {
        code.function(&list_new_fn(alloc));
        code.function(&list_slot_fn(alloc));
    }
    if needs_str_eq {
        code.function(&str_eq_fn());
    }

    // The type descriptors the bodies named follow the literals, so the heap starts past both.
    data.extend(descs.into_inner().bytes);
    let mut mems = MemorySection::new();
    // Reserve the interned literal image plus HEAP_PAGES of bump-allocation heap to start; `__alloc` grows the memory past
    // them when an allocation needs it (RW 3.4).
    let data_pages = (data.len() as u64).div_ceil(65536);
    let min_pages = data_pages + HEAP_PAGES;
    mems.memory(MemoryType { minimum: min_pages, maximum: None, memory64: false, shared: false, page_size_log2: None });

    // The bump-heap pointer: a mutable i32 global starting just past the interned literals.
    let heap_start = data.len() as i32;
    let mut globals = GlobalSection::new();
    globals.global(GlobalType { val_type: ValType::I32, mutable: true, shared: false }, &ConstExpr::i32_const(heap_start));
    for a in &table.actors {
        let fields =
            i32::try_from(a.field_count).map_err(|_| CompileError::Unsupported("an actor with too many fields".into()))?;
        globals.global(GlobalType { val_type: ValType::I32, mutable: false, shared: false }, &ConstExpr::i32_const(fields));
    }

    let mut datas = DataSection::new();
    if !data.is_empty() {
        datas.active(0, &ConstExpr::i32_const(0), data.iter().copied());
    }

    // Section order: Type(1), Import(2), Function(3), Memory(5), Global(6), Export(7), Code(10),
    // Data(11).
    let mut m = WasmModule::new();
    m.section(&types);
    if needs_console
        || needs_clock
        || needs_rand
        || needs_fs
        || needs_fs_write
        || needs_http
        || needs_foreign
        || needs_actors
        || needs_lists
        || needs_str_methods
        || needs_parse_int
        || needs_parse_float
        || needs_floats
        || needs_values
    {
        m.section(&imports);
    }
    m.section(&funcsec);
    m.section(&mems);
    m.section(&globals);
    m.section(&exports);
    m.section(&code);
    m.section(&datas);
    Ok(m.finish())
}

fn result_valtypes(ret: Ty) -> Vec<ValType> {
    match wasm_valtype(ret) {
        Some(v) => vec![v],
        None => vec![],
    }
}

fn collect_strings_block(b: &Block, off: &mut HashMap<String, u32>, data: &mut Vec<u8>) {
    for s in &b.stmts {
        match s {
            Stmt::Let { value, .. } | Stmt::Assign { value, .. } => collect_strings_expr(value, off, data),
            Stmt::For { iter, body, .. } => {
                collect_strings_expr(iter, off, data);
                collect_strings_block(body, off, data);
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
            Stmt::While { cond, body, .. } => {
                collect_strings_expr(cond, off, data);
                collect_strings_block(body, off, data);
            }
            Stmt::Return { value: Some(e), .. } => collect_strings_expr(e, off, data),
            Stmt::Return { value: None, .. } => {}
            Stmt::Expr(e) => collect_strings_expr(e, off, data),
        }
    }
}

fn collect_strings_expr(e: &Expr, off: &mut HashMap<String, u32>, data: &mut Vec<u8>) {
    match e {
        Expr::Lit { kind: LitKind::Str(s), .. } => intern_string(s, off, data),
        Expr::Method { recv, args, .. } => {
            collect_strings_expr(recv, off, data);
            for a in args {
                collect_strings_expr(a, off, data);
            }
        }
        Expr::Call { callee, args, .. } => {
            collect_strings_expr(callee, off, data);
            for a in args {
                collect_strings_expr(a, off, data);
            }
        }
        Expr::Binary { lhs, rhs, .. } => {
            collect_strings_expr(lhs, off, data);
            collect_strings_expr(rhs, off, data);
        }
        Expr::Unary { operand, .. } => collect_strings_expr(operand, off, data),
        Expr::If { cond, then_, else_, .. } => {
            collect_strings_expr(cond, off, data);
            collect_strings_block(then_, off, data);
            if let Some(e) = else_ {
                collect_strings_expr(e, off, data);
            }
        }
        Expr::Match { scrutinee, arms, .. } => {
            collect_strings_expr(scrutinee, off, data);
            for arm in arms {
                collect_strings_expr(&arm.body, off, data);
            }
        }
        Expr::Try { inner, .. } => collect_strings_expr(inner, off, data),
        Expr::Block(b) => collect_strings_block(b, off, data),
        // Stage 7: spawn/send arguments and recover bodies carry literals like any call.
        Expr::Spawn { args, .. } => {
            for a in args {
                collect_strings_expr(a, off, data);
            }
        }
        Expr::Recover { body, .. } => collect_strings_block(body, off, data),
        // WASM-INTERN-1: a literal inside a lambda, a list, a record or an index was never interned, and compiling it
        // panicked at `expect("interned")` — `apply(fn(s: Str) -> Str { s + "!" }, "a")` crashed `--engine wasm`.
        Expr::Lambda { body, .. } => collect_strings_block(body, off, data),
        Expr::List { items, .. } => {
            for i in items {
                collect_strings_expr(i, off, data);
            }
        }
        Expr::Record { fields, .. } => {
            for (_, e) in fields {
                collect_strings_expr(e, off, data);
            }
        }
        Expr::Field { recv, .. } => collect_strings_expr(recv, off, data),
        Expr::Index { recv, index, .. } => {
            collect_strings_expr(recv, off, data);
            collect_strings_expr(index, off, data);
        }
        Expr::Lit { .. } | Expr::Var { .. } | Expr::Consume { .. } => {}
    }
}

fn intern_string(s: &str, off: &mut HashMap<String, u32>, data: &mut Vec<u8>) {
    if off.contains_key(s) {
        return;
    }
    let ptr = data.len() as u32;
    data.extend_from_slice(&(s.len() as u32).to_le_bytes());
    data.extend_from_slice(s.as_bytes());
    off.insert(s.to_string(), ptr);
}

/// Number of synthetic checked-arithmetic helper functions emitted before user functions:
/// `__ovf_add`, `__ovf_sub`, `__ovf_mul`, `__chk_rem` (in that order). `Div` uses `i64.div_s`
/// directly, which traps on div-by-zero and `INT_MIN/-1` exactly like the interpreter's
/// `checked_div`, so it needs no helper.
const N_ARITH_HELPERS: u32 = 4;

/// Number of synthetic helpers emitted after the arithmetic helpers: `__concat`, `__int_to_str`, `__alloc`, then
/// `__str_len` (in that order).
const N_STR_HELPERS: u32 = 4;

/// Pages (64 KiB each) reserved for the runtime bump-allocation heap at start, above the interned literals. Every guest
/// allocation goes through `__alloc`, which grows the memory past them when it must (RW 3.4: a loop that builds a string
/// outgrew a fixed 1 MiB); the run's memory budget holds the growth, as it holds the interpreter's.
const HEAP_PAGES: u64 = 16;

/// Index of the bump-heap pointer global (the module emits exactly one global).
const HEAP_GLOBAL: u32 = 0;

/// A function VALUE the backend inlines rather than represents at runtime: a lambda, or a generic
/// function specialised at a call site. `params`/`body` are owned clones of the AST (Phase 3r).
#[derive(Clone)]
struct Callable {
    params: Vec<Param>,
    body: Block,
    /// Where it was defined (WASM-HYGIENE-1): the frames hidden there, and how many scope and callable frames there
    /// were. Inlined anywhere, its body sees those frames and no other — as the interpreter's closure sees its own.
    hidden: Vec<Hidden>,
    scopes_at: usize,
    callables_at: usize,
}

/// A run of scope frames and a run of callable frames, `start..end` each, that a body being inlined cannot see
/// (WASM-HYGIENE-1): a generic function's body sees none of its caller's frames, a lambda's only its definition's.
#[derive(Clone, Copy, Debug)]
struct Hidden {
    scopes: (usize, usize),
    callables: (usize, usize),
}

/// Whose names an inlined body sees (WASM-HYGIENE-1).
enum Lexical<'c> {
    /// A generic function: its parameters, and nothing of its caller's.
    Generic,
    /// A function value: what was visible where it was defined.
    Value(&'c Callable),
}

struct Cx<'a> {
    index: &'a HashMap<String, (u32, Vec<Ty>, Ty)>,
    str_off: &'a HashMap<String, u32>,
    /// The module's named sum types (Phase 3o Checkpoint 3).
    env: &'a EnumEnv,
    /// Generic user functions, inlined (monomorphised) at each call site (Phase 3r).
    generics: &'a HashMap<String, FnDecl>,
    /// Each function a value may name — `apply(double, 21)`, `xs.map(double)`, `let h = double` (RW 3.4) — built by
    /// `named_value`. Seen wherever no local or function value of the same name hides it.
    named: &'a HashMap<String, Callable>,
    scopes: Vec<HashMap<String, (u32, Ty)>>,
    /// Function-value bindings (lambdas / generic fn params of function type), a stack of frames
    /// pushed at each inline boundary. A name is a local xor a callable.
    callables: Vec<HashMap<String, Callable>>,
    extra_locals: Vec<ValType>,
    nparams: u32,
    /// Recursion/blow-up guard for inlining.
    inline_depth: u32,
    /// Function index of the first arithmetic helper (`__ovf_add`); the others follow.
    arith_base: u32,
    /// Function index of the `__concat` helper (`Str + Str`).
    concat_fn: u32,
    /// Function index of the `__int_to_str` helper (`str(Int)`).
    int_to_str_fn: u32,
    /// Function index of the `__alloc` helper — every guest allocation (RW 3.4).
    alloc_fn: u32,
    /// Function index of the `__str_len` helper (`Str.len()`, RW 3.4).
    str_len_fn: u32,
    /// The WASM labels open at this point of `instrs` — each `block`, `loop` and `if` opens one, each `end` closes it —
    /// which is what a `br` counts outward (RW 3.4).
    depth: u32,
    /// Each enclosing `while`, innermost last: the label depths of its `block` (`break` leaves it) and its `loop`
    /// (`continue` runs it again).
    loops: Vec<(u32, u32)>,
    /// The first scope of the body being inlined (`inline_call`), or 0: an assignment inside an inlined body may name only
    /// a variable that body declared — what a function value does to its caller's variables is not this backend's to guess.
    inline_floor: usize,
    /// The frames the body being compiled cannot see — empty in a function's own body (WASM-HYGIENE-1).
    hidden: Vec<Hidden>,
    /// Host import function indices (`root.console()`/`println`, `root.clock()`/`now_ms`).
    imp: Imports,
    /// The checker's `root.foreign(load)` bind-site → lib-name map (Stage 4 phase 4g): recovers which
    /// lib a bind site resolves to (the grammar has no method type-argument syntax).
    foreign_binds: &'a HashMap<NodeId, String>,
    /// The checker's type for each node (`compile_checked`; empty through `compile_module`): what an empty list holds
    /// when no `let` says (RW 3.4).
    node_types: &'a HashMap<NodeId, delulu_check::Type>,
    /// The type descriptors `str` and `==` of a compound value name (`desc_ptr`), shared by every body of the module.
    descs: &'a RefCell<Descs>,
    /// The module's actor table (Stage 7 phase 7h): `spawn`/send index resolution + ctor arity.
    actors: &'a ActorTable,
    /// The `delulu:actors` host import indices (Stage 7 phase 7h).
    actor_imp: ActorImports,
    /// Present while compiling an actor member body (Stage 7 phase 7h) — `self`, self-fields,
    /// self-sends. `None` in ordinary functions (which may still `spawn`/send).
    actor: Option<ActorCtx>,
    /// The function's declared return type — the expected type for `return`/`?` (Phase 3o).
    ret: Ty,
    instrs: Vec<Instruction<'static>>,
}

impl<'a> Cx<'a> {
    fn lookup(&self, name: &str) -> Option<(u32, Ty)> {
        self.lookup_frame(name).map(|(_, b)| b)
    }
    /// A local's binding and the index of the scope frame that holds it — the innermost frame this body can see.
    fn lookup_frame(&self, name: &str) -> Option<(usize, (u32, Ty))> {
        let seen = |i: usize| !self.hidden.iter().any(|h| (h.scopes.0..h.scopes.1).contains(&i));
        self.scopes.iter().enumerate().rev().filter(|(i, _)| seen(*i)).find_map(|(i, s)| s.get(name).map(|b| (i, *b)))
    }
    fn lookup_callable(&self, name: &str) -> Option<Callable> {
        let seen = |i: usize| !self.hidden.iter().any(|h| (h.callables.0..h.callables.1).contains(&i));
        self.callables.iter().enumerate().rev().filter(|(i, _)| seen(*i)).find_map(|(_, s)| s.get(name).cloned())
    }
    /// A function value defined here: it will see what this body sees (WASM-HYGIENE-1).
    fn callable(&self, params: &[Param], body: &Block) -> Callable {
        Callable {
            params: params.to_vec(),
            body: body.clone(),
            hidden: self.hidden.clone(),
            scopes_at: self.scopes.len(),
            callables_at: self.callables.len(),
        }
    }
    fn alloc_local(&mut self, ty: Ty) -> Result<u32, CompileError> {
        let vt = wasm_valtype(ty).ok_or_else(|| CompileError::Unsupported("a `let` binding of type Unit".into()))?;
        let idx = self.nparams + self.extra_locals.len() as u32;
        self.extra_locals.push(vt);
        Ok(idx)
    }
    fn emit(&mut self, i: Instruction<'static>) {
        match i {
            Instruction::Block(_) | Instruction::Loop(_) | Instruction::If(_) => self.depth += 1,
            Instruction::End => self.depth = self.depth.saturating_sub(1),
            _ => {}
        }
        self.instrs.push(i);
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_fn<'a>(
    f: &FnDecl,
    env: &'a EnumEnv,
    generics: &'a HashMap<String, FnDecl>,
    named: &'a HashMap<String, Callable>,
    index: &'a HashMap<String, (u32, Vec<Ty>, Ty)>,
    str_off: &'a HashMap<String, u32>,
    arith_base: u32,
    imp: Imports,
    actor_imp: ActorImports,
    actors: &'a ActorTable,
    foreign_binds: &'a HashMap<NodeId, String>,
    node_types: &'a HashMap<NodeId, delulu_check::Type>,
    descs: &'a RefCell<Descs>,
) -> Result<Function, CompileError> {
    let mut params = HashMap::new();
    for (i, p) in f.params.iter().enumerate() {
        params.insert(p.name.name.clone(), (i as u32, wasm_ty(env, &p.ty).unwrap()));
    }
    let ret = ret_ty(env, f).unwrap();
    let mut cx = Cx {
        index,
        str_off,
        env,
        generics,
        named,
        scopes: vec![params],
        callables: vec![HashMap::new()],
        extra_locals: Vec::new(),
        nparams: f.params.len() as u32,
        inline_depth: 0,
        arith_base,
        concat_fn: arith_base + N_ARITH_HELPERS,
        int_to_str_fn: arith_base + N_ARITH_HELPERS + 1,
        alloc_fn: arith_base + N_ARITH_HELPERS + 2,
        str_len_fn: arith_base + N_ARITH_HELPERS + 3,
        depth: 0,
        loops: Vec::new(),
        inline_floor: 0,
        hidden: Vec::new(),
        imp,
        foreign_binds,
        node_types,
        descs,
        actors,
        actor_imp,
        actor: None,
        ret,
        instrs: Vec::new(),
    };
    // Compile the body expecting the declared return type (expected-type-directed, so variant
    // constructions in tail position know their full `Result`/`Option` type — Phase 3o).
    compile_block_as(&f.body, &mut cx, ret)?;
    let mut func = Function::new(cx.extra_locals.iter().map(|&t| (1u32, t)));
    for ins in &cx.instrs {
        func.instruction(ins);
    }
    func.instruction(&Instruction::End);
    Ok(func)
}

/// Compile one actor member (the ctor or a behaviour) to a wasm function (Stage 7 phase 7h).
/// The export signature is all-i64: `(self_slot, param0, param1, …) -> ()`. `Int` and actor-ref
/// params are i64 directly; the body reaches actor state through the `delulu:actors.field_*`
/// imports and sends through `spawn`/`send`. A param or field outside the subset (`Int`/actor-ref)
/// makes the member — and therefore the whole module — non-compilable (honest DL1201 fallback).
#[allow(clippy::too_many_arguments)]
fn compile_actor_member<'a>(
    actor_idx: u32,
    decl: &ActorDecl,
    params: &[Param],
    body: &Block,
    env: &'a EnumEnv,
    generics: &'a HashMap<String, FnDecl>,
    named: &'a HashMap<String, Callable>,
    index: &'a HashMap<String, (u32, Vec<Ty>, Ty)>,
    str_off: &'a HashMap<String, u32>,
    arith_base: u32,
    imp: Imports,
    actor_imp: ActorImports,
    actors: &'a ActorTable,
    foreign_binds: &'a HashMap<NodeId, String>,
    node_types: &'a HashMap<NodeId, delulu_check::Type>,
    descs: &'a RefCell<Descs>,
) -> Result<Function, CompileError> {
    // Field layout: name → (slot index, lowered type). Only `Int`/actor-ref fields are in-subset.
    let mut fields: HashMap<String, (u32, Ty)> = HashMap::new();
    for (i, fld) in decl.fields.iter().enumerate() {
        let fty = actor_boundary_ty(env, &fld.ty).ok_or_else(|| {
            CompileError::Unsupported(format!(
                "an actor field `{}` of a type outside the Int/actor-reference subset",
                fld.name.name
            ))
        })?;
        fields.insert(fld.name.name.clone(), (i as u32, fty));
    }
    // The implicit `self_slot` is local 0; declared params follow (each an in-subset i64 value).
    let mut scope: HashMap<String, (u32, Ty)> = HashMap::new();
    for (i, p) in params.iter().enumerate() {
        let pty = actor_boundary_ty(env, &p.ty).ok_or_else(|| {
            CompileError::Unsupported(format!(
                "an actor parameter `{}` of a type outside the Int/actor-reference subset",
                p.name.name
            ))
        })?;
        scope.insert(p.name.name.clone(), (i as u32 + 1, pty));
    }
    let nparams = params.len() as u32 + 1;
    let mut cx = Cx {
        index,
        str_off,
        env,
        generics,
        named,
        scopes: vec![scope],
        callables: vec![HashMap::new()],
        extra_locals: Vec::new(),
        nparams,
        inline_depth: 0,
        arith_base,
        concat_fn: arith_base + N_ARITH_HELPERS,
        int_to_str_fn: arith_base + N_ARITH_HELPERS + 1,
        alloc_fn: arith_base + N_ARITH_HELPERS + 2,
        str_len_fn: arith_base + N_ARITH_HELPERS + 3,
        depth: 0,
        loops: Vec::new(),
        inline_floor: 0,
        hidden: Vec::new(),
        imp,
        foreign_binds,
        node_types,
        descs,
        actors,
        actor_imp,
        actor: Some(ActorCtx { self_slot: 0, actor_idx, fields }),
        ret: Ty::Unit,
        instrs: Vec::new(),
    };
    // A behaviour/ctor body yields Unit (invariant 34 — a behaviour returns nothing at the send
    // site; the ctor returns nothing).
    compile_block_as(body, &mut cx, Ty::Unit)?;
    let mut func = Function::new(cx.extra_locals.iter().map(|&t| (1u32, t)));
    for ins in &cx.instrs {
        func.instruction(ins);
    }
    func.instruction(&Instruction::End);
    Ok(func)
}

/// The lowered type of an actor field / behaviour-ctor parameter, restricted to the phase-7h
/// subset: `Int` (i64) and actor references (`tag A` → i64 slot). Anything else → `None`
/// (honest DL1201). Kept separate from `wasm_ty` so the subset boundary is explicit and testable.
fn actor_boundary_ty(env: &EnumEnv, t: &TypeExpr) -> Option<Ty> {
    match wasm_ty(env, t)? {
        Ty::I64 => Some(Ty::I64),
        Ty::ActorRef(i) => Some(Ty::ActorRef(i)),
        _ => None,
    }
}

/// Synthesis: compile a block and report its tail value type.
fn compile_block(b: &Block, cx: &mut Cx) -> Result<Ty, CompileError> {
    compile_block_inner(b, cx, None)
}

/// Expected-type-directed: compile a block whose tail must produce `expected`.
fn compile_block_as(b: &Block, cx: &mut Cx, expected: Ty) -> Result<(), CompileError> {
    compile_block_inner(b, cx, Some(expected)).map(|_| ())
}

/// A block opens a frame of locals AND a frame of function values: a `let f = fn…` inside an `if`, a `while` or a
/// match arm is that block's, and the `f` outside it is the outer one again once it ends (WASM-HYGIENE-2 — the binding
/// had gone into the function's own frame, so an inner `f` replaced the outer for the rest of the function).
fn compile_block_inner(b: &Block, cx: &mut Cx, expected: Option<Ty>) -> Result<Ty, CompileError> {
    cx.scopes.push(HashMap::new());
    cx.callables.push(HashMap::new());
    let mut result = Ty::Unit;
    let n = b.stmts.len();
    for (i, stmt) in b.stmts.iter().enumerate() {
        let is_last = i + 1 == n;
        let r = compile_stmt(stmt, cx, is_last, expected);
        match r {
            Ok(ty) => result = ty,
            Err(e) => {
                cx.callables.pop();
                cx.scopes.pop();
                return Err(e);
            }
        }
    }
    cx.callables.pop();
    cx.scopes.pop();
    Ok(result)
}

fn compile_stmt(stmt: &Stmt, cx: &mut Cx, is_last: bool, expected: Option<Ty>) -> Result<Ty, CompileError> {
    match stmt {
        Stmt::Let { name, ty, value, .. } => {
            // Binding a lambda binds a function VALUE (inlined at its call sites), not a runtime local.
            if let Expr::Lambda { params, body, .. } = value {
                let c = cx.callable(params, body);
                cx.callables.last_mut().unwrap().insert(name.name.clone(), c);
                return Ok(Ty::Unit);
            }
            // So does binding a name that is one — `let h = double`, `let g = h` (RW 3.4).
            if let Expr::Var { path, .. } = value {
                if path.segs.len() == 1 && cx.lookup(&path.segs[0].name).is_none() {
                    if let Ok(c) = resolve_callable(value, cx) {
                        cx.callables.last_mut().unwrap().insert(name.name.clone(), c);
                        return Ok(Ty::Unit);
                    }
                }
            }
            // A written type is the value's expected one: an empty list's element type (`let xs: List[Int] = []`), a
            // constructor's other half (`let r: Result[Int, Str] = Ok(1)`, `var cur: Option[ref Node] = None`).
            let ty = match (value, ty.as_ref().and_then(|t| wasm_ty(cx.env, t))) {
                (Expr::List { items, .. }, Some(Ty::List(s))) if items.is_empty() => compile_list_literal(items, cx, Some(s))?,
                (_, Some(t)) => {
                    compile_expr_as(value, cx, t)?;
                    t
                }
                _ => compile_expr(value, cx)?,
            };
            let idx = cx.alloc_local(ty)?;
            cx.emit(Instruction::LocalSet(idx));
            cx.scopes.last_mut().unwrap().insert(name.name.clone(), (idx, ty));
            Ok(Ty::Unit)
        }
        Stmt::Expr(e) => {
            // Stage 7 phase 7h: `if cond { … }` with no `else`, used as a statement (a Unit-typed
            // then-branch — e.g. a self-send guarded by a condition). The value-`if` path
            // (`compile_expr`) still requires an `else`; this is the statement form.
            if let Expr::If { cond, then_, else_: None, .. } = e {
                if compile_expr(cond, cx)? != Ty::I32 {
                    return Err(CompileError::Unsupported("a non-Bool `if` condition".into()));
                }
                cx.emit(Instruction::If(BlockType::Empty));
                compile_block_as(then_, cx, Ty::Unit)?;
                cx.emit(Instruction::End);
                return Ok(Ty::Unit);
            }
            if is_last {
                match expected {
                    Some(exp) => {
                        compile_expr_as(e, cx, exp)?;
                        Ok(exp)
                    }
                    None => compile_expr(e, cx),
                }
            } else {
                let ty = compile_expr(e, cx)?;
                if ty != Ty::Unit {
                    cx.emit(Instruction::Drop);
                }
                Ok(Ty::Unit)
            }
        }
        // `return e` always targets the function's declared return type (`cx.ret`).
        Stmt::Return { value: Some(e), .. } => {
            let ret = cx.ret;
            compile_expr_as(e, cx, ret)?;
            cx.emit(Instruction::Return);
            Ok(if is_last { ret } else { Ty::Unit })
        }
        Stmt::Return { value: None, .. } => {
            cx.emit(Instruction::Return);
            Ok(Ty::Unit)
        }
        // Stage 7 phase 7h: `self.field = …` lowers to the actor-state host import. Other
        // assignment targets (locals, indices) stay outside the subset (honest DL1201).
        Stmt::Assign { target, value, .. } => compile_assign(target, value, cx),
        // RW 3.4: `while c { body }` is `block { loop { br_if 1 (!c); body; br 0 } }` — `break` leaves the block,
        // `continue` runs the loop again. The run's processor budget holds an endless one, as it holds the interpreter's.
        Stmt::While { cond, body, .. } => {
            cx.emit(Instruction::Block(BlockType::Empty));
            let brk = cx.depth;
            cx.emit(Instruction::Loop(BlockType::Empty));
            let cont = cx.depth;
            if compile_expr(cond, cx)? != Ty::I32 {
                return Err(CompileError::Unsupported("a non-Bool `while` condition".into()));
            }
            cx.emit(Instruction::I32Eqz);
            cx.emit(Instruction::BrIf(cx.depth - brk));
            cx.loops.push((brk, cont));
            let body_ty = compile_block(body, cx);
            cx.loops.pop();
            if body_ty? != Ty::Unit {
                cx.emit(Instruction::Drop);
            }
            cx.emit(Instruction::Br(cx.depth - cont));
            cx.emit(Instruction::End);
            cx.emit(Instruction::End);
            Ok(Ty::Unit)
        }
        Stmt::For { var, iter, body, .. } => compile_for(var, iter, body, cx),
        Stmt::Break { .. } | Stmt::Continue { .. } => {
            let Some(&(brk, cont)) = cx.loops.last() else {
                return Err(CompileError::Unsupported("`break` or `continue` outside a compiled `while`".into()));
            };
            let to = if matches!(stmt, Stmt::Break { .. }) { brk } else { cont };
            cx.emit(Instruction::Br(cx.depth - to));
            Ok(Ty::Unit)
        }
    }
}

/// Compile an assignment: `self.field = …` (phase 7h: actor state lives host-side, reached through the
/// `delulu:actors.field_set` import — an actor may write only its OWN fields, the checker's viewpoint/tag rules deny
/// anything else), and `x = …` to a local (RW 3.4: the checker has already held it to a `let mut` of the value's type).
fn compile_assign(target: &LValue, value: &Expr, cx: &mut Cx) -> Result<Ty, CompileError> {
    if let LValue::Var(v) = target {
        let Some((depth, (idx, ty))) = cx.lookup_frame(&v.name) else {
            return Err(CompileError::Unsupported(format!("an assignment to `{}`, which is no local here", v.name)));
        };
        if depth < cx.inline_floor {
            return Err(CompileError::Unsupported(format!(
                "an assignment to `{}` from inside an inlined function value",
                v.name
            )));
        }
        compile_expr_as(value, cx, ty)?;
        cx.emit(Instruction::LocalSet(idx));
        return Ok(Ty::Unit);
    }
    if let LValue::Field(base, name) = target {
        if let LValue::Var(v) = &**base {
            if v.name == "self" {
                let (fidx, fty, self_slot) = self_field(cx, &name.name)?;
                if cx.actor_imp.field_set == u32::MAX {
                    return Err(CompileError::Unsupported("a self-field assignment without the actor host interface".into()));
                }
                cx.emit(Instruction::LocalGet(self_slot)); // slot: i64
                cx.emit(Instruction::I32Const(fidx as i32)); // field_idx: i32
                compile_expr_as(value, cx, fty)?; // value at the field's logical type
                emit_extend_to_i64(cx, fty)?; // -> the i64 boundary ABI
                cx.emit(Instruction::Call(cx.actor_imp.field_set));
                return Ok(Ty::Unit);
            }
        }
    }
    if let LValue::Field(base, name) = target {
        return compile_field_assign(base, &name.name, value, cx);
    }
    Err(CompileError::Unsupported("an assignment target other than a local or a field (indices unsupported)".into()))
}

// ----- RW 3.4: records ---------------------------------------------------------------------------------------------
//
// A record is an i32 pointer to its cell: one 8-byte slot per field, in declaration order (`RecordDesc`). The cell is
// shared — `let q = p` names it — as the interpreter's `Rc<RefCell<…>>` is, so a field written through one name is read
// through the other. `==` and `str` of a whole record stay outside the fragment (DL1201); a field's value is any scalar,
// or an `Option`, `Result` or `List` of scalars (`field_ty`) — so a record can name another, or itself: `next: Option[ref
// Node]`.

/// A record literal: its fields computed in the order written, as the interpreter computes them, each into a local; then
/// the cell, each value stored in its field's slot.
fn compile_record_literal(path: &Path, fields: &[(Ident, Expr)], cx: &mut Cx) -> Result<Ty, CompileError> {
    let name = &path.segs.last().expect("a record literal names its type").name;
    let id = cx
        .env
        .record_id(name)
        .ok_or_else(|| CompileError::Unsupported(format!("the record `{name}` (generic, or not a record type)")))?;
    let layout = cx.env.record_fields(id)?.to_vec();
    if fields.len() != layout.len() {
        return Err(CompileError::Unsupported(format!("a `{name}` literal that does not name each field once")));
    }
    let mut held: Vec<Option<u32>> = vec![None; layout.len()];
    for (fname, fexpr) in fields {
        let k = layout
            .iter()
            .position(|(n, _)| *n == fname.name)
            .ok_or_else(|| CompileError::Unsupported(format!("`{name}` has no field `{}`", fname.name)))?;
        let t = layout[k].1;
        compile_expr_as(fexpr, cx, t)?;
        if t != Ty::Unit {
            let l = cx.alloc_local(t)?;
            cx.emit(Instruction::LocalSet(l));
            held[k] = Some(l);
        }
    }
    let cell = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::I32Const(i32::try_from(layout.len() * 8).unwrap_or(i32::MAX)));
    cx.emit(Instruction::Call(cx.alloc_fn));
    cx.emit(Instruction::LocalSet(cell));
    for (k, (_, t)) in layout.iter().enumerate() {
        if let Some(l) = held[k] {
            cx.emit(Instruction::LocalGet(cell));
            cx.emit(Instruction::I32Const(k as i32 * 8));
            cx.emit(Instruction::I32Add);
            cx.emit(Instruction::LocalGet(l));
            store_ty(cx, *t);
        }
    }
    cx.emit(Instruction::LocalGet(cell));
    Ok(Ty::Record(id))
}

/// Field `name` of record `id`: its slot and its type.
fn record_slot(cx: &Cx, id: u32, name: &str) -> Result<(usize, Ty), CompileError> {
    cx.env
        .record_fields(id)?
        .iter()
        .enumerate()
        .find_map(|(k, (n, t))| (n == name).then_some((k, *t)))
        .ok_or_else(|| CompileError::Unsupported(format!("a record with no field `{name}`")))
}

/// `r.name`: the record, then its field's slot read.
fn compile_field(recv: &Expr, name: &str, cx: &mut Cx) -> Result<Ty, CompileError> {
    let Ty::Record(id) = compile_expr(recv, cx)? else {
        return Err(CompileError::Unsupported("a field of a value that is not a record (or `self` outside an actor)".into()));
    };
    let (k, t) = record_slot(cx, id, name)?;
    if t == Ty::Unit {
        cx.emit(Instruction::Drop);
        return Ok(Ty::Unit);
    }
    cx.emit(Instruction::I32Const(k as i32 * 8));
    cx.emit(Instruction::I32Add);
    load_ty(cx, t);
    Ok(t)
}

/// The type of an assignment's place, from the names alone — nothing is emitted — so its value can be computed first.
fn lvalue_ty(lv: &LValue, cx: &Cx) -> Result<Ty, CompileError> {
    match lv {
        LValue::Var(v) => cx
            .lookup(&v.name)
            .map(|(_, t)| t)
            .ok_or_else(|| CompileError::Unsupported(format!("an assignment through `{}`, which is no local here", v.name))),
        LValue::Field(base, f) => match lvalue_ty(base, cx)? {
            Ty::Record(id) => Ok(record_slot(cx, id, &f.name)?.1),
            _ => Err(CompileError::Unsupported("a field of a value that is not a record".into())),
        },
        // A field written through a list element (`ps[i].x = v`): the checker refuses the write (DL1604, an element
        // read through a list is `val`), so the backend holds no path for it.
        LValue::Index(..) => Err(CompileError::Unsupported("a field written through a list element".into())),
    }
}

/// The expression an assignment's place reads (`p`, `p.next.v`), compiled as any read is.
fn lvalue_expr(lv: &LValue) -> Result<Expr, CompileError> {
    Ok(match lv {
        LValue::Var(v) => Expr::Var { path: Path { segs: vec![v.clone()] }, id: NodeId(0), span: v.span },
        LValue::Field(base, f) => {
            Expr::Field { recv: Box::new(lvalue_expr(base)?), name: f.clone(), id: NodeId(0), span: f.span }
        }
        LValue::Index(..) => return Err(CompileError::Unsupported("a field written through a list element".into())),
    })
}

/// `base.name = value`: the value first, then the record it goes into, as the interpreter evaluates them; then the store.
fn compile_field_assign(base: &LValue, name: &str, value: &Expr, cx: &mut Cx) -> Result<Ty, CompileError> {
    let Ty::Record(id) = lvalue_ty(base, cx)? else {
        return Err(CompileError::Unsupported("a field assignment to a value that is not a record".into()));
    };
    let (k, t) = record_slot(cx, id, name)?;
    compile_expr_as(value, cx, t)?;
    if t == Ty::Unit {
        return Ok(Ty::Unit);
    }
    let v = cx.alloc_local(t)?;
    cx.emit(Instruction::LocalSet(v));
    if compile_expr(&lvalue_expr(base)?, cx)? != Ty::Record(id) {
        return Err(CompileError::Unsupported("a field assignment whose record changed type".into()));
    }
    cx.emit(Instruction::I32Const(k as i32 * 8));
    cx.emit(Instruction::I32Add);
    cx.emit(Instruction::LocalGet(v));
    store_ty(cx, t);
    Ok(Ty::Unit)
}

/// Resolve a `self` field to `(field index, lowered type, self-slot local)` in the current actor
/// context, or an honest error if outside an actor member / no such field.
fn self_field(cx: &Cx, name: &str) -> Result<(u32, Ty, u32), CompileError> {
    let ctx = cx.actor.as_ref().ok_or_else(|| CompileError::Unsupported("`self` outside an actor member".into()))?;
    let (fidx, fty) =
        *ctx.fields.get(name).ok_or_else(|| CompileError::Unsupported(format!("this actor has no field `{name}`")))?;
    Ok((fidx, fty, ctx.self_slot))
}

/// Zero-extend an on-stack value to the i64 actor-boundary ABI. `Int`/actor-ref are already i64;
/// the i32-backed logical types (unused in the Int-only subset but kept general) zero-extend.
fn emit_extend_to_i64(cx: &mut Cx, t: Ty) -> Result<(), CompileError> {
    match t {
        Ty::I64 | Ty::ActorRef(_) => Ok(()),
        Ty::F64 => Err(CompileError::Unsupported("a Float value crossing an actor boundary".into())),
        Ty::Unit => Err(CompileError::Unsupported("a Unit value crossing an actor boundary".into())),
        _ => {
            cx.emit(Instruction::I64ExtendI32U);
            Ok(())
        }
    }
}

/// Narrow an i64 actor-boundary value back to its logical wasm type after a `field_get` (no-op for
/// `Int`/actor-ref; i32-backed logical types wrap).
fn emit_wrap_from_i64(cx: &mut Cx, t: Ty) {
    match t {
        Ty::I64 | Ty::ActorRef(_) | Ty::F64 | Ty::Unit => {}
        _ => cx.emit(Instruction::I32WrapI64),
    }
}

/// Bump-allocate an `argc * 8` i64 argument vector in guest memory, storing each arg (extended to
/// the i64 boundary ABI). Returns the local holding the buffer base (0 when there are no args —
/// the host reads nothing). Used by `spawn` and behaviour sends.
fn emit_args_buffer(cx: &mut Cx, args: &[Expr]) -> Result<u32, CompileError> {
    let base = cx.alloc_local(Ty::I32)?;
    if args.is_empty() {
        cx.emit(Instruction::I32Const(0));
        cx.emit(Instruction::LocalSet(base));
        return Ok(base);
    }
    cx.emit(Instruction::I32Const(args.len() as i32 * 8));
    cx.emit(Instruction::Call(cx.alloc_fn));
    cx.emit(Instruction::LocalSet(base));
    for (i, arg) in args.iter().enumerate() {
        cx.emit(Instruction::LocalGet(base));
        cx.emit(Instruction::I32Const(i as i32 * 8));
        cx.emit(Instruction::I32Add);
        let t = compile_expr(arg, cx)?;
        emit_extend_to_i64(cx, t)?;
        cx.emit(Instruction::I64Store(i64_at()));
    }
    Ok(base)
}

/// `spawn A(args)` (Stage 7 phase 7h, T-Spawn): lower to the `actor_spawn` import carrying the
/// ctor args, yielding the new slot id typed `tag A` (`Ty::ActorRef`).
fn compile_spawn(actor: &Path, args: &[Expr], cx: &mut Cx) -> Result<Ty, CompileError> {
    let name = actor.segs.last().expect("spawn path has segments").name.clone();
    let aidx = *cx
        .actors
        .by_name
        .get(&name)
        .ok_or_else(|| CompileError::Unsupported(format!("spawn of `{name}` (no such compilable actor)")))?;
    let arity = cx.actors.actors[aidx as usize].ctor_arity;
    if args.len() != arity {
        return Err(CompileError::Unsupported(format!("spawn {name}() with {} args (its `new` expects {arity})", args.len())));
    }
    if cx.actor_imp.spawn == u32::MAX {
        return Err(CompileError::Unsupported("spawn without the actor host interface".into()));
    }
    let base = emit_args_buffer(cx, args)?;
    cx.emit(Instruction::I32Const(aidx as i32)); // actor_idx
    cx.emit(Instruction::LocalGet(base)); // args_ptr
    cx.emit(Instruction::I32Const(args.len() as i32)); // argc
    cx.emit(Instruction::Call(cx.actor_imp.spawn));
    Ok(Ty::ActorRef(aidx))
}

/// The actor a method's receiver refers to, read from the scopes without compiling it: a local or parameter of
/// actor-reference type, `self` in an actor member, or a `self.field` holding a reference — each compiled to that type by
/// `compile_expr`. `None` is "not known to be an actor": the receiver is then compiled and dispatched by its type.
fn recv_actor(recv: &Expr, cx: &Cx) -> Option<u32> {
    match recv {
        Expr::Var { path, .. } if path.segs.len() == 1 => match cx.lookup(&path.segs[0].name) {
            Some((_, Ty::ActorRef(a))) => Some(a),
            Some(_) => None,
            None if path.segs[0].name == "self" => cx.actor.as_ref().map(|c| c.actor_idx),
            None => None,
        },
        Expr::Field { recv, name, .. } => match &**recv {
            Expr::Var { path, .. } if path.segs.len() == 1 && path.segs[0].name == "self" => match self_field(cx, &name.name) {
                Ok((_, Ty::ActorRef(a), _)) => Some(a),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// A behaviour send `recv.beh(args)` (Stage 7 phase 7h, T-Send). The receiver slot (i64) is
/// already on the wasm stack; stash it, build the args vector, then call `actor_send`. Self-sends
/// (`self.beh(…)`) reach here too — `self` compiled to the self-slot with type `ActorRef(self)`.
fn compile_send(actor_idx: u32, method: &str, args: &[Expr], cx: &mut Cx) -> Result<Ty, CompileError> {
    let beh_idx = cx.actors.behavior_index(actor_idx, method).ok_or_else(|| {
        CompileError::Unsupported(format!("actor `{}` has no behaviour `{method}`", cx.actors.actors[actor_idx as usize].name))
    })?;
    let arity = cx.actors.actors[actor_idx as usize].behaviors[beh_idx as usize].arity;
    if args.len() != arity {
        return Err(CompileError::Unsupported(format!(
            "send `{method}` with {} args (the behaviour expects {arity})",
            args.len()
        )));
    }
    if cx.actor_imp.send == u32::MAX {
        return Err(CompileError::Unsupported("a behaviour send without the actor host interface".into()));
    }
    let slot_local = cx.alloc_local(Ty::I64)?;
    cx.emit(Instruction::LocalSet(slot_local)); // stash the receiver slot
    let base = emit_args_buffer(cx, args)?;
    cx.emit(Instruction::LocalGet(slot_local)); // slot: i64
    cx.emit(Instruction::I32Const(beh_idx as i32)); // behavior_idx: i32
    cx.emit(Instruction::LocalGet(base)); // args_ptr: i32
    cx.emit(Instruction::I32Const(args.len() as i32)); // argc: i32
    cx.emit(Instruction::Call(cx.actor_imp.send));
    Ok(Ty::Unit)
}

fn i64_at() -> MemArg {
    MemArg { offset: 0, align: 3, memory_index: 0 }
}

/// A variant cell is `[tag:i32 @0][field @4][field @12]…` — each field slot is 8 bytes (loaded/stored
/// at the payload's actual width), so a whole enum's cells are one uniform size: `4 + 8 * maxfields`.
fn variant_cell_size(ctors: &[(String, Vec<Scalar>)]) -> i32 {
    let max_fields = ctors.iter().map(|(_, p)| p.len()).max().unwrap_or(0);
    4 + 8 * max_fields as i32
}

fn field_offset(i: usize) -> i32 {
    4 + 8 * i as i32
}

/// Push a `(file, start, end)` span triple for an effect host call (Stage 4 phase 4g) — the host
/// records the effect's `TraceRecord` with it, matching the interpreter's span byte-for-byte.
fn emit_span(cx: &mut Cx, span: delulu_diag::Span) {
    cx.emit(Instruction::I32Const(span.file as i32));
    cx.emit(Instruction::I32Const(span.start as i32));
    cx.emit(Instruction::I32Const(span.end as i32));
}

/// One marshalled foreign-argument cell in the guest args buffer: `[tag:i32 @0][pad @4][payload:8 @8]`
/// (Stage 4 phase 4g). The host reads `argc` of these to reconstruct the `Vec<FVal>` for
/// `foreign::call`. 16 bytes keeps the 8-byte payload slot naturally sized for an i64/f64.
const FVAL_CELL: i32 = 16;

/// The args-buffer tag for a marshalled value of the given foreign parameter `Ty` (must agree with the
/// host's `foreign_call` decoder).
fn fval_tag(t: Ty) -> i32 {
    match t {
        Ty::I64 => 0,        // Int
        Ty::F64 => 1,        // Float
        Ty::I32 => 2,        // Bool
        Ty::Str => 3,        // Str (payload = guest str ptr)
        Ty::ForeignPtr => 4, // ForeignPtr (payload = host foreign-ptr table index)
        _ => 5,              // Unit (no native argument)
    }
}

/// Compile a method call on a bound `foreign` lib handle (Stage 4 phase 4g). The receiver handle is
/// already on the stack. Marshal each argument into a guest args buffer, call `foreign_call` (which
/// runs `foreign::call` host-side — one code path with the interpreter), and decode the marshalled
/// i64 return per the declared return type. A `ForeignErr::BadReturn` becomes a host-recorded DL1306
/// refusal surfaced after the run (never a trap from inside the callback).
fn compile_foreign_call(
    lib_id: u32,
    method: &str,
    args: &[Expr],
    span: delulu_diag::Span,
    cx: &mut Cx,
) -> Result<Ty, CompileError> {
    if cx.imp.foreign_call == u32::MAX {
        return Err(CompileError::Unsupported("a foreign call without the foreign host interface".into()));
    }
    let m = cx
        .env
        .foreign_method(lib_id, method)
        .ok_or_else(|| CompileError::Unsupported(format!("unknown foreign method `{method}`")))?
        .clone();
    if args.len() != m.params.len() {
        return Err(CompileError::Unsupported(format!(
            "foreign method `{method}` called with {} args (expected {})",
            args.len(),
            m.params.len()
        )));
    }
    // Stash the receiver handle in a local so building the args buffer (bump-alloc + stores) does not
    // disturb it.
    let hloc = cx.alloc_local(Ty::Foreign(lib_id))?;
    cx.emit(Instruction::LocalSet(hloc));

    // Bump-allocate an `argc * FVAL_CELL` argument buffer and remember its base.
    let argc = args.len() as i32;
    let base = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::I32Const(argc * FVAL_CELL));
    cx.emit(Instruction::Call(cx.alloc_fn));
    cx.emit(Instruction::LocalSet(base));

    for (i, (arg, pty)) in args.iter().zip(m.params.iter()).enumerate() {
        let off = i as i32 * FVAL_CELL;
        // Tag @ off.
        cx.emit(Instruction::LocalGet(base));
        cx.emit(Instruction::I32Const(off));
        cx.emit(Instruction::I32Add);
        cx.emit(Instruction::I32Const(fval_tag(*pty)));
        cx.emit(Instruction::I32Store(u32_at()));
        if *pty == Ty::Unit {
            // A `Unit` argument marshals to no native argument; still evaluate it for effect.
            compile_expr_as(arg, cx, Ty::Unit)?;
            continue;
        }
        // Payload @ off+8.
        cx.emit(Instruction::LocalGet(base));
        cx.emit(Instruction::I32Const(off + 8));
        cx.emit(Instruction::I32Add);
        compile_expr_as(arg, cx, *pty)?;
        match pty {
            Ty::I64 => cx.emit(Instruction::I64Store(i64_at())),
            Ty::F64 => cx.emit(Instruction::F64Store(i64_at())),
            Ty::I32 | Ty::Str | Ty::ForeignPtr => cx.emit(Instruction::I32Store(u32_at())),
            other => return Err(CompileError::Unsupported(format!("a non-marshallable foreign argument of type {other:?}"))),
        }
    }

    // foreign_call(lib, method_ptr, args_ptr, argc, file, start, end) -> i64.
    let method_ptr = *cx.str_off.get(method).expect("foreign method name interned") as i32;
    cx.emit(Instruction::LocalGet(hloc));
    cx.emit(Instruction::I32Const(method_ptr));
    cx.emit(Instruction::LocalGet(base));
    cx.emit(Instruction::I32Const(argc));
    emit_span(cx, span);
    cx.emit(Instruction::Call(cx.imp.foreign_call));

    // Decode the marshalled i64 return into the declared return type.
    match m.ret {
        Ty::I64 => Ok(Ty::I64),
        Ty::F64 => {
            cx.emit(Instruction::F64ReinterpretI64);
            Ok(Ty::F64)
        }
        Ty::I32 => {
            cx.emit(Instruction::I32WrapI64);
            Ok(Ty::I32)
        }
        Ty::Str => {
            cx.emit(Instruction::I32WrapI64);
            Ok(Ty::Str)
        }
        Ty::ForeignPtr => {
            cx.emit(Instruction::I32WrapI64);
            Ok(Ty::ForeignPtr)
        }
        Ty::Unit => {
            cx.emit(Instruction::Drop);
            Ok(Ty::Unit)
        }
        other => Err(CompileError::Unsupported(format!("a foreign return of type {other:?}"))),
    }
}

/// The name + args of a constructor-shaped expression: `Name(args)` or a bare `Name`.
fn ctor_shape(e: &Expr) -> Option<(&str, &[Expr])> {
    match e {
        Expr::Call { callee, args, .. } => {
            if let Expr::Var { path, .. } = &**callee {
                if path.segs.len() == 1 {
                    return Some((path.segs[0].name.as_str(), args.as_slice()));
                }
            }
            None
        }
        Expr::Var { path, .. } if path.segs.len() == 1 => Some((path.segs[0].name.as_str(), &[])),
        _ => None,
    }
}

/// Expected-type-directed compilation: emit code producing a value of type `expected`. Only the
/// forms that need the expected type (variant construction, and the control-flow that carries it to
/// a tail) are special-cased; everything else synthesises and is checked against `expected`.
fn compile_expr_as(e: &Expr, cx: &mut Cx, expected: Ty) -> Result<(), CompileError> {
    // A constructor of the expected variant type (`Ok`/`Some`/`None` or a user enum's ctor).
    if let Some((name, args)) = ctor_shape(e) {
        if let Some(ctors) = cx.env.ctors_of(expected) {
            if ctors.iter().any(|(n, _)| n == name) {
                return compile_ctor(name, args, cx, expected);
            }
        }
    }
    match e {
        Expr::If { cond, then_, else_, .. } => {
            if compile_expr(cond, cx)? != Ty::I32 {
                return Err(CompileError::Unsupported("a non-Bool `if` condition".into()));
            }
            let else_ =
                else_.as_ref().ok_or_else(|| CompileError::Unsupported("an `if` without `else` used as a value".into()))?;
            let bt = match wasm_valtype(expected) {
                Some(v) => BlockType::Result(v),
                None => BlockType::Empty,
            };
            cx.emit(Instruction::If(bt));
            compile_block_as(then_, cx, expected)?;
            cx.emit(Instruction::Else);
            compile_expr_as(else_, cx, expected)?;
            cx.emit(Instruction::End);
            Ok(())
        }
        Expr::Block(b) => compile_block_as(b, cx, expected),
        Expr::Match { scrutinee, arms, .. } => compile_match(scrutinee, arms, cx, Some(expected)).map(|_| ()),
        Expr::List { items, .. } => match expected {
            Ty::List(s) => compile_list_literal(items, cx, Some(s)).map(|_| ()),
            _ => Err(CompileError::Unsupported(format!("a list where {expected:?} was expected"))),
        },
        _ => {
            let got = compile_expr(e, cx)?;
            if got != expected {
                return Err(CompileError::Unsupported(format!("a value of type {got:?} where {expected:?} was expected")));
            }
            Ok(())
        }
    }
}

/// The type of constructor `name` used where no type is expected (`let c = Red`, `var o = None`): a named sum's when one
/// declares it, else — `None`, `Ok`, `Err` — the type the checker inferred for the node, where the backend holds it.
fn synthesized_ctor_ty(name: &str, node: NodeId, cx: &Cx) -> Option<Ty> {
    if let Some(id) = cx.env.enum_of_ctor(name) {
        return Some(Ty::Enum(id));
    }
    if !matches!(name, "None" | "Ok" | "Err") {
        return None;
    }
    cx.node_types.get(&node).and_then(checked_ty).filter(|t| matches!(t, Ty::Option(_) | Ty::Result(..)))
}

/// A type the checker inferred, as the backend holds it — the scalars, and an `Option`, `Result` or `List` of them; a
/// named type is not resolved here (its id is the checker's table's, which the backend is not given).
fn checked_ty(t: &delulu_check::Type) -> Option<Ty> {
    use delulu_check::Type;
    Some(match t {
        Type::Int => Ty::I64,
        Type::Bool => Ty::I32,
        Type::Float => Ty::F64,
        Type::Str => Ty::Str,
        Type::Unit => Ty::Unit,
        Type::Option(p) => Ty::Option(ty_to_scalar(checked_ty(p)?)?),
        Type::Result(o, e) => Ty::Result(ty_to_scalar(checked_ty(o)?)?, ty_to_scalar(checked_ty(e)?)?),
        Type::List(e) => Ty::List(list_element(checked_ty(e)?)?),
        _ => return None,
    })
}

/// `Some(x)` where no type is expected: `x` first, as the interpreter computes it, then the cell of `Option` of its type.
fn compile_some(arg: &Expr, cx: &mut Cx) -> Result<Ty, CompileError> {
    let t = compile_expr(arg, cx)?;
    let s = ty_to_scalar(t).ok_or_else(|| CompileError::Unsupported(format!("`Some` of a {t:?}")))?;
    let opt = Ty::Option(s);
    let ctors = cx.env.ctors_of(opt).expect("an `Option` has its two constructors");
    let tag = ctors.iter().position(|(n, _)| n == "Some").expect("an `Option` has `Some`");
    let held = if s == Scalar::Unit {
        None
    } else {
        let l = cx.alloc_local(t)?;
        cx.emit(Instruction::LocalSet(l));
        Some(l)
    };
    let rp = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::I32Const(variant_cell_size(&ctors)));
    cx.emit(Instruction::Call(cx.alloc_fn));
    cx.emit(Instruction::LocalSet(rp));
    cx.emit(Instruction::LocalGet(rp));
    cx.emit(Instruction::I32Const(tag as i32));
    cx.emit(Instruction::I32Store(u32_at()));
    if let Some(l) = held {
        cx.emit(Instruction::LocalGet(rp));
        cx.emit(Instruction::I32Const(field_offset(0)));
        cx.emit(Instruction::I32Add);
        cx.emit(Instruction::LocalGet(l));
        store_scalar(cx, s);
    }
    cx.emit(Instruction::LocalGet(rp));
    Ok(opt)
}

/// Compile a variant constructor to a fresh heap `[tag][field…]` cell of type `expected`.
fn compile_ctor(name: &str, args: &[Expr], cx: &mut Cx, expected: Ty) -> Result<(), CompileError> {
    let ctors = cx
        .env
        .ctors_of(expected)
        .ok_or_else(|| CompileError::Unsupported(format!("constructor `{name}` where {expected:?} was expected")))?;
    let tag = ctors
        .iter()
        .position(|(n, _)| n == name)
        .ok_or_else(|| CompileError::Unsupported(format!("`{name}` is not a constructor of {expected:?}")))?;
    let payloads = ctors[tag].1.clone();
    if args.len() != payloads.len() {
        return Err(CompileError::Unsupported(format!(
            "constructor `{name}` with {} args (expected {})",
            args.len(),
            payloads.len()
        )));
    }
    let cell = variant_cell_size(&ctors);

    let rp = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::I32Const(cell));
    cx.emit(Instruction::Call(cx.alloc_fn));
    cx.emit(Instruction::LocalSet(rp));
    cx.emit(Instruction::LocalGet(rp));
    cx.emit(Instruction::I32Const(tag as i32));
    cx.emit(Instruction::I32Store(u32_at()));
    for (i, (arg, ps)) in args.iter().zip(payloads.iter()).enumerate() {
        if *ps == Scalar::Unit {
            compile_expr_as(arg, cx, Ty::Unit)?;
            continue;
        }
        cx.emit(Instruction::LocalGet(rp));
        cx.emit(Instruction::I32Const(field_offset(i)));
        cx.emit(Instruction::I32Add);
        // Compile the field at its expected type, so a nested constructor (e.g. `Err(NotFound)`) works.
        compile_expr_as(arg, cx, ps.to_ty())?;
        store_scalar(cx, *ps);
    }
    cx.emit(Instruction::LocalGet(rp));
    Ok(())
}

/// Compile a `match` on a variant scrutinee (`Result`/`Option`/user enum) to a tag test chain with
/// typed field binding. `expected` is the arms' result type; in synthesis it comes from the 1st arm.
fn compile_match(scrutinee: &Expr, arms: &[Arm], cx: &mut Cx, expected: Option<Ty>) -> Result<Ty, CompileError> {
    let sty = compile_expr(scrutinee, cx)?;
    let ctors = cx
        .env
        .ctors_of(sty)
        .filter(|c| !c.is_empty())
        .ok_or_else(|| CompileError::Unsupported(format!("`match` on the non-variant type {sty:?}")))?;
    let ntags = ctors.len();
    let sp = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::LocalSet(sp));

    // Map each tag to the arm handling it (a variant pattern by name, else a wildcard/bind catch-all).
    let mut arm_for_tag: Vec<Option<usize>> = vec![None; ntags];
    let mut catch_all: Option<usize> = None;
    for (ai, arm) in arms.iter().enumerate() {
        match &arm.pattern {
            Pattern::Variant { path, .. } if path.segs.len() == 1 => {
                if let Some(t) = ctors.iter().position(|(n, _)| *n == path.segs[0].name) {
                    if arm_for_tag[t].is_none() {
                        arm_for_tag[t] = Some(ai);
                    }
                }
            }
            Pattern::Wildcard(_) | Pattern::Bind(_) => {
                if catch_all.is_none() {
                    catch_all = Some(ai);
                }
            }
            _ => return Err(CompileError::Unsupported("a `match` pattern the WASM backend can't compile".into())),
        }
    }
    let arm_idx =
        |t: usize| arm_for_tag[t].or(catch_all).ok_or_else(|| CompileError::Unsupported("a non-exhaustive `match`".into()));

    // The result type is the expected one, or the type the first arm compiles to: each `if` of the chain is given it once
    // it is known (WASM-IFTY-1: it was guessed from the first arm's syntax, a variable guessed `Int`, and an arm binding a
    // `Str` was refused).
    let mut result_ty = expected;
    let mut ifs: Vec<usize> = Vec::new();

    // Load the tag once, then a nested `if tag==t { arm } else { … }` chain (final tag = last else).
    let tv = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::LocalGet(sp));
    cx.emit(Instruction::I32Load(u32_at()));
    cx.emit(Instruction::LocalSet(tv));
    for t in 0..ntags - 1 {
        cx.emit(Instruction::LocalGet(tv));
        cx.emit(Instruction::I32Const(t as i32));
        cx.emit(Instruction::I32Eq);
        ifs.push(cx.instrs.len());
        cx.emit(Instruction::If(BlockType::Empty));
        let got = compile_arm(&arms[arm_idx(t)?], &ctors[t].1, sp, sty, cx, result_ty)?;
        result_ty.get_or_insert(got);
        cx.emit(Instruction::Else);
    }
    let got = compile_arm(&arms[arm_idx(ntags - 1)?], &ctors[ntags - 1].1, sp, sty, cx, result_ty)?;
    let result_ty = result_ty.unwrap_or(got);
    for at in ifs {
        cx.instrs[at] = Instruction::If(block_type(result_ty));
    }
    for _ in 0..ntags - 1 {
        cx.emit(Instruction::End);
    }
    Ok(result_ty)
}

fn compile_arm(
    arm: &Arm,
    payloads: &[Scalar],
    sp: u32,
    scrut_ty: Ty,
    cx: &mut Cx,
    result_ty: Option<Ty>,
) -> Result<Ty, CompileError> {
    cx.scopes.push(HashMap::new());
    match &arm.pattern {
        // `Ctor(f0, f1, …)` — bind each field named by a `Bind` pattern at its slot.
        Pattern::Variant { fields, .. } => {
            for (i, (fp, ps)) in fields.iter().zip(payloads.iter()).enumerate() {
                if let (Pattern::Bind(name), false) = (fp, *ps == Scalar::Unit) {
                    let fty = ps.to_ty();
                    let flocal = cx.alloc_local(fty)?;
                    cx.emit(Instruction::LocalGet(sp));
                    cx.emit(Instruction::I32Const(field_offset(i)));
                    cx.emit(Instruction::I32Add);
                    load_scalar(cx, *ps);
                    cx.emit(Instruction::LocalSet(flocal));
                    cx.scopes.last_mut().unwrap().insert(name.name.clone(), (flocal, fty));
                }
            }
        }
        // A bare binding catches the whole scrutinee (`other => …`).
        Pattern::Bind(name) => {
            cx.scopes.last_mut().unwrap().insert(name.name.clone(), (sp, scrut_ty));
        }
        _ => {}
    }
    // Held to the match's type once it is known; the first arm of a match with none expected says what it is.
    let r = match result_ty {
        Some(t) => compile_expr_as(&arm.body, cx, t).map(|()| t),
        None => compile_expr(&arm.body, cx),
    };
    cx.scopes.pop();
    r
}

fn is_fn_type(t: &TypeExpr) -> bool {
    matches!(t, TypeExpr::Fn { .. })
}

/// A function named as a value (RW 3.4). A generic one is its own body, inlined as a call to it is; any other is a body
/// that calls it, so a recursion or a `return` in it stays the call's own. Either sees what a function's own body sees:
/// its parameters, and nothing of where it is named.
fn named_value(f: &FnDecl) -> Callable {
    let body = if f.generics.is_empty() {
        let (id, span) = (f.body.id, f.name.span);
        let var = |n: &Ident| Expr::Var { path: Path { segs: vec![n.clone()] }, id, span };
        let args = f.params.iter().map(|p| var(&p.name)).collect();
        Block { stmts: vec![Stmt::Expr(Expr::Call { callee: Box::new(var(&f.name)), args, id, span })], id, span }
    } else {
        f.body.clone()
    };
    Callable { params: f.params.clone(), body, hidden: Vec::new(), scopes_at: 0, callables_at: 0 }
}

/// Resolve a function-typed argument to the `Callable` we will inline: a lambda literal, a name already bound to a
/// callable, or a function's own name. (A local is never one: the checker types a function-typed argument, and no local
/// of a function type compiles.)
fn resolve_callable(arg: &Expr, cx: &Cx) -> Result<Callable, CompileError> {
    match arg {
        Expr::Lambda { params, body, .. } => Ok(cx.callable(params, body)),
        Expr::Var { path, .. } if path.segs.len() == 1 => {
            let name = &path.segs[0].name;
            cx.lookup_callable(name)
                .or_else(|| cx.named.get(name).cloned())
                .ok_or_else(|| CompileError::Unsupported(format!("`{}` is not a function value", path.dotted())))
        }
        _ => Err(CompileError::Unsupported("a function argument that is not a lambda or function value".into())),
    }
}

/// Inline (monomorphise) a call to a generic function or a function value: evaluate value arguments
/// into fresh locals in the current scope, bind function-typed arguments as callables, then compile
/// the body in a new frame. This is how the backend handles generics and non-capturing lambdas —
/// `apply(fn(x) { x * 2 }, 21)` reduces to inlined arithmetic, no function table needed (Phase 3r).
fn inline_call(params: &[Param], body: &Block, args: &[Expr], cx: &mut Cx, lexical: Lexical) -> Result<Ty, CompileError> {
    if cx.inline_depth >= 64 {
        return Err(CompileError::Unsupported("inlining too deep (recursive generic or lambda?)".into()));
    }
    if params.len() != args.len() {
        return Err(CompileError::Unsupported("an inlined call with the wrong number of arguments".into()));
    }
    // Evaluate args in the CURRENT scope: value args into fresh locals; function args to callables.
    let mut local_binds: Vec<(String, u32, Ty)> = Vec::new();
    let mut fn_binds: Vec<(String, Callable)> = Vec::new();
    for (p, arg) in params.iter().zip(args) {
        if is_fn_type(&p.ty) {
            fn_binds.push((p.name.name.clone(), resolve_callable(arg, cx)?));
        } else {
            let ty = compile_expr(arg, cx)?;
            let l = cx.alloc_local(ty)?;
            cx.emit(Instruction::LocalSet(l));
            local_binds.push((p.name.name.clone(), l, ty));
        }
    }
    inline_body(body, local_binds, fn_binds, cx, lexical)
}

/// Apply a one-parameter function value to the value held in `local` (`List.map`'s per-element call): the same inlining
/// as a call, with the argument already evaluated.
fn inline_apply(c: &Callable, local: u32, ty: Ty, cx: &mut Cx) -> Result<Ty, CompileError> {
    if cx.inline_depth >= 64 {
        return Err(CompileError::Unsupported("inlining too deep (recursive generic or lambda?)".into()));
    }
    let [p] = c.params.as_slice() else {
        return Err(CompileError::Unsupported("a function value of other than one parameter where one is applied".into()));
    };
    if is_fn_type(&p.ty) {
        return Err(CompileError::Unsupported("a function value applied to a function".into()));
    }
    inline_body(&c.body, vec![(p.name.name.clone(), local, ty)], Vec::new(), cx, Lexical::Value(c))
}

/// Compile an inlined body with its parameters bound (`inline_call`, `inline_apply`).
fn inline_body(
    body: &Block,
    local_binds: Vec<(String, u32, Ty)>,
    fn_binds: Vec<(String, Callable)>,
    cx: &mut Cx,
    lexical: Lexical,
) -> Result<Ty, CompileError> {
    // Enter the inlined body with the parameters bound. It sees the names of where it was written and no others
    // (WASM-HYGIENE-1); its `break`/`continue` reach no loop of the caller's, and its assignments no variable of the
    // caller's (RW 3.4).
    let here = Hidden { scopes: (0, cx.scopes.len()), callables: (0, cx.callables.len()) };
    let env = match lexical {
        Lexical::Generic => vec![here],
        Lexical::Value(c) => {
            let mut env = c.hidden.clone();
            env.push(Hidden { scopes: (c.scopes_at, here.scopes.1), callables: (c.callables_at, here.callables.1) });
            env
        }
    };
    let hidden = std::mem::replace(&mut cx.hidden, env);
    cx.inline_depth += 1;
    let floor = std::mem::replace(&mut cx.inline_floor, cx.scopes.len());
    let loops = std::mem::take(&mut cx.loops);
    cx.scopes.push(local_binds.into_iter().map(|(n, l, t)| (n, (l, t))).collect());
    cx.callables.push(fn_binds.into_iter().collect());
    let r = compile_block(body, cx);
    cx.scopes.pop();
    cx.callables.pop();
    cx.loops = loops;
    cx.inline_floor = floor;
    cx.inline_depth -= 1;
    cx.hidden = hidden;
    r
}

fn compile_expr(e: &Expr, cx: &mut Cx) -> Result<Ty, CompileError> {
    match e {
        Expr::Lit { kind, .. } => match kind {
            LitKind::Int(n) => {
                cx.emit(Instruction::I64Const(*n));
                Ok(Ty::I64)
            }
            LitKind::Bool(b) => {
                cx.emit(Instruction::I32Const(if *b { 1 } else { 0 }));
                Ok(Ty::I32)
            }
            LitKind::Str(s) => {
                // A literal the collector did not reach is refused (DL1201), never a panic (WASM-INTERN-1).
                let ptr = *cx
                    .str_off
                    .get(s)
                    .ok_or_else(|| CompileError::Unsupported("a string literal where the backend collected none".into()))?
                    as i32;
                cx.emit(Instruction::I32Const(ptr));
                Ok(Ty::Str)
            }
            // Stage 4 phase 4g: a `Float` literal is an f64 (used as a `foreign "c"` scalar argument).
            LitKind::Float(x) => {
                cx.emit(Instruction::F64Const(*x));
                Ok(Ty::F64)
            }
        },
        Expr::Var { path, id, .. } => {
            if path.segs.len() == 1 {
                if let Some((idx, ty)) = cx.lookup(&path.segs[0].name) {
                    cx.emit(Instruction::LocalGet(idx));
                    return Ok(ty);
                }
                // Stage 7 phase 7h: `self` in an actor member is the actor's own slot id (`tag`).
                if path.segs[0].name == "self" {
                    if let Some((slot, aidx)) = cx.actor.as_ref().map(|c| (c.self_slot, c.actor_idx)) {
                        cx.emit(Instruction::LocalGet(slot));
                        return Ok(Ty::ActorRef(aidx));
                    }
                }
                // A nullary constructor with no expected type: its sum's, or the checker's (`None`).
                if let Some(t) = synthesized_ctor_ty(&path.segs[0].name, *id, cx) {
                    compile_ctor(&path.segs[0].name, &[], cx, t)?;
                    return Ok(t);
                }
            }
            Err(CompileError::Unsupported(format!("the name `{}` (not a local)", path.dotted())))
        }
        // Stage 7 phase 7h: `self.field` reads the actor's own field host-side. Only self-field
        // reads are in-subset (an actor cannot reach another actor's fields — checker-denied).
        Expr::Field { recv, name, .. } => {
            if let Expr::Var { path, .. } = &**recv {
                if path.segs.len() == 1 && path.segs[0].name == "self" {
                    let (fidx, fty, self_slot) = self_field(cx, &name.name)?;
                    if cx.actor_imp.field_get == u32::MAX {
                        return Err(CompileError::Unsupported("a self-field read without the actor host interface".into()));
                    }
                    cx.emit(Instruction::LocalGet(self_slot));
                    cx.emit(Instruction::I32Const(fidx as i32));
                    cx.emit(Instruction::Call(cx.actor_imp.field_get));
                    emit_wrap_from_i64(cx, fty);
                    return Ok(fty);
                }
            }
            compile_field(recv, &name.name, cx)
        }
        Expr::Record { path, fields, .. } => compile_record_literal(path, fields, cx),
        // Stage 7 phase 7h, T-Spawn.
        Expr::Spawn { actor, args, .. } => compile_spawn(actor, args, cx),
        Expr::Unary { op, operand, .. } => {
            let ty = compile_expr(operand, cx)?;
            match op {
                // `-x` is `x * -1` through the checked multiply, which traps on `Int::MIN` exactly where the
                // interpreter's `checked_neg` faults DL0901 (WASM-NEG-1: a bare `i64.mul` wrapped to `Int::MIN`).
                UnOp::Neg if ty == Ty::I64 => {
                    cx.emit(Instruction::I64Const(-1));
                    cx.emit(Instruction::Call(cx.arith_base + 2));
                    Ok(Ty::I64)
                }
                // A `Float`'s sign flipped, as the interpreter's `-f`.
                UnOp::Neg if ty == Ty::F64 => {
                    cx.emit(Instruction::F64Neg);
                    Ok(Ty::F64)
                }
                UnOp::Not if ty == Ty::I32 => {
                    cx.emit(Instruction::I32Eqz);
                    Ok(Ty::I32)
                }
                _ => Err(CompileError::Unsupported("this unary operator on this type".into())),
            }
        }
        Expr::Binary { op, lhs, rhs, .. } => compile_binary(*op, lhs, rhs, cx),
        // An empty list's element type, when no `let` writes it, is the one the checker inferred (`let out = []`, then
        // `out.push(i)`); a list with items says its own.
        Expr::List { items, id, .. } => {
            let inferred = if items.is_empty() { cx.node_types.get(id).and_then(list_scalar) } else { None };
            compile_list_literal(items, cx, inferred)
        }
        Expr::Index { recv, index, .. } => compile_index(recv, index, cx),
        Expr::If { cond, then_, else_, .. } => {
            let ct = compile_expr(cond, cx)?;
            if ct != Ty::I32 {
                return Err(CompileError::Unsupported("a non-Bool `if` condition".into()));
            }
            let else_ =
                else_.as_ref().ok_or_else(|| CompileError::Unsupported("an `if` without `else` used as a value".into()))?;
            // The `if`'s block type is the type the then-branch compiles to, set once it has compiled (WASM-IFTY-1: it was
            // guessed from the branch's syntax, a variable guessed `Int`, so `if c { a } else { b }` of two `Str`
            // variables built an `if` typed i64 holding an i32 — a module wasmtime refused, at run time, as invalid).
            let at = cx.instrs.len();
            cx.emit(Instruction::If(BlockType::Empty));
            let tt = compile_block(then_, cx)?;
            cx.instrs[at] = Instruction::If(block_type(tt));
            cx.emit(Instruction::Else);
            compile_expr_as(else_, cx, tt).map_err(|e| match e {
                CompileError::Unsupported(m) => CompileError::Unsupported(format!("`if` branches of differing types: {m}")),
                other => other,
            })?;
            cx.emit(Instruction::End);
            Ok(tt)
        }
        Expr::Method { recv, name, args, span, id } => {
            // The receiver decides before the name does: a method on an actor reference is a behaviour send, whatever it
            // is called. The arms below match capability methods by NAME, so until 2026-10-21 a behaviour named
            // `narrow`, `read_text`, `int` or `expose` was refused as that method on the wrong receiver (DL1201).
            if let Some(aidx) = recv_actor(recv, cx) {
                compile_expr(recv, cx)?; // the slot `compile_send` takes from the stack
                return compile_send(aidx, &name.name, args, cx);
            }
            // Phase 3e: Root.console() -> host import minting a Console handle.
            if name.name == "console" && args.is_empty() {
                let rt = compile_expr(recv, cx)?;
                if rt != Ty::Root {
                    return Err(CompileError::Unsupported("console() on a non-Root receiver".into()));
                }
                cx.emit(Instruction::Call(cx.imp.root_console));
                return Ok(Ty::Cap);
            }
            // Phase 3k: Root.clock() -> host import minting a Clock handle.
            if name.name == "clock" && args.is_empty() {
                let rt = compile_expr(recv, cx)?;
                if rt != Ty::Root {
                    return Err(CompileError::Unsupported("clock() on a non-Root receiver".into()));
                }
                cx.emit(Instruction::Call(cx.imp.root_clock));
                return Ok(Ty::Clock);
            }
            // Phase 3b: Cap[Console].println(str) -> host import; Unit result. The `(file,start,end)`
            // span is passed so the host records the Write `TraceRecord` (phase 4g).
            if name.name == "println" && args.len() == 1 {
                let rt = compile_expr(recv, cx)?;
                if rt != Ty::Cap {
                    return Err(CompileError::Unsupported("println on a non-Console receiver".into()));
                }
                let at = compile_expr(&args[0], cx)?;
                if at != Ty::Str {
                    return Err(CompileError::Unsupported("println of a non-Str argument".into()));
                }
                emit_span(cx, *span);
                cx.emit(Instruction::Call(cx.imp.console_println));
                return Ok(Ty::Unit);
            }
            // Phase 3k: Cap[Clock].now_ms() -> host import returning the (fixed or wall) clock as Int.
            if name.name == "now_ms" && args.is_empty() {
                let rt = compile_expr(recv, cx)?;
                if rt != Ty::Clock {
                    return Err(CompileError::Unsupported("now_ms() on a non-Clock receiver".into()));
                }
                emit_span(cx, *span);
                cx.emit(Instruction::Call(cx.imp.clock_now_ms));
                return Ok(Ty::I64);
            }
            // Phase 3l: Root.rand() -> host import minting a Rand handle.
            if name.name == "rand" && args.is_empty() {
                let rt = compile_expr(recv, cx)?;
                if rt != Ty::Root {
                    return Err(CompileError::Unsupported("rand() on a non-Root receiver".into()));
                }
                cx.emit(Instruction::Call(cx.imp.root_rand));
                return Ok(Ty::Rand);
            }
            // Phase 3l: Cap[Rand].int(lo, hi) -> host import returning a seeded random Int in [lo, hi).
            if name.name == "int" && args.len() == 2 {
                let rt = compile_expr(recv, cx)?;
                if rt != Ty::Rand {
                    return Err(CompileError::Unsupported("int(lo, hi) on a non-Rand receiver".into()));
                }
                if compile_expr(&args[0], cx)? != Ty::I64 || compile_expr(&args[1], cx)? != Ty::I64 {
                    return Err(CompileError::Unsupported("rand.int with non-Int bounds".into()));
                }
                emit_span(cx, *span);
                cx.emit(Instruction::Call(cx.imp.rand_int));
                return Ok(Ty::I64);
            }
            // Phase 3p: Root.fs_read(path) -> host import minting a scoped FsRead handle.
            if name.name == "fs_read" && args.len() == 1 {
                if compile_expr(recv, cx)? != Ty::Root {
                    return Err(CompileError::Unsupported("fs_read() on a non-Root receiver".into()));
                }
                if compile_expr(&args[0], cx)? != Ty::Str {
                    return Err(CompileError::Unsupported("fs_read with a non-Str path".into()));
                }
                cx.emit(Instruction::Call(cx.imp.root_fs_read));
                return Ok(Ty::FsRead);
            }
            // Phase 3p: Cap[FsRead].read_text(path) -> host import returning Result[Str, IoErr].
            if name.name == "read_text" && args.len() == 1 {
                if compile_expr(recv, cx)? != Ty::FsRead {
                    return Err(CompileError::Unsupported("read_text() on a non-FsRead receiver".into()));
                }
                if compile_expr(&args[0], cx)? != Ty::Str {
                    return Err(CompileError::Unsupported("read_text with a non-Str path".into()));
                }
                let io_err = cx
                    .env
                    .id("IoErr")
                    .ok_or_else(|| CompileError::Unsupported("read_text without the IoErr prelude type".into()))?;
                emit_span(cx, *span);
                cx.emit(Instruction::Call(cx.imp.fs_read_text));
                return Ok(Ty::Result(Scalar::Str, Scalar::Enum(io_err)));
            }
            // `Cap[FsRead].narrow(sub)` -> a narrower `Cap[FsRead]`, minted host-side under the interpreter's own rule; pure, as
            // there — not traced.
            if name.name == "narrow" && args.len() == 1 {
                if compile_expr(recv, cx)? != Ty::FsRead {
                    return Err(CompileError::Unsupported("narrow() on a receiver that is not a Cap[FsRead]".into()));
                }
                if compile_expr(&args[0], cx)? != Ty::Str {
                    return Err(CompileError::Unsupported("narrow with a non-Str path".into()));
                }
                cx.emit(Instruction::Call(cx.imp.fs_narrow));
                return Ok(Ty::FsRead);
            }
            // `root.fs_write(path)` -> a scoped `Cap[FsWrite]`, minted host-side as the interpreter mints it.
            if name.name == "fs_write" && args.len() == 1 {
                if compile_expr(recv, cx)? != Ty::Root {
                    return Err(CompileError::Unsupported("fs_write() on a non-Root receiver".into()));
                }
                if compile_expr(&args[0], cx)? != Ty::Str {
                    return Err(CompileError::Unsupported("fs_write with a non-Str path".into()));
                }
                cx.emit(Instruction::Call(cx.imp.root_fs_write));
                return Ok(Ty::FsWrite);
            }
            // `Cap[FsWrite].write_text(path, body)` and `append_text` -> `Result[Unit, IoErr]`, written host-side.
            if (name.name == "write_text" || name.name == "append_text") && args.len() == 2 {
                if compile_expr(recv, cx)? != Ty::FsWrite {
                    return Err(CompileError::Unsupported(format!("{}() on a non-FsWrite receiver", name.name)));
                }
                if compile_expr(&args[0], cx)? != Ty::Str || compile_expr(&args[1], cx)? != Ty::Str {
                    return Err(CompileError::Unsupported(format!("{} with a non-Str argument", name.name)));
                }
                let io_err = cx
                    .env
                    .id("IoErr")
                    .ok_or_else(|| CompileError::Unsupported("a write without the IoErr prelude type".into()))?;
                cx.emit(Instruction::I32Const(i32::from(name.name == "append_text")));
                emit_span(cx, *span);
                cx.emit(Instruction::Call(cx.imp.fs_write_text));
                return Ok(Ty::Result(Scalar::Unit, Scalar::Enum(io_err)));
            }
            // `root.http(hosts)` -> a `Cap[Http]`, minted host-side by the interpreter's own rule over the list the host
            // rebuilds from its descriptor (each entry within a granted one, the `net.special=` subset carried).
            if name.name == "http" && args.len() == 1 {
                if cx.imp.root_http == u32::MAX {
                    return Err(CompileError::Unsupported("http() without the http host interface".into()));
                }
                if compile_expr(recv, cx)? != Ty::Root {
                    return Err(CompileError::Unsupported("http() on a non-Root receiver".into()));
                }
                let hosts = compile_expr(&args[0], cx)?;
                if hosts != Ty::List(Scalar::Str) {
                    return Err(CompileError::Unsupported("http() of a host list that is not a List[Str]".into()));
                }
                cx.emit(Instruction::I32Const(desc_ptr(cx, hosts)?));
                cx.emit(Instruction::Call(cx.imp.root_http));
                return Ok(Ty::Http);
            }
            // Stage 4 phase 4g: root.foreign_load() -> Cap[ForeignLoad]. Minted host-side (grant check),
            // pure like the other root derivations — not traced.
            if name.name == "foreign_load" && args.is_empty() {
                if compile_expr(recv, cx)? != Ty::Root {
                    return Err(CompileError::Unsupported("foreign_load() on a non-Root receiver".into()));
                }
                if cx.imp.foreign_load == u32::MAX {
                    return Err(CompileError::Unsupported("foreign_load without the foreign host interface".into()));
                }
                cx.emit(Instruction::Call(cx.imp.foreign_load));
                return Ok(Ty::ForeignLoad);
            }
            // Stage 4 phase 4g: root.foreign(load) -> Result[M, ForeignErr]. The lib `M` is recovered
            // from the checker's bind-site map (keyed by this call's NodeId — the grammar has no method
            // type-argument syntax). The host binds + resolves every symbol (fail-fast) and CONSTRUCTS
            // the `Result[M, ForeignErr]` cell in guest memory. Binding is pure — not traced.
            if name.name == "foreign" && args.len() == 1 {
                if compile_expr(recv, cx)? != Ty::Root {
                    return Err(CompileError::Unsupported("foreign() on a non-Root receiver".into()));
                }
                // `foreign_bind` takes `(load, name_ptr)` — the loader authority is carried by the
                // `Cap[ForeignLoad]`, not the root — so type-check `root` then drop it off the stack.
                cx.emit(Instruction::Drop);
                if compile_expr(&args[0], cx)? != Ty::ForeignLoad {
                    return Err(CompileError::Unsupported("foreign() with a non-Cap[ForeignLoad] argument".into()));
                }
                if cx.imp.foreign_bind == u32::MAX {
                    return Err(CompileError::Unsupported("foreign() without the foreign host interface".into()));
                }
                let lib_name = cx
                    .foreign_binds
                    .get(id)
                    .ok_or_else(|| CompileError::Unsupported("a foreign bind site with no checker-resolved lib".into()))?;
                let lib_id = cx
                    .env
                    .foreign_id(lib_name)
                    .ok_or_else(|| CompileError::Unsupported("root.foreign() bound a lib with no `foreign` block".into()))?;
                let foreign_err = cx
                    .env
                    .id("ForeignErr")
                    .ok_or_else(|| CompileError::Unsupported("root.foreign() without the ForeignErr prelude type".into()))?;
                let name_ptr = *cx.str_off.get(lib_name.as_str()).expect("foreign lib name interned") as i32;
                cx.emit(Instruction::I32Const(name_ptr)); // stack: [load_handle, name_ptr]
                cx.emit(Instruction::Call(cx.imp.foreign_bind));
                return Ok(Ty::Result(Scalar::Foreign(lib_id), Scalar::Enum(foreign_err)));
            }
            // Phase 3n (secrets stay host-side, §4.4/DL1205): minting a secret (`root.secret(...)`) or
            // declassifying one (`secret.expose(...)`) would put secret bytes in guest linear memory.
            // Refuse so the program runs on the interpreter — where secrets never cross into a guest.
            if name.name == "secret" {
                return Err(CompileError::SecretInGuest("`root.secret(...)` mints a secret in the guest".into()));
            }
            if name.name == "expose" {
                return Err(CompileError::SecretInGuest("`Secret.expose(...)` reveals secret bytes to the guest".into()));
            }
            // Stage 4 phase 4g: a method on a bound `foreign` lib handle marshals + calls host-side
            // (the ForeignCall is traced). This is last so the built-in/secret dispatch above wins;
            // the receiver TYPE (`Ty::Foreign(id)`) selects it.
            let recv_ty = compile_expr(recv, cx)?;
            if let Ty::Foreign(lib_id) = recv_ty {
                return compile_foreign_call(lib_id, &name.name, args, *span, cx);
            }
            // Stage 7 phase 7h, T-Send: a method on an actor reference is a behaviour send (the
            // receiver slot is already on the stack). Self-sends reach here too (recv = `self`).
            if let Ty::ActorRef(aidx) = recv_ty {
                return compile_send(aidx, &name.name, args, cx);
            }
            // `Cap[Http].get(url)` -> `Result[Str, NetErr]`, performed host-side by the interpreter's own `get`.
            if recv_ty == Ty::Http && name.name == "get" && args.len() == 1 {
                if cx.imp.http_get == u32::MAX {
                    return Err(CompileError::Unsupported("`get` on a Cap[Http] in a module that mints none".into()));
                }
                if compile_expr(&args[0], cx)? != Ty::Str {
                    return Err(CompileError::Unsupported("http get of a URL that is not a Str".into()));
                }
                let net_err = cx
                    .env
                    .id("NetErr")
                    .ok_or_else(|| CompileError::Unsupported("http get without the NetErr prelude type".into()))?;
                emit_span(cx, *span);
                cx.emit(Instruction::Call(cx.imp.http_get));
                return Ok(Ty::Result(Scalar::Str, Scalar::Enum(net_err)));
            }
            // RW 3.4: `Str.len()`, counted as the interpreter counts it (`__str_len`).
            if recv_ty == Ty::Str && name.name == "len" && args.is_empty() {
                cx.emit(Instruction::Call(cx.str_len_fn));
                return Ok(Ty::I64);
            }
            // RW 3.4: `Str.split(sep)`, split host-side by the interpreter's own rule (an empty separator splits into
            // characters) into a `List[Str]` in the guest's memory.
            if recv_ty == Ty::Str && name.name == "split" && args.len() == 1 {
                if cx.imp.str_split == u32::MAX {
                    return Err(CompileError::Unsupported("`split` without the list interface".into()));
                }
                if compile_expr(&args[0], cx)? != Ty::Str {
                    return Err(CompileError::Unsupported("`split` by a separator that is not a Str".into()));
                }
                cx.emit(Instruction::Call(cx.imp.str_split));
                return Ok(Ty::List(Scalar::Str));
            }
            if let (Ty::Str, Some((id, params, ret))) = (recv_ty, str_method_sig(&name.name)) {
                if params.len() == args.len() {
                    return compile_str_method(id, params, ret, args, cx);
                }
            }
            if let Ty::List(s) = recv_ty {
                return compile_list_method(s, &name.name, args, cx);
            }
            Err(CompileError::Unsupported(format!("the method `.{}`", name.name)))
        }
        Expr::Call { callee, args, .. } => {
            let name = match &**callee {
                Expr::Var { path, .. } if path.segs.len() == 1 => path.segs[0].name.clone(),
                _ => return Err(CompileError::Unsupported("an indirect or builtin call".into())),
            };
            // Phase 3j: `str(x)` — Int formats via `__int_to_str`; a Str is already a string (identity).
            // RW 3.4: `parse_int(s)`, parsed host-side by the interpreter's own rule into `Some(n)` or `None`.
            if name == "parse_int" && args.len() == 1 {
                return compile_parse_int(&args[0], cx);
            }
            if name == "parse_float" && args.len() == 1 {
                return compile_parse_float(&args[0], cx);
            }
            // `float(x)` and `int(x)` as the interpreter's `as` casts: an `Int` rounded to the nearest `Float`
            // (`f64.convert_i64_s`), and a `Float` truncated toward zero and saturated at the `Int` range, a NaN to 0
            // (`i64.trunc_sat_f64_s` — exactly Rust's `as i64`).
            if (name == "float" || name == "int") && args.len() == 1 {
                let at = compile_expr(&args[0], cx)?;
                return match (name.as_str(), at) {
                    ("float", Ty::I64) => {
                        cx.emit(Instruction::F64ConvertI64S);
                        Ok(Ty::F64)
                    }
                    ("int", Ty::F64) => {
                        cx.emit(Instruction::I64TruncSatF64S);
                        Ok(Ty::I64)
                    }
                    ("float", Ty::F64) => Ok(Ty::F64),
                    ("int", Ty::I64) => Ok(Ty::I64),
                    _ => Err(CompileError::Unsupported(format!("`{name}` of {at:?}"))),
                };
            }
            if name == "str" && args.len() == 1 {
                let at = compile_expr(&args[0], cx)?;
                return match at {
                    Ty::I64 => {
                        cx.emit(Instruction::Call(cx.int_to_str_fn));
                        Ok(Ty::Str)
                    }
                    Ty::Str => Ok(Ty::Str), // str(<Str>) is the identity; the pointer is already on the stack
                    // Stage 4 phase 4g: `str(Float)` formats host-side (`float_to_str`) with the EXACT
                    // `Value::Float` display logic, so both engines print a foreign Float identically.
                    Ty::F64 if cx.imp.float_to_str != u32::MAX => {
                        cx.emit(Instruction::Call(cx.imp.float_to_str));
                        Ok(Ty::Str)
                    }
                    // Stage 4 phase 4g: `str(Bool)` selects the interned "true"/"false" literal (matches
                    // `Value::Bool.display()`). Every module interns them (RW 3.4).
                    Ty::I32 => {
                        let t = *cx
                            .str_off
                            .get("true")
                            .ok_or_else(|| CompileError::Unsupported("str(Bool) without its interned literals".into()))?
                            as i32;
                        let f = *cx
                            .str_off
                            .get("false")
                            .ok_or_else(|| CompileError::Unsupported("str(Bool) without its interned literals".into()))?
                            as i32;
                        cx.emit(Instruction::If(BlockType::Result(ValType::I32)));
                        cx.emit(Instruction::I32Const(t));
                        cx.emit(Instruction::Else);
                        cx.emit(Instruction::I32Const(f));
                        cx.emit(Instruction::End);
                        Ok(Ty::Str)
                    }
                    // A list, an option, a result, a sum or a record: rendered host-side by the interpreter's own
                    // `display`, over the value rebuilt by its type's descriptor.
                    t @ (Ty::List(_) | Ty::Option(_) | Ty::Result(..) | Ty::Enum(_) | Ty::Record(_))
                        if cx.imp.value_display != u32::MAX =>
                    {
                        cx.emit(Instruction::I32Const(desc_ptr(cx, t)?));
                        cx.emit(Instruction::Call(cx.imp.value_display));
                        Ok(Ty::Str)
                    }
                    _ => Err(CompileError::Unsupported(
                        "str() of this type (only Int/Str, and Float/Bool with the foreign interface, compile so far)".into(),
                    )),
                };
            }
            // Phase 3r: a call to a function VALUE (a bound lambda) or a generic function → inline it.
            if let Some(c) = cx.lookup_callable(&name) {
                return inline_call(&c.params, &c.body, args, cx, Lexical::Value(&c));
            }
            if let Some(g) = cx.generics.get(&name).cloned() {
                return inline_call(&g.params, &g.body, args, cx, Lexical::Generic);
            }
            // A constructor with no expected type: `Some(x)` takes its payload's type; another, its sum's or the checker's.
            if !cx.index.contains_key(&name) {
                if let ("Some", [arg]) = (name.as_str(), &args[..]) {
                    return compile_some(arg, cx);
                }
                if let Some(t) = synthesized_ctor_ty(&name, e.id(), cx) {
                    compile_ctor(&name, args, cx, t)?;
                    return Ok(t);
                }
            }
            let (fidx, ptys, ret) = {
                let e = cx
                    .index
                    .get(&name)
                    .ok_or_else(|| CompileError::Unsupported(format!("a call to `{name}` (not a compilable function)")))?;
                (e.0, e.1.clone(), e.2)
            };
            if args.len() != ptys.len() {
                return Err(CompileError::Unsupported(format!(
                    "`{name}` called with {} args (expected {})",
                    args.len(),
                    ptys.len()
                )));
            }
            // Compile each argument at its declared parameter type, so a variant constructor passed
            // directly (`f(Some(x))`, `f(Say("hi"))`) knows its type.
            for (a, pty) in args.iter().zip(ptys.iter()) {
                compile_expr_as(a, cx, *pty)?;
            }
            cx.emit(Instruction::Call(fidx));
            Ok(ret)
        }
        Expr::Match { scrutinee, arms, .. } => compile_match(scrutinee, arms, cx, None),
        // Phase 3o Checkpoint 2: `expr?` — if `expr` is Err, return it (same cell layout) as the
        // function's Err; otherwise the `?` expression is the unwrapped Ok payload.
        Expr::Try { inner, .. } => {
            let (t, e) = match compile_expr(inner, cx)? {
                Ty::Result(t, e) => (t, e),
                other => return Err(CompileError::Unsupported(format!("`?` on the non-Result type {other:?}"))),
            };
            // The enclosing function must return `Result[_, e]` (checker guarantees this via DL0409).
            match cx.ret {
                Ty::Result(_, re) if re == e => {}
                _ => return Err(CompileError::Unsupported("`?` outside a matching Result-returning function".into())),
            }
            let sp = cx.alloc_local(Ty::I32)?;
            cx.emit(Instruction::LocalSet(sp));
            // if Ok (tag == 0): fall through; else return the scrutinee as this function's Err.
            cx.emit(Instruction::LocalGet(sp));
            cx.emit(Instruction::I32Load(u32_at()));
            cx.emit(Instruction::I32Eqz);
            cx.emit(Instruction::If(BlockType::Empty));
            cx.emit(Instruction::Else);
            cx.emit(Instruction::LocalGet(sp));
            cx.emit(Instruction::Return);
            cx.emit(Instruction::End);
            // Ok path: the value of `expr?` is the Ok payload at `sp + 4`.
            if t == Scalar::Unit {
                Ok(Ty::Unit)
            } else {
                cx.emit(Instruction::LocalGet(sp));
                cx.emit(Instruction::I32Const(4));
                cx.emit(Instruction::I32Add);
                load_scalar(cx, t);
                Ok(t.to_ty())
            }
        }
        Expr::Block(b) => compile_block(b, cx),
        _ => Err(CompileError::Unsupported("this expression form".into())),
    }
}

/// `parse_int(s)` (RW 3.4): the host parses by the interpreter's own rule (`call_builtin`) and answers whether it parsed
/// and the number; the guest builds the `Option[Int]` from the two, as `List.get` builds its own.
fn compile_parse_int(arg: &Expr, cx: &mut Cx) -> Result<Ty, CompileError> {
    compile_parse(arg, cx, cx.imp.parse_int, Scalar::I64, "parse_int")
}

/// `parse_float(s)`, as `parse_int` — the interpreter's rule refuses the text a `Float` literal may not be (`inf`, a
/// magnitude past the range), so neither engine reads a non-finite number from text.
fn compile_parse_float(arg: &Expr, cx: &mut Cx) -> Result<Ty, CompileError> {
    compile_parse(arg, cx, cx.imp.parse_float, Scalar::F64, "parse_float")
}

/// Call the host's `import` on the `Str` `arg` — answering whether it parsed and the number, a `Scalar` `s` — and build
/// the `Option` from the two.
fn compile_parse(arg: &Expr, cx: &mut Cx, import: u32, s: Scalar, name: &str) -> Result<Ty, CompileError> {
    if import == u32::MAX {
        return Err(CompileError::Unsupported(format!("`{name}` in a module compiled without the string interface")));
    }
    compile_expr_as(arg, cx, Ty::Str)?;
    cx.emit(Instruction::Call(import));
    let (parsed, value) = (cx.alloc_local(Ty::I32)?, cx.alloc_local(s.to_ty())?);
    cx.emit(Instruction::LocalSet(value));
    cx.emit(Instruction::LocalSet(parsed));
    let opt = Ty::Option(s);
    let ctors = cx.env.ctors_of(opt).expect("Option has constructors");
    let tag = |n: &str| ctors.iter().position(|(c, _)| c == n).expect("Option's constructor") as i32;
    let (some, none) = (tag("Some"), tag("None"));
    let cell = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::I32Const(variant_cell_size(&ctors)));
    cx.emit(Instruction::Call(cx.alloc_fn));
    cx.emit(Instruction::LocalSet(cell));
    cx.emit(Instruction::LocalGet(parsed));
    cx.emit(Instruction::If(BlockType::Empty));
    cx.emit(Instruction::LocalGet(cell));
    cx.emit(Instruction::I32Const(some));
    cx.emit(Instruction::I32Store(u32_at()));
    cx.emit(Instruction::LocalGet(cell));
    cx.emit(Instruction::I32Const(field_offset(0)));
    cx.emit(Instruction::I32Add);
    cx.emit(Instruction::LocalGet(value));
    store_scalar(cx, s);
    cx.emit(Instruction::Else);
    cx.emit(Instruction::LocalGet(cell));
    cx.emit(Instruction::I32Const(none));
    cx.emit(Instruction::I32Store(u32_at()));
    cx.emit(Instruction::End);
    cx.emit(Instruction::LocalGet(cell));
    Ok(opt)
}

/// A `Str` method run host-side (`delulu:str.str_method`), the receiver on the stack: its arguments are computed in order,
/// then passed in the import's slots — `Str`s in `a` and `b`, `Int`s in `lo` and `hi`, the unused ones zero.
fn compile_str_method(id: u32, params: &[Ty], ret: Ty, args: &[Expr], cx: &mut Cx) -> Result<Ty, CompileError> {
    if cx.imp.str_method == u32::MAX {
        return Err(CompileError::Unsupported("a `Str` method in a module compiled without the string interface".into()));
    }
    let recv = cx.alloc_local(Ty::Str)?;
    cx.emit(Instruction::LocalSet(recv));
    let mut strs = Vec::new();
    let mut ints = Vec::new();
    for (arg, want) in args.iter().zip(params) {
        compile_expr_as(arg, cx, *want)?;
        let l = cx.alloc_local(*want)?;
        cx.emit(Instruction::LocalSet(l));
        if *want == Ty::Str {
            strs.push(l);
        } else {
            ints.push(l);
        }
    }
    cx.emit(Instruction::LocalGet(recv));
    cx.emit(Instruction::I32Const(id as i32));
    for k in 0..2 {
        match strs.get(k) {
            Some(l) => cx.emit(Instruction::LocalGet(*l)),
            None => cx.emit(Instruction::I32Const(0)),
        }
    }
    for k in 0..2 {
        match ints.get(k) {
            Some(l) => cx.emit(Instruction::LocalGet(*l)),
            None => cx.emit(Instruction::I64Const(0)),
        }
    }
    cx.emit(Instruction::Call(cx.imp.str_method));
    Ok(ret)
}

// ----- RW 3.4: the WASM `List` -------------------------------------------------------------------------------------
//
// A list is a pointer to its header `[len][cap][data]` (`Ty::List`); its elements sit in 8-byte slots at `data`, written
// and read with `store_scalar`/`load_scalar` as a variant's fields are. `__list_new(len)` makes one with `len` slots in
// use; `__list_slot(list)` makes room for one more (doubling the slots into a fresh allocation when full — the bump heap
// never frees, so the old slots stay as they were) and returns the new slot's address. Every operation is held to the
// interpreter's: `get` answers `None` outside the list, an index outside it faults DL0903 in the interpreter's words, and
// `for` walks the list as it was when the loop began.

fn list_mem(offset: u64) -> MemArg {
    MemArg { offset, align: 2, memory_index: 0 }
}

/// Push the address of slot `index` (an i32 local) of the slots at `data` (an i32 local).
fn emit_slot_addr(cx: &mut Cx, data: u32, index: u32) {
    cx.emit(Instruction::LocalGet(data));
    cx.emit(Instruction::LocalGet(index));
    cx.emit(Instruction::I32Const(8));
    cx.emit(Instruction::I32Mul);
    cx.emit(Instruction::I32Add);
}

fn list_helpers(cx: &Cx) -> Result<(), CompileError> {
    if cx.imp.list_new == u32::MAX || cx.imp.list_slot == u32::MAX {
        return Err(CompileError::Unsupported("a list in a module compiled without the list helpers".into()));
    }
    Ok(())
}

/// `[a, b, …]`: a list of the items, each at the element type (the first item's, or `want`, the annotation's).
fn compile_list_literal(items: &[Expr], cx: &mut Cx, want: Option<Scalar>) -> Result<Ty, CompileError> {
    list_helpers(cx)?;
    let list = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::I32Const(items.len() as i32));
    cx.emit(Instruction::Call(cx.imp.list_new));
    cx.emit(Instruction::LocalSet(list));
    let mut elem = want;
    for (k, item) in items.iter().enumerate() {
        cx.emit(Instruction::LocalGet(list));
        cx.emit(Instruction::I32Load(list_mem(LIST_DATA)));
        cx.emit(Instruction::I32Const(k as i32 * 8));
        cx.emit(Instruction::I32Add);
        let s = match elem {
            Some(s) => {
                compile_expr_as(item, cx, s.to_ty())?;
                s
            }
            None => {
                let t = compile_expr(item, cx)?;
                list_element(t).ok_or_else(|| CompileError::Unsupported(format!("a list of {t:?}")))?
            }
        };
        store_scalar(cx, s);
        elem = Some(s);
    }
    let s = elem.ok_or_else(|| CompileError::Unsupported("an empty list whose element type the backend was not given".into()))?;
    cx.emit(Instruction::LocalGet(list));
    Ok(Ty::List(s))
}

/// The element type of a `List` the checker inferred, where the backend holds it: an `Int`, `Bool`, `Float` or `Str`.
fn list_scalar(t: &delulu_check::Type) -> Option<Scalar> {
    use delulu_check::Type;
    match t {
        Type::List(e) => match **e {
            Type::Int => Some(Scalar::I64),
            Type::Bool => Some(Scalar::I32),
            Type::Float => Some(Scalar::F64),
            Type::Str => Some(Scalar::Str),
            _ => None,
        },
        _ => None,
    }
}

/// `xs[i]`: the element, or DL0903 in the interpreter's words when `i` is outside the list.
fn compile_index(recv: &Expr, index: &Expr, cx: &mut Cx) -> Result<Ty, CompileError> {
    let Ty::List(s) = compile_expr(recv, cx)? else {
        return Err(CompileError::Unsupported("indexing a value that is not a List".into()));
    };
    if cx.imp.index_fault == u32::MAX {
        return Err(CompileError::Unsupported("an index without the list interface".into()));
    }
    let list = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::LocalSet(list));
    if compile_expr(index, cx)? != Ty::I64 {
        return Err(CompileError::Unsupported("an index that is not an Int".into()));
    }
    let ix = cx.alloc_local(Ty::I64)?;
    cx.emit(Instruction::LocalSet(ix));
    // Outside the list: `i < 0 || i >= len`.
    cx.emit(Instruction::LocalGet(ix));
    cx.emit(Instruction::I64Const(0));
    cx.emit(Instruction::I64LtS);
    cx.emit(Instruction::LocalGet(ix));
    cx.emit(Instruction::LocalGet(list));
    cx.emit(Instruction::I32Load(list_mem(LIST_LEN)));
    cx.emit(Instruction::I64ExtendI32U);
    cx.emit(Instruction::I64GeS);
    cx.emit(Instruction::I32Or);
    cx.emit(Instruction::If(BlockType::Empty));
    cx.emit(Instruction::LocalGet(ix));
    cx.emit(Instruction::LocalGet(list));
    cx.emit(Instruction::I32Load(list_mem(LIST_LEN)));
    cx.emit(Instruction::Call(cx.imp.index_fault));
    cx.emit(Instruction::Unreachable);
    cx.emit(Instruction::End);
    let data = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::LocalGet(list));
    cx.emit(Instruction::I32Load(list_mem(LIST_DATA)));
    cx.emit(Instruction::LocalSet(data));
    let i32ix = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::LocalGet(ix));
    cx.emit(Instruction::I32WrapI64);
    cx.emit(Instruction::LocalSet(i32ix));
    emit_slot_addr(cx, data, i32ix);
    load_scalar(cx, s);
    Ok(s.to_ty())
}

/// A method of a `List[s]` whose header pointer is on the stack.
fn compile_list_method(s: Scalar, method: &str, args: &[Expr], cx: &mut Cx) -> Result<Ty, CompileError> {
    match (method, args) {
        ("len", []) => {
            cx.emit(Instruction::I32Load(list_mem(LIST_LEN)));
            cx.emit(Instruction::I64ExtendI32U);
            Ok(Ty::I64)
        }
        ("is_empty", []) => {
            cx.emit(Instruction::I32Load(list_mem(LIST_LEN)));
            cx.emit(Instruction::I32Eqz);
            Ok(Ty::I32)
        }
        // `Some(xs[i])`, or `None` for an index outside the list — a negative one included, as the interpreter's
        // `usize` conversion makes it.
        ("get", [index]) => {
            let list = cx.alloc_local(Ty::I32)?;
            cx.emit(Instruction::LocalSet(list));
            if compile_expr(index, cx)? != Ty::I64 {
                return Err(CompileError::Unsupported("`get` with an index that is not an Int".into()));
            }
            let ix = cx.alloc_local(Ty::I64)?;
            cx.emit(Instruction::LocalSet(ix));
            let opt = Ty::Option(s);
            let ctors = cx.env.ctors_of(opt).expect("Option has constructors");
            let tag = |n: &str| ctors.iter().position(|(c, _)| c == n).expect("Option's constructor") as i32;
            let (some, none) = (tag("Some"), tag("None"));
            let cell = cx.alloc_local(Ty::I32)?;
            cx.emit(Instruction::I32Const(variant_cell_size(&ctors)));
            cx.emit(Instruction::Call(cx.alloc_fn));
            cx.emit(Instruction::LocalSet(cell));
            // Inside: `0 <= i && i < len`.
            cx.emit(Instruction::LocalGet(ix));
            cx.emit(Instruction::I64Const(0));
            cx.emit(Instruction::I64GeS);
            cx.emit(Instruction::LocalGet(ix));
            cx.emit(Instruction::LocalGet(list));
            cx.emit(Instruction::I32Load(list_mem(LIST_LEN)));
            cx.emit(Instruction::I64ExtendI32U);
            cx.emit(Instruction::I64LtS);
            cx.emit(Instruction::I32And);
            cx.emit(Instruction::If(BlockType::Empty));
            cx.emit(Instruction::LocalGet(cell));
            cx.emit(Instruction::I32Const(some));
            cx.emit(Instruction::I32Store(u32_at()));
            cx.emit(Instruction::LocalGet(cell));
            cx.emit(Instruction::I32Const(field_offset(0)));
            cx.emit(Instruction::I32Add);
            let data = cx.alloc_local(Ty::I32)?;
            cx.emit(Instruction::LocalGet(list));
            cx.emit(Instruction::I32Load(list_mem(LIST_DATA)));
            cx.emit(Instruction::LocalSet(data));
            let i32ix = cx.alloc_local(Ty::I32)?;
            cx.emit(Instruction::LocalGet(ix));
            cx.emit(Instruction::I32WrapI64);
            cx.emit(Instruction::LocalSet(i32ix));
            emit_slot_addr(cx, data, i32ix);
            load_scalar(cx, s);
            store_scalar(cx, s);
            cx.emit(Instruction::Else);
            cx.emit(Instruction::LocalGet(cell));
            cx.emit(Instruction::I32Const(none));
            cx.emit(Instruction::I32Store(u32_at()));
            cx.emit(Instruction::End);
            cx.emit(Instruction::LocalGet(cell));
            Ok(opt)
        }
        // The value is computed before the list grows, as the interpreter computes it before pushing: `xs.push(xs.len())`
        // pushes the old length.
        ("push", [value]) => {
            list_helpers(cx)?;
            let list = cx.alloc_local(Ty::I32)?;
            cx.emit(Instruction::LocalSet(list));
            compile_expr_as(value, cx, s.to_ty())?;
            let v = cx.alloc_local(s.to_ty())?;
            cx.emit(Instruction::LocalSet(v));
            cx.emit(Instruction::LocalGet(list));
            cx.emit(Instruction::Call(cx.imp.list_slot));
            cx.emit(Instruction::LocalGet(v));
            store_scalar(cx, s);
            Ok(Ty::Unit)
        }
        // A new list of `f(x)` for each element, the function value inlined per element.
        ("map", [f]) => {
            list_helpers(cx)?;
            let c = resolve_callable(f, cx)?;
            let list = cx.alloc_local(Ty::I32)?;
            cx.emit(Instruction::LocalSet(list));
            let (n, data, out, out_data, i) = (
                cx.alloc_local(Ty::I32)?,
                cx.alloc_local(Ty::I32)?,
                cx.alloc_local(Ty::I32)?,
                cx.alloc_local(Ty::I32)?,
                cx.alloc_local(Ty::I32)?,
            );
            cx.emit(Instruction::LocalGet(list));
            cx.emit(Instruction::I32Load(list_mem(LIST_LEN)));
            cx.emit(Instruction::LocalSet(n));
            cx.emit(Instruction::LocalGet(list));
            cx.emit(Instruction::I32Load(list_mem(LIST_DATA)));
            cx.emit(Instruction::LocalSet(data));
            cx.emit(Instruction::LocalGet(n));
            cx.emit(Instruction::Call(cx.imp.list_new));
            cx.emit(Instruction::LocalTee(out));
            cx.emit(Instruction::I32Load(list_mem(LIST_DATA)));
            cx.emit(Instruction::LocalSet(out_data));
            cx.emit(Instruction::I32Const(0));
            cx.emit(Instruction::LocalSet(i));
            let x = cx.alloc_local(s.to_ty())?;
            cx.emit(Instruction::Block(BlockType::Empty));
            let brk = cx.depth;
            cx.emit(Instruction::Loop(BlockType::Empty));
            let top = cx.depth;
            cx.emit(Instruction::LocalGet(i));
            cx.emit(Instruction::LocalGet(n));
            cx.emit(Instruction::I32GeU);
            cx.emit(Instruction::BrIf(cx.depth - brk));
            emit_slot_addr(cx, data, i);
            load_scalar(cx, s);
            cx.emit(Instruction::LocalSet(x));
            let t = inline_apply(&c, x, s.to_ty(), cx)?;
            let es = list_element(t).ok_or_else(|| CompileError::Unsupported(format!("`map` to a list of {t:?}")))?;
            let y = cx.alloc_local(t)?;
            cx.emit(Instruction::LocalSet(y));
            emit_slot_addr(cx, out_data, i);
            cx.emit(Instruction::LocalGet(y));
            store_scalar(cx, es);
            cx.emit(Instruction::LocalGet(i));
            cx.emit(Instruction::I32Const(1));
            cx.emit(Instruction::I32Add);
            cx.emit(Instruction::LocalSet(i));
            cx.emit(Instruction::Br(cx.depth - top));
            cx.emit(Instruction::End);
            cx.emit(Instruction::End);
            cx.emit(Instruction::LocalGet(out));
            Ok(Ty::List(es))
        }
        _ => Err(CompileError::Unsupported(format!("the List method `.{method}`"))),
    }
}

/// `for x in xs { body }` over the list as it is when the loop begins — its length and its slots are read once, so a
/// `push` in the body neither extends the walk nor moves it (the interpreter iterates a snapshot). `break` leaves the
/// walk; `continue` goes on to the next element.
fn compile_for(var: &Ident, iter: &Expr, body: &Block, cx: &mut Cx) -> Result<Ty, CompileError> {
    let Ty::List(s) = compile_expr(iter, cx)? else {
        return Err(CompileError::Unsupported("`for` over a value that is not a List".into()));
    };
    let (list, n, data, i) =
        (cx.alloc_local(Ty::I32)?, cx.alloc_local(Ty::I32)?, cx.alloc_local(Ty::I32)?, cx.alloc_local(Ty::I32)?);
    cx.emit(Instruction::LocalSet(list));
    cx.emit(Instruction::LocalGet(list));
    cx.emit(Instruction::I32Load(list_mem(LIST_LEN)));
    cx.emit(Instruction::LocalSet(n));
    cx.emit(Instruction::LocalGet(list));
    cx.emit(Instruction::I32Load(list_mem(LIST_DATA)));
    cx.emit(Instruction::LocalSet(data));
    cx.emit(Instruction::I32Const(0));
    cx.emit(Instruction::LocalSet(i));
    let x = cx.alloc_local(s.to_ty())?;
    cx.emit(Instruction::Block(BlockType::Empty));
    let brk = cx.depth;
    cx.emit(Instruction::Loop(BlockType::Empty));
    let top = cx.depth;
    cx.emit(Instruction::LocalGet(i));
    cx.emit(Instruction::LocalGet(n));
    cx.emit(Instruction::I32GeU);
    cx.emit(Instruction::BrIf(cx.depth - brk));
    emit_slot_addr(cx, data, i);
    load_scalar(cx, s);
    cx.emit(Instruction::LocalSet(x));
    // `continue` leaves this block, to the step to the next element.
    cx.emit(Instruction::Block(BlockType::Empty));
    let cont = cx.depth;
    cx.loops.push((brk, cont));
    cx.scopes.push(HashMap::from([(var.name.clone(), (x, s.to_ty()))]));
    cx.callables.push(HashMap::new());
    let body_ty = compile_block(body, cx);
    cx.callables.pop();
    cx.scopes.pop();
    cx.loops.pop();
    if body_ty? != Ty::Unit {
        cx.emit(Instruction::Drop);
    }
    cx.emit(Instruction::End);
    cx.emit(Instruction::LocalGet(i));
    cx.emit(Instruction::I32Const(1));
    cx.emit(Instruction::I32Add);
    cx.emit(Instruction::LocalSet(i));
    cx.emit(Instruction::Br(cx.depth - top));
    cx.emit(Instruction::End);
    cx.emit(Instruction::End);
    Ok(Ty::Unit)
}

fn compile_binary(op: BinOp, lhs: &Expr, rhs: &Expr, cx: &mut Cx) -> Result<Ty, CompileError> {
    // `&&` and `||` short-circuit, as the interpreter's do: the right side runs only when the left does not decide
    // the answer, so its fault or effect happens on exactly the runs it happens on there (WASM-SHORT-1: both sides
    // were computed and then combined with `i32.and`/`i32.or`).
    if matches!(op, BinOp::And | BinOp::Or) {
        if compile_expr(lhs, cx)? != Ty::I32 {
            return Err(CompileError::Unsupported("`&&`/`||` on a non-Bool".into()));
        }
        cx.emit(Instruction::If(BlockType::Result(ValType::I32)));
        let rt = if op == BinOp::And {
            let rt = compile_expr(rhs, cx)?;
            cx.emit(Instruction::Else);
            cx.emit(Instruction::I32Const(0));
            rt
        } else {
            cx.emit(Instruction::I32Const(1));
            cx.emit(Instruction::Else);
            compile_expr(rhs, cx)?
        };
        cx.emit(Instruction::End);
        if rt != Ty::I32 {
            return Err(CompileError::Unsupported("`&&`/`||` on a non-Bool".into()));
        }
        return Ok(Ty::I32);
    }
    let lt = compile_expr(lhs, cx)?;
    let rt = compile_expr(rhs, cx)?;
    use BinOp::*;
    if lt == Ty::I64 && rt == Ty::I64 {
        // Checked arithmetic (parity with the interpreter, spec §3.3): +,-,*,% go through the
        // synthetic helpers that trap on overflow / INT_MIN%-1; / uses i64.div_s (already traps on
        // div-by-zero and INT_MIN/-1, matching `checked_div`). Comparisons emit directly.
        let base = cx.arith_base;
        let (ins, ty): (Instruction<'static>, Ty) = match op {
            Add => (Instruction::Call(base), Ty::I64),
            Sub => (Instruction::Call(base + 1), Ty::I64),
            Mul => (Instruction::Call(base + 2), Ty::I64),
            Rem => (Instruction::Call(base + 3), Ty::I64),
            Div => (Instruction::I64DivS, Ty::I64),
            Lt => (Instruction::I64LtS, Ty::I32),
            Le => (Instruction::I64LeS, Ty::I32),
            Gt => (Instruction::I64GtS, Ty::I32),
            Ge => (Instruction::I64GeS, Ty::I32),
            Eq => (Instruction::I64Eq, Ty::I32),
            Ne => (Instruction::I64Ne, Ty::I32),
            And | Or => unreachable!("`&&`/`||` are compiled above, before either operand"),
        };
        cx.emit(ins);
        return Ok(ty);
    }
    // `Float` arithmetic is IEEE 754 on both engines — the interpreter's `f64` and WebAssembly's `f64` are the same
    // arithmetic, so the results agree to the bit, and a comparison with a NaN is false (`!=`, true) on both. `%` has no
    // WebAssembly instruction; the host computes it as the interpreter does (Rust's `%`, a truncated remainder).
    if lt == Ty::F64 && rt == Ty::F64 {
        let (ins, ty): (Instruction<'static>, Ty) = match op {
            Add => (Instruction::F64Add, Ty::F64),
            Sub => (Instruction::F64Sub, Ty::F64),
            Mul => (Instruction::F64Mul, Ty::F64),
            Div => (Instruction::F64Div, Ty::F64),
            Rem if cx.imp.float_rem != u32::MAX => (Instruction::Call(cx.imp.float_rem), Ty::F64),
            Rem => return Err(CompileError::Unsupported("`Float % Float` in a module compiled without `delulu:float`".into())),
            Lt => (Instruction::F64Lt, Ty::I32),
            Le => (Instruction::F64Le, Ty::I32),
            Gt => (Instruction::F64Gt, Ty::I32),
            Ge => (Instruction::F64Ge, Ty::I32),
            Eq => (Instruction::F64Eq, Ty::I32),
            Ne => (Instruction::F64Ne, Ty::I32),
            And | Or => unreachable!("`&&`/`||` are compiled above, before either operand"),
        };
        cx.emit(ins);
        return Ok(ty);
    }
    if lt == Ty::I32 && rt == Ty::I32 {
        let ins: Instruction<'static> = match op {
            Eq => Instruction::I32Eq,
            Ne => Instruction::I32Ne,
            _ => return Err(CompileError::Unsupported("this operator on Bool".into())),
        };
        cx.emit(ins);
        return Ok(Ty::I32);
    }
    // Phase 3i: `Str + Str` concatenates at runtime via the `__concat` bump-allocating helper. Both
    // operand pointers are already on the stack; the helper allocates and returns the new pointer.
    // `Str == Str` and `!=` compare the bytes (`__str_eq`), as the interpreter's `String` equality does.
    if lt == Ty::Str && rt == Ty::Str && matches!(op, Eq | Ne) {
        if cx.imp.str_eq == u32::MAX {
            return Err(CompileError::Unsupported("`Str` equality in a module compiled without `__str_eq`".into()));
        }
        cx.emit(Instruction::Call(cx.imp.str_eq));
        if op == Ne {
            cx.emit(Instruction::I32Eqz);
        }
        return Ok(Ty::I32);
    }
    if lt == Ty::Str && rt == Ty::Str && op == Add {
        cx.emit(Instruction::Call(cx.concat_fn));
        return Ok(Ty::Str);
    }
    // `==` and `!=` of two lists, options, results, sums or records: the interpreter's own `eq`, host-side, over the two
    // values rebuilt by their type's descriptor through one map of cells.
    if lt == rt
        && matches!(op, Eq | Ne)
        && matches!(lt, Ty::List(_) | Ty::Option(_) | Ty::Result(..) | Ty::Enum(_) | Ty::Record(_))
        && cx.imp.value_eq != u32::MAX
    {
        cx.emit(Instruction::I32Const(desc_ptr(cx, lt)?));
        cx.emit(Instruction::Call(cx.imp.value_eq));
        if op == Ne {
            cx.emit(Instruction::I32Eqz);
        }
        return Ok(Ty::I32);
    }
    Err(CompileError::Unsupported("a binary operator on these operand types".into()))
}

// ----- checked-arithmetic helper functions (parity with the interpreter, §3.3) --------------
//
// Each takes (a: i64, b: i64) -> i64 and traps (via `unreachable`) exactly where the interpreter's
// checked arithmetic faults, so the two engines agree on both the value and the fault.

fn build(locals: &[(u32, ValType)], instrs: &[Instruction<'static>]) -> Function {
    let mut f = Function::new(locals.iter().copied());
    for i in instrs {
        f.instruction(i);
    }
    f.instruction(&Instruction::End);
    f
}

/// `a + b`, trapping on signed overflow: overflow iff `((a^r) & (b^r)) < 0`.
fn ovf_add_fn() -> Function {
    use Instruction::*;
    build(
        &[(1, ValType::I64)], // local 2 = r
        &[
            LocalGet(0),
            LocalGet(1),
            I64Add,
            LocalSet(2),
            LocalGet(0),
            LocalGet(2),
            I64Xor,
            LocalGet(1),
            LocalGet(2),
            I64Xor,
            I64And,
            I64Const(0),
            I64LtS,
            If(BlockType::Empty),
            Unreachable,
            End,
            LocalGet(2),
        ],
    )
}

/// `a - b`, trapping on signed overflow: overflow iff `((a^b) & (a^r)) < 0`.
fn ovf_sub_fn() -> Function {
    use Instruction::*;
    build(
        &[(1, ValType::I64)], // local 2 = r
        &[
            LocalGet(0),
            LocalGet(1),
            I64Sub,
            LocalSet(2),
            LocalGet(0),
            LocalGet(1),
            I64Xor,
            LocalGet(0),
            LocalGet(2),
            I64Xor,
            I64And,
            I64Const(0),
            I64LtS,
            If(BlockType::Empty),
            Unreachable,
            End,
            LocalGet(2),
        ],
    )
}

/// `a * b`, trapping on signed overflow. If `a == 0` no overflow. If `a == -1`, overflow iff
/// `b == INT_MIN`. Otherwise overflow iff `r / a != b` (safe: `a != -1`, so `i64.div_s` cannot
/// trap on `INT_MIN/-1`).
fn ovf_mul_fn() -> Function {
    use Instruction::*;
    build(
        &[(1, ValType::I64)], // local 2 = r
        &[
            LocalGet(0),
            LocalGet(1),
            I64Mul,
            LocalSet(2),
            Block(BlockType::Empty),
            LocalGet(0),
            I64Eqz,
            BrIf(0), // a == 0 -> no overflow
            LocalGet(0),
            I64Const(-1),
            I64Eq,
            If(BlockType::Empty),
            LocalGet(1),
            I64Const(i64::MIN),
            I64Eq,
            If(BlockType::Empty),
            Unreachable,
            End,
            Else,
            LocalGet(2),
            LocalGet(0),
            I64DivS,
            LocalGet(1),
            I64Ne,
            If(BlockType::Empty),
            Unreachable,
            End,
            End,
            End, // block
            LocalGet(2),
        ],
    )
}

/// `a % b`, trapping on `b == 0` and on `INT_MIN % -1` (matching `checked_rem`; core `i64.rem_s`
/// would return 0 for the latter rather than trap).
fn chk_rem_fn() -> Function {
    use Instruction::*;
    build(
        &[],
        &[
            LocalGet(1),
            I64Eqz,
            If(BlockType::Empty),
            Unreachable,
            End,
            LocalGet(0),
            I64Const(i64::MIN),
            I64Eq,
            LocalGet(1),
            I64Const(-1),
            I64Eq,
            I32And,
            If(BlockType::Empty),
            Unreachable,
            End,
            LocalGet(0),
            LocalGet(1),
            I64RemS,
        ],
    )
}

/// A 4-byte memory access (the `u32` length header of a `Str`).
fn u32_at() -> MemArg {
    MemArg { offset: 0, align: 2, memory_index: 0 }
}

/// A single-byte memory access (a string's character byte).
fn byte_at() -> MemArg {
    MemArg { offset: 0, align: 0, memory_index: 0 }
}

/// `__concat(a, b)`: allocate `[len_a+len_b : u32-le][a-bytes][b-bytes]` (`__alloc`, function `alloc`) and return the
/// new buffer's pointer. `memory.copy` (bulk memory) moves the operand bytes.
fn concat_fn(alloc: u32) -> Function {
    use Instruction::*;
    // params: local 0 = a, local 1 = b; extra locals: 2 = len_a, 3 = len_b, 4 = result_ptr, 5 = total.
    build(
        &[(4, ValType::I32)],
        &[
            LocalGet(0),
            I32Load(u32_at()),
            LocalSet(2), // len_a = mem[a]
            LocalGet(1),
            I32Load(u32_at()),
            LocalSet(3), // len_b = mem[b]
            LocalGet(2),
            LocalGet(3),
            I32Add,
            LocalSet(5), // total = len_a + len_b
            LocalGet(5),
            I32Const(4),
            I32Add,
            Call(alloc),
            LocalSet(4), // result_ptr = __alloc(4 + total)
            LocalGet(4),
            LocalGet(5),
            I32Store(u32_at()), // mem[result_ptr] = total
            // copy a's bytes: dst = result_ptr+4, src = a+4, len = len_a
            LocalGet(4),
            I32Const(4),
            I32Add,
            LocalGet(0),
            I32Const(4),
            I32Add,
            LocalGet(2),
            MemoryCopy { src_mem: 0, dst_mem: 0 },
            // copy b's bytes: dst = result_ptr+4+len_a, src = b+4, len = len_b
            LocalGet(4),
            I32Const(4),
            I32Add,
            LocalGet(2),
            I32Add,
            LocalGet(1),
            I32Const(4),
            I32Add,
            LocalGet(3),
            MemoryCopy { src_mem: 0, dst_mem: 0 },
            LocalGet(4), // return result_ptr
        ],
    )
}

/// `__int_to_str(n)`: format a signed i64 as decimal into a fresh `[len:u32-le][ascii]` buffer in
/// the bump heap and return its pointer. Matches the interpreter's `str(Int)` (`i64::to_string`),
/// including negatives and `i64::MIN`. The trick for `i64::MIN`: its magnitude overflows i64, so we
/// take `0 - n` (two's-complement wrap gives the `i64::MIN` bit pattern) and format it with UNSIGNED
/// division — reading that bit pattern as u64 is exactly the right magnitude (9223372036854775808).
///
/// locals: 0 = n (param); 1 = mag, 2 = tmp (i64); 3 = sign, 4 = count, 5 = ptr, 6 = pos, 7 = len (i32). The buffer
/// comes from `__alloc` (function `alloc`).
fn int_to_str_fn(alloc: u32) -> Function {
    use Instruction::*;
    build(
        &[(2, ValType::I64), (5, ValType::I32)],
        &[
            // sign = n < 0
            LocalGet(0),
            I64Const(0),
            I64LtS,
            LocalSet(3),
            // mag = sign ? (0 - n) : n
            LocalGet(3),
            If(BlockType::Result(ValType::I64)),
            I64Const(0),
            LocalGet(0),
            I64Sub,
            Else,
            LocalGet(0),
            End,
            LocalSet(1),
            // count decimal digits of mag (unsigned); mag == 0 counts as one digit
            I32Const(0),
            LocalSet(4),
            LocalGet(1),
            LocalSet(2), // tmp = mag
            LocalGet(1),
            I64Eqz,
            If(BlockType::Empty),
            I32Const(1),
            LocalSet(4),
            Else,
            Block(BlockType::Empty),
            Loop(BlockType::Empty),
            LocalGet(2),
            I64Eqz,
            BrIf(1), // tmp == 0 -> done
            LocalGet(2),
            I64Const(10),
            I64DivU,
            LocalSet(2), // tmp /= 10
            LocalGet(4),
            I32Const(1),
            I32Add,
            LocalSet(4), // count++
            Br(0),
            End,
            End,
            End,
            // total_len = count + sign
            LocalGet(4),
            LocalGet(3),
            I32Add,
            LocalSet(7),
            // ptr = __alloc(4 + total_len); store total_len header
            LocalGet(7),
            I32Const(4),
            I32Add,
            Call(alloc),
            LocalSet(5),
            LocalGet(5),
            LocalGet(7),
            I32Store(u32_at()),
            // if sign, write '-' at ptr+4
            LocalGet(3),
            If(BlockType::Empty),
            LocalGet(5),
            I32Const(4),
            I32Add,
            I32Const(45),
            I32Store8(byte_at()),
            End,
            // pos = ptr + 4 + sign + count - 1  (position of the last digit)
            LocalGet(5),
            I32Const(4),
            I32Add,
            LocalGet(3),
            I32Add,
            LocalGet(4),
            I32Add,
            I32Const(1),
            I32Sub,
            LocalSet(6),
            // write digits from least-significant at `pos` downward; mag == 0 writes a single '0'
            LocalGet(1),
            LocalSet(2), // tmp = mag
            LocalGet(1),
            I64Eqz,
            If(BlockType::Empty),
            LocalGet(6),
            I32Const(48),
            I32Store8(byte_at()),
            Else,
            Block(BlockType::Empty),
            Loop(BlockType::Empty),
            LocalGet(2),
            I64Eqz,
            BrIf(1),
            // mem[pos] = '0' + (tmp % 10)
            LocalGet(6),
            LocalGet(2),
            I64Const(10),
            I64RemU,
            I32WrapI64,
            I32Const(48),
            I32Add,
            I32Store8(byte_at()),
            LocalGet(6),
            I32Const(1),
            I32Sub,
            LocalSet(6), // pos--
            LocalGet(2),
            I64Const(10),
            I64DivU,
            LocalSet(2), // tmp /= 10
            Br(0),
            End,
            End,
            End,
            // return ptr
            LocalGet(5),
        ],
    )
}

/// `__str_len(s)`: the string's length as the interpreter counts it — Unicode scalar values (`chars().count()`). A guest
/// string is UTF-8 (literals, concatenations, `str(Int)` and the host's own writes), so that is the count of its bytes
/// that do not continue a character (`0b10xx_xxxx`). RW 3.4.
///
/// locals: 0 = s (param); 1 = at, 2 = end (i32); 3 = n (i64).
/// `__list_new(len) -> list` (RW 3.4): a header and `max(len, 4)` slots, `len` of them in use (the caller fills them).
fn list_new_fn(alloc: u32) -> Function {
    use Instruction::*;
    build(
        &[(2, ValType::I32)], // 1 = the header, 2 = the capacity
        &[
            LocalGet(0),
            I32Const(4),
            LocalGet(0),
            I32Const(4),
            I32GtS,
            Select,
            LocalSet(2), // cap = len > 4 ? len : 4
            I32Const(LIST_HEADER_BYTES),
            Call(alloc),
            LocalSet(1),
            LocalGet(1),
            LocalGet(0),
            I32Store(list_mem(LIST_LEN)),
            LocalGet(1),
            LocalGet(2),
            I32Store(list_mem(LIST_CAP)),
            LocalGet(1),
            LocalGet(2),
            I32Const(8),
            I32Mul,
            Call(alloc),
            I32Store(list_mem(LIST_DATA)),
            LocalGet(1),
        ],
    )
}

/// `__list_slot(list) -> address` (RW 3.4): one more slot in use, its address returned. A full list's slots are copied
/// to twice as many in a fresh allocation; the old ones are left exactly as they were, which is what lets a `for` that
/// began before the growth go on reading them.
fn list_slot_fn(alloc: u32) -> Function {
    use Instruction::*;
    build(
        &[(5, ValType::I32)], // 1 = len, 2 = cap, 3 = the new slots, 4 = i, 5 = the old slots
        &[
            LocalGet(0),
            I32Load(list_mem(LIST_LEN)),
            LocalSet(1),
            LocalGet(0),
            I32Load(list_mem(LIST_CAP)),
            LocalSet(2),
            LocalGet(1),
            LocalGet(2),
            I32GeU,
            If(BlockType::Empty),
            // cap = cap * 2 > 4 ? cap * 2 : 4
            LocalGet(2),
            I32Const(2),
            I32Mul,
            I32Const(4),
            LocalGet(2),
            I32Const(2),
            I32Mul,
            I32Const(4),
            I32GtU,
            Select,
            LocalSet(2),
            LocalGet(2),
            I32Const(8),
            I32Mul,
            Call(alloc),
            LocalSet(3),
            LocalGet(0),
            I32Load(list_mem(LIST_DATA)),
            LocalSet(5),
            I32Const(0),
            LocalSet(4),
            Block(BlockType::Empty),
            Loop(BlockType::Empty),
            LocalGet(4),
            LocalGet(1),
            I32GeU,
            BrIf(1),
            LocalGet(3),
            LocalGet(4),
            I32Const(8),
            I32Mul,
            I32Add,
            LocalGet(5),
            LocalGet(4),
            I32Const(8),
            I32Mul,
            I32Add,
            I64Load(i64_at()),
            I64Store(i64_at()),
            LocalGet(4),
            I32Const(1),
            I32Add,
            LocalSet(4),
            Br(0),
            End,
            End,
            LocalGet(0),
            LocalGet(3),
            I32Store(list_mem(LIST_DATA)),
            LocalGet(0),
            LocalGet(2),
            I32Store(list_mem(LIST_CAP)),
            End,
            LocalGet(0),
            LocalGet(1),
            I32Const(1),
            I32Add,
            I32Store(list_mem(LIST_LEN)),
            LocalGet(0),
            I32Load(list_mem(LIST_DATA)),
            LocalGet(1),
            I32Const(8),
            I32Mul,
            I32Add,
        ],
    )
}

/// `__str_eq(a, b) -> Bool`: the same bytes — what the interpreter's `Str == Str` compares (Rust's `String` equality).
fn str_eq_fn() -> Function {
    use Instruction::*;
    let byte = MemArg { offset: 4, align: 0, memory_index: 0 }; // past the length prefix
    build(
        &[(2, ValType::I32)], // 2 = the length, 3 = i
        &[
            // The same string is equal to itself.
            LocalGet(0),
            LocalGet(1),
            I32Eq,
            If(BlockType::Empty),
            I32Const(1),
            Return,
            End,
            LocalGet(0),
            I32Load(u32_at()),
            LocalTee(2),
            LocalGet(1),
            I32Load(u32_at()),
            I32Ne,
            If(BlockType::Empty),
            I32Const(0),
            Return,
            End,
            I32Const(0),
            LocalSet(3),
            Block(BlockType::Empty),
            Loop(BlockType::Empty),
            LocalGet(3),
            LocalGet(2),
            I32GeU,
            BrIf(1),
            LocalGet(0),
            LocalGet(3),
            I32Add,
            I32Load8U(byte),
            LocalGet(1),
            LocalGet(3),
            I32Add,
            I32Load8U(byte),
            I32Ne,
            If(BlockType::Empty),
            I32Const(0),
            Return,
            End,
            LocalGet(3),
            I32Const(1),
            I32Add,
            LocalSet(3),
            Br(0),
            End,
            End,
            I32Const(1),
        ],
    )
}

fn str_len_fn() -> Function {
    use Instruction::*;
    build(
        &[(2, ValType::I32), (1, ValType::I64)],
        &[
            LocalGet(0),
            I32Const(4),
            I32Add,
            LocalTee(1), // at = s + 4
            LocalGet(0),
            I32Load(u32_at()),
            I32Add,
            LocalSet(2), // end = at + byte length
            Block(BlockType::Empty),
            Loop(BlockType::Empty),
            LocalGet(1),
            LocalGet(2),
            I32GeU,
            BrIf(1),
            LocalGet(1),
            I32Load8U(byte_at()),
            I32Const(0xC0),
            I32And,
            I32Const(0x80),
            I32Ne,
            If(BlockType::Empty),
            LocalGet(3),
            I64Const(1),
            I64Add,
            LocalSet(3),
            End,
            LocalGet(1),
            I32Const(1),
            I32Add,
            LocalSet(1),
            Br(0),
            End,
            End,
            LocalGet(3),
        ],
    )
}

/// `__alloc(n)`: `n` bytes from the bump heap — the heap pointer's old value, the pointer moved past them. The memory
/// grows by the pages the allocation needs when it would end past the memory's size (RW 3.4); an allocation that cannot be
/// had — past 4 GiB, or refused by the engine — traps as an access out of bounds (a load at 0xFFFF_FFFF, past any 32-bit
/// memory), which the host reports as DL0903, the code the fixed heap's end had. Never `unreachable`: the host reads that
/// as the arithmetic helpers' overflow, and an exhausted memory was reported as `DL0901: integer overflow`. Every guest
/// allocation goes through here; the host allocates by the same rule (`host.rs` `guest_alloc`). The run's memory budget
/// holds the growth.
///
/// locals: 0 = n (param); 1 = ptr, 2 = end (i32).
fn alloc_fn(heap_global: u32) -> Function {
    use Instruction::*;
    // The memory's size in bytes, as an i64 (65,536 pages is 2^32 bytes, past an i32).
    let size_bytes = [MemorySize(0), I64ExtendI32U, I64Const(16), I64Shl];
    let mut code = vec![
        GlobalGet(heap_global),
        LocalTee(1),
        LocalGet(0),
        I32Add,
        LocalSet(2), // end = ptr + n
        // end < ptr: the sum wrapped past 4 GiB.
        LocalGet(2),
        LocalGet(1),
        I32LtU,
        If(BlockType::Empty),
        I32Const(-1),
        I32Load(u32_at()),
        Drop,
        End,
        // end > size: grow by ceil((end - size) / 65536) pages, or trap.
        LocalGet(2),
        I64ExtendI32U,
    ];
    code.extend(size_bytes.iter().cloned());
    code.extend([I64GtU, If(BlockType::Empty), LocalGet(2), I64ExtendI32U]);
    code.extend(size_bytes.iter().cloned());
    code.extend([
        I64Sub,
        I64Const(65535),
        I64Add,
        I64Const(16),
        I64ShrU,
        I32WrapI64,
        MemoryGrow(0),
        I32Const(-1),
        I32Eq,
        If(BlockType::Empty),
        I32Const(-1),
        I32Load(u32_at()),
        Drop,
        End,
        End,
        LocalGet(2),
        GlobalSet(heap_global),
        LocalGet(1),
    ]);
    build(&[(2, ValType::I32)], &code)
}

#[cfg(test)]
mod alloc_tests {
    use super::*;

    /// A module holding `__alloc` alone (function 0) and an exported `t(n)` that calls it, its heap pointer starting at 16
    /// in a memory of one page that may grow to `max_pages`.
    fn alloc_module(max_pages: u64) -> Vec<u8> {
        let mut types = TypeSection::new();
        types.ty().function([ValType::I32], [ValType::I32]);
        let mut funcs = FunctionSection::new();
        funcs.function(0);
        funcs.function(0);
        let mut mems = MemorySection::new();
        mems.memory(MemoryType { minimum: 1, maximum: Some(max_pages), memory64: false, shared: false, page_size_log2: None });
        let mut globals = GlobalSection::new();
        globals.global(GlobalType { val_type: ValType::I32, mutable: true, shared: false }, &ConstExpr::i32_const(16));
        let mut exports = ExportSection::new();
        exports.export("t", ExportKind::Func, 1);
        let mut code = CodeSection::new();
        code.function(&alloc_fn(0));
        code.function(&build(&[], &[Instruction::LocalGet(0), Instruction::Call(0)]));
        let mut m = WasmModule::new();
        m.section(&types);
        m.section(&funcs);
        m.section(&mems);
        m.section(&globals);
        m.section(&exports);
        m.section(&code);
        m.finish()
    }

    /// What `t(n)` answers: the pointer, or the trap as the host reports it to the operator.
    fn call(wasm: &[u8], n: i32) -> Result<i32, String> {
        let engine = crate::host::optimizing_engine();
        let module = wasmtime::Module::new(&engine, wasm).expect("the test module validates");
        let mut store = wasmtime::Store::new(&engine, ());
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).expect("it instantiates");
        let t = instance.get_typed_func::<i32, i32>(&mut store, "t").expect("t is exported");
        t.call(&mut store, n).map_err(|e| crate::host::clean_trap(&e))
    }

    /// An allocation the memory cannot give is the WASM engine's memory exhausted — and was reported as `DL0901: integer
    /// overflow` (RW 3.4's `__alloc` trapped with `unreachable`, which the host reads as the arithmetic helpers' overflow):
    /// a program that doubled a string past WebAssembly's 4 GiB under `--limits mem=12G` printed exactly that, with no
    /// arithmetic in it. Both ways to fail: a size whose end wraps past 4 GiB, and a memory that may not grow.
    #[test]
    fn an_allocation_the_memory_cannot_give_is_not_an_overflow() {
        // From a heap pointer of 16, `-8` is 0xFFFF_FFF8 bytes: its end wraps past 2^32.
        let wasm = alloc_module(4);
        assert_eq!(call(&wasm, 100), Ok(16), "an allocation that fits is the heap pointer's old value");
        for (what, n, max_pages) in [("wraps past 4 GiB", -8, 65_536), ("needs a page the memory may not have", 300_000, 4)] {
            let got = call(&alloc_module(max_pages), n);
            let err = got.clone().expect_err(what);
            assert!(err.starts_with("DL0903") && err.contains("exhausted") && !err.contains("overflow"), "{what}: {got:?}");
        }
    }
}

#[cfg(test)]
mod ctor_tests {
    use super::*;

    /// A constructor used with no expected type builds the sum that declares it — and, as the checker's `variant_ctor`
    /// rules, a name two sums declare builds neither: `Other` is `IoErr`'s and `NetErr`'s, and here a user sum's too. The
    /// checker refuses such a name first, so no checked program reaches the rule; this is where it is held.
    #[test]
    fn a_constructor_two_sums_declare_names_neither() {
        let checked = delulu_check::check_source(0, "module m\ntype E = Ready | Other(Str)\n");
        let env = build_enum_env(&checked.module);
        assert_eq!(env.enum_of_ctor("Other"), None, "three sums declare `Other`");
        assert_eq!(env.enum_of_ctor("Ready"), env.id("E"));
        assert_eq!(env.enum_of_ctor("NotFound"), env.id("IoErr"));
        assert_eq!(env.enum_of_ctor("Missing"), None);
    }
}
