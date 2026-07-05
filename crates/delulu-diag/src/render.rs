//! Human-facing rendering. This text is presentation, not contract: it may change
//! freely between versions (the JSON envelope is the stable surface). Optimized for
//! the human audiences the constitution names: learners and reviewers of AI code.

use std::fmt::Write as _;

use crate::diagnostic::Diagnostic;
use crate::source::SourceMap;

/// Render one diagnostic in a compact rustc-like layout:
///
/// ```text
/// error[DL0501]: function `fetch` performs effect `Net` not declared in its row
///   --> src/main.delulu:14:3
///    |
/// 14 |   out.println("hi")
///    |   ^^^^^^^^^^^^^^^^^ this call performs `Net`
/// ```
pub fn render_human(d: &Diagnostic, map: &SourceMap) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{}[{}]: {}", d.severity.as_str(), d.code, d.message);

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
        let mut underline = String::new();
        for _ in 0..line_start_col {
            underline.push(' ');
        }
        for _ in 0..underline_len.max(1) {
            underline.push_str(marker);
        }
        match &ls.label {
            Some(label) => {
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
        let _ = writeln!(out, "  repair: {} ({}){}", r.id, r.confidence.as_str(), flag_text);
    }

    let _ = writeln!(out, "  explain: delulu explain {}", d.explanation_id());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::span::Span;

    #[test]
    fn renders_with_caret_line() {
        let mut map = SourceMap::new();
        let f = map.add_file("src/main.delulu", "fn main() {\n  boom()\n}\n");
        let d = Diagnostic::error("DL0301", "unknown name `boom`")
            .with_span(Span::new(f, 14, 18), "not found in this scope");
        let text = render_human(&d, &map);
        assert!(text.contains("error[DL0301]"));
        assert!(text.contains("src/main.delulu:2:3"));
        assert!(text.contains("boom()"));
        assert!(text.contains("^^^^"));
        assert!(text.contains("not found in this scope"));
    }
}
