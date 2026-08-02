//! DeluluLang diagnostics.
//!
//! The machine contract (Stage-1 spec §10): every diagnostic has a stable code, byte-precise
//! spans, and — where applicable — typed repairs. English text is presentation; the JSON is
//! the contract. Repairs that widen authority are flagged `authority_widening` so agents and
//! CI can refuse them by policy; repairs toward secret declassification are never emitted.

mod catalog;
mod codes;
mod diagnostic;
mod json;
mod palette;
mod render;
mod source;
mod span;

pub use codes::{
    code_explain, code_title, is_registered, topic_explain, unallocated, CodeInfo, Disposition,
    UnallocatedCode, GUARD_BYPASS_BANNER, GUARD_CAVEAT, GUARD_POLICY_BOUND, MIN_EXPLAIN_BODY,
    REGISTRY, REVOCATION_BOUND, UNALLOCATED,
};
pub use diagnostic::{Confidence, Diagnostic, Edit, LabeledSpan, Repair, Severity};
pub use json::{envelope, envelope_to_string};
pub use palette::{
    color_enabled, named_color_sgr, resolve_theme, ColorChoice, Palette, Role, Theme, RESET,
};
pub use catalog::{cli_string, placeholders_for, Catalog, CLI_STRINGS};
pub use render::{render_human, render_human_localized, render_human_with};
pub use source::{SourceFile, SourceMap};
pub use span::{FileId, Span};
