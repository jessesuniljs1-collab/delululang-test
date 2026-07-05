//! DeluluLang syntax: tokens, lexer, AST, parser (Stage-1 spec §2–§4).
//!
//! Design commitments implemented here:
//! - Go-style automatic statement termination (§2.2), including the `} else` consequence.
//! - Reserved words rejected at declaration sites only (§2.3 precision) — `root.secret(…)`
//!   stays legal while `secret` stays reserved.
//! - Record literals not parsed in `if`/`while` conditions or `match` scrutinees (§3).
//! - Error-recovering parsing: multiple diagnostics per run, statement-boundary resync.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

use delulu_diag::{Diagnostic, FileId};

/// Lex + parse one file. Always returns a `Module` (possibly partial) so later
/// pipeline stages can keep producing diagnostics; callers decide on errors.
pub fn parse_file(file: FileId, src: &str) -> (ast::Module, Vec<Diagnostic>) {
    let (tokens, mut diags) = lexer::lex(file, src);
    let (module, parse_diags) = parser::parse(file, tokens);
    diags.extend(parse_diags);
    (module, diags)
}
