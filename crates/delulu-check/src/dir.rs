//! DIR — the Delulu typed IR (Stage 6 "Live", spec §2.3).
//!
//! DIR is the **post-check typed AST**, canonically serialized (versioned CBOR): the source
//! module, every node's resolved type and effect row, the whole-module authority facts, and the
//! versions of the two contracts it was checked against (the DIR wire format and the primitive
//! table). It carries **no machine code** — a Verified plugin ships DIR and is *re-verified* at
//! load, then run by the host's own engine.
//!
//! Two operations live here:
//! - [`serialize`] turns a checked module into DIR bytes (Phase 6a).
//! - [`deserialize`] reads DIR bytes back, hostile-input hardened: malformed/truncated CBOR, an
//!   unsupported version, or a structurally impossible AST all refuse cleanly and **never panic**.
//! - [`verify`] (Phase 6b) replays the checking pass — reusing the *exact same rule code* as
//!   `check_source` — and refuses (DL1504) any DIR whose stored facts do not match what re-checking
//!   the code produces. That single-implementation rule is what guarantees `plugin verify` and a
//!   real load can never diverge (spec §9.9).
//!
//! **The security stance:** a DIR is untrusted bytes. Deserialized `DefId`s and `NodeId`s are only
//! ever *compared*, never used to index a table — so an out-of-range id can lie but cannot crash.
//! The reconstructed AST is structurally validated before it is ever re-checked, so a hostile
//! payload cannot reach a `panic!` inside the checker.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use delulu_syntax::ast::*;

use crate::check::{CheckResult, FnFacts};
use crate::ty::{Effect, Row, Type};

/// The DIR wire-format version. Bumped whenever the serialized shape changes; a DIR written by a
/// different version is refused with DL1503 (rebuild the plugin). Activates in Phase 6a.
pub const DIR_VERSION: u32 = 1;

/// The version of the checker's primitive table (`check::method_sig`, spec §7.3) that a DIR was
/// checked against. Re-verifying a DIR against a *different* primitive table would silently change
/// what its capability operations mean, so a mismatch is refused with DL1503 — same "rebuild the
/// plugin" repair as a wire-format bump. Bump this constant whenever the primitive table changes.
pub const PRIM_TABLE_VERSION: u32 = 1;

/// Why a DIR payload was refused. Everything that is not a clean version mismatch maps to DL1504
/// (the Verified re-check failed) — a corrupt or dishonest DIR is *unverifiable*, and per invariant
/// 29 it never falls back to Contained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirError {
    /// The bytes are not decodable DIR: malformed/truncated CBOR, or a structurally impossible AST
    /// (e.g. an empty name path). Refused as DL1504 — it cannot be re-verified.
    Malformed(String),
    /// The DIR wire-format version is not the one this toolchain understands (DL1503).
    UnsupportedVersion { found: u32, supported: u32 },
    /// The DIR was checked against a different primitive-table version (DL1503).
    PrimTableMismatch { found: u32, supported: u32 },
    /// The DIR decoded, but replaying the checking pass refuted it (DL1504). Carries a short reason.
    VerifyFailed(String),
}

impl DirError {
    /// The diagnostic code the loader/CLI reports for this fault.
    pub fn code(&self) -> &'static str {
        match self {
            DirError::UnsupportedVersion { .. } | DirError::PrimTableMismatch { .. } => "DL1503",
            DirError::Malformed(_) | DirError::VerifyFailed(_) => "DL1504",
        }
    }

    /// A DL1504 refusal is `requires_human: true` (spec §7): corrupt or dishonest Verified code is
    /// never machine-repairable and never silently downgraded. A DL1503 mismatch has the exact
    /// "rebuild the plugin" repair instead.
    pub fn requires_human(&self) -> bool {
        matches!(self, DirError::Malformed(_) | DirError::VerifyFailed(_))
    }

    pub fn message(&self) -> String {
        match self {
            DirError::Malformed(w) => format!("DIR payload is not decodable: {w}"),
            DirError::UnsupportedVersion { found, supported } => format!(
                "DIR wire-format version {found} is not supported by this toolchain (v{supported}) — rebuild the plugin"
            ),
            DirError::PrimTableMismatch { found, supported } => format!(
                "DIR was checked against primitive-table version {found}, but this toolchain is v{supported} — rebuild the plugin"
            ),
            DirError::VerifyFailed(w) => format!("DIR re-verification failed: {w}"),
        }
    }
}

/// The serialized DIR. Every collection is a `BTreeMap`/`BTreeSet`/`Vec` — never a `HashMap` — so
/// the CBOR encoding is **canonical**: identical inputs produce byte-identical DIR across runs
/// (house rule 8). `NodeId`s are stored as their raw `u32` for a compact, stable key order.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dir {
    /// Wire-format version (checked against [`DIR_VERSION`] on decode).
    pub dir_version: u32,
    /// Primitive-table version this DIR was checked against (checked against [`PRIM_TABLE_VERSION`]).
    pub prim_table_version: u32,
    /// The plugin's source module — the post-resolve AST, replayed (not re-parsed) at verify time.
    pub module: Module,
    /// Per-function authority facts (effects, capability kinds, callees, secrets), by name.
    pub facts: BTreeMap<String, FnFacts>,
    /// Each function's static type (rows never erase — invariant 3), by name.
    pub fn_types: BTreeMap<String, Type>,
    /// Every expression/block node's resolved type, keyed by `NodeId` (spec §2.3).
    pub node_types: BTreeMap<u32, Type>,
    /// Every expression/block node's resolved effect row, keyed by `NodeId` (spec §2.3).
    pub node_rows: BTreeMap<u32, Row>,
    /// Whether the module declares `fn main` (a plugin package must not — enforced in Phase 6c).
    pub main_present: bool,
    /// `main`'s resolved effect set, if present.
    pub main_row: Option<BTreeSet<Effect>>,
    /// Which `foreign` lib each `root.foreign(load)` call site binds, by call-site `NodeId`.
    pub foreign_binds: BTreeMap<u32, String>,
}

impl Dir {
    /// Build a DIR from a checked module. The caller has already run `check_source`/`check_module`;
    /// this is a pure projection of that result plus the source module and the two contract
    /// versions. `HashMap`s are lowered to `BTreeMap`s so the encoding is canonical.
    pub fn from_checked(module: &Module, result: &CheckResult) -> Dir {
        Dir {
            dir_version: DIR_VERSION,
            prim_table_version: PRIM_TABLE_VERSION,
            module: module.clone(),
            facts: result.facts.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            fn_types: result.fn_types.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            node_types: result.node_types.iter().map(|(k, v)| (k.0, v.clone())).collect(),
            node_rows: result.node_rows.iter().map(|(k, v)| (k.0, v.clone())).collect(),
            main_present: result.main_present,
            main_row: result.main_row.clone(),
            foreign_binds: result.foreign_binds.iter().map(|(k, v)| (k.0, v.clone())).collect(),
        }
    }

    /// Encode to canonical DIR bytes. Serialization of a well-formed `Dir` is infallible (the sink
    /// is an in-memory `Vec`), so a failure here is a programming error, not a runtime condition.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf).expect("DIR serialization to a Vec cannot fail");
        buf
    }

    /// Decode DIR bytes with full hostile-input hardening: bad CBOR, an unsupported version, or a
    /// structurally impossible AST all return an `Err` — never a panic.
    pub fn decode(bytes: &[u8]) -> Result<Dir, DirError> {
        let dir: Dir =
            ciborium::from_reader(bytes).map_err(|e| DirError::Malformed(e.to_string()))?;
        if dir.dir_version != DIR_VERSION {
            return Err(DirError::UnsupportedVersion {
                found: dir.dir_version,
                supported: DIR_VERSION,
            });
        }
        if dir.prim_table_version != PRIM_TABLE_VERSION {
            return Err(DirError::PrimTableMismatch {
                found: dir.prim_table_version,
                supported: PRIM_TABLE_VERSION,
            });
        }
        // Structural hardening: the checker relies on a handful of AST invariants (chiefly that
        // every name path has at least one segment — `Path::span`/`segs[0]` would otherwise panic).
        // A hostile CBOR payload can satisfy the *schema* while violating them, so we validate the
        // reconstructed module before any code path can touch it.
        validate_module(&dir.module)?;
        Ok(dir)
    }
}

/// Serialize a checked module to canonical DIR bytes (Phase 6a).
pub fn serialize(module: &Module, result: &CheckResult) -> Vec<u8> {
    Dir::from_checked(module, result).encode()
}

/// Deserialize DIR bytes, hostile-input hardened (Phase 6a). See [`Dir::decode`].
pub fn deserialize(bytes: &[u8]) -> Result<Dir, DirError> {
    Dir::decode(bytes)
}

/// Re-verify DIR bytes — the Verified guarantee (Phase 6b, spec §3.1 step 5-Verified).
///
/// This **replays the checking pass by reusing the exact rule code of `check_source`**: it
/// reconstructs the module from DIR and re-runs the very same `resolve` + `check_module` the
/// compiler ran originally — never a second, drifting implementation (playbook trap 3; this is
/// what makes `plugin verify` and a real load provably identical, criterion 9). A DIR is accepted
/// only when **both**:
///
/// 1. Re-checking the code raises **no error** — so a body that performs an effect its declared row
///    omits (a narrowed row) is caught by the checker's own boundary rule (DL0501 → DL1504), and
/// 2. The DIR's **stored** facts/types/rows equal what re-checking produces — so a DIR that was
///    validly re-encoded but *lies* about its authority (a forged narrower row, a forged export
///    type) is refuted by the comparison.
///
/// Any failure is **DL1504** (`requires_human`), and per invariant 29 it **never** falls back to
/// Contained. Deserialized ids are only compared here, never used to index, and the module was
/// structurally validated at decode — so this runs the checker over hostile input without a panic
/// surface.
pub fn verify(bytes: &[u8]) -> Result<Dir, DirError> {
    let dir = deserialize(bytes)?;

    // Reuse `check_source`'s exact pipeline, minus the parse step (the AST is replayed, not
    // re-parsed): single-module resolve, then the T-* judgment. See build-order Deviation 1.
    let (table, rdiags) = crate::resolve::resolve(&dir.module);
    let result = crate::check::check_module(&dir.module, &table);

    // (1) Re-checking must be clean. Any error from resolve or check refutes the DIR — including the
    // row-subset boundary check (DL0501) that catches a row narrower than the code actually needs.
    if let Some(d) = rdiags.iter().chain(result.diags.iter()).find(|d| d.is_error()) {
        return Err(DirError::VerifyFailed(format!(
            "re-checking the code raised {}: {}",
            d.code, d.message
        )));
    }

    // (2) The stored facts/types/rows must be exactly what honest re-checking produces. Comparing
    // the whole recomputed projection catches any forged authority claim.
    let recomputed = Dir::from_checked(&dir.module, &result);
    if recomputed.facts != dir.facts {
        return Err(DirError::VerifyFailed(
            "stored authority facts disagree with the re-checked code".into(),
        ));
    }
    if recomputed.fn_types != dir.fn_types {
        return Err(DirError::VerifyFailed(
            "stored function types disagree with the re-checked code".into(),
        ));
    }
    if recomputed.node_types != dir.node_types {
        return Err(DirError::VerifyFailed(
            "a stored node type disagrees with the re-checked code".into(),
        ));
    }
    if recomputed.node_rows != dir.node_rows {
        return Err(DirError::VerifyFailed(
            "a stored node effect row disagrees with the re-checked code".into(),
        ));
    }
    if recomputed.main_present != dir.main_present || recomputed.main_row != dir.main_row {
        return Err(DirError::VerifyFailed(
            "stored `main` facts disagree with the re-checked code".into(),
        ));
    }
    if recomputed.foreign_binds != dir.foreign_binds {
        return Err(DirError::VerifyFailed(
            "stored foreign bind sites disagree with the re-checked code".into(),
        ));
    }

    Ok(dir)
}

// ===== structural validation (panic-proofing the reconstructed AST) =========================

/// Reject a reconstructed module that violates an AST invariant the checker assumes. Today the one
/// load-bearing invariant is **every `Path` has at least one segment**; a zero-segment path would
/// panic `Path::span`/`Path::segs[0]` deep inside `resolve`/`check`. Walking the tree here keeps the
/// panic surface at zero for arbitrary CBOR input.
fn validate_module(m: &Module) -> Result<(), DirError> {
    check_path(&m.name)?;
    for imp in &m.imports {
        check_path(&imp.path)?;
    }
    for item in &m.items {
        validate_item(item)?;
    }
    Ok(())
}

fn check_path(p: &Path) -> Result<(), DirError> {
    if p.segs.is_empty() {
        return Err(DirError::Malformed("a name path has zero segments".into()));
    }
    Ok(())
}

fn validate_item(item: &Item) -> Result<(), DirError> {
    match item {
        Item::Fn(f) => {
            for p in &f.params {
                validate_type_expr(&p.ty)?;
            }
            if let Some(r) = &f.ret {
                validate_type_expr(r)?;
            }
            if let Some(row) = &f.row {
                validate_row(row)?;
            }
            validate_block(&f.body)
        }
        Item::Type(td) => match &td.kind {
            TypeDeclKind::Record(fields) => {
                for fd in fields {
                    validate_type_expr(&fd.ty)?;
                }
                Ok(())
            }
            TypeDeclKind::Sum(variants) => {
                for v in variants {
                    for t in &v.fields {
                        validate_type_expr(t)?;
                    }
                }
                Ok(())
            }
            TypeDeclKind::Alias(t) => validate_type_expr(t),
        },
        Item::Const(c) => {
            if let Some(t) = &c.ty {
                validate_type_expr(t)?;
            }
            validate_expr(&c.value)
        }
        Item::Foreign(fd) => {
            for f in &fd.fns {
                for p in &f.params {
                    validate_type_expr(&p.ty)?;
                }
                if let Some(r) = &f.ret {
                    validate_type_expr(r)?;
                }
            }
            Ok(())
        }
        Item::Effect(_) => Ok(()),
    }
}

fn validate_type_expr(t: &TypeExpr) -> Result<(), DirError> {
    match t {
        TypeExpr::Named { path, args, .. } => {
            check_path(path)?;
            for a in args {
                validate_type_expr(a)?;
            }
            Ok(())
        }
        TypeExpr::Fn { params, ret, row, .. } => {
            for p in params {
                validate_type_expr(p)?;
            }
            if let Some(r) = ret {
                validate_type_expr(r)?;
            }
            if let Some(row) = row {
                validate_row(row)?;
            }
            Ok(())
        }
    }
}

fn validate_row(r: &RowExpr) -> Result<(), DirError> {
    for e in &r.effects {
        check_path(e)?;
    }
    Ok(())
}

fn validate_block(b: &Block) -> Result<(), DirError> {
    for stmt in &b.stmts {
        match stmt {
            Stmt::Let { ty, value, .. } => {
                if let Some(t) = ty {
                    validate_type_expr(t)?;
                }
                validate_expr(value)?;
            }
            Stmt::Assign { target, value, .. } => {
                validate_lvalue(target)?;
                validate_expr(value)?;
            }
            Stmt::While { cond, body, .. } => {
                validate_expr(cond)?;
                validate_block(body)?;
            }
            Stmt::Return { value, .. } => {
                if let Some(e) = value {
                    validate_expr(e)?;
                }
            }
            Stmt::Expr(e) => validate_expr(e)?,
        }
    }
    Ok(())
}

fn validate_lvalue(lv: &LValue) -> Result<(), DirError> {
    match lv {
        LValue::Var(_) => Ok(()),
        LValue::Field(base, _) => validate_lvalue(base),
        LValue::Index(base, idx) => {
            validate_lvalue(base)?;
            validate_expr(idx)
        }
    }
}

fn validate_expr(e: &Expr) -> Result<(), DirError> {
    match e {
        Expr::Lit { .. } => Ok(()),
        Expr::Var { path, .. } => check_path(path),
        Expr::List { items, .. } => {
            for it in items {
                validate_expr(it)?;
            }
            Ok(())
        }
        Expr::Record { path, fields, .. } => {
            check_path(path)?;
            for (_, fe) in fields {
                validate_expr(fe)?;
            }
            Ok(())
        }
        Expr::Call { callee, args, .. } => {
            validate_expr(callee)?;
            for a in args {
                validate_expr(a)?;
            }
            Ok(())
        }
        Expr::Method { recv, args, .. } => {
            validate_expr(recv)?;
            for a in args {
                validate_expr(a)?;
            }
            Ok(())
        }
        Expr::Field { recv, .. } => validate_expr(recv),
        Expr::Index { recv, index, .. } => {
            validate_expr(recv)?;
            validate_expr(index)
        }
        Expr::Unary { operand, .. } => validate_expr(operand),
        Expr::Binary { lhs, rhs, .. } => {
            validate_expr(lhs)?;
            validate_expr(rhs)
        }
        Expr::If { cond, then_, else_, .. } => {
            validate_expr(cond)?;
            validate_block(then_)?;
            if let Some(e) = else_ {
                validate_expr(e)?;
            }
            Ok(())
        }
        Expr::Match { scrutinee, arms, .. } => {
            validate_expr(scrutinee)?;
            for arm in arms {
                validate_pattern(&arm.pattern)?;
                validate_expr(&arm.body)?;
            }
            Ok(())
        }
        Expr::Lambda { params, ret, row, body, .. } => {
            for p in params {
                validate_type_expr(&p.ty)?;
            }
            if let Some(r) = ret {
                validate_type_expr(r)?;
            }
            if let Some(row) = row {
                validate_row(row)?;
            }
            validate_block(body)
        }
        Expr::Try { inner, .. } => validate_expr(inner),
        Expr::Block(b) => validate_block(b),
    }
}

fn validate_pattern(p: &Pattern) -> Result<(), DirError> {
    match p {
        Pattern::Wildcard(_) | Pattern::Lit(_, _) | Pattern::Bind(_) => Ok(()),
        Pattern::Variant { path, fields, .. } => {
            check_path(path)?;
            for f in fields {
                validate_pattern(f)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check_source;

    /// A spread of programs that exercise the interesting corners of the typed AST: pure
    /// arithmetic, effect rows, capabilities, secrets, records/sums/match, generics/lambdas with
    /// row polymorphism, and the foreign surface. The DIR round-trip must be lossless for all.
    fn corpus() -> Vec<&'static str> {
        vec![
            "module m\nfn fib(n: Int) -> Int { if n < 2 { n } else { fib(n-1) + fib(n-2) } }\n",
            "module m\nfn greet(out: Cap[Console], n: Str) ! {Write} { out.println(n) }\n",
            "module m\nfn main(root: Root) ! {Write} { let out = root.console()\n out.println(\"hi\") }\n",
            "module m\nfn r(fs: Cap[FsRead]) -> Result[Str, IoErr] ! {Read} { fs.read_text(\"a.txt\") }\n",
            "module m\ntype Point { x: Int, y: Int }\nfn mk() -> Point { Point { x: 1, y: 2 } }\nfn gx(p: Point) -> Int { p.x }\n",
            "module m\ntype Color = Red | Green | Blue\nfn name(c: Color) -> Str { match c { Red => \"r\", Green => \"g\", Blue => \"b\" } }\n",
            "module m\nfn apply[T, U, e](f: fn(T) -> U ! e, x: T) -> U ! e { f(x) }\nfn g() -> Int { apply(fn(x: Int) -> Int { x * 2 }, 21) }\n",
            "module m\nfn f(s: Secret[Str]) -> Secret[Str] { s }\n",
            "module m\nforeign \"c\" lib mathlib { fn cos(x: Float) -> Float }\nfn leaf(m: mathlib) -> Float ! {ForeignCall} { m.cos(1.0) }\n",
            "module m\nfn poly() -> Int { let xs = [1, 2, 3]\n xs[0] }\n",
        ]
    }

    #[test]
    fn round_trip_is_lossless_and_byte_stable_for_the_corpus() {
        for src in corpus() {
            let c = check_source(0, src);
            assert!(!c.has_errors(), "corpus program must check clean: {src}\n{:?}", c.diagnostics);

            let bytes = serialize(&c.module, &c.result);
            let dir = deserialize(&bytes).unwrap_or_else(|e| panic!("decode `{src}`: {}", e.message()));

            // The CheckResult data survives the round-trip exactly.
            let want_facts: BTreeMap<String, FnFacts> =
                c.result.facts.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            let want_fn_types: BTreeMap<String, Type> =
                c.result.fn_types.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            let want_node_types: BTreeMap<u32, Type> =
                c.result.node_types.iter().map(|(k, v)| (k.0, v.clone())).collect();
            let want_node_rows: BTreeMap<u32, Row> =
                c.result.node_rows.iter().map(|(k, v)| (k.0, v.clone())).collect();
            assert_eq!(dir.facts, want_facts, "facts round-trip: {src}");
            assert_eq!(dir.fn_types, want_fn_types, "fn_types round-trip: {src}");
            assert_eq!(dir.node_types, want_node_types, "node_types round-trip: {src}");
            assert_eq!(dir.node_rows, want_node_rows, "node_rows round-trip: {src}");
            assert_eq!(dir.main_present, c.result.main_present, "main_present round-trip: {src}");
            assert_eq!(dir.main_row, c.result.main_row, "main_row round-trip: {src}");

            // Re-encoding the decoded DIR reproduces the exact bytes: canonical + lossless.
            assert_eq!(dir.encode(), bytes, "DIR is not byte-stable across a round-trip: {src}");
        }
    }

    #[test]
    fn node_tables_cover_every_expression_and_block() {
        // The DIR must carry a type/row for every node — an empty table would be a silent gap.
        let c = check_source(0, "module m\nfn f(n: Int) -> Int { let x = n + 1\n x * 2 }\n");
        assert!(!c.result.node_types.is_empty(), "node_types must be populated");
        assert!(!c.result.node_rows.is_empty(), "node_rows must be populated");
        // Every node with a recorded type also has a recorded row (they are captured together).
        for id in c.result.node_types.keys() {
            assert!(c.result.node_rows.contains_key(id), "node {id:?} has a type but no row");
        }
    }

    #[test]
    fn encoding_is_deterministic_across_independent_checks() {
        // Two independent checks of the same source must produce byte-identical DIR (house rule 8).
        let src = "module m\nfn f(out: Cap[Console]) ! {Write} { out.println(\"x\") }\n";
        let a = check_source(0, src);
        let b = check_source(0, src);
        assert_eq!(serialize(&a.module, &a.result), serialize(&b.module, &b.result));
    }

    #[test]
    fn wrong_dir_version_is_dl1503() {
        let c = check_source(0, "module m\nfn f() -> Int { 1 }\n");
        let mut dir = Dir::from_checked(&c.module, &c.result);
        dir.dir_version = DIR_VERSION + 1;
        let bytes = dir.encode();
        match deserialize(&bytes) {
            Err(e @ DirError::UnsupportedVersion { .. }) => assert_eq!(e.code(), "DL1503"),
            other => panic!("a version bump must be DL1503, got {other:?}"),
        }
    }

    #[test]
    fn wrong_prim_table_version_is_dl1503() {
        let c = check_source(0, "module m\nfn f() -> Int { 1 }\n");
        let mut dir = Dir::from_checked(&c.module, &c.result);
        dir.prim_table_version = PRIM_TABLE_VERSION + 7;
        let bytes = dir.encode();
        match deserialize(&bytes) {
            Err(e @ DirError::PrimTableMismatch { .. }) => assert_eq!(e.code(), "DL1503"),
            other => panic!("a primitive-table bump must be DL1503, got {other:?}"),
        }
    }

    #[test]
    fn malformed_cbor_refuses_cleanly() {
        // Random and truncated bytes must never panic, always refuse.
        assert!(deserialize(b"").is_err());
        assert!(deserialize(b"not cbor at all, just ascii").is_err());
        assert!(deserialize(&[0xff, 0xff, 0xff, 0xff, 0x00, 0x01]).is_err());

        let c = check_source(0, "module m\nfn f() -> Int { 1 }\n");
        let bytes = serialize(&c.module, &c.result);
        for cut in 0..bytes.len() {
            // Every truncation of a valid DIR must refuse cleanly (no panic, no accept).
            let _ = deserialize(&bytes[..cut]);
        }
    }

    #[test]
    fn every_single_byte_flip_refuses_or_reproduces_never_panics() {
        // The hostile-DIR witness for Phase 6a: flipping any one byte of a valid DIR must never
        // panic. It either fails to decode, or decodes to *some* structurally valid DIR — it is
        // Phase 6b (`verify`) that then refutes a decoded-but-dishonest DIR.
        let c = check_source(0, "module m\nfn f(out: Cap[Console]) ! {Write} { out.println(\"x\") }\n");
        let bytes = serialize(&c.module, &c.result);
        for i in 0..bytes.len() {
            for bit in 0..8u32 {
                let mut t = bytes.clone();
                t[i] ^= 1 << bit;
                // Must return (Ok or Err) without panicking; correctness of Ok cases is 6b's job.
                let _ = deserialize(&t);
            }
        }
    }

    // ===== Phase 6b — DIR re-verification (the Verified guarantee) ===========================

    #[test]
    fn valid_dir_verifies_and_agrees_with_check_source() {
        // The reuse guarantee (playbook trap 3 / criterion 9 in miniature): `verify` reproduces
        // exactly what `check_source` computed, for every corpus program.
        for src in corpus() {
            let c = check_source(0, src);
            assert!(!c.has_errors(), "{src}");
            let bytes = serialize(&c.module, &c.result);
            let verified = verify(&bytes).unwrap_or_else(|e| panic!("verify `{src}`: {}", e.message()));

            let want_facts: BTreeMap<String, FnFacts> =
                c.result.facts.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            assert_eq!(verified.facts, want_facts, "verify must agree with check_source: {src}");
        }
    }

    #[test]
    fn verify_is_comfortably_fast_for_per_load_use() {
        // Build-order deviation 1 condition (b): a rough timing witness that re-verification is fast
        // enough to run at EVERY load. Re-verification is a FULL same-code-path re-check (single-
        // module `resolve` + `check_module`) PLUS a stored-truth comparison — strictly stronger than
        // the §2.3 assert-replay description, and still linear and fast.
        let src = "module p\n\
            fn fib(n: Int) -> Int { if n < 2 { n } else { fib(n-1) + fib(n-2) } }\n\
            fn greet(out: Cap[Console], n: Str) ! {Write} { out.println(n) }\n\
            fn r(fs: Cap[FsRead]) -> Result[Str, IoErr] ! {Read} { fs.read_text(\"a.txt\") }\n\
            pub fn scan(fs: Cap[FsRead], p: Str) -> Result[Str, IoErr] ! {Read} { fs.read_text(p) }\n";
        let c = check_source(0, src);
        assert!(!c.has_errors(), "{:?}", c.diagnostics);
        let bytes = serialize(&c.module, &c.result);

        let n = 200u32;
        let start = std::time::Instant::now();
        for _ in 0..n {
            verify(&bytes).expect("verifies");
        }
        let per = start.elapsed() / n;
        // Comfortably fast: well under a millisecond per verify on a dev machine. The 5 ms ceiling is
        // generous enough to stay green on a loaded CI box while still catching a pathological blowup.
        assert!(per < std::time::Duration::from_millis(5), "verify per-load must be fast for per-load use: {per:?}");
        eprintln!("dir::verify per-load ≈ {per:?} (build-order deviation 1b timing witness)");
    }

    #[test]
    fn narrowed_declared_row_is_dl1504_via_the_boundary_rule() {
        // Tamper the code itself: drop the declared row from an effectful function. Re-checking the
        // body then performs Write with no declared effect — the checker's own T-Fn boundary rule
        // (DL0501) refutes it, surfaced as DL1504. This is the re-check catching a narrowed row.
        let c = check_source(0, "module m\nfn f(out: Cap[Console]) ! {Write} { out.println(\"x\") }\n");
        let mut dir = Dir::from_checked(&c.module, &c.result);
        for item in &mut dir.module.items {
            if let Item::Fn(f) = item {
                f.row = None; // now declares purity while the body still writes
            }
        }
        // Even a self-consistent forgery (also zero the stored facts) is caught, because the *body*
        // genuinely performs Write.
        if let Some(facts) = dir.facts.get_mut("f") {
            facts.effects.clear();
            facts.pure = true;
        }
        let bytes = dir.encode();
        match verify(&bytes) {
            Err(e @ DirError::VerifyFailed(_)) => {
                assert_eq!(e.code(), "DL1504");
                assert!(e.requires_human(), "DL1504 is requires_human and never falls back to Contained");
            }
            other => panic!("a narrowed row must be DL1504, got {other:?}"),
        }
    }

    #[test]
    fn forged_narrower_stored_facts_is_dl1504_via_comparison() {
        // Leave the code honest (so re-checking is clean) but forge the *stored* authority to claim
        // purity. The re-check recomputes {Write}; the comparison against the forged {} refutes it.
        let c = check_source(0, "module m\nfn f(out: Cap[Console]) ! {Write} { out.println(\"x\") }\n");
        let mut dir = Dir::from_checked(&c.module, &c.result);
        if let Some(facts) = dir.facts.get_mut("f") {
            facts.effects.clear();
            facts.pure = true;
        }
        let bytes = dir.encode();
        match verify(&bytes) {
            Err(DirError::VerifyFailed(_)) => {}
            other => panic!("forged narrower stored facts must be DL1504, got {other:?}"),
        }
    }

    #[test]
    fn forged_export_type_is_dl1504() {
        // Forge a function's stored static type. Re-checking recomputes the true type; the mismatch
        // is DL1504 — the foundation of the Verified per-function export check.
        let c = check_source(0, "module m\nfn f(n: Int) -> Int { n + 1 }\n");
        let mut dir = Dir::from_checked(&c.module, &c.result);
        if let Some(t) = dir.fn_types.get_mut("f") {
            *t = Type::Fn { params: vec![Type::Str], ret: Box::new(Type::Str), row: Row::pure() };
        }
        let bytes = dir.encode();
        assert!(matches!(verify(&bytes), Err(DirError::VerifyFailed(_))), "forged export type must be DL1504");
    }

    #[test]
    fn no_byte_flip_ever_verifies_with_altered_authority() {
        // The soundness fuzz witness for Phase 6b: over every single-byte, single-bit flip of a
        // valid DIR, `verify` either refuses or accepts — but an accepted flip's authority (facts,
        // types, node rows) is byte-identical to the original. A tamper can never escalate or alter
        // a *verifying* plugin's authority, and running the checker over thousands of mutated
        // modules never panics.
        let c = check_source(
            0,
            "module m\nfn f(out: Cap[Console], n: Str) ! {Write} { out.println(n) }\nfn g(x: Int) -> Int { x + 1 }\n",
        );
        let bytes = serialize(&c.module, &c.result);
        let orig = verify(&bytes).expect("the untampered DIR verifies");
        let (of, oft, ont, onr) =
            (orig.facts.clone(), orig.fn_types.clone(), orig.node_types.clone(), orig.node_rows.clone());

        for i in 0..bytes.len() {
            for bit in 0..8u32 {
                let mut t = bytes.clone();
                t[i] ^= 1 << bit;
                if let Ok(v) = verify(&t) {
                    assert_eq!(v.facts, of, "flip @{i}.{bit} verified with altered facts");
                    assert_eq!(v.fn_types, oft, "flip @{i}.{bit} verified with altered fn_types");
                    assert_eq!(v.node_types, ont, "flip @{i}.{bit} verified with altered node_types");
                    assert_eq!(v.node_rows, onr, "flip @{i}.{bit} verified with altered node_rows");
                }
            }
        }
    }

    #[test]
    fn a_malformed_dir_verifies_as_dl1504_never_falls_back() {
        // Truncated/garbage bytes are unverifiable Verified code: DL1504, requires_human, and by
        // invariant 29 there is no Contained fallback path to take.
        for bad in [&b""[..], &b"garbage"[..], &[0x9f, 0x9f, 0x00][..]] {
            match verify(bad) {
                Err(e) => assert_eq!(e.code(), "DL1504", "malformed DIR must verify as DL1504"),
                Ok(_) => panic!("malformed bytes must never verify"),
            }
        }
    }
}
