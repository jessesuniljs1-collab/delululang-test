# DeluluLang — Stage 1 Implementation Specification

**Version:** 0.1 ("Skeleton"). **Status:** Committed — hand to a coding model and build.
**Governing document:** `CONSTITUTION.md`. Where they conflict, the constitution wins.

---

## 0. Scope

Stage 1 ships a working `delulu` CLI containing a lexer, parser, name resolver, **type and effect
checker**, tree-walking interpreter, REPL, machine-readable diagnostics, and the human-in-the-loop
authority grant flow. The authority/effect skeleton is present from the first commit, because it
is the one thing that can never be retrofitted.

**In scope (must ship):**
- Full lexical + grammar per §2–§3; AST per §4.
- Types: `Int`, `Float`, `Bool`, `Str`, `Unit`, `List[T]`, `Option[T]`, `Result[T, E]`,
  records, sum types, function types, generics with **effect-row polymorphism**, `Cap[R]`,
  `Secret[T]`, `Root`.
- Core effects: `Read`, `Write`, `Net`, `Clock`, `Rand`, `Declassify` (audit rule R-2); user-declared effects.
- Capability standard library: `FsRead`, `FsWrite`, `Console`, `Http`, `Clock`, `Rand`,
  `Declassify`; the `Root` object; attenuation.
- Authority manifest (`delulu.toml`) + CLI grants + interactive grant prompt.
- `delulu check | run | repl | authority`, all with `--json` diagnostics.

**Designed here, implemented in later stages:** plugin loading (§8 — types and manifest format
are fixed now; runtime lands Stage 6), FFI (`Cap[Foreign]`/`ForeignCall` reserved), actors and
reference capabilities (keywords reserved), `Async`, WASM backend (§9.4 decides it now).

**Non-goals for Stage 1:** performance, concurrency, package resolution (single file + local
module tree only), string interpolation, traits/interfaces, pattern-match exhaustiveness beyond
`Option`/`Result`/user sums (which *are* checked).

---

## 1. Invariants that must hold from the first commit

1. **No ambient authority.** No global `print`, no global filesystem, no global clock. Every
   primitive effect is a method on a capability value.
2. **Capabilities are unforgeable.** `Cap[R]` has no literal, no constructor, no cast. Values of
   type `Cap[R]` originate only from `Root` (injected into `main`) or by attenuation of another
   capability.
3. **Effect rows never erase.** Function values keep their rows in their types everywhere —
   parameters, returns, fields, lists.
4. **Omitted row = pure.** A named function with no `!{...}` clause is checked as `!{}`.
5. **No module-level mutable state.** Module-level `var` is a compile error (DL0305).
6. **Secrets don't unwrap silently.** `Secret[T]` participates in no coercion; only
   `verify`/`expose` eliminate it.
7. **The runtime re-checks scope.** Every capability operation validates its scope (path prefix,
   host allowlist) at runtime even though the type checker already verified the kind. Two layers,
   always.
8. **Reserved words stay reserved** (§2.3) so Stages 4–7 break no Stage-1 code.
9. **Every diagnostic has a stable code, a span, and (where applicable) a typed repair.** English
   text is presentation; the JSON is the contract.

---

## 2. Lexical structure (token model)

Source is UTF-8. Identifiers are ASCII in v0.1 (revisit post-1.0).

### 2.1 Tokens

```
IDENT     = [A-Za-z_][A-Za-z0-9_]*        (minus keywords)
INT       = [0-9][0-9_]* | 0x[0-9a-fA-F_]+
FLOAT     = [0-9][0-9_]* "." [0-9][0-9_]* ( ("e"|"E") ("+"|"-")? [0-9]+ )?
STRING    = '"' ( char | escape )* '"'
            escape = \n \t \r \\ \" \0 \u{1-6 hex}
COMMENT   = "//" to end of line | "/*" ... "*/" (nesting allowed) | "///" doc comment (attached)
```

Punctuation/operators (each a distinct token):
`( ) { } [ ] , . : ; -> => ! ? | = == != < <= > >= + - * / % && || _ @`

### 2.2 Statement termination (Go-style automatic insertion)

The lexer inserts a statement terminator at a newline **iff** the previous token can end a
statement: `IDENT INT FLOAT STRING true false return ) ] } ?`. Explicit `;` is always allowed.
This keeps the grammar regular for both humans and LLMs without semicolon noise.
Consequence (shared with Go): `else` must sit on the same line as its `if` block's closing brace
(`} else {`), because a newline after `}` inserts a terminator. The formatter enforces this.
A block comment containing a newline counts as a newline for insertion purposes.

### 2.3 Keywords

Active in Stage 1:
```
fn let var if else while return match module import pub type effect true false
```
**Reserved** (compile error DL0106 "reserved for a future stage"):
```
actor async await spawn iso val ref box tag trn plugin foreign secret cap
for in break continue trait impl where pure
```
Precision (implementation-forced clarification): reserved words are rejected where a **new name
is declared** — function/type/effect/variant/field/generic names, parameters, `let`/`var`
bindings, module path segments. They remain legal as **member names after `.`** — which is how
`root.secret(…)` coexists with `secret` being reserved. The lexer emits them as ordinary
identifier tokens; the parser enforces DL0106 at declaration sites.

### 2.4 Predefined (non-keyword, resolvable) names

`Int Float Bool Str Unit List Option Some None Result Ok Err Cap Secret Root` and effect names
`Read Write Net Clock Rand Declassify` (+ reserved effect names `Async ForeignCall Load Actuate
Alloc`).
These live in an implicit prelude module, not the keyword table.

---

## 3. Grammar (EBNF)

`{X}` = zero or more, `[X]` = optional, `|` = alternative, terminals quoted. `term` is `;` or an
inserted terminator (§2.2).

```ebnf
file          = module_decl , { import_decl } , { item } ;
module_decl   = "module" , path , term ;
import_decl   = "import" , path , [ "as" , IDENT ] , term ;
path          = IDENT , { "." , IDENT } ;

item          = [ "pub" ] , ( fn_decl | type_decl | effect_decl | const_decl ) ;

fn_decl       = "fn" , IDENT , [ generics ] , "(" , [ params ] , ")" ,
                [ "->" , type ] , [ effect_row ] , block ;
generics      = "[" , IDENT , { "," , IDENT } , "]" ;      (* type vars and row vars share the
                                                              bracket; a var used after "!" is a
                                                              row var — kinds are inferred and
                                                              must be consistent (DL0410) *)
params        = param , { "," , param } ;
param         = IDENT , ":" , type ;

effect_row    = "!" , ( "{" [ effects ] "}" | IDENT ) ;
effects       = effect_name , { "," , effect_name } , [ "|" , IDENT ] ;
effect_name   = path ;

const_decl    = "let" , IDENT , [ ":" , type ] , "=" , expr , term ;   (* module level: pure only *)

type_decl     = "type" , IDENT , [ generics ] ,
                ( "{" , field , { "," , field } , [ "," ] , "}"        (* record  *)
                | "=" , variant , { "|" , variant }                    (* sum     *)
                | "=" , type ) ;                                       (* alias   *)
field         = IDENT , ":" , type ;
variant       = IDENT , [ "(" , type , { "," , type } , ")" ] ;

effect_decl   = "effect" , IDENT , term ;

type          = "fn" , "(" , [ type , { "," , type } ] , ")" , [ "->" , type ] , [ effect_row ]
              | path , [ "[" , type , { "," , type } , "]" ]
              | "(" , type , ")" ;

block         = "{" , { stmt } , "}" ;
stmt          = let_stmt | var_stmt | assign_stmt | while_stmt | return_stmt | expr_stmt ;
let_stmt      = "let" , IDENT , [ ":" , type ] , "=" , expr , term ;
var_stmt      = "var" , IDENT , [ ":" , type ] , "=" , expr , term ;
assign_stmt   = lvalue , "=" , expr , term ;
lvalue        = IDENT , { "." , IDENT | "[" , expr , "]" } ;
while_stmt    = "while" , expr , block ;
return_stmt   = "return" , [ expr ] , term ;
expr_stmt     = expr , term ;

(* Block value: the value of the final expr_stmt if the block ends with one; otherwise Unit.
   This makes `if` usable as an expression without Rust's tail-expression special casing. *)

expr          = or_expr ;
or_expr       = and_expr , { "||" , and_expr } ;
and_expr      = eq_expr , { "&&" , eq_expr } ;
eq_expr       = cmp_expr , [ ( "==" | "!=" ) , cmp_expr ] ;
cmp_expr      = add_expr , [ ( "<" | "<=" | ">" | ">=" ) , add_expr ] ;
add_expr      = mul_expr , { ( "+" | "-" ) , mul_expr } ;
mul_expr      = unary , { ( "*" | "/" | "%" ) , unary } ;
unary         = ( "-" | "!" ) , unary | postfix ;
postfix       = primary , { call | method | field_access | index | try } ;
call          = "(" , [ expr , { "," , expr } ] , ")" ;
method        = "." , IDENT , "(" , [ expr , { "," , expr } ] , ")" ;
field_access  = "." , IDENT ;
index         = "[" , expr , "]" ;
try           = "?" ;                                       (* Result propagation *)

primary       = INT | FLOAT | STRING | "true" | "false"
              | path                                        (* var, or variant like Some *)
              | path , "{" , [ field_init , { "," , field_init } ] , "}"   (* record literal.
                     Restriction (implementation-forced): record literals are NOT parsed in
                     `if`/`while` conditions or `match` scrutinees — parenthesize there
                     (`if (P { x: 1 } == q) { … }`). This resolves the `path {` vs block
                     ambiguity exactly as Rust does. *)
              | "(" , expr , ")"
              | if_expr | match_expr | lambda | list_lit ;
field_init    = IDENT , ":" , expr ;
list_lit      = "[" , [ expr , { "," , expr } ] , "]" ;
if_expr       = "if" , expr , block , [ "else" , ( if_expr | block ) ] ;
match_expr    = "match" , expr , "{" , arm , { "," , arm } , [ "," ] , "}" ;
arm           = pattern , "=>" , ( expr | block ) ;
pattern       = "_" | INT | STRING | "true" | "false"
              | IDENT                                       (* binding *)
              | path , [ "(" , pattern , { "," , pattern } , ")" ] ;   (* variant *)
lambda        = "fn" , "(" , [ params ] , ")" , [ "->" , type ] , [ effect_row ] , block ;
```

### 3.1 Reference program (the Stage-1 demo)

Implementation-forced correction (2026-07-05): the earlier draft used `?` inside `main`, but
`main` returns `Unit` and T-Try (§5.8/§6.2) requires `?` to sit in a `Result`-returning function.
The example now demonstrates `?` in a `Result`-returning helper (`read_config`) and uses `match`
in `main`, so the reference program type-checks under the rules it illustrates. The canonical
copy is `examples/demo.delulu`; this block mirrors it.

```delulu
module demo

// Pure: omitted row means !{}. The checker PROVES fib performs no effect.
fn fib(n: Int) -> Int {
  if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

// Effectful: the row declares Write; the Cap[Console] param is the permission.
fn greet(out: Cap[Console], name: Str) ! {Write} {
  out.println("hello, " + name)
}

// Higher-order with row polymorphism: apply's row is exactly f's row.
fn apply[T, U, e](f: fn(T) -> U ! e, x: T) -> U ! e {
  f(x)
}

// `?` demonstrated where it is well-typed: a Result-returning function whose error
// type matches read_text's.
fn read_config(fs: Cap[FsRead]) -> Result[Str, IoErr] ! {Read} {
  let text = fs.read_text("app.txt")?
  Ok(text)
}

fn main(root: Root) ! {Write, Read} {
  let out = root.console()
  greet(out, "delulu world")
  out.println("fib(10) = " + str(fib(10)))

  let doubled = apply(fn(x: Int) -> Int { x * 2 }, 21)
  out.println("apply = " + str(doubled))

  let fs = root.fs_read("./config")          // attenuated: read-only, this subtree
  match read_config(fs) {
    Ok(text) => out.println(text),
    Err(_) => out.println("(no config found)")
  }

  let key = root.secret("API_KEY")           // Secret[Str]
  // out.println(key)                         // DL0602: Secret[Str] is not Str
  // out.println(str(key))                    // DL0604: opaque type cannot be stringified
}
```

---

## 4. AST (Rust definitions — normative shape)

```rust
pub struct Span { pub file: FileId, pub start: u32, pub end: u32 }   // byte offsets
pub struct NodeId(pub u32);                                          // unique per compilation
pub struct Ident { pub name: Symbol, pub span: Span }
pub struct Path  { pub segs: Vec<Ident> }

pub struct Module { pub name: Path, pub imports: Vec<Import>, pub items: Vec<Item> }
pub struct Import { pub path: Path, pub alias: Option<Ident>, pub span: Span }

pub enum Item { Fn(FnDecl), Type(TypeDecl), Effect(EffectDecl), Const(ConstDecl) }

pub struct FnDecl {
    pub public: bool,
    pub name: Ident,
    pub generics: Vec<Ident>,             // type vars + row vars, kind inferred
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,            // None => Unit
    pub row: Option<RowExpr>,             // None => pure !{}
    pub body: Block,
    pub id: NodeId, pub span: Span,
}
pub struct Param { pub name: Ident, pub ty: TypeExpr }

pub enum TypeExpr {
    Named { path: Path, args: Vec<TypeExpr>, span: Span },     // Int, List[T], Cap[FsRead]
    Fn { params: Vec<TypeExpr>, ret: Box<TypeExpr>, row: RowExpr, span: Span },
}
pub struct RowExpr { pub effects: Vec<Path>, pub tail: Option<Ident>, pub span: Span }

pub struct TypeDecl { pub public: bool, pub name: Ident, pub generics: Vec<Ident>,
                      pub kind: TypeDeclKind, pub id: NodeId, pub span: Span }
pub enum TypeDeclKind {
    Record(Vec<FieldDef>),
    Sum(Vec<VariantDef>),
    Alias(TypeExpr),
}
pub struct FieldDef   { pub name: Ident, pub ty: TypeExpr }
pub struct VariantDef { pub name: Ident, pub fields: Vec<TypeExpr> }
pub struct EffectDecl { pub public: bool, pub name: Ident, pub id: NodeId, pub span: Span }
pub struct ConstDecl  { pub public: bool, pub name: Ident, pub ty: Option<TypeExpr>,
                        pub value: Expr, pub id: NodeId, pub span: Span }

pub struct Block { pub stmts: Vec<Stmt>, pub id: NodeId, pub span: Span }
pub enum Stmt {
    Let    { name: Ident, ty: Option<TypeExpr>, value: Expr, mutable: bool, span: Span },
    Assign { target: LValue, value: Expr, span: Span },
    While  { cond: Expr, body: Block, span: Span },
    Return { value: Option<Expr>, span: Span },
    Expr   (Expr),
}
pub enum LValue { Var(Ident), Field(Box<LValue>, Ident), Index(Box<LValue>, Expr) }

pub enum Expr {
    Lit    { kind: LitKind, id: NodeId, span: Span },              // Int/Float/Str/Bool
    Var    { path: Path, id: NodeId, span: Span },
    List   { items: Vec<Expr>, id: NodeId, span: Span },
    Record { path: Path, fields: Vec<(Ident, Expr)>, id: NodeId, span: Span },
    Call   { callee: Box<Expr>, args: Vec<Expr>, id: NodeId, span: Span },
    Method { recv: Box<Expr>, name: Ident, args: Vec<Expr>, id: NodeId, span: Span },
    Field  { recv: Box<Expr>, name: Ident, id: NodeId, span: Span },
    Index  { recv: Box<Expr>, index: Box<Expr>, id: NodeId, span: Span },
    Unary  { op: UnOp, operand: Box<Expr>, id: NodeId, span: Span },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>, id: NodeId, span: Span },
    If     { cond: Box<Expr>, then_: Block, else_: Option<Box<Expr>>, id: NodeId, span: Span },
    Match  { scrutinee: Box<Expr>, arms: Vec<Arm>, id: NodeId, span: Span },
    Lambda { params: Vec<Param>, ret: Option<TypeExpr>, row: Option<RowExpr>,
             body: Block, id: NodeId, span: Span },
    Try    { inner: Box<Expr>, id: NodeId, span: Span },           // expr?
    Block  (Block),   // added (implementation-forced): needed for `else { … }` branches and
                      // block-bodied match arms; not a primary production on its own
}
pub struct Arm { pub pattern: Pattern, pub body: Expr, pub span: Span }
pub enum Pattern { Wildcard(Span), Lit(LitKind, Span), Bind(Ident),
                   Variant { path: Path, fields: Vec<Pattern>, span: Span } }
```

Every node that can carry a diagnostic has a `Span`; every expression has a `NodeId` so the type
checker can attach inferred types/rows in side tables (`HashMap<NodeId, Type>`,
`HashMap<NodeId, Row>`) — never mutate the AST.

---

## 5. Types, names, and inference

- **Name resolution** is a separate pass producing a symbol table; forward references between
  top-level items are legal within a module. Modules form a tree mirroring directories
  (`import demo.util` → `./demo/util.delulu` relative to the root file); cycles are DL0304.
- **Type checking is bidirectional**: signatures are explicit (declared param/return types on all
  named functions), so checking is mostly propagation; local `let` types and lambda rows are
  inferred by unification. Generics are checked per call site by unification (no turbofish, no
  explicit instantiation in v0.1).
- **Generic variables** in one bracket: a variable used after `!` anywhere in the signature has
  row kind; used in type position, type kind; both uses → DL0410.
- **No subtyping** except row width subsumption (§6.2). No implicit numeric coercions
  (`Int` + `Float` is DL0402; use `float(i)` / `int(f)`).
- Records are nominal; structural comparison only via `==` on identical nominal types.
- `match` on sums must be exhaustive (DL0407) — this is what makes `Result`-based errors safe.

---

## 6. The authority/effect core (the heart — hand-write, never delegate)

### 6.1 Representation

```rust
pub struct Row { pub effects: BTreeSet<EffectId>, pub tail: Option<RowVar> }   // {Net, Read | e}
pub enum Type {
    Int, Float, Bool, Str, Unit,
    List(Box<Type>), Option_(Box<Type>), Result_(Box<Type>, Box<Type>),
    Record(TypeDefId, Vec<Type>), Sum(TypeDefId, Vec<Type>),
    Fn { params: Vec<Type>, ret: Box<Type>, row: Row },
    Cap(ResourceKind),                    // FsRead | FsWrite | Console | Http | ClockRes |
                                          // RandRes | Declassify | PluginHost | Foreign
    Secret(Box<Type>),
    Root,
    Var(TypeVar),
}
```

### 6.2 Typing judgment

`Γ ⊢ e : τ ! ε` — under environment Γ, expression `e` has type τ and may perform effects ε.
Key rules (informal but binding):

- **T-Lit / T-Var:** literals and variable reads are pure: `ε = {}`.
- **T-Call:** if `callee : fn(τ₁..τₙ) -> τᵣ ! ε_f` and each `argᵢ : τᵢ ! εᵢ`, then the call has
  type `τᵣ` and effects `ε₁ ∪ … ∪ εₙ ∪ ε_f` (after instantiating type and row variables by
  unification).
- **T-CapOp:** each capability method has an intrinsic effect from the primitive table (§7.3);
  `fs.read_text(p)` contributes `{Read}`. **Primitive effects arise only here.** User-declared
  effects arise only from functions that declare them.
- **T-Fn (declaration check):** check the body, obtaining `ε_body`. Require
  `ε_body ⊆ row_declared` (with row variables of the signature treated as opaque). If not:
  DL0501 with an `add_effect_to_row` repair, flagged `authority_widening: true`. Declaring more
  than the body performs is legal (width subsumption) but produces warning DL0502 (unused
  declared effect) with a `remove_effect_from_row` repair — narrowing, therefore auto-applicable.
- **T-Lambda:** row inferred as exactly `ε_body` unless annotated (then checked like T-Fn).
- **T-If / T-Match / T-While:** effects are the union of all branches (static may-perform
  over-approximation — a branch not taken at runtime still counts; this conservatism is the
  guarantee, not a bug).
- **T-Try (`e?`):** `e : Result[τ, ε_t] ! ε` in a function returning `Result[_, ε_t']` with
  `ε_t = ε_t'`; yields `τ ! ε`. Pure control flow — no effect.
- **Row unification:** closed rows unify iff equal sets; `{ℓ̄ | ρ}` unifies against a row
  containing at least `ℓ̄`, binding ρ to the remainder. Occurs check applies. This is deliberately
  simpler than full Rémy-style rows: effect labels are a set (no duplicates, no order), and v0.1
  allows **at most one row variable per signature** (DL0503 otherwise) — enough for `map`/`apply`
  composition, small enough to hand-verify.
- **R-3 (single subsumption site; invariant unification — soundness-critical, see
  `SOUNDNESS_AUDIT.md` F-3):** there is **no subsumption inside unification**. Rows unify by
  equality (plus row-variable binding); subsumption `ε_body ⊆ ε_declared` exists at exactly two
  sites — the T-Fn declaration check and the row-annotated-lambda check. Parameter-position rows
  are therefore never widened or narrowed implicitly; the contravariance exploit in the audit must
  stay a rejection test forever. Workaround for legitimate row coercion: eta-expansion.
- **R-3b (no union-merge):** a row variable receiving conflicting closed bindings is an error
  (DL0504) — the checker must never merge bindings by union.
- **R-4 (builtin-callback law, audit F-4):** every builtin that may invoke a function argument
  carries that argument's row variable in its own row. A meta-test walks the primitive table and
  asserts it; each higher-order builtin has a laundering test (effectful callback under a pure
  caller → DL0501).

### 6.3 The agreement property (what "verified" means in Stage 1)

> **Theorem (design-level, test-enforced):** in a well-typed Stage-1 program, no primitive effect
> is performed except by invoking a method on a capability value of the matching resource kind,
> and every function's declared row contains every effect any call path through it can perform.
> Consequently the whole-program authority is `row(main)` for kinds, and the manifest ∩ CLI grant
> for scopes.

Enforced by the conformance suite (§12), not yet by mechanized proof (future work "Delulu Core" —
a small calculus with progress/preservation including rows; post-Stage-2).

### 6.4 Secret rules (checker-enforced)

1. `Secret[T]` unifies only with `Secret[T]`. No coercion to/from `T` (DL0602).
2. `s.map(f)` where `f : fn(T) -> U ! {}` (pure — enforced, DL0603 otherwise) yields `Secret[U]`.
3. `Secret.verify(a: Secret[Str], b: Secret[Str]) -> Bool` — constant-time; pure.
4. `s.expose(d: Cap[Declassify]) -> T ! {Declassify}` — the only unwrap; requires the root-issued
   cap **and carries the core effect `Declassify`** (audit R-2), so every function that can reveal
   a secret says so in its row and in the authority report.
5. No repair object ever inserts `expose` or requests `Cap[Declassify]` (§10.4).
6. **R-5 (opaque types, audit F-5):** `Secret[T]`, `Cap[R]`, `Root`, `Plugin[_]` — and any record,
   variant, or collection containing one (a compiler-computed `opaque` property, propagated
   structurally) — are excluded from `str`, `==`/`!=`, and all serialization. DL0604
   (stringify/serialize opaque), DL0605 (equality on opaque). Constant-time `Secret.verify` is
   the only equality on secrets.

### 6.5 Capability laundering — closed channels (checked by tests)

- Module-level `var` → DL0305 (no ambient storage).
- Closures capturing caps: legal; their rows carry their behavior (T-Lambda).
- Caps in records/lists: legal; using them still surfaces effects at every call site.
- Returning caps: legal (attenuation patterns need it); the *effect* of later use still appears
  in the user's row.
- No reflection, no `eval`, no dynamic import in Stage 1 — nothing bypasses the table in §7.3.
- Generic builtins over opaque types: closed by R-5 (`str(secret)`, `==` on secrets/caps rejected).
- Parameter-position row widening (contravariance exploit): closed by R-3; permanent rejection test.
- Row-variable union-merge: closed by R-3b (DL0504).
- Full channel-by-channel analysis, minimal exploit programs, and the soundness argument:
  **`SOUNDNESS_AUDIT.md`** (normative companion to this section).

---

## 7. Runtime and interpreter

### 7.1 Values

```rust
pub enum Value {
    Int(i64), Float(f64), Bool(bool), Str(Rc<str>), Unit,
    List(Rc<RefCell<Vec<Value>>>),
    Record(TypeDefId, Rc<RefCell<Vec<Value>>>),
    Variant(TypeDefId, u32, Rc<Vec<Value>>),
    Closure(Rc<Closure>),                 // params, body, captured env, checked row
    Cap(Rc<CapVal>),
    Secret(Rc<SecretVal>),                // zeroized on drop; Debug/Display print "«secret»"
    Root(Rc<RootVal>),
}
pub struct CapVal { pub kind: ResourceKind, pub scope: Scope, pub granted_by: GrantId }
pub enum Scope {
    FsSubtree { root: PathBuf, write: bool },
    NetHosts  { allow: Vec<HostPattern> },
    ConsoleIo, ClockAccess, RandAccess,
    DeclassifyNames { names: Vec<String> },
}
```

Checked arithmetic: `Int` overflow, division by zero, and out-of-bounds indexing are runtime
panics (DL09xx) that abort with a JSON diagnostic — defined behavior, never UB. Recursion depth
is limited (default 10_000; `--max-depth`). The interpreter is a straightforward environment-
passing tree-walker over the *typed* AST; `Rc`/`RefCell` is the Stage-1 memory story (single-
threaded; reference capabilities arrive Stage 7).

### 7.2 Root injection and the grant flow (the Stage-1 capability broker)

`fn main(root: Root)` is the only place `Root` enters the program. Flow for `delulu run app.delulu`:

1. Compile; compute `row(main)` and the full authority report (§10.5).
2. Read the manifest (`delulu.toml`, optional for single files):

```toml
[package]
name = "demo"
version = "0.1.0"

[authority]
effects  = ["Read", "Write"]
fs.read  = ["./config"]
fs.write = []
net      = []
secrets  = ["API_KEY"]
```

3. Verify `row(main) ⊆ authority.effects` (DL0701 otherwise).
4. Collect grants from flags:
   `--grant fs.read=./config --grant fs.write=./out --grant net=api.example.com`
   `--grant console --grant clock --grant rand --grant secret:API_KEY=env:API_KEY`
5. If grants < manifest: **interactive prompt** showing exactly what the program requests and why
   (which functions need it — spans included), asking the human to approve. `--grant-manifest`
   accepts the manifest wholesale; `--no-prompt` fails instead of asking (CI mode; DL0702).
6. Construct `Root` exposing only the granted slice; run `main`.

The CLI *is* the human-controlled broker in Stage 1. Stage 5 replaces it with a process-separated
broker + WASM containment; the grant model and manifest format do not change.

### 7.3 Primitive table (the single source of effect truth)

| Capability | Method | Type | Effect | Runtime scope check |
|---|---|---|---|---|
| `Root` | `console()` | `-> Cap[Console]` | — (pure) | console granted |
| `Root` | `fs_read(p: Str)` | `-> Cap[FsRead]` | — | p under a granted read root |
| `Root` | `fs_write(p: Str)` | `-> Cap[FsWrite]` | — | p under a granted write root |
| `Root` | `http(hosts: List[Str])` | `-> Cap[Http]` | — | hosts ⊆ granted hosts |
| `Root` | `clock()` / `rand()` | `-> Cap[Clock]` / `Cap[Rand]` | — | granted |
| `Root` | `secret(name: Str)` | `-> Secret[Str]` | — | name in granted secrets |
| `Root` | `declassify()` | `-> Cap[Declassify]` | — | explicitly granted only |
| `Cap[Console]` | `println(s: Str)` / `print(s: Str)` | `-> Unit` | `Write` | — |
| `Cap[Console]` | `readline()` | `-> Result[Str, IoErr]` | `Read` | — |
| `Cap[FsRead]` | `read_text(p: Str)` | `-> Result[Str, IoErr]` | `Read` | path under scope root |
| `Cap[FsRead]` | `list_dir(p: Str)` | `-> Result[List[Str], IoErr]` | `Read` | path under scope root |
| `Cap[FsRead]` | `narrow(p: Str)` | `-> Cap[FsRead]` | — (attenuation) | sub-path of scope |
| `Cap[FsWrite]` | `write_text(p, s)` / `append_text(p, s)` | `-> Result[Unit, IoErr]` | `Write` | path under scope root |
| `Cap[Http]` | `get(url: Str)` | `-> Result[Str, NetErr]` | `Net` | host in allowlist, https only |
| `Cap[Clock]` | `now_ms()` | `-> Int` | `Clock` | — |
| `Cap[Rand]` | `int(lo, hi)` / `float()` | `-> Int` / `-> Float` | `Rand` | — |
| `Secret[T]` | `map(f: fn(T) -> U ! {})` | `-> Secret[U]` | — (f must be pure, DL0603) | — |
| `Secret[T]` | `Secret.verify(a, b)` | `-> Bool` | — (constant-time) | — |
| `Secret[T]` | `expose(d: Cap[Declassify])` | `-> T` | `Declassify` | secret's origin ∈ granted declassify names |

Root methods **fail at startup**, not mid-run: `Root` is constructed with only granted slices, so
a `root.http(...)` call without a net grant is a startup-time refusal with DL0703 naming the span.
Attenuation (`narrow`, and `Root` constructors) is pure: deriving weaker authority is not an
effect; *using* authority is.

Builtin free functions (prelude, all pure): `str(x)`, `len(x)`, `int(f)`, `float(i)`,
`parse_int(s) -> Option[Int]`, `push(list, x)`, `range(lo, hi) -> List[Int]`.

---

## 8. The plugin/add-on system (design fixed now; runtime lands Stage 6)

### 8.1 Model

A plugin is a DeluluLang module compiled against a **plugin manifest** — its authority contract:

```toml
[plugin]
name = "summarize"
version = "0.1.0"
api = 1

[plugin.authority]
effects  = ["Read"]            # hard upper bound on every exported function's row
requires = ["Cap[FsRead]"]     # capability parameters the host must supply

[plugin.exports]
summarize = "fn(Cap[FsRead], Str) -> Result[Str, IoErr] ! {Read}"
```

### 8.2 Host API (types shipped in Stage 1 so signatures are stable)

```delulu
// Loading requires authority to load — plugin hosting is itself capability-gated.
// Cap[PluginHost] comes only from Root; effect Load is reserved now, active Stage 6.
fn load[P](host: Cap[PluginHost], path: Str, grant: Grant)
    -> Result[Plugin[P], PluginErr] ! {Load, Read}
```

- **`Plugin[Verified]`** — source or signed typed-IR; the loader re-runs the full §6 check at
  load time against the manifest, then checks `manifest.authority ⊆ grant`. Per-function
  verification, compile-time grade.
- **`Plugin[Contained]`** — opaque WASM; the loader maps the grant to WASI imports
  (deny-by-default) and enforces module-granularity containment only. The type name says so;
  documentation and `delulu authority` report it as a containment boundary, not a proof.
  **R-1 (audit F-1):** every export of a Contained plugin is typed at the row equal to the
  plugin's **entire granted authority** (`effects(grant)` ⊆ manifest effects, checked at load);
  per-export rows in a Contained manifest are documentation and never enter the type system.
  **R-6a:** function-typed arguments (or function-typed fields inside arguments) to Contained
  exports are a compile error (DL0803) — an opaque module holding a funcref could invoke it at
  times no caller row accounts for. Verified plugins accept callbacks, rows composed per R-4.
- **R-Get:** `p.get[F](name)` succeeds only if the plugin's export type matches `F` with
  `row(export) ⊆ row(F)`; for Contained plugins, `row(export) := effects(grant)`.
- Exported functions surface as ordinary typed values (`plugin.get[T](name)`), their rows joining
  the caller's row like any call. **A plugin call can therefore never widen the host's checked
  authority** — the host's row already accounts for it, and the runtime cap scopes still apply.
- **R-6b:** host values passed into a plugin live only for the synchronous duration of the export
  call; runtime handles are invalidated on return — no persistent host references across calls.
- `plugin.unload()` revokes the grant id at the broker; subsequent calls through retained
  references fail with DL0801 (revocation is runtime, honest and immediate). **R-6c:** function
  values from a plugin bind the load-time `GrantId`, re-checked per call; reload mints a fresh id
  and fresh values — a live reference is never re-bound to different authority.
- **R-7 (holder model / monotone attenuation):** every grant issued to a plugin, sub-agent, or
  child context must satisfy `sub ⊑ holder's grant` (attenuation order per `SOUNDNESS_AUDIT.md`
  §A.3); wider requests are DL0802. Revocation is transitive down the grant tree. This holds
  identically whether the holder is a human, an LLM orchestrating parallel agents, or any future
  AI — no discrimination between holders; no grantee can exceed its grantor.

### 8.3 Why this is the proof of the core

Loading code *after* compile time and still bounding it is the one scenario every mainstream
language fails (a loaded `.so`/module runs with full host authority). Here it is the same
mechanism as §6 — more authority-typed code, checked and contained. Verified vs. Contained states
exactly where verification ends and containment begins, so the claim is never inflated.

---

## 9. Compiler architecture

### 9.1 Implementation language: **Rust**

Memory-safe compiler for a security language (dogfooding the posture); the WASM/sandbox ecosystem
we need in Stages 5–6 (wasmtime, cranelift, wasi) is Rust-native; single static binary on
Windows/macOS/Linux; best-in-class lexer/LSP/testing crates. Rejected: Go (fine, but the sandbox
ecosystem isn't native to it); OCaml (excellent for compilers, weaker single-binary + embedding
story); TypeScript (no).

### 9.2 Workspace layout

```
delulu/
  Cargo.toml                # workspace
  crates/
    delulu-syntax/          # lexer, parser, AST, spans  (no deps on the rest)
    delulu-check/           # resolve, types, effects, authority report
    delulu-runtime/         # values, cap table, interpreter, grant broker
    delulu-diag/            # diagnostic model, JSON encoder, repair objects
    delulu/                 # CLI: check | run | repl | authority
  tests/
    conformance/            # .delulu files + expected JSON diagnostics (the language's law)
    laundering/             # every §6.5 channel, must stay closed
  examples/
```

Keep files small (agents lose attention on long files). The conformance suite is the primary
control: every rule in §6 has at least one accepting and one rejecting test.

### 9.3 Pipeline

```
source ─ lex ─ parse (recursive descent, error-recovering) ─ AST
       ─ resolve (symbols, modules) ─ type+effect check (bidirectional + row unification)
       ─ side tables (types, rows, authority report)
       ─ Stage 1: tree-walk interpret        ─ Stage ≥3: WASM emission
```

Handwritten lexer and parser (no generator): error recovery and precise machine diagnostics are
product features, and generators fight both. Parser must recover at statement boundaries and emit
multiple diagnostics per run.

### 9.4 Backend decision (hardened — overrides the old roadmap's "transpile to C")

**The first compiled backend is WASM** (WAT emission via `wasm-encoder`, executed under Wasmtime
with deny-by-default WASI), merging old Stages 3+5. Reasons: the C transpile was a throwaway
backend with **zero runtime containment** — a DeluluLang artifact whose sandbox floor doesn't
exist yet contradicts the identity; it also drags a C toolchain dependency (painful on Windows).
WASM is portability *and* the enforcement floor in one move, and the interpreter remains the
reference semantics meanwhile. Fallback if WASM-GC ergonomics bite: keep interpreting longer;
never resurrect the C detour.

### 9.5 CLI contract

```
delulu check  <file> [--json]              # exit 0 ok / 1 diagnostics / 2 internal
delulu run    <file> [--json] [--grant ...] [--grant-manifest] [--no-prompt]
delulu repl   [--grant ...]                # rows and types printed for each binding
delulu authority <file> [--json]           # the whole-program authority report
```

`--json` moves *all* compiler output to stdout as one JSON document; human text goes to stderr
only. Exit codes are part of the stable contract.

---

## 10. Machine-readable diagnostics contract

### 10.1 Envelope

```json
{
  "delulu_version": "0.1.0",
  "schema": 1,
  "command": "check",
  "diagnostics": [ ... ],
  "authority": { ... },          // present for `authority`, optional for `check`
  "summary": { "errors": 1, "warnings": 0 }
}
```

### 10.2 Diagnostic object

```json
{
  "code": "DL0501",
  "severity": "error",
  "message": "function `fetch_price` performs effect `Net` not declared in its row",
  "spans": [
    { "file": "src/main.delulu", "start": { "line": 14, "col": 3, "byte": 210 },
      "end": { "line": 14, "col": 34, "byte": 241 },
      "label": "this call performs `Net`" },
    { "file": "src/main.delulu", "start": { "line": 12, "col": 1, "byte": 150 },
      "end": { "line": 12, "col": 42, "byte": 192 },
      "label": "declared row `!{Read}` is here", "secondary": true }
  ],
  "explanation_id": "E-DL0501",
  "repairs": [
    {
      "id": "add_effect_to_row",
      "confidence": "exact",
      "authority_widening": true,
      "requires_human": false,
      "edits": [
        { "file": "src/main.delulu",
          "range": { "start_byte": 190, "end_byte": 190 }, "insert": ", Net" } ]
    }
  ]
}
```

Rules: `line`/`col` are 1-based, `byte` offsets are the machine truth; messages are presentation
and may change, **codes and repair ids may not** (removal only via deprecation). Every span-free
diagnostic is a bug.

### 10.3 Code registry (allocate within ranges; never reuse)

| Range | Domain |
|---|---|
| DL01xx | lexical — allocated: DL0101 unexpected character, DL0102 unterminated string, DL0103 invalid escape, DL0104 invalid numeric literal, DL0105 unterminated block comment, DL0106 reserved word declared |
| DL02xx | parse — allocated: DL0201 expected token, DL0202 expected expression, DL0203 expected type, DL0204 missing `module` header, DL0205 expected pattern, DL0206 chained comparison (non-associative), DL0207 invalid assignment target, DL0208 expected item, DL0209 expected statement terminator, DL0210 reserved |
| DL03xx | names/modules (DL0304 import cycle, DL0305 module-level `var`) |
| DL04xx | types (DL0402 numeric mix, DL0407 non-exhaustive match, DL0410 var kind conflict) |
| DL05xx | effects (DL0501 undeclared effect, DL0502 unused declared effect, DL0503 >1 row var, DL0504 conflicting row-var bindings — never union-merge) |
| DL06xx | capabilities & secrets (DL0602 secret flow, DL0603 impure secret map, DL0604 stringify/serialize opaque, DL0605 equality on opaque) |
| DL07xx | manifest/authority (DL0701 main exceeds manifest, DL0702 grant refused, DL0703 ungranted root slice) |
| DL08xx | plugins & grants (DL0801 revoked-plugin call, DL0802 grant exceeds holder's grant — attenuation violation, DL0803 function-typed argument to Contained export) |
| DL09xx | runtime (overflow, div-by-zero, OOB, scope violation, depth) |

### 10.4 Repair policy (normative)

- `authority_widening: true` on any repair that adds effects, capabilities, grants, or scopes;
  agents/CI can refuse these by policy.
- Repairs involving `Secret` **never** insert `expose` or `Cap[Declassify]`; they may only
  suggest removing the offending flow (`requires_human: true` where intent is ambiguous).
- Narrowing repairs (DL0502's `remove_effect_from_row`) are safe to auto-apply.

### 10.5 Authority report (`delulu authority --json`)

```json
{
  "program": "demo",
  "effects": ["Read", "Write"],
  "capabilities": [
    { "kind": "FsRead", "scopes": ["./config"], "used_at": [ {"file": "...", "line": 20} ] },
    { "kind": "Console", "scopes": ["stdio"], "used_at": [ ... ] }
  ],
  "secrets": ["API_KEY"],
  "foreign_calls": [],
  "contained_plugins": [],
  "pure_functions": ["fib", "apply"]
}
```

This report is the Possibility-1 demo (`CONSTITUTION.md` §4): the mechanical answer to "what can
this program do?"

---

## 11. Standard library (Stage 1 surface — deliberately tiny)

`std.core` (prelude, auto-imported): builtins of §7.3, `Option`, `Result`, `List` methods
(`push`, `pop`, `get`, `set`, `map`, `filter` — all row-polymorphic), `Str` methods (`split`,
`trim`, `contains`, `starts_with`, `slice`). `std.fs`, `std.net`, `std.io`: the capability types
and error types (`IoErr`, `NetErr` as sums). Nothing else. Every addition must state its row and
pass review; the stdlib is the first authority-typed ecosystem code and sets the standard.

---

## 12. Acceptance criteria (Stage 1 is done when all of these pass)

1. `delulu repl`: `fib(10)` → `55`; the printed type is `fn(Int) -> Int !{}`.
2. A named function calling `out.println` without `!{Write}` fails with DL0501, span on the call,
   repair flagged `authority_widening: true`.
3. There is no way to write a program performing filesystem I/O without a `Cap[FsRead]`/
   `Cap[FsWrite]` value threading from `main`'s `Root` — verified by the laundering suite
   (module-level `var` rejected; closures/records/returns covered; effects still surface).
4. `delulu run` with grants smaller than the manifest prompts; `--no-prompt` fails with DL0702;
   granted run reads only under `./config` and a runtime path escape (`../`) is refused (DL09xx
   scope violation).
5. `Secret[Str]` passed to `println` → DL0602 at compile time; no repair suggests `expose`;
   `verify` works; runtime `Debug` output prints `«secret»`.
6. `apply(fn(x: Int) -> Int { x * 2 }, 21)` type-checks with row `{}`;
   `apply(fn(_: Int) -> Unit !{Write} { out.println("hi") }, 1)` requires `{Write}` in the caller
   — row polymorphism works end to end.
7. `delulu authority` on the reference program (§3.1) emits exactly the report of §10.5 shape.
8. Every diagnostic in the conformance suite matches its expected JSON (codes, spans, repairs).
9. `delulu check --json` output round-trips through a JSON schema validator.
10. All of the above run green on Windows, macOS, and Linux CI.
11. The audit's F-3 contravariance program is rejected (row mismatch at the `let` ascription);
    the F-3b conflicting-bindings program is DL0504 — never a union.
12. The primitive-table meta-test (R-4) passes, and `[1,2,3].map(effectful_lambda)` inside a
    pure function is DL0501.
13. `str(secret)` is DL0604; `secret == secret` and `==` on a record containing a Cap are DL0605.
14. `expose` outside a `!{Declassify}` row is DL0501; `delulu authority` lists every
    declassifying function.

## 12a. Implementation status (2026-07-05)

Stage 1 is **implemented and verified**. Rust workspace under `crates/` (`delulu-diag`,
`delulu-syntax`, `delulu-check`, `delulu-runtime`, `delulu`); 65 tests green on MSVC Windows,
including the laundering suite (`crates/delulu-check/tests/laundering.rs`) encoding audit findings
F-2…F-5 and the core non-escape guarantee. `delulu authority examples/demo.delulu`, `delulu run`,
`delulu check` (human + `--json`), and a REPL all run. Implementation-forced spec deltas already
folded in: the §3.1 reference program (the `?`-in-`Unit`-`main` fix), the §2.2 `} else` rule, the
§2.3 reserved-at-declaration-site precision, the §3 record-literal-not-in-conditions rule, the §4
`Expr::Block` node, and the DL01xx/02xx code allocations. One soundness bug (audit F-4: a
higher-order `map` not surfacing its callback's row) was found *by* the laundering suite during
implementation and fixed (R-4 now enforced in `check_method`).

## 13. First-commit checklist

- [ ] Workspace + crates of §9.2; CI (fmt, clippy, test) on three OSes.
- [ ] Lexer with §2 tokens incl. terminator insertion; golden tests.
- [ ] Parser for §3 with recovery; AST of §4; pretty-printer (needed by repairs).
- [ ] `Row`/`Type` of §6.1; unification incl. single row var; primitive table §7.3 as data.
- [ ] Checker rules of §6.2 with DL05xx/DL06xx diagnostics + repairs.
- [ ] Interpreter + `Root` grant flow of §7.2; manifest parsing.
- [ ] `delulu authority`; JSON envelope of §10.
- [ ] Conformance + laundering suites seeded with every example in this document.

*The skeleton ships first. The guarantee is never added late.*
