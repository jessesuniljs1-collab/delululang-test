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
    /// Why this repair carries no edits — the judgement the tool will not make.
    ///
    /// **A repair with `edits: []` is documentation of a decision, not something to apply**, and
    /// verification finding NE-07 found the two channels saying opposite things about the same
    /// one: `check --json` reported DL0502's `remove_effect_from_row` as `confidence: "safe"`,
    /// `requires_human: false`, `edits: []` — flags that read as *apply me* — while
    /// `fix --dry-run --json` correctly refused it with `verdict: "requires-human"`, and the LSP
    /// offered it as `isPreferred: true` with no `edit` at all. A harness following the flags
    /// would have concluded there was a fix and found nothing to do.
    ///
    /// [`Diagnostic::with_repair`] now makes an editless repair `requires_human` on the way in, so
    /// no site can forget, and this field says *why* rather than leaving a caller to guess from the
    /// id. `None` for a repair that carries edits.
    pub reason: Option<&'static str>,
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
    /// Typed placeholder values for catalog rendering (Stage 8, spec §6.1): `("fn", "greet")`.
    /// HUMAN-surface only — the JSON envelope never serializes these (invariant 39: `message`
    /// stays the in-code en-US prose on every machine channel). A catalog entry renders only
    /// when every placeholder it uses has a value here; otherwise the en-US message stands.
    pub args: Vec<(String, String)>,
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
            args: Vec::new(),
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
            args: Vec::new(),
        }
    }

    /// Attach a typed placeholder value for catalog rendering (Stage 8). The key must be one
    /// of the code's declared placeholders (`catalog::placeholders_for`) for any catalog to
    /// use it; unknown keys are harmless (they simply never render).
    pub fn with_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.args.push((key.into(), value.into()));
        self
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

    /// Attach a typed repair — the ONE path a repair reaches a diagnostic by, which is why the
    /// editless-repair rule is enforced here rather than at each construction site (NE-07).
    ///
    /// A repair with no edits cannot be applied by anything, so it must not advertise otherwise:
    /// `requires_human` is set on the way in. The `debug_assert` makes a site that forgot the
    /// `reason` fail loudly in tests instead of shipping a flag with no explanation; in release the
    /// normalization still holds, because failing closed matters more than being noisy.
    pub fn with_repair(mut self, mut repair: Repair) -> Self {
        if repair.edits.is_empty() {
            debug_assert!(
                repair.reason.is_some(),
                "repair `{}` carries no edits and no reason — say why a human must decide",
                repair.id
            );
            repair.requires_human = true;
        }
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
