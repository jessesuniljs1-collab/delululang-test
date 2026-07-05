//! Phase 3a code generation: the pure-Int/Bool fragment of DeluluLang → core WebAssembly.
//!
//! Supported: `Int` (i64), `Bool` (i32), arithmetic, comparisons, `&&`/`||`, unary `-`/`!`,
//! `if`/`else` as an expression, `let`, calls to other pure functions, and recursion. Anything
//! outside this fragment (strings, capabilities, GC types, `match`, `while`, foreign, …) is a
//! `CompileError::Unsupported` — DL1201 — and that function is simply not compiled to WASM; the
//! interpreter remains the reference engine for it. This is the honest floor Phase 3a stands on;
//! later phases add the `delulu:cap` host interface, strings, and GC types.

use std::collections::HashMap;

use delulu_syntax::ast::*;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module as WasmModule,
    TypeSection, ValType,
};

/// A code-generation failure. Maps to diagnostic DL1201 at the CLI boundary.
#[derive(Clone, Debug)]
pub enum CompileError {
    Unsupported(String),
}

impl CompileError {
    pub fn message(&self) -> String {
        match self {
            CompileError::Unsupported(what) => format!("WASM codegen (Phase 3a) does not support {what}"),
        }
    }
}

/// The two scalar WASM types Phase 3a uses: `Int` is i64, `Bool` is i32.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ty {
    I64,
    I32,
}

fn valtype(t: Ty) -> ValType {
    match t {
        Ty::I64 => ValType::I64,
        Ty::I32 => ValType::I32,
    }
}

fn scalar_ty(t: &TypeExpr) -> Option<Ty> {
    if let TypeExpr::Named { path, args, .. } = t {
        if args.is_empty() && path.segs.len() == 1 {
            return match path.segs[0].name.as_str() {
                "Int" => Some(Ty::I64),
                "Bool" => Some(Ty::I32),
                _ => None,
            };
        }
    }
    None
}

/// A function is compilable in Phase 3a iff it is non-generic with all-scalar params and a scalar
/// return type. (Its body may still fail — that surfaces as a `CompileError` while compiling.)
fn is_compilable(f: &FnDecl) -> bool {
    f.generics.is_empty()
        && f.params.iter().all(|p| scalar_ty(&p.ty).is_some())
        && f.ret.as_ref().and_then(scalar_ty).is_some()
}

/// Compile a checked module's pure-Int/Bool functions to a WASM module that exports each by name.
/// Returns the WASM bytes. Errors if a compilable function's body uses an unsupported construct.
pub fn compile_module(module: &Module) -> Result<Vec<u8>, CompileError> {
    let mut fns: Vec<&FnDecl> = Vec::new();
    let mut index: HashMap<String, (u32, Ty)> = HashMap::new();
    for item in &module.items {
        if let Item::Fn(f) = item {
            if is_compilable(f) {
                let ret = scalar_ty(f.ret.as_ref().unwrap()).unwrap();
                index.insert(f.name.name.clone(), (fns.len() as u32, ret));
                fns.push(f);
            }
        }
    }

    let mut types = TypeSection::new();
    let mut funcsec = FunctionSection::new();
    let mut exports = ExportSection::new();
    let mut code = CodeSection::new();

    for (i, f) in fns.iter().enumerate() {
        let params: Vec<ValType> = f.params.iter().map(|p| valtype(scalar_ty(&p.ty).unwrap())).collect();
        let ret = valtype(scalar_ty(f.ret.as_ref().unwrap()).unwrap());
        types.ty().function(params, [ret]);
        funcsec.function(i as u32);
        exports.export(&f.name.name, ExportKind::Func, i as u32);
        code.function(&compile_fn(f, &index)?);
    }

    let mut m = WasmModule::new();
    m.section(&types);
    m.section(&funcsec);
    m.section(&exports);
    m.section(&code);
    Ok(m.finish())
}

struct Cx<'a> {
    index: &'a HashMap<String, (u32, Ty)>,
    scopes: Vec<HashMap<String, (u32, Ty)>>,
    extra_locals: Vec<ValType>,
    nparams: u32,
    instrs: Vec<Instruction<'static>>,
}

impl<'a> Cx<'a> {
    fn lookup(&self, name: &str) -> Option<(u32, Ty)> {
        for s in self.scopes.iter().rev() {
            if let Some(v) = s.get(name) {
                return Some(*v);
            }
        }
        None
    }
    fn alloc_local(&mut self, ty: Ty) -> u32 {
        let idx = self.nparams + self.extra_locals.len() as u32;
        self.extra_locals.push(valtype(ty));
        idx
    }
    fn emit(&mut self, i: Instruction<'static>) {
        self.instrs.push(i);
    }
}

fn compile_fn(f: &FnDecl, index: &HashMap<String, (u32, Ty)>) -> Result<Function, CompileError> {
    let mut params = HashMap::new();
    for (i, p) in f.params.iter().enumerate() {
        params.insert(p.name.name.clone(), (i as u32, scalar_ty(&p.ty).unwrap()));
    }
    let mut cx = Cx {
        index,
        scopes: vec![params],
        extra_locals: Vec::new(),
        nparams: f.params.len() as u32,
        instrs: Vec::new(),
    };
    let ret = scalar_ty(f.ret.as_ref().unwrap()).unwrap();
    let body_ty = compile_block(&f.body, &mut cx)?;
    if body_ty != ret {
        return Err(CompileError::Unsupported("a body whose value type differs from the declared return type".into()));
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
    let mut result: Option<Ty> = None;
    let n = b.stmts.len();
    for (i, stmt) in b.stmts.iter().enumerate() {
        let is_last = i + 1 == n;
        match stmt {
            Stmt::Let { name, value, .. } => {
                let ty = compile_expr(value, cx)?;
                let idx = cx.alloc_local(ty);
                cx.emit(Instruction::LocalSet(idx));
                cx.scopes.last_mut().unwrap().insert(name.name.clone(), (idx, ty));
            }
            Stmt::Expr(e) => {
                let ty = compile_expr(e, cx)?;
                if is_last {
                    result = Some(ty);
                } else {
                    cx.emit(Instruction::Drop);
                }
            }
            Stmt::Return { value, .. } => {
                match value {
                    Some(e) => {
                        let ty = compile_expr(e, cx)?;
                        cx.emit(Instruction::Return);
                        if is_last && result.is_none() {
                            result = Some(ty);
                        }
                    }
                    None => return Err(CompileError::Unsupported("`return` without a value".into())),
                }
            }
            Stmt::While { .. } | Stmt::Assign { .. } => {
                cx.scopes.pop();
                return Err(CompileError::Unsupported("`while`/assignment (Phase 3a)".into()));
            }
        }
    }
    cx.scopes.pop();
    result.ok_or_else(|| CompileError::Unsupported("a block whose value is Unit".into()))
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
            LitKind::Float(_) => Err(CompileError::Unsupported("Float literals (Phase 3a)".into())),
            LitKind::Str(_) => Err(CompileError::Unsupported("Str literals (Phase 3a)".into())),
        },
        Expr::Var { path, .. } => {
            if path.segs.len() == 1 {
                if let Some((idx, ty)) = cx.lookup(&path.segs[0].name) {
                    cx.emit(Instruction::LocalGet(idx));
                    return Ok(ty);
                }
            }
            Err(CompileError::Unsupported(format!("the name `{}` (not a local — consts/caps are Phase 3a+)", path.dotted())))
        }
        Expr::Unary { op, operand, .. } => {
            let ty = compile_expr(operand, cx)?;
            match op {
                UnOp::Neg if ty == Ty::I64 => {
                    // 0 - operand: emit 0 first, but operand is already on the stack. Compute via
                    // a temp: (operand) already pushed; multiply by -1.
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
            // We need the block type before emitting; compile the then-branch into a sub-buffer to
            // learn its type, then splice. Simpler: emit If(Empty) placeholder is not possible with
            // a result. So compile then first into cx (after emitting a placeholder If we patch).
            // Instead: peek the then type by compiling into a temporary Cx-less path is complex;
            // emit `If` with the then-branch's type discovered by compiling then-branch first.
            let then_ty = block_result_ty(then_)?;
            cx.emit(Instruction::If(wasm_encoder::BlockType::Result(valtype(then_ty))));
            let tt = compile_block(then_, cx)?;
            cx.emit(Instruction::Else);
            let et = compile_expr(else_, cx)?;
            cx.emit(Instruction::End);
            if tt != et {
                return Err(CompileError::Unsupported("`if` branches of differing WASM types".into()));
            }
            Ok(tt)
        }
        Expr::Call { callee, args, .. } => {
            let name = match &**callee {
                Expr::Var { path, .. } if path.segs.len() == 1 => path.segs[0].name.clone(),
                _ => return Err(CompileError::Unsupported("an indirect or builtin call (Phase 3a)".into())),
            };
            let (fidx, ret) = *cx
                .index
                .get(&name)
                .ok_or_else(|| CompileError::Unsupported(format!("a call to `{name}` (not a pure compilable function)")))?;
            for a in args {
                compile_expr(a, cx)?;
            }
            cx.emit(Instruction::Call(fidx));
            Ok(ret)
        }
        Expr::Block(b) => compile_block(b, cx),
        _ => Err(CompileError::Unsupported("this expression form (Phase 3a)".into())),
    }
}

/// Statically determine a block's WASM result type without emitting code (needed to type an `if`
/// before its then-branch is emitted). Only handles the pure fragment; errors otherwise.
fn block_result_ty(b: &Block) -> Result<Ty, CompileError> {
    match b.stmts.last() {
        Some(Stmt::Expr(e)) => expr_result_ty(e),
        Some(Stmt::Return { value: Some(e), .. }) => expr_result_ty(e),
        _ => Err(CompileError::Unsupported("a block whose value is Unit".into())),
    }
}

fn expr_result_ty(e: &Expr) -> Result<Ty, CompileError> {
    match e {
        Expr::Lit { kind, .. } => match kind {
            LitKind::Int(_) => Ok(Ty::I64),
            LitKind::Bool(_) => Ok(Ty::I32),
            _ => Err(CompileError::Unsupported("Float/Str (Phase 3a)".into())),
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
        // Var and Call types are context-dependent; the concrete emission path resolves them, and
        // the branch-type equality check catches any mismatch. Default the branch type to I64 for
        // typing purposes; if wrong, `compile_expr`'s equality check errors cleanly.
        Expr::Var { .. } | Expr::Call { .. } => Ok(Ty::I64),
        _ => Err(CompileError::Unsupported("this expression form (Phase 3a)".into())),
    }
}

fn compile_binary(op: BinOp, lhs: &Expr, rhs: &Expr, cx: &mut Cx) -> Result<Ty, CompileError> {
    let lt = compile_expr(lhs, cx)?;
    let rt = compile_expr(rhs, cx)?;
    use BinOp::*;
    // Arithmetic and comparison on Int (i64).
    if lt == Ty::I64 && rt == Ty::I64 {
        let (ins, ty): (Instruction<'static>, Ty) = match op {
            Add => (Instruction::I64Add, Ty::I64),
            Sub => (Instruction::I64Sub, Ty::I64),
            Mul => (Instruction::I64Mul, Ty::I64),
            Div => (Instruction::I64DivS, Ty::I64),
            Rem => (Instruction::I64RemS, Ty::I64),
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
    // Logical / equality on Bool (i32).
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
    Err(CompileError::Unsupported("a binary operator on mixed WASM types".into()))
}
