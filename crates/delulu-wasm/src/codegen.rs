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
//! Constructs outside this fragment (`while`, embedded Python (`root.python`/`py.*` — head-chef
//! ruling: the DL1201 interpreter fallback, since `py.list` needs guest `List` values), GC types,
//! user enums with non-scalar payloads) are `CompileError::Unsupported` (DL1201) and stay on the
//! interpreter, which remains the reference engine. Secret-handling constructs (`root.secret(...)`,
//! `Secret.expose(...)`) are refused as `CompileError::SecretInGuest` (DL1205, Phase 3n) so secret
//! bytes never enter guest linear memory.

use std::collections::HashMap;

use delulu_syntax::ast::*;
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, DataSection, EntityType, ExportKind, ExportSection, Function,
    FunctionSection, GlobalSection, GlobalType, ImportSection, Instruction, MemArg, MemorySection,
    MemoryType, Module as WasmModule, TypeSection, ValType,
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
    I64,   // Int
    I32,   // Bool
    F64,   // Float (Stage 4 phase 4g — used for `foreign "c"` scalar marshalling)
    Str,   // i32 pointer into linear memory
    Cap,    // i32 host handle (Console)
    Clock,  // i32 host handle (Clock, Phase 3k)
    Rand,   // i32 host handle (Rand, Phase 3l)
    FsRead, // i32 host handle (Cap[FsRead], Phase 3p)
    Root,   // i32 host handle to the root authority (Phase 3e)
    // Stage 4 phase 4g: `Cap[ForeignLoad]` (i32 host handle) and a bound `foreign` lib handle
    // `Foreign(lib_id)` (i32 host handle; `lib_id` indexes the module's foreign blocks so a method
    // call knows its marshalling signature). `ForeignPtr` is the opaque `void*` handle (i32 index
    // into the host's foreign-pointer table — never a raw address in the guest).
    ForeignLoad,
    Foreign(u32),
    ForeignPtr,
    Unit,   // no value
    // Phase 3o: sum types. A variant value is an i32 pointer to `[tag:i32][field…]` in the heap.
    // `Result`/`Option` carry their payload scalars inline; user/prelude enums (Phase 3o Checkpoint 3)
    // are `Enum(id)` where `id` indexes the module's `EnumEnv` (so nested enums work — e.g. an
    // `IoErr` in `Result[Str, IoErr]` is `Scalar::Enum(io_err_id)`).
    Result(Scalar, Scalar), // Ok(T), Err(E)
    Option(Scalar),         // None, Some(T)
    Enum(u32),              // a named sum type (index into EnumEnv)
}

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
        _ => None,
    }
}

/// Emit the store instruction for a variant field slot of the given scalar payload kind.
fn store_scalar(cx: &mut Cx, s: Scalar) {
    match s {
        Scalar::I64 => cx.emit(Instruction::I64Store(i64_at())),
        Scalar::F64 => cx.emit(Instruction::F64Store(i64_at())),
        Scalar::I32 | Scalar::Str | Scalar::Enum(_) | Scalar::Foreign(_) | Scalar::ForeignPtr => {
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
        Scalar::I32 | Scalar::Str | Scalar::Enum(_) | Scalar::Foreign(_) | Scalar::ForeignPtr => {
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
}

impl EnumEnv {
    fn id(&self, name: &str) -> Option<u32> {
        self.by_name.get(name).copied()
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

    let mut env = EnumEnv {
        descs: vec![EnumDesc { ctors: Vec::new() }; by_name.len()],
        by_name,
        foreign_libs: Vec::new(),
        foreign_by_name,
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
    let mut foreign_libs: Vec<ForeignLibInfo> = vec![
        ForeignLibInfo { name: String::new(), methods: Vec::new() };
        env.foreign_by_name.len()
    ];
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
                                    None => { ok = false; break; }
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
    env
}

fn set_ctors(env: &mut EnumEnv, name: &str, ctors: &[(&str, Vec<Scalar>)]) {
    if let Some(&id) = env.by_name.get(name) {
        env.descs[id as usize].ctors = ctors
            .iter()
            .map(|(n, p)| EnumCtorDesc { name: (*n).to_string(), payloads: p.clone() })
            .collect();
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
    // Stage 4 phase 4g — the `delulu:foreign` host interface.
    foreign_load: u32,
    foreign_bind: u32,
    foreign_call: u32,
    float_to_str: u32,
}

fn wasm_valtype(t: Ty) -> Option<ValType> {
    match t {
        Ty::I64 => Some(ValType::I64),
        Ty::F64 => Some(ValType::F64),
        // Str/Cap/variant handles are all i32 (a pointer or a host handle).
        Ty::I32 | Ty::Str | Ty::Cap | Ty::Clock | Ty::Rand | Ty::FsRead | Ty::Root
        | Ty::ForeignLoad | Ty::Foreign(_) | Ty::ForeignPtr | Ty::Result(..) | Ty::Option(..) | Ty::Enum(_) => {
            Some(ValType::I32)
        }
        Ty::Unit => None,
    }
}

fn wasm_ty(env: &EnumEnv, t: &TypeExpr) -> Option<Ty> {
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
            if args.is_empty() {
                return match name {
                    "Int" => Some(Ty::I64),
                    "Bool" => Some(Ty::I32),
                    "Float" => Some(Ty::F64),
                    "Str" => Some(Ty::Str),
                    "Root" => Some(Ty::Root),
                    "Unit" => Some(Ty::Unit),
                    "ForeignPtr" => Some(Ty::ForeignPtr),
                    // A named sum type (prelude or user `enum`), else a `foreign … lib M` handle type.
                    other => env.id(other).map(Ty::Enum).or_else(|| env.foreign_id(other).map(Ty::Foreign)),
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

/// Does any compilable function call a method whose name is in `names`? Used to decide which host
/// imports the module needs (`console`/`println` → the console imports; `clock`/`now_ms` → clock).
fn module_calls_method(env: &EnumEnv, module: &Module, names: &[&str]) -> bool {
    fn in_block(b: &Block, names: &[&str]) -> bool {
        b.stmts.iter().any(|s| match s {
            Stmt::Let { value, .. } | Stmt::Assign { value, .. } => in_expr(value, names),
            Stmt::While { cond, body, .. } => in_expr(cond, names) || in_block(body, names),
            Stmt::Return { value: Some(e), .. } => in_expr(e, names),
            Stmt::Return { value: None, .. } => false,
            Stmt::Expr(e) => in_expr(e, names),
        })
    }
    fn in_expr(e: &Expr, names: &[&str]) -> bool {
        match e {
            Expr::Method { name, recv, args, .. } => {
                names.contains(&name.name.as_str()) || in_expr(recv, names) || args.iter().any(|a| in_expr(a, names))
            }
            Expr::Call { callee, args, .. } => in_expr(callee, names) || args.iter().any(|a| in_expr(a, names)),
            Expr::Binary { lhs, rhs, .. } => in_expr(lhs, names) || in_expr(rhs, names),
            Expr::Unary { operand, .. } => in_expr(operand, names),
            Expr::If { cond, then_, else_, .. } => {
                in_expr(cond, names) || in_block(then_, names) || else_.as_ref().is_some_and(|e| in_expr(e, names))
            }
            Expr::Match { scrutinee, arms, .. } => {
                in_expr(scrutinee, names) || arms.iter().any(|a| in_expr(&a.body, names))
            }
            Expr::Try { inner, .. } => in_expr(inner, names),
            Expr::Block(b) => in_block(b, names),
            _ => false,
        }
    }
    module.items.iter().any(|it| matches!(it, Item::Fn(f) if is_compilable(env, f) && in_block(&f.body, names)))
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
pub fn compile_module_with(
    module: &Module,
    foreign_binds: &HashMap<NodeId, String>,
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

    let needs_console = module_calls_method(&env, module, &["console", "println"]);
    let needs_clock = module_calls_method(&env, module, &["clock", "now_ms"]);
    let needs_rand = module_calls_method(&env, module, &["rand", "int"]);
    let needs_fs = module_calls_method(&env, module, &["fs_read", "read_text"]);
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
        foreign_load: u32::MAX,
        foreign_bind: u32::MAX,
        foreign_call: u32::MAX,
        float_to_str: u32::MAX,
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
        n_imports += 2;
    }
    if needs_foreign {
        imp.foreign_load = n_imports;
        imp.foreign_bind = n_imports + 1;
        imp.foreign_call = n_imports + 2;
        imp.float_to_str = n_imports + 3;
        n_imports += 4;
    }
    let arith_base = n_imports; // the 4 checked-arithmetic helpers occupy [n_imports, n_imports+4)
    // The string helpers (`__concat`, `__int_to_str`) follow; user functions start after them.
    let user_base = n_imports + N_ARITH_HELPERS + N_STR_HELPERS;

    // Collect string literals into a length-prefixed linear-memory image.
    let mut str_off: HashMap<String, u32> = HashMap::new();
    let mut data: Vec<u8> = Vec::new();
    for f in &fns {
        collect_strings_block(&f.body, &mut str_off, &mut data);
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
        // `str(Bool)` (a Bool from a foreign return, spec §4.2 marshalling matrix) selects one of these.
        intern_string("true", &mut str_off, &mut data);
        intern_string("false", &mut str_off, &mut data);
    }

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
        types.ty().function([ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32], [ValType::I64]);
        float_to_str_ty = next_type + 3;
        types.ty().function([ValType::F64], [ValType::I32]); // float_to_str(x) -> str ptr (matches Value::Float display)
        next_type += 4;
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
    let mut user_types = Vec::new();
    for f in &fns {
        let params: Vec<ValType> = f.params.iter().map(|p| wasm_valtype(wasm_ty(&env, &p.ty).unwrap()).unwrap()).collect();
        let results = result_valtypes(ret_ty(&env, f).unwrap());
        types.ty().function(params, results);
        user_types.push(next_type);
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
    }
    if needs_foreign {
        imports.import("delulu:foreign", "foreign_load", EntityType::Function(foreign_load_ty));
        imports.import("delulu:foreign", "foreign_bind", EntityType::Function(foreign_bind_ty));
        imports.import("delulu:foreign", "foreign_call", EntityType::Function(foreign_call_ty));
        imports.import("delulu:foreign", "float_to_str", EntityType::Function(float_to_str_ty));
    }

    // Functions (in code order): the 4 arithmetic helpers, `__concat`, `__int_to_str`, then users.
    let mut funcsec = FunctionSection::new();
    for _ in 0..N_ARITH_HELPERS {
        funcsec.function(helper_type);
    }
    funcsec.function(concat_type);
    funcsec.function(int_to_str_type);
    for t in &user_types {
        funcsec.function(*t);
    }

    let mut mems = MemorySection::new();
    // Reserve the interned literal image plus a fixed bump-allocation heap for runtime strings.
    // The heap has no `memory.grow` yet, so a concatenation-heavy run could exhaust it (a trap =
    // an error, honestly surfaced); HEAP_PAGES keeps everyday programs comfortably within bounds.
    let data_pages = (data.len() as u64 + 65535) / 65536;
    let min_pages = data_pages + HEAP_PAGES;
    mems.memory(MemoryType { minimum: min_pages, maximum: None, memory64: false, shared: false, page_size_log2: None });

    // The bump-heap pointer: a mutable i32 global starting just past the interned literals.
    let heap_start = data.len() as i32;
    let mut globals = GlobalSection::new();
    globals.global(
        GlobalType { val_type: ValType::I32, mutable: true, shared: false },
        &ConstExpr::i32_const(heap_start),
    );

    let mut exports = ExportSection::new();
    exports.export("memory", ExportKind::Memory, 0);
    // Export the bump-heap pointer so the host can allocate in guest memory (the filesystem cap
    // writes the file content + `Result[Str, IoErr]` cells into the guest heap — Phase 3p).
    exports.export("__heap", ExportKind::Global, HEAP_GLOBAL);
    for (i, f) in fns.iter().enumerate() {
        exports.export(&f.name.name, ExportKind::Func, user_base + i as u32);
    }

    let mut code = CodeSection::new();
    code.function(&ovf_add_fn());
    code.function(&ovf_sub_fn());
    code.function(&ovf_mul_fn());
    code.function(&chk_rem_fn());
    code.function(&concat_fn(HEAP_GLOBAL));
    code.function(&int_to_str_fn(HEAP_GLOBAL));
    for f in &fns {
        code.function(&compile_fn(f, &env, &generics, &index, &str_off, arith_base, imp, foreign_binds)?);
    }

    let mut datas = DataSection::new();
    if !data.is_empty() {
        datas.active(0, &ConstExpr::i32_const(0), data.iter().copied());
    }

    // Section order: Type(1), Import(2), Function(3), Memory(5), Global(6), Export(7), Code(10),
    // Data(11).
    let mut m = WasmModule::new();
    m.section(&types);
    if needs_console || needs_clock || needs_rand || needs_fs || needs_foreign {
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
        _ => {}
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

/// Number of synthetic string helpers emitted after the arithmetic helpers: `__concat` then
/// `__int_to_str` (in that order).
const N_STR_HELPERS: u32 = 2;

/// Pages (64 KiB each) reserved for the runtime bump-allocation heap, above the interned literals.
const HEAP_PAGES: u64 = 16; // 1 MiB — no `memory.grow` yet, so this is the fixed heap ceiling

/// Index of the bump-heap pointer global (the module emits exactly one global).
const HEAP_GLOBAL: u32 = 0;

/// A function VALUE the backend inlines rather than represents at runtime: a lambda, or a generic
/// function specialised at a call site. `params`/`body` are owned clones of the AST (Phase 3r).
#[derive(Clone)]
struct Callable {
    params: Vec<Param>,
    body: Block,
}

struct Cx<'a> {
    index: &'a HashMap<String, (u32, Vec<Ty>, Ty)>,
    str_off: &'a HashMap<String, u32>,
    /// The module's named sum types (Phase 3o Checkpoint 3).
    env: &'a EnumEnv,
    /// Generic user functions, inlined (monomorphised) at each call site (Phase 3r).
    generics: &'a HashMap<String, FnDecl>,
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
    /// Host import function indices (`root.console()`/`println`, `root.clock()`/`now_ms`).
    imp: Imports,
    /// The checker's `root.foreign(load)` bind-site → lib-name map (Stage 4 phase 4g): recovers which
    /// lib a bind site resolves to (the grammar has no method type-argument syntax).
    foreign_binds: &'a HashMap<NodeId, String>,
    /// The function's declared return type — the expected type for `return`/`?` (Phase 3o).
    ret: Ty,
    instrs: Vec<Instruction<'static>>,
}

impl<'a> Cx<'a> {
    fn lookup(&self, name: &str) -> Option<(u32, Ty)> {
        self.scopes.iter().rev().find_map(|s| s.get(name).copied())
    }
    fn lookup_callable(&self, name: &str) -> Option<Callable> {
        self.callables.iter().rev().find_map(|s| s.get(name).cloned())
    }
    fn alloc_local(&mut self, ty: Ty) -> Result<u32, CompileError> {
        let vt = wasm_valtype(ty).ok_or_else(|| CompileError::Unsupported("a `let` binding of type Unit".into()))?;
        let idx = self.nparams + self.extra_locals.len() as u32;
        self.extra_locals.push(vt);
        Ok(idx)
    }
    fn emit(&mut self, i: Instruction<'static>) {
        self.instrs.push(i);
    }
}

#[allow(clippy::too_many_arguments)]
fn compile_fn<'a>(f: &FnDecl, env: &'a EnumEnv, generics: &'a HashMap<String, FnDecl>, index: &'a HashMap<String, (u32, Vec<Ty>, Ty)>, str_off: &'a HashMap<String, u32>, arith_base: u32, imp: Imports, foreign_binds: &'a HashMap<NodeId, String>) -> Result<Function, CompileError> {
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
        scopes: vec![params],
        callables: vec![HashMap::new()],
        extra_locals: Vec::new(),
        nparams: f.params.len() as u32,
        inline_depth: 0,
        arith_base,
        concat_fn: arith_base + N_ARITH_HELPERS,
        int_to_str_fn: arith_base + N_ARITH_HELPERS + 1,
        imp,
        foreign_binds,
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

/// Synthesis: compile a block and report its tail value type.
fn compile_block(b: &Block, cx: &mut Cx) -> Result<Ty, CompileError> {
    compile_block_inner(b, cx, None)
}

/// Expected-type-directed: compile a block whose tail must produce `expected`.
fn compile_block_as(b: &Block, cx: &mut Cx, expected: Ty) -> Result<(), CompileError> {
    compile_block_inner(b, cx, Some(expected)).map(|_| ())
}

fn compile_block_inner(b: &Block, cx: &mut Cx, expected: Option<Ty>) -> Result<Ty, CompileError> {
    cx.scopes.push(HashMap::new());
    let mut result = Ty::Unit;
    let n = b.stmts.len();
    for (i, stmt) in b.stmts.iter().enumerate() {
        let is_last = i + 1 == n;
        let r = compile_stmt(stmt, cx, is_last, expected);
        match r {
            Ok(ty) => result = ty,
            Err(e) => {
                cx.scopes.pop();
                return Err(e);
            }
        }
    }
    cx.scopes.pop();
    Ok(result)
}

fn compile_stmt(stmt: &Stmt, cx: &mut Cx, is_last: bool, expected: Option<Ty>) -> Result<Ty, CompileError> {
    match stmt {
        Stmt::Let { name, value, .. } => {
            // Binding a lambda binds a function VALUE (inlined at its call sites), not a runtime local.
            if let Expr::Lambda { params, body, .. } = value {
                let c = Callable { params: params.clone(), body: body.clone() };
                cx.callables.last_mut().unwrap().insert(name.name.clone(), c);
                return Ok(Ty::Unit);
            }
            let ty = compile_expr(value, cx)?;
            let idx = cx.alloc_local(ty)?;
            cx.emit(Instruction::LocalSet(idx));
            cx.scopes.last_mut().unwrap().insert(name.name.clone(), (idx, ty));
            Ok(Ty::Unit)
        }
        Stmt::Expr(e) => {
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
        Stmt::While { .. } | Stmt::Assign { .. } => {
            Err(CompileError::Unsupported("`while`/assignment".into()))
        }
    }
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
fn compile_foreign_call(lib_id: u32, method: &str, args: &[Expr], span: delulu_diag::Span, cx: &mut Cx) -> Result<Ty, CompileError> {
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
    cx.emit(Instruction::GlobalGet(HEAP_GLOBAL));
    cx.emit(Instruction::LocalTee(base));
    cx.emit(Instruction::I32Const(argc * FVAL_CELL));
    cx.emit(Instruction::I32Add);
    cx.emit(Instruction::GlobalSet(HEAP_GLOBAL));

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
            let else_ = else_.as_ref().ok_or_else(|| CompileError::Unsupported("an `if` without `else` used as a value".into()))?;
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
        _ => {
            let got = compile_expr(e, cx)?;
            if got != expected {
                return Err(CompileError::Unsupported(format!("a value of type {got:?} where {expected:?} was expected")));
            }
            Ok(())
        }
    }
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
        return Err(CompileError::Unsupported(format!("constructor `{name}` with {} args (expected {})", args.len(), payloads.len())));
    }
    let cell = variant_cell_size(&ctors);

    let rp = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::GlobalGet(HEAP_GLOBAL));
    cx.emit(Instruction::LocalTee(rp));
    cx.emit(Instruction::I32Const(cell));
    cx.emit(Instruction::I32Add);
    cx.emit(Instruction::GlobalSet(HEAP_GLOBAL));
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
    let arm_idx = |t: usize| arm_for_tag[t].or(catch_all).ok_or_else(|| CompileError::Unsupported("a non-exhaustive `match`".into()));

    let result_ty = match expected {
        Some(t) => t,
        None => expr_result_ty(&arms[arm_idx(0)?].body)?,
    };
    let bt = match wasm_valtype(result_ty) {
        Some(v) => BlockType::Result(v),
        None => BlockType::Empty,
    };

    // Load the tag once, then a nested `if tag==t { arm } else { … }` chain (final tag = last else).
    let tv = cx.alloc_local(Ty::I32)?;
    cx.emit(Instruction::LocalGet(sp));
    cx.emit(Instruction::I32Load(u32_at()));
    cx.emit(Instruction::LocalSet(tv));
    for t in 0..ntags - 1 {
        cx.emit(Instruction::LocalGet(tv));
        cx.emit(Instruction::I32Const(t as i32));
        cx.emit(Instruction::I32Eq);
        cx.emit(Instruction::If(bt));
        compile_arm(&arms[arm_idx(t)?], &ctors[t].1, sp, sty, cx, result_ty, expected.is_some())?;
        cx.emit(Instruction::Else);
    }
    compile_arm(&arms[arm_idx(ntags - 1)?], &ctors[ntags - 1].1, sp, sty, cx, result_ty, expected.is_some())?;
    for _ in 0..ntags - 1 {
        cx.emit(Instruction::End);
    }
    Ok(result_ty)
}

fn compile_arm(arm: &Arm, payloads: &[Scalar], sp: u32, scrut_ty: Ty, cx: &mut Cx, result_ty: Ty, expected_mode: bool) -> Result<(), CompileError> {
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
    let r = if expected_mode {
        compile_expr_as(&arm.body, cx, result_ty)
    } else {
        match compile_expr(&arm.body, cx) {
            Ok(t) if t == result_ty => Ok(()),
            Ok(t) => Err(CompileError::Unsupported(format!("a match arm of type {t:?} where {result_ty:?} was expected"))),
            Err(e) => Err(e),
        }
    };
    cx.scopes.pop();
    r
}

fn is_fn_type(t: &TypeExpr) -> bool {
    matches!(t, TypeExpr::Fn { .. })
}

/// Resolve a function-typed argument to the `Callable` we will inline: a lambda literal, or a name
/// already bound to a callable.
fn resolve_callable(arg: &Expr, cx: &Cx) -> Result<Callable, CompileError> {
    match arg {
        Expr::Lambda { params, body, .. } => Ok(Callable { params: params.clone(), body: body.clone() }),
        Expr::Var { path, .. } if path.segs.len() == 1 => cx
            .lookup_callable(&path.segs[0].name)
            .ok_or_else(|| CompileError::Unsupported(format!("`{}` is not a function value", path.dotted()))),
        _ => Err(CompileError::Unsupported("a function argument that is not a lambda or function value".into())),
    }
}

/// Inline (monomorphise) a call to a generic function or a function value: evaluate value arguments
/// into fresh locals in the current scope, bind function-typed arguments as callables, then compile
/// the body in a new frame. This is how the backend handles generics and non-capturing lambdas —
/// `apply(fn(x) { x * 2 }, 21)` reduces to inlined arithmetic, no function table needed (Phase 3r).
fn inline_call(params: &[Param], body: &Block, args: &[Expr], cx: &mut Cx) -> Result<Ty, CompileError> {
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
    // Enter the inlined body with the parameters bound.
    cx.inline_depth += 1;
    cx.scopes.push(local_binds.into_iter().map(|(n, l, t)| (n, (l, t))).collect());
    cx.callables.push(fn_binds.into_iter().collect());
    let r = compile_block(body, cx);
    cx.scopes.pop();
    cx.callables.pop();
    cx.inline_depth -= 1;
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
                let ptr = *cx.str_off.get(s).expect("interned") as i32;
                cx.emit(Instruction::I32Const(ptr));
                Ok(Ty::Str)
            }
            // Stage 4 phase 4g: a `Float` literal is an f64 (used as a `foreign "c"` scalar argument).
            LitKind::Float(x) => {
                cx.emit(Instruction::F64Const(*x));
                Ok(Ty::F64)
            }
        },
        Expr::Var { path, .. } => {
            if path.segs.len() == 1 {
                if let Some((idx, ty)) = cx.lookup(&path.segs[0].name) {
                    cx.emit(Instruction::LocalGet(idx));
                    return Ok(ty);
                }
            }
            Err(CompileError::Unsupported(format!("the name `{}` (not a local)", path.dotted())))
        }
        Expr::Unary { op, operand, .. } => {
            let ty = compile_expr(operand, cx)?;
            match op {
                UnOp::Neg if ty == Ty::I64 => {
                    cx.emit(Instruction::I64Const(-1));
                    cx.emit(Instruction::I64Mul);
                    Ok(Ty::I64)
                }
                UnOp::Not if ty == Ty::I32 => {
                    cx.emit(Instruction::I32Eqz);
                    Ok(Ty::I32)
                }
                _ => Err(CompileError::Unsupported("this unary operator on this type".into())),
            }
        }
        Expr::Binary { op, lhs, rhs, .. } => compile_binary(*op, lhs, rhs, cx),
        Expr::If { cond, then_, else_, .. } => {
            let ct = compile_expr(cond, cx)?;
            if ct != Ty::I32 {
                return Err(CompileError::Unsupported("a non-Bool `if` condition".into()));
            }
            let else_ = else_.as_ref().ok_or_else(|| CompileError::Unsupported("an `if` without `else` used as a value".into()))?;
            let then_ty = block_result_ty(then_)?;
            let bt = match wasm_valtype(then_ty) {
                Some(v) => BlockType::Result(v),
                None => BlockType::Empty,
            };
            cx.emit(Instruction::If(bt));
            let tt = compile_block(then_, cx)?;
            cx.emit(Instruction::Else);
            let et = compile_expr(else_, cx)?;
            cx.emit(Instruction::End);
            if tt != et {
                return Err(CompileError::Unsupported("`if` branches of differing types".into()));
            }
            Ok(tt)
        }
        Expr::Method { recv, name, args, span, id } => {
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
            Err(CompileError::Unsupported(format!("the method `.{}`", name.name)))
        }
        Expr::Call { callee, args, .. } => {
            let name = match &**callee {
                Expr::Var { path, .. } if path.segs.len() == 1 => path.segs[0].name.clone(),
                _ => return Err(CompileError::Unsupported("an indirect or builtin call".into())),
            };
            // Phase 3j: `str(x)` — Int formats via `__int_to_str`; a Str is already a string (identity).
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
                    // `Value::Bool.display()`). Available with the foreign interface (which interns them).
                    Ty::I32 => {
                        let t = *cx.str_off.get("true").ok_or_else(|| CompileError::Unsupported("str(Bool) without the foreign interface".into()))? as i32;
                        let f = *cx.str_off.get("false").ok_or_else(|| CompileError::Unsupported("str(Bool) without the foreign interface".into()))? as i32;
                        cx.emit(Instruction::If(BlockType::Result(ValType::I32)));
                        cx.emit(Instruction::I32Const(t));
                        cx.emit(Instruction::Else);
                        cx.emit(Instruction::I32Const(f));
                        cx.emit(Instruction::End);
                        Ok(Ty::Str)
                    }
                    _ => Err(CompileError::Unsupported("str() of this type (only Int/Str, and Float/Bool with the foreign interface, compile so far)".into())),
                };
            }
            // Phase 3r: a call to a function VALUE (a bound lambda) or a generic function → inline it.
            if let Some(c) = cx.lookup_callable(&name) {
                return inline_call(&c.params, &c.body, args, cx);
            }
            if let Some(g) = cx.generics.get(&name).cloned() {
                return inline_call(&g.params, &g.body, args, cx);
            }
            let (fidx, ptys, ret) = {
                let e = cx
                    .index
                    .get(&name)
                    .ok_or_else(|| CompileError::Unsupported(format!("a call to `{name}` (not a compilable function)")))?;
                (e.0, e.1.clone(), e.2)
            };
            if args.len() != ptys.len() {
                return Err(CompileError::Unsupported(format!("`{name}` called with {} args (expected {})", args.len(), ptys.len())));
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

/// Statically determine a block's result type (to type an `if` before its then-branch is emitted).
fn block_result_ty(b: &Block) -> Result<Ty, CompileError> {
    match b.stmts.last() {
        Some(Stmt::Expr(e)) => expr_result_ty(e),
        Some(Stmt::Return { value: Some(e), .. }) => expr_result_ty(e),
        Some(Stmt::Let { .. }) | Some(Stmt::Return { value: None, .. }) | None => Ok(Ty::Unit),
        Some(Stmt::While { .. }) | Some(Stmt::Assign { .. }) => Ok(Ty::Unit),
    }
}

fn expr_result_ty(e: &Expr) -> Result<Ty, CompileError> {
    match e {
        Expr::Lit { kind, .. } => match kind {
            LitKind::Int(_) => Ok(Ty::I64),
            LitKind::Bool(_) => Ok(Ty::I32),
            LitKind::Str(_) => Ok(Ty::Str),
            LitKind::Float(_) => Ok(Ty::F64),
        },
        Expr::Unary { op, operand, .. } => match op {
            UnOp::Neg => expr_result_ty(operand),
            UnOp::Not => Ok(Ty::I32),
        },
        Expr::Binary { op, lhs, .. } => match op {
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Eq | BinOp::Ne | BinOp::And | BinOp::Or => Ok(Ty::I32),
            _ => expr_result_ty(lhs),
        },
        Expr::If { then_, .. } => block_result_ty(then_),
        Expr::Match { arms, .. } => arms.first().map(|a| expr_result_ty(&a.body)).unwrap_or(Ok(Ty::Unit)),
        Expr::Block(b) => block_result_ty(b),
        Expr::Method { name, .. } if name.name == "println" => Ok(Ty::Unit),
        Expr::Method { name, .. } if name.name == "console" => Ok(Ty::Cap),
        Expr::Method { name, .. } if name.name == "clock" => Ok(Ty::Clock),
        Expr::Method { name, .. } if name.name == "now_ms" => Ok(Ty::I64),
        Expr::Method { name, .. } if name.name == "rand" => Ok(Ty::Rand),
        Expr::Method { name, args, .. } if name.name == "int" && args.len() == 2 => Ok(Ty::I64),
        // `str(...)` always yields a Str — type it precisely so it can tail an `if` branch.
        Expr::Call { callee, .. }
            if matches!(&**callee, Expr::Var { path, .. } if path.segs.len() == 1 && path.segs[0].name == "str") =>
        {
            Ok(Ty::Str)
        }
        // Var/Call are context-dependent; default I64 for typing. A mismatch is caught by the
        // branch-equality check in `compile_expr`.
        Expr::Var { .. } | Expr::Call { .. } => Ok(Ty::I64),
        _ => Err(CompileError::Unsupported("this expression form".into())),
    }
}

fn compile_binary(op: BinOp, lhs: &Expr, rhs: &Expr, cx: &mut Cx) -> Result<Ty, CompileError> {
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
            And | Or => return Err(CompileError::Unsupported("`&&`/`||` on Int".into())),
        };
        cx.emit(ins);
        return Ok(ty);
    }
    if lt == Ty::I32 && rt == Ty::I32 {
        let ins: Instruction<'static> = match op {
            And => Instruction::I32And,
            Or => Instruction::I32Or,
            Eq => Instruction::I32Eq,
            Ne => Instruction::I32Ne,
            _ => return Err(CompileError::Unsupported("this operator on Bool".into())),
        };
        cx.emit(ins);
        return Ok(Ty::I32);
    }
    // Phase 3i: `Str + Str` concatenates at runtime via the `__concat` bump-allocating helper. Both
    // operand pointers are already on the stack; the helper allocates and returns the new pointer.
    if lt == Ty::Str && rt == Ty::Str && op == Add {
        cx.emit(Instruction::Call(cx.concat_fn));
        return Ok(Ty::Str);
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
            LocalGet(0), LocalGet(1), I64Add, LocalSet(2),
            LocalGet(0), LocalGet(2), I64Xor,
            LocalGet(1), LocalGet(2), I64Xor,
            I64And, I64Const(0), I64LtS,
            If(BlockType::Empty), Unreachable, End,
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
            LocalGet(0), LocalGet(1), I64Sub, LocalSet(2),
            LocalGet(0), LocalGet(1), I64Xor,
            LocalGet(0), LocalGet(2), I64Xor,
            I64And, I64Const(0), I64LtS,
            If(BlockType::Empty), Unreachable, End,
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
            LocalGet(0), LocalGet(1), I64Mul, LocalSet(2),
            Block(BlockType::Empty),
            LocalGet(0), I64Eqz, BrIf(0), // a == 0 -> no overflow
            LocalGet(0), I64Const(-1), I64Eq,
            If(BlockType::Empty),
            LocalGet(1), I64Const(i64::MIN), I64Eq,
            If(BlockType::Empty), Unreachable, End,
            Else,
            LocalGet(2), LocalGet(0), I64DivS,
            LocalGet(1), I64Ne,
            If(BlockType::Empty), Unreachable, End,
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
            LocalGet(1), I64Eqz,
            If(BlockType::Empty), Unreachable, End,
            LocalGet(0), I64Const(i64::MIN), I64Eq,
            LocalGet(1), I64Const(-1), I64Eq,
            I32And,
            If(BlockType::Empty), Unreachable, End,
            LocalGet(0), LocalGet(1), I64RemS,
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

/// `__concat(a, b)`: bump-allocate `[len_a+len_b : u32-le][a-bytes][b-bytes]` at the heap pointer,
/// advance the pointer, and return the new buffer's pointer. `memory.copy` (bulk memory) moves the
/// operand bytes. An allocation past the reserved heap traps on the store — an honest error, matched
/// against the interpreter only for pathological sizes (everyday strings stay well within HEAP_PAGES).
fn concat_fn(heap_global: u32) -> Function {
    use Instruction::*;
    // params: local 0 = a, local 1 = b; extra locals: 2 = len_a, 3 = len_b, 4 = result_ptr, 5 = total.
    build(
        &[(4, ValType::I32)],
        &[
            LocalGet(0), I32Load(u32_at()), LocalSet(2), // len_a = mem[a]
            LocalGet(1), I32Load(u32_at()), LocalSet(3), // len_b = mem[b]
            LocalGet(2), LocalGet(3), I32Add, LocalSet(5), // total = len_a + len_b
            GlobalGet(heap_global), LocalSet(4),           // result_ptr = heap
            LocalGet(4), LocalGet(5), I32Store(u32_at()),  // mem[result_ptr] = total
            // copy a's bytes: dst = result_ptr+4, src = a+4, len = len_a
            LocalGet(4), I32Const(4), I32Add,
            LocalGet(0), I32Const(4), I32Add,
            LocalGet(2),
            MemoryCopy { src_mem: 0, dst_mem: 0 },
            // copy b's bytes: dst = result_ptr+4+len_a, src = b+4, len = len_b
            LocalGet(4), I32Const(4), I32Add, LocalGet(2), I32Add,
            LocalGet(1), I32Const(4), I32Add,
            LocalGet(3),
            MemoryCopy { src_mem: 0, dst_mem: 0 },
            // heap = result_ptr + 4 + total
            LocalGet(4), I32Const(4), I32Add, LocalGet(5), I32Add,
            GlobalSet(heap_global),
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
/// locals: 0 = n (param); 1 = mag, 2 = tmp (i64); 3 = sign, 4 = count, 5 = ptr, 6 = pos, 7 = len (i32).
fn int_to_str_fn(heap_global: u32) -> Function {
    use Instruction::*;
    build(
        &[(2, ValType::I64), (5, ValType::I32)],
        &[
            // sign = n < 0
            LocalGet(0), I64Const(0), I64LtS, LocalSet(3),
            // mag = sign ? (0 - n) : n
            LocalGet(3),
            If(BlockType::Result(ValType::I64)),
            I64Const(0), LocalGet(0), I64Sub,
            Else,
            LocalGet(0),
            End,
            LocalSet(1),
            // count decimal digits of mag (unsigned); mag == 0 counts as one digit
            I32Const(0), LocalSet(4),
            LocalGet(1), LocalSet(2), // tmp = mag
            LocalGet(1), I64Eqz,
            If(BlockType::Empty),
            I32Const(1), LocalSet(4),
            Else,
            Block(BlockType::Empty),
            Loop(BlockType::Empty),
            LocalGet(2), I64Eqz, BrIf(1), // tmp == 0 -> done
            LocalGet(2), I64Const(10), I64DivU, LocalSet(2), // tmp /= 10
            LocalGet(4), I32Const(1), I32Add, LocalSet(4),   // count++
            Br(0),
            End,
            End,
            End,
            // total_len = count + sign
            LocalGet(4), LocalGet(3), I32Add, LocalSet(7),
            // ptr = heap; store total_len header
            GlobalGet(heap_global), LocalSet(5),
            LocalGet(5), LocalGet(7), I32Store(u32_at()),
            // if sign, write '-' at ptr+4
            LocalGet(3),
            If(BlockType::Empty),
            LocalGet(5), I32Const(4), I32Add, I32Const(45), I32Store8(byte_at()),
            End,
            // pos = ptr + 4 + sign + count - 1  (position of the last digit)
            LocalGet(5), I32Const(4), I32Add, LocalGet(3), I32Add, LocalGet(4), I32Add, I32Const(1), I32Sub, LocalSet(6),
            // write digits from least-significant at `pos` downward; mag == 0 writes a single '0'
            LocalGet(1), LocalSet(2), // tmp = mag
            LocalGet(1), I64Eqz,
            If(BlockType::Empty),
            LocalGet(6), I32Const(48), I32Store8(byte_at()),
            Else,
            Block(BlockType::Empty),
            Loop(BlockType::Empty),
            LocalGet(2), I64Eqz, BrIf(1),
            // mem[pos] = '0' + (tmp % 10)
            LocalGet(6),
            LocalGet(2), I64Const(10), I64RemU, I32WrapI64, I32Const(48), I32Add,
            I32Store8(byte_at()),
            LocalGet(6), I32Const(1), I32Sub, LocalSet(6), // pos--
            LocalGet(2), I64Const(10), I64DivU, LocalSet(2), // tmp /= 10
            Br(0),
            End,
            End,
            End,
            // heap = ptr + 4 + total_len ; return ptr
            LocalGet(5), I32Const(4), I32Add, LocalGet(7), I32Add, GlobalSet(heap_global),
            LocalGet(5),
        ],
    )
}
