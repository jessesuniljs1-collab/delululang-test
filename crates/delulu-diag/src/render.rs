//! Human-facing rendering. This text is presentation, not contract: it may change
//! freely between versions (the JSON envelope is the stable surface). Optimized for
//! the human audiences the constitution names: learners and reviewers of AI code.

use std::fmt::Write as _;

use crate::catalog::Catalog;
use crate::diagnostic::{Diagnostic, Severity};
use crate::palette::{Palette, Role};
use crate::source::SourceMap;

/// Render one diagnostic in a compact rustc-like layout, with no color:
///
/// ```text
/// error[DL0501]: function `fetch` performs effect `Net` not declared in its row
///   --> src/main.delulu:14:3
///    |
/// 14 |   out.println("hi")
///    |   ^^^^^^^^^^^^^^^^^ this call performs `Net`
/// ```
///
/// This is exactly [`render_human_with`] under a disabled palette — the two are byte-identical,
/// which is why every pre-existing (non-TTY) test keeps passing (criterion 10).
pub fn render_human(d: &Diagnostic, map: &SourceMap) -> String {
    render_human_with(d, map, &Palette::none())
}

/// The color-aware renderer. With `palette` disabled the output is byte-for-byte identical to
/// [`render_human`]; when enabled it paints the severity/code, span carets + labels, and repairs
/// via semantic [`Role`]s so the theme decides the colors.
pub fn render_human_with(d: &Diagnostic, map: &SourceMap, palette: &Palette) -> String {
    render_human_localized(d, map, palette, None)
}

/// The locale-aware renderer (Stage 8, spec §6.1): with a catalog, the HEADER message renders
/// from the catalog's template when the entry exists and every placeholder it uses has a value
/// in `d.args` — otherwise (and always with `None`) the in-code en-US message stands. This is
/// the ONLY seam a catalog touches: codes, spans, labels, repairs, and the entire JSON envelope
/// never pass through here (invariant 39 by construction).
pub fn render_human_localized(
    d: &Diagnostic,
    map: &SourceMap,
    palette: &Palette,
    catalog: Option<&Catalog>,
) -> String {
    let localized = catalog.and_then(|c| c.render(d.code, &d.args));
    let message: &str = localized.as_deref().unwrap_or(&d.message);
    let sev_role = match d.severity {
        Severity::Error => Role::Error,
        Severity::Warning => Role::Warning,
        Severity::Note => Role::Note,
    };
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{}[{}]: {}",
        palette.paint(sev_role, d.severity.as_str()),
        palette.paint(Role::Code, d.code),
        message
    );

    for ls in &d.spans {
        let span = ls.span;
        let (line, col) = map.position(span.file, span.start);
        let arrow = if ls.secondary { "---" } else { "-->" };
        let _ = writeln!(out, "  {} {}:{}:{}", arrow, map.name(span.file), line, col);

        let text = map.line_text(span.file, line);
        let gutter_w = line.to_string().len().max(2);
        let _ = writeln!(out, "{:w$} |", "", w = gutter_w);
        let _ = writeln!(out, "{:w$} | {}", line, text, w = gutter_w);

        // Underline: clamp to the first line of the span.
        let line_start_col = (col - 1) as usize;
        let (end_line, end_col) = map.position(span.file, span.end);
        let underline_len = if end_line == line && end_col > col {
            (end_col - col) as usize
        } else {
            (text.chars().count().saturating_sub(line_start_col)).max(1)
        };
        let marker = if ls.secondary { "-" } else { "^" };
        let span_role = if ls.secondary { Role::SpanSecondary } else { Role::SpanPrimary };
        let mut underline = String::new();
        for _ in 0..line_start_col {
            underline.push(' ');
        }
        for _ in 0..underline_len.max(1) {
            underline.push_str(marker);
        }
        let underline = palette.paint(span_role, &underline);
        match &ls.label {
            Some(label) => {
                let label = palette.paint(span_role, label);
                let _ = writeln!(out, "{:w$} | {} {}", "", underline, label, w = gutter_w);
            }
            None => {
                let _ = writeln!(out, "{:w$} | {}", "", underline, w = gutter_w);
            }
        }
    }

    for r in &d.repairs {
        let mut flags = Vec::new();
        if r.authority_widening {
            flags.push("widens authority — review before applying");
        }
        if r.requires_human {
            flags.push("requires human decision");
        }
        let flag_text = if flags.is_empty() { String::new() } else { format!("  [{}]", flags.join("; ")) };
        let _ = writeln!(
            out,
            "  repair: {} ({}){}",
            palette.paint(Role::Repair, r.id),
            r.confidence.as_str(),
            flag_text
        );
    }

    let _ = writeln!(out, "  explain: delulu explain {}", d.explanation_id());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette::{Palette, Theme};
    use crate::span::Span;

    fn sample() -> (SourceMap, Diagnostic) {
        let mut map = SourceMap::new();
        let f = map.add_file("src/main.delulu", "fn main() {\n  boom()\n}\n");
        let d = Diagnostic::error("DL0301", "unknown name `boom`")
            .with_span(Span::new(f, 14, 18), "not found in this scope");
        (map, d)
    }

    #[test]
    fn renders_with_caret_line() {
        let (map, d) = sample();
        let text = render_human(&d, &map);
        assert!(text.contains("error[DL0301]"));
        assert!(text.contains("src/main.delulu:2:3"));
        assert!(text.contains("boom()"));
        assert!(text.contains("^^^^"));
        assert!(text.contains("not found in this scope"));
    }

    /// A disabled palette renders byte-for-byte identically to `render_human` — the guarantee that
    /// keeps every pre-existing non-TTY test green (criterion 10).
    #[test]
    fn disabled_palette_matches_render_human_byte_for_byte() {
        let (map, d) = sample();
        assert_eq!(render_human(&d, &map), render_human_with(&d, &map, &Palette::none()));
    }

    /// An enabled palette paints the severity, the code, and the caret — and never colors the
    /// machine text elsewhere (criterion 8: color is presentation only).
    #[test]
    fn enabled_palette_colors_severity_code_and_caret() {
        let (map, d) = sample();
        let text = render_human_with(&d, &map, &Palette::new(true, Theme::default_theme()));
        assert!(text.contains("\x1b[1;31merror\x1b[0m"), "severity painted: {text}");
        assert!(text.contains("\x1b[1mDL0301\x1b[0m"), "code painted: {text}");
        assert!(text.contains("\x1b["), "carets painted");
        // The underlying message text survives intact for a human to read.
        assert!(text.contains("unknown name `boom`"));
    }
}
