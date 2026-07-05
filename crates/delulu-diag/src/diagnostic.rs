use crate::span::{FileId, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        }
    }
}

/// How mechanical a repair is. `Exact` repairs are byte-precise and safe to apply
/// blindly *unless* flagged `authority_widening` or `requires_human`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    Exact,
    Safe,
    Suggest,
}

impl Confidence {
    pub fn as_str(self) -> &'static str {
        match self {
            Confidence::Exact => "exact",
            Confidence::Safe => "safe",
            Confidence::Suggest => "suggest",
        }
    }
}

/// A byte-range text edit. `start_byte == end_byte` is a pure insertion.
#[derive(Clone, Debug)]
pub struct Edit {
    pub file: FileId,
    pub start_byte: u32,
    pub end_byte: u32,
    pub insert: String,
}

/// A typed repair (spec §10.2/§10.4). Repair ids are stable API, like codes.
/// Policy invariants enforced at construction sites, tested in the suite:
/// - anything that adds effects/capabilities/grants sets `authority_widening: true`;
/// - no repair ever inserts `expose` or requests `Cap[Declassify]`.
#[derive(Clone, Debug)]
pub struct Repair {
    pub id: &'static str,
    pub confidence: Confidence,
    pub authority_widening: bool,
    pub requires_human: bool,
    pub edits: Vec<Edit>,
}

#[derive(Clone, Debug)]
pub struct LabeledSpan {
    pub span: Span,
    pub label: Option<String>,
    pub secondary: bool,
}

/// One diagnostic. `code` must be registered in [`crate::REGISTRY`]; the first
/// non-secondary span is the primary location.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
    pub spans: Vec<LabeledSpan>,
    pub repairs: Vec<Repair>,
}

impl Diagnostic {
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        debug_assert!(crate::codes::is_registered(code), "unregistered code {code}");
        Diagnostic {
            code,
            severity: Severity::Error,
            message: message.into(),
            spans: Vec::new(),
            repairs: Vec::new(),
        }
    }

    pub fn warning(code: &'static str, message: impl Into<String>) -> Self {
        debug_assert!(crate::codes::is_registered(code), "unregistered code {code}");
        Diagnostic {
            code,
            severity: Severity::Warning,
            message: message.into(),
            spans: Vec::new(),
            repairs: Vec::new(),
        }
    }

    pub fn with_span(mut self, span: Span, label: impl Into<String>) -> Self {
        self.spans.push(LabeledSpan { span, label: Some(label.into()), secondary: false });
        self
    }

    pub fn with_bare_span(mut self, span: Span) -> Self {
        self.spans.push(LabeledSpan { span, label: None, secondary: false });
        self
    }

    pub fn with_secondary_span(mut self, span: Span, label: impl Into<String>) -> Self {
        self.spans.push(LabeledSpan { span, label: Some(label.into()), secondary: true });
        self
    }

    pub fn with_repair(mut self, repair: Repair) -> Self {
        self.repairs.push(repair);
        self
    }

    pub fn explanation_id(&self) -> String {
        format!("E-{}", self.code)
    }

    pub fn primary_span(&self) -> Option<Span> {
        self.spans
            .iter()
            .find(|s| !s.secondary)
            .or(self.spans.first())
            .map(|s| s.span)
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_and_primary_span() {
        let d = Diagnostic::error("DL0501", "row missing Net")
            .with_secondary_span(Span::new(0, 1, 2), "declared here")
            .with_span(Span::new(0, 10, 20), "this call performs `Net`");
        assert_eq!(d.primary_span(), Some(Span::new(0, 10, 20)));
        assert_eq!(d.explanation_id(), "E-DL0501");
        assert!(d.is_error());
    }
}
