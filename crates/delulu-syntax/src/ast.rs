//! The Stage-1 AST (spec §4). Every node that can carry a diagnostic has a `Span`;
//! every expression and block has a `NodeId` so the checker attaches inferred
//! types and rows in side tables — the AST itself is never mutated after parse.

use delulu_diag::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

#[derive(Clone, Debug)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Path {
    pub segs: Vec<Ident>,
}

impl Path {
    pub fn span(&self) -> Span {
        let first = self.segs.first().expect("path has at least one segment").span;
        let last = self.segs.last().expect("path has at least one segment").span;
        first.to(last)
    }

    pub fn dotted(&self) -> String {
        self.segs.iter().map(|s| s.name.as_str()).collect::<Vec<_>>().join(".")
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    pub name: Path,
    pub imports: Vec<Import>,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub struct Import {
    /// `pub import` re-exports the target module's public items from this module (Stage 2, §2).
    pub public: bool,
    pub path: Path,
    pub alias: Option<Ident>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Item {
    Fn(FnDecl),
    Type(TypeDecl),
    Effect(EffectDecl),
    Const(ConstDecl),
    /// A `foreign "c" lib M { … }` block (Stage 4, spec §2).
    Foreign(ForeignDecl),
}

/// A `foreign <abi> lib <name> { … }` block. `name` is BOTH the nominal opaque lib-handle type
/// and the logical grant name; the block introduces no effect-row syntax — every foreign function
/// has the implicit row `!{ForeignCall}` (spec §2, §3 T-ForeignCall).
#[derive(Clone, Debug)]
pub struct ForeignDecl {
    pub public: bool,
    /// The ABI string; `"c"` is the only value in v0.4 (others → DL1308).
    pub abi: String,
    /// Span of the ABI string literal (for DL1308).
    pub abi_span: Span,
    pub name: Ident,
    pub fns: Vec<ForeignFn>,
    pub id: NodeId,
    pub span: Span,
}

/// A single `fn name(params) -> ret` inside a foreign block. There is deliberately no effect-row:
/// the row is implicitly `!{ForeignCall}`, always (spec §2 EBNF).
#[derive(Clone, Debug)]
pub struct ForeignFn {
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FnDecl {
    pub public: bool,
    pub name: Ident,
    /// Type variables and row variables share the bracket; a variable used
    /// after `!` has row kind — kinds are inferred and must be consistent (DL0410).
    pub generics: Vec<Ident>,
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,
    /// `None` means pure `!{}` (Stage-1 invariant 4).
    pub row: Option<RowExpr>,
    pub body: Block,
    pub id: NodeId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: Ident,
    pub ty: TypeExpr,
}

#[derive(Clone, Debug)]
pub enum TypeExpr {
    /// `Int`, `List[T]`, `Cap[FsRead]`, `Secret[Str]`, `demo.util.Point`
    Named { path: Path, args: Vec<TypeExpr>, span: Span },
    /// `fn(T, U) -> V ! {Read | e}` — rows never erase from function types.
    Fn { params: Vec<TypeExpr>, ret: Option<Box<TypeExpr>>, row: Option<RowExpr>, span: Span },
}

impl TypeExpr {
    pub fn span(&self) -> Span {
        match self {
            TypeExpr::Named { span, .. } | TypeExpr::Fn { span, .. } => *span,
        }
    }
}

/// `! {Read, Net | e}` or `! e`. `effects` may be empty (`!{}` explicit purity;
/// tail-only rows are written `! e`).
#[derive(Clone, Debug)]
pub struct RowExpr {
    pub effects: Vec<Path>,
    pub tail: Option<Ident>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct TypeDecl {
    pub public: bool,
    pub name: Ident,
    pub generics: Vec<Ident>,
    pub kind: TypeDeclKind,
    pub id: NodeId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum TypeDeclKind {
    Record(Vec<FieldDef>),
    Sum(Vec<VariantDef>),
    Alias(TypeExpr),
}

#[derive(Clone, Debug)]
pub struct FieldDef {
    pub name: Ident,
    pub ty: TypeExpr,
}

#[derive(Clone, Debug)]
pub struct VariantDef {
    pub name: Ident,
    pub fields: Vec<TypeExpr>,
}

#[derive(Clone, Debug)]
pub struct EffectDecl {
    pub public: bool,
    pub name: Ident,
    pub id: NodeId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ConstDecl {
    pub public: bool,
    pub name: Ident,
    pub ty: Option<TypeExpr>,
    pub value: Expr,
    pub id: NodeId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub id: NodeId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Let { name: Ident, ty: Option<TypeExpr>, value: Expr, mutable: bool, span: Span },
    Assign { target: LValue, value: Expr, span: Span },
    While { cond: Expr, body: Block, span: Span },
    Return { value: Option<Expr>, span: Span },
    Expr(Expr),
}

#[derive(Clone, Debug)]
pub enum LValue {
    Var(Ident),
    Field(Box<LValue>, Ident),
    Index(Box<LValue>, Expr),
}

impl LValue {
    pub fn span(&self) -> Span {
        match self {
            LValue::Var(i) => i.span,
            LValue::Field(base, name) => base.span().to(name.span),
            LValue::Index(base, _) => base.span(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LitKind {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Lit { kind: LitKind, id: NodeId, span: Span },
    /// A variable reference. The parser produces single-segment paths here;
    /// dotted chains are `Field`/`Method` and are resolved by the checker
    /// (module member vs. record field), Go-selector style.
    Var { path: Path, id: NodeId, span: Span },
    List { items: Vec<Expr>, id: NodeId, span: Span },
    Record { path: Path, fields: Vec<(Ident, Expr)>, id: NodeId, span: Span },
    Call { callee: Box<Expr>, args: Vec<Expr>, id: NodeId, span: Span },
    Method { recv: Box<Expr>, name: Ident, args: Vec<Expr>, id: NodeId, span: Span },
    Field { recv: Box<Expr>, name: Ident, id: NodeId, span: Span },
    Index { recv: Box<Expr>, index: Box<Expr>, id: NodeId, span: Span },
    Unary { op: UnOp, operand: Box<Expr>, id: NodeId, span: Span },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>, id: NodeId, span: Span },
    If { cond: Box<Expr>, then_: Block, else_: Option<Box<Expr>>, id: NodeId, span: Span },
    Match { scrutinee: Box<Expr>, arms: Vec<Arm>, id: NodeId, span: Span },
    Lambda {
        params: Vec<Param>,
        ret: Option<TypeExpr>,
        row: Option<RowExpr>,
        body: Block,
        id: NodeId,
        span: Span,
    },
    /// `expr?` — Result propagation; pure control flow.
    Try { inner: Box<Expr>, id: NodeId, span: Span },
    /// Needed for `else { … }` branches and block-bodied match arms
    /// (implementation-forced addition, spec §4).
    Block(Block),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Lit { span, .. }
            | Expr::Var { span, .. }
            | Expr::List { span, .. }
            | Expr::Record { span, .. }
            | Expr::Call { span, .. }
            | Expr::Method { span, .. }
            | Expr::Field { span, .. }
            | Expr::Index { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. }
            | Expr::If { span, .. }
            | Expr::Match { span, .. }
            | Expr::Lambda { span, .. }
            | Expr::Try { span, .. } => *span,
            Expr::Block(b) => b.span,
        }
    }

    pub fn id(&self) -> NodeId {
        match self {
            Expr::Lit { id, .. }
            | Expr::Var { id, .. }
            | Expr::List { id, .. }
            | Expr::Record { id, .. }
            | Expr::Call { id, .. }
            | Expr::Method { id, .. }
            | Expr::Field { id, .. }
            | Expr::Index { id, .. }
            | Expr::Unary { id, .. }
            | Expr::Binary { id, .. }
            | Expr::If { id, .. }
            | Expr::Match { id, .. }
            | Expr::Lambda { id, .. }
            | Expr::Try { id, .. } => *id,
            Expr::Block(b) => b.id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Arm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Pattern {
    Wildcard(Span),
    Lit(LitKind, Span),
    /// A single lowercase-or-uppercase identifier; the checker decides whether
    /// it binds or names a variant of the scrutinee's type (Rust-style resolution).
    Bind(Ident),
    Variant { path: Path, fields: Vec<Pattern>, span: Span },
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard(s) | Pattern::Lit(_, s) | Pattern::Variant { span: s, .. } => *s,
            Pattern::Bind(i) => i.span,
        }
    }
}
