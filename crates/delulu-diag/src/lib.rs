//! DeluluLang diagnostics.
//!
//! The machine contract (Stage-1 spec §10): every diagnostic has a stable code, byte-precise
//! spans, and — where applicable — typed repairs. English text is presentation; the JSON is
//! the contract. Repairs that widen authority are flagged `authority_widening` so agents and
//! CI can refuse them by policy; repairs toward secret declassification are never emitted.

mod codes;
mod diagnostic;
mod json;
mod render;
mod source;
mod span;

pub use codes::{code_title, is_registered, CodeInfo, REGISTRY};
pub use diagnostic::{Confidence, Diagnostic, Edit, LabeledSpan, Repair, Severity};
pub use json::{envelope, envelope_to_string};
pub use render::render_human;
pub use source::{SourceFile, SourceMap};
pub use span::{FileId, Span};
