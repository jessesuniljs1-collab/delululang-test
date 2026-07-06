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
//! Constructs outside this fragment (other capabilities, `match`, `while`, foreign, GC types) are
//! `CompileError::Unsupported` (DL1201) and stay on the interpreter, which remains the reference
//! engine.

use std::collections::HashMap;

use delulu_syntax::ast::*;
use wasm_encoder::{
    BlockType, CodeSection, ConstExpr, DataSection, EntityType, ExportKind, ExportSection, Function,
    FunctionSection, GlobalSection, GlobalType, ImportSection, Instruction, MemArg, MemorySection,
    MemoryType, Module as WasmModule, TypeSection, ValType,
};

#[derive(Clone, Debug)]
pub enum CompileError {
    Unsupported(String),
}

impl CompileError {
    pub fn message(&self) -> String {
        match self {
            CompileError::Unsupported(what) => format!("WASM codegen does not support {what}"),
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
    Str,   // i32 pointer into linear memory
    Cap,   // i32 host handle (Console)
    Clock, // i32 host handle (Clock, Phase 3k)
    Rand,  // i32 host handle (Rand, Phase 3l)
    Root,  // i32 host handle to the root authority (Phase 3e)
    Unit,  // no value
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
}

fn wasm_valtype(t: Ty) -> Option<ValType> {
    match t {
        Ty::I64 => Some(ValType::I64),
        Ty::I32 | Ty::Str | Ty::Cap | Ty::Clock | Ty::Rand | Ty::Root => Some(ValType::I32),
        Ty::Unit => None,
    }
}

fn wasm_ty(t: &TypeExpr) -> Option<Ty> {
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
                            _ => None,
                        };
                    }
                }
                return None; // other capability kinds are later phases
            }
            if args.is_empty() {
                return match name {
                    "Int" => Some(Ty::I64),
                    "Bool" => Some(Ty::I32),
                    "Str" => Some(Ty::Str),
                    "Root" => Some(Ty::Root),
                    "Unit" => Some(Ty::Unit),
                    _ => None,
                };
            }
        }
    }
    None
}

fn ret_ty(f: &FnDecl) -> Option<Ty> {
    match &f.ret {
        None => Some(Ty::Unit),
        Some(t) => wasm_ty(t),
    }
}

fn is_compilable(f: &FnDecl) -> bool {
    f.generics.is_empty()
        && f.params.iter().all(|p| matches!(wasm_ty(&p.ty), Some(t) if t != Ty::Unit))
        && ret_ty(f).is_some()
}

/// Does any compilable function call a method whose name is in `names`? Used to decide which host
/// imports the module needs (`console`/`println` → the console imports; `clock`/`now_ms` → clock).
fn module_calls_method(module: &Module, names: &[&str]) -> bool {
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
            Expr::Block(b) => in_block(b, names),
            _ => false,
        }
    }
    module.items.iter().any(|it| matches!(it, Item::Fn(f) if is_compilable(f) && in_block(&f.body, names)))
}

/// Does the module perform console output (so it needs the console host imports)?
pub fn uses_console(module: &Module) -> bool {
    module_calls_method(module, &["console", "println"])
}

/// Does the module read the clock (so it needs the clock host imports)?
fn uses_clock(module: &Module) -> bool {
    module_calls_method(module, &["clock", "now_ms"])
}

/// Does the module draw randomness (so it needs the rand host imports)? `int` is the `Cap[Rand]`
/// method `r.int(lo, hi)` (a `Method`); the free `int(float)` builtin is a `Call`, so it doesn't match.
fn uses_rand(module: &Module) -> bool {
    module_calls_method(module, &["rand", "int"])
}

/// Compile a checked module's compilable functions to a WASM module exporting each by name.
pub fn compile_module(module: &Module) -> Result<Vec<u8>, CompileError> {
    let fns: Vec<&FnDecl> = module
        .items
        .iter()
        .filter_map(|it| if let Item::Fn(f) = it { Some(f) } else { None })
        .filter(|f| is_compilable(f))
        .collect();

    let needs_console = uses_console(module);
    let needs_clock = uses_clock(module);
    let needs_rand = uses_rand(module);

    // Assign imported-function indices in a fixed order (console, then clock, then rand pairs). Only
    // the present ones consume indices; the rest are left as sentinels codegen never reads.
    let mut n_imports = 0u32;
    let mut imp = Imports {
        root_console: u32::MAX,
        console_println: u32::MAX,
        root_clock: u32::MAX,
        clock_now_ms: u32::MAX,
        root_rand: u32::MAX,
        rand_int: u32::MAX,
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
    let arith_base = n_imports; // the 4 checked-arithmetic helpers occupy [n_imports, n_imports+4)
    // The string helpers (`__concat`, `__int_to_str`) follow; user functions start after them.
    let user_base = n_imports + N_ARITH_HELPERS + N_STR_HELPERS;

    // Collect string literals into a length-prefixed linear-memory image.
    let mut str_off: HashMap<String, u32> = HashMap::new();
    let mut data: Vec<u8> = Vec::new();
    for f in &fns {
        collect_strings_block(&f.body, &mut str_off, &mut data);
    }

    // Function index map: name -> (absolute wasm function index, return type).
    let mut index: HashMap<String, (u32, Ty)> = HashMap::new();
    for (i, f) in fns.iter().enumerate() {
        index.insert(f.name.name.clone(), (user_base + i as u32, ret_ty(f).unwrap()));
    }

    // Types: [import types...], the shared helper type (i64,i64)->i64, the string helpers, each user
    // fn's. Import types are emitted in the same fixed order the indices were assigned above.
    let mut types = TypeSection::new();
    let mut next_type = 0u32;
    let mut console_root_ty = 0;
    let mut console_println_ty = 0;
    if needs_console {
        console_root_ty = next_type;
        types.ty().function([ValType::I32], [ValType::I32]); // root_console(root) -> cap
        console_println_ty = next_type + 1;
        types.ty().function([ValType::I32, ValType::I32], []); // console_println(cap, ptr)
        next_type += 2;
    }
    let mut clock_root_ty = 0;
    let mut clock_now_ty = 0;
    if needs_clock {
        clock_root_ty = next_type;
        types.ty().function([ValType::I32], [ValType::I32]); // root_clock(root) -> cap
        clock_now_ty = next_type + 1;
        types.ty().function([ValType::I32], [ValType::I64]); // clock_now_ms(cap) -> i64
        next_type += 2;
    }
    let mut rand_root_ty = 0;
    let mut rand_int_ty = 0;
    if needs_rand {
        rand_root_ty = next_type;
        types.ty().function([ValType::I32], [ValType::I32]); // root_rand(root) -> cap
        rand_int_ty = next_type + 1;
        types.ty().function([ValType::I32, ValType::I64, ValType::I64], [ValType::I64]); // rand_int(cap, lo, hi) -> i64
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
    let mut user_types = Vec::new();
    for f in &fns {
        let params: Vec<ValType> = f.params.iter().map(|p| wasm_valtype(wasm_ty(&p.ty).unwrap()).unwrap()).collect();
        let results = result_valtypes(ret_ty(f).unwrap());
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
        code.function(&compile_fn(f, &index, &str_off, arith_base, imp)?);
    }

    let mut datas = DataSection::new();
    if !data.is_empty() {
        datas.active(0, &ConstExpr::i32_const(0), data.iter().copied());
    }

    // Section order: Type(1), Import(2), Function(3), Memory(5), Global(6), Export(7), Code(10),
    // Data(11).
    let mut m = WasmModule::new();
    m.section(&types);
    if needs_console || needs_clock || needs_rand {
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

struct Cx<'a> {
    index: &'a HashMap<String, (u32, Ty)>,
    str_off: &'a HashMap<String, u32>,
    scopes: Vec<HashMap<String, (u32, Ty)>>,
    extra_locals: Vec<ValType>,
    nparams: u32,
    /// Function index of the first arithmetic helper (`__ovf_add`); the others follow.
    arith_base: u32,
    /// Function index of the `__concat` helper (`Str + Str`).
    concat_fn: u32,
    /// Function index of the `__int_to_str` helper (`str(Int)`).
    int_to_str_fn: u32,
    /// Host import function indices (`root.console()`/`println`, `root.clock()`/`now_ms`).
    imp: Imports,
    instrs: Vec<Instruction<'static>>,
}

impl<'a> Cx<'a> {
    fn lookup(&self, name: &str) -> Option<(u32, Ty)> {
        self.scopes.iter().rev().find_map(|s| s.get(name).copied())
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

fn compile_fn(f: &FnDecl, index: &HashMap<String, (u32, Ty)>, str_off: &HashMap<String, u32>, arith_base: u32, imp: Imports) -> Result<Function, CompileError> {
    let mut params = HashMap::new();
    for (i, p) in f.params.iter().enumerate() {
        params.insert(p.name.name.clone(), (i as u32, wasm_ty(&p.ty).unwrap()));
    }
    let mut cx = Cx {
        index,
        str_off,
        scopes: vec![params],
        extra_locals: Vec::new(),
        nparams: f.params.len() as u32,
        arith_base,
        concat_fn: arith_base + N_ARITH_HELPERS,
        int_to_str_fn: arith_base + N_ARITH_HELPERS + 1,
        imp,
        instrs: Vec::new(),
    };
    let ret = ret_ty(f).unwrap();
    let body_ty = compile_block(&f.body, &mut cx)?;
    if body_ty != ret {
        return Err(CompileError::Unsupported("a body whose value type differs from the return type".into()));
    }
    let mut func = Function::new(cx.extra_locals.iter().map(|&t| (1u32, t)));
    for ins in &cx.instrs {
        func.instruction(ins);
    }
    func.instruction(&Instruction::End);
    Ok(func)
}

fn compile_block(b: &Block, cx: &mut Cx) -> Result<Ty, CompileError> {
    cx.scopes.push(HashMap::new());
    let mut result = Ty::Unit;
    let n = b.stmts.len();
    for (i, stmt) in b.stmts.iter().enumerate() {
        let is_last = i + 1 == n;
        match stmt {
            Stmt::Let { name, value, .. } => {
                let ty = compile_expr(value, cx)?;
                let idx = cx.alloc_local(ty)?;
                cx.emit(Instruction::LocalSet(idx));
                cx.scopes.last_mut().unwrap().insert(name.name.clone(), (idx, ty));
                result = Ty::Unit;
            }
            Stmt::Expr(e) => {
                let ty = compile_expr(e, cx)?;
                if is_last {
                    result = ty;
                } else if ty != Ty::Unit {
                    cx.emit(Instruction::Drop);
                }
            }
            Stmt::Return { value: Some(e), .. } => {
                let ty = compile_expr(e, cx)?;
                cx.emit(Instruction::Return);
                if is_last {
                    result = ty;
                }
            }
            Stmt::Return { value: None, .. } => {
                cx.emit(Instruction::Return);
                result = Ty::Unit;
            }
            Stmt::While { .. } | Stmt::Assign { .. } => {
                cx.scopes.pop();
                return Err(CompileError::Unsupported("`while`/assignment (Phase 3b)".into()));
            }
        }
    }
    cx.scopes.pop();
    Ok(result)
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
            LitKind::Float(_) => Err(CompileError::Unsupported("Float literals".into())),
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
        Expr::Method { recv, name, args, .. } => {
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
            // Phase 3b: Cap[Console].println(str) -> host import; Unit result.
            if name.name == "println" && args.len() == 1 {
                let rt = compile_expr(recv, cx)?;
                if rt != Ty::Cap {
                    return Err(CompileError::Unsupported("println on a non-Console receiver".into()));
                }
                let at = compile_expr(&args[0], cx)?;
                if at != Ty::Str {
                    return Err(CompileError::Unsupported("println of a non-Str argument".into()));
                }
                cx.emit(Instruction::Call(cx.imp.console_println));
                return Ok(Ty::Unit);
            }
            // Phase 3k: Cap[Clock].now_ms() -> host import returning the (fixed or wall) clock as Int.
            if name.name == "now_ms" && args.is_empty() {
                let rt = compile_expr(recv, cx)?;
                if rt != Ty::Clock {
                    return Err(CompileError::Unsupported("now_ms() on a non-Clock receiver".into()));
                }
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
                cx.emit(Instruction::Call(cx.imp.rand_int));
                return Ok(Ty::I64);
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
                    _ => Err(CompileError::Unsupported("str() of this type (only Int and Str compile so far)".into())),
                };
            }
            let (fidx, ret) = *cx
                .index
                .get(&name)
                .ok_or_else(|| CompileError::Unsupported(format!("a call to `{name}` (not a compilable function)")))?;
            for a in args {
                compile_expr(a, cx)?;
            }
            cx.emit(Instruction::Call(fidx));
            Ok(ret)
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
            LitKind::Float(_) => Err(CompileError::Unsupported("Float".into())),
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
