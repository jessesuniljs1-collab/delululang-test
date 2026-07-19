//! The Stage-1 AST (spec §4). Every node that can carry a diagnostic has a `Span`;
//! every expression and block has a `NodeId` so the checker attaches inferred
//! types and rows in side tables — the AST itself is never mutated after parse.

use delulu_diag::Span;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Module {
    pub name: Path,
    pub imports: Vec<Import>,
    pub items: Vec<Item>,
    /// Execution-mode hints on the module header (Stage 10, spec §2.2). Hints, never semantics
    /// (invariant 45): nothing downstream of the parser may branch on them except a scheduler.
    /// Serde default + skip keeps every attribute-free artifact byte-identical to 1.0.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attrs: Vec<Attribute>,
}

/// `attribute = "@" IDENT [ "(" STRING ")" ]` (Stage 10, spec §2.2 — a reserved 1.0 activation).
/// v1.x-defined names: `aot`, `interpret`, `jit`, `inline` (arg `"never"`|`"always"`). All are
/// HINTS: the scheduler may ignore them; none changes semantics; unknown names are DL1901 —
/// there is no silent vendor attribute space.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attribute {
    pub name: Ident,
    pub arg: Option<String>,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Import {
    /// `pub import` re-exports the target module's public items from this module (Stage 2, §2).
    pub public: bool,
    pub path: Path,
    pub alias: Option<Ident>,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Item {
    Fn(FnDecl),
    Type(TypeDecl),
    Effect(EffectDecl),
    Const(ConstDecl),
    /// A `foreign "c" lib M { … }` block (Stage 4, spec §2).
    Foreign(ForeignDecl),
    /// An `actor A { … }` declaration (Stage 7, spec §2).
    Actor(ActorDecl),
    /// A `test "name" [! {row}] { … }` block (Stage 8, spec §2) — checked like a
    /// Unit-returning fn, compiled out of non-test builds (invariant 41's carrier).
    Test(TestDecl),
}

/// `test STRING [effect_row] block` — `test` is a keyword only in item position (contextual).
/// The body receives `test_root: Root` scoped by the test manifest (§5.1; the runner is 8g).
/// Omitted row = pure. Tests are never `pub`: they are not exported items.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestDecl {
    pub name: String,
    /// Span of the name string — where "declared row is here" diagnostics point.
    pub name_span: Span,
    pub row: Option<RowExpr>,
    pub body: Block,
    pub id: NodeId,
    pub span: Span,
}

/// A reference capability (Stage 7, spec §3 — Pony's system, adopted not redesigned).
/// The deny-property definitions are normative; `alias`/`sendable`/viewpoint adaptation live in
/// the checker (`delulu-check`), not here — the AST only records what was written.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Rcap {
    Iso,
    Trn,
    Ref,
    Val,
    Box_,
    Tag,
}

impl Rcap {
    /// The source spelling. The six words are RESERVED identifiers recognized contextually in
    /// type position (spec §2; build-order deviation 5).
    pub fn from_name(s: &str) -> Option<Rcap> {
        Some(match s {
            "iso" => Rcap::Iso,
            "trn" => Rcap::Trn,
            "ref" => Rcap::Ref,
            "val" => Rcap::Val,
            "box" => Rcap::Box_,
            "tag" => Rcap::Tag,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Rcap::Iso => "iso",
            Rcap::Trn => "trn",
            Rcap::Ref => "ref",
            Rcap::Val => "val",
            Rcap::Box_ => "box",
            Rcap::Tag => "tag",
        }
    }
}

/// An `actor A { fields, new, behaviors, fns }` declaration (spec §2, §2.1). Exactly one `new`
/// per actor — the parser enforces the count and synthesizes an empty one on error so the
/// checker always has a constructor to look at.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActorDecl {
    pub public: bool,
    pub name: Ident,
    pub generics: Vec<Ident>,
    pub fields: Vec<ActorField>,
    pub ctor: CtorDecl,
    pub behaviors: Vec<BehaviorDecl>,
    pub fns: Vec<FnDecl>,
    pub id: NodeId,
    pub span: Span,
    /// Execution-mode hints (Stage 10, spec §2.2); see [`Attribute`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attrs: Vec<Attribute>,
    /// Stage 10 (10c, spec §3): `actor A(mailbox = N)` — this actor's mailbox bound. `None`
    /// (and no manifest default) = unbounded, the 1.0 behavior; bounding is opt-in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mailbox: Option<u64>,
}

/// `("let" | "var") name ":" type` — no initializer in the grammar; fields are assigned in
/// `new`. `var` fields are actor-internal state (legal: owned, isolated, reached only via the
/// actor's own turn — NOT the banned module-level ambient `var`, spec §4 T-Actor).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActorField {
    pub mutable: bool,
    pub name: Ident,
    pub ty: TypeExpr,
    pub span: Span,
}

/// `new(params) [row] { … }` — construction is a send to the new actor, so parameter
/// sendability rules match behaviors (T-Ctor).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CtorDecl {
    pub params: Vec<Param>,
    pub row: Option<RowExpr>,
    pub body: Block,
    pub id: NodeId,
    pub span: Span,
}

/// `be name(params) [row] { … }` — behaviors have no return type: they yield `Unit` at the
/// send site (a written return type is DL1606 with an exact delete repair).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BehaviorDecl {
    pub name: Ident,
    pub params: Vec<Param>,
    pub row: Option<RowExpr>,
    pub body: Block,
    pub id: NodeId,
    pub span: Span,
}

/// A `foreign <abi> lib <name> { … }` block. `name` is BOTH the nominal opaque lib-handle type
/// and the logical grant name; the block introduces no effect-row syntax — every foreign function
/// has the implicit row `!{ForeignCall}` (spec §2, §3 T-ForeignCall).
#[derive(Clone, Debug, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ForeignFn {
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    /// Execution-mode hints (Stage 10, spec §2.2); see [`Attribute`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attrs: Vec<Attribute>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Param {
    pub name: Ident,
    pub ty: TypeExpr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TypeExpr {
    /// `Int`, `List[T]`, `Cap[FsRead]`, `Secret[Str]`, `demo.util.Point`
    Named { path: Path, args: Vec<TypeExpr>, span: Span },
    /// `fn(T, U) -> V ! {Read | e}` — rows never erase from function types.
    Fn { params: Vec<TypeExpr>, ret: Option<Box<TypeExpr>>, row: Option<RowExpr>, span: Span },
    /// `iso T`, `val fn(val T) -> Unit ! e`, … — a reference-capability prefix (Stage 7,
    /// spec §2). One wrapper variant instead of the spec sketch's `TypeExprR` struct
    /// (build-order deviation 1): the mechanism — an optional rcap at any type position,
    /// including fn-type parameters (spec §8 writes `val fn(val T)`) — is what is normative.
    Rcap { rcap: Rcap, inner: Box<TypeExpr>, span: Span },
}

impl TypeExpr {
    pub fn span(&self) -> Span {
        match self {
            TypeExpr::Named { span, .. } | TypeExpr::Fn { span, .. } | TypeExpr::Rcap { span, .. } => *span,
        }
    }

    /// The written rcap prefix, if any (`None` means the spec §2 default rule applies —
    /// resolved by the checker, never here).
    pub fn written_rcap(&self) -> Option<Rcap> {
        match self {
            TypeExpr::Rcap { rcap, .. } => Some(*rcap),
            _ => None,
        }
    }

    /// The type under any rcap prefix.
    pub fn core(&self) -> &TypeExpr {
        match self {
            TypeExpr::Rcap { inner, .. } => inner,
            other => other,
        }
    }
}

/// `! {Read, Net | e}` or `! e`. `effects` may be empty (`!{}` explicit purity;
/// tail-only rows are written `! e`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RowExpr {
    pub effects: Vec<Path>,
    pub tail: Option<Ident>,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TypeDecl {
    pub public: bool,
    pub name: Ident,
    pub generics: Vec<Ident>,
    pub kind: TypeDeclKind,
    pub id: NodeId,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TypeDeclKind {
    Record(Vec<FieldDef>),
    Sum(Vec<VariantDef>),
    Alias(TypeExpr),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: Ident,
    pub ty: TypeExpr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VariantDef {
    pub name: Ident,
    pub fields: Vec<TypeExpr>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EffectDecl {
    pub public: bool,
    pub name: Ident,
    pub id: NodeId,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstDecl {
    pub public: bool,
    pub name: Ident,
    pub ty: Option<TypeExpr>,
    pub value: Expr,
    pub id: NodeId,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub id: NodeId,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Stmt {
    Let { name: Ident, ty: Option<TypeExpr>, value: Expr, mutable: bool, span: Span },
    Assign { target: LValue, value: Expr, span: Span },
    While { cond: Expr, body: Block, span: Span },
    Return { value: Option<Expr>, span: Span },
    Expr(Expr),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LitKind {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    /// `spawn A(args)` — creates an actor; type `tag A`, row `{Async} ∪ row(A.new)`
    /// (Stage 7, T-Spawn).
    Spawn { actor: Path, args: Vec<Expr>, id: NodeId, span: Span },
    /// `consume x` — yields `x`'s full rcap and kills the binding (locals/params only in
    /// v0.7; any later use is DL1602).
    Consume { name: Ident, id: NodeId, span: Span },
    /// `recover [rcap] { … }` — checks the block in a restricted environment and lifts the
    /// result to `iso` (default) or `val` (Stage 7, spec §3).
    Recover { target: Option<Rcap>, body: Block, id: NodeId, span: Span },
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
            | Expr::Try { span, .. }
            | Expr::Spawn { span, .. }
            | Expr::Consume { span, .. }
            | Expr::Recover { span, .. } => *span,
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
            | Expr::Try { id, .. }
            | Expr::Spawn { id, .. }
            | Expr::Consume { id, .. }
            | Expr::Recover { id, .. } => *id,
            Expr::Block(b) => b.id,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Arm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
