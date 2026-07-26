//! The language's one rule for turning written decimal text into a `Float`.
//!
//! Two places in the toolchain read a float out of text: the **lexer**, for a literal in source,
//! and the **runtime**, for `parse_float` on program input. They must agree. A lexer that refuses
//! `1.0e400` while `parse_float("1.0e400")` answers `Some(inf)` does not have one rule with two
//! callers — it has two rules, and the campaign has already paid for that shape once (C40, where
//! the broker and the runtime resolved a repeated envelope term in opposite directions).
//!
//! So the rule lives here, and both callers ask this function.
//!
//! The rule itself exists because `f64::from_str` does not fail on a magnitude it cannot hold.
//! It **saturates** to infinity and **flushes** to zero, and it accepts the words `inf`, `NaN`
//! and their variants outright. Every one of those returns a value that is not the value the
//! text names, silently. For source that is a typo; for input it is a way to put infinity into a
//! program that never wrote it.

/// What a piece of text denotes as a `Float`.
///
/// The failures are distinguished rather than collapsed into one `None`, because the lexer has to
/// say *which* mistake was made — "too large" and "too small" want different words in front of a
/// person, even though `parse_float` maps both to `None`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatText {
    /// A real number `f64` can hold. Subnormals are included: they lose precision, but they keep
    /// their magnitude, which is the property this rule is about.
    Value(f64),
    /// Larger than `f64::MAX` — `from_str` would answer infinity.
    Overflow,
    /// Nonzero, and smaller than the smallest subnormal — `from_str` would answer zero.
    Underflow,
    /// Not a number at all, or one of the non-finite *words* (`inf`, `infinity`, `NaN`).
    Malformed,
}

/// Read `text` as a `Float` under the language's rule.
///
/// `text` is taken as written: no trimming, and no underscore stripping. Callers normalise first
/// if their surface allows it (the lexer strips `_`; `parse_float` trims, and rejects `_` exactly
/// as `parse_int` does).
pub fn float_from_text(text: &str) -> FloatText {
    match text.parse::<f64>() {
        // Non-finite covers both the saturating overflow and the literal words. Checked before
        // anything else so no later arm has to remember that `inf` is possible here.
        Ok(v) if !v.is_finite() => {
            if has_nonzero_digit(text) {
                FloatText::Overflow
            } else {
                // `inf`, `-Infinity`, `NaN`: no digits, so nothing overflowed — the text simply
                // does not name a number this language will accept from text.
                FloatText::Malformed
            }
        }
        Ok(v) if v == 0.0 && has_nonzero_digit(text) => FloatText::Underflow,
        Ok(v) => FloatText::Value(v),
        Err(_) => FloatText::Malformed,
    }
}

/// Does the MANTISSA of `text` contain a digit other than zero?
///
/// Only the mantissa: `1.0e-400` has a nonzero digit in its exponent, and that says nothing about
/// whether the number written was zero. This is what separates the two ways text reaches `0.0` —
/// because it was written as zero (fine) or because its magnitude was flushed away (not fine).
fn has_nonzero_digit(text: &str) -> bool {
    let mantissa = text.split(['e', 'E']).next().unwrap_or(text);
    mantissa.chars().any(|c| c.is_ascii_digit() && c != '0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn representable_values_come_back_unchanged() {
        assert_eq!(float_from_text("1.5"), FloatText::Value(1.5));
        assert_eq!(float_from_text("0.0"), FloatText::Value(0.0));
        assert_eq!(float_from_text("-2.75"), FloatText::Value(-2.75));
        assert_eq!(float_from_text("1.7976931348623157e308"), FloatText::Value(f64::MAX));
        // Written as zero, so zero — the exponent cannot make it an underflow.
        assert_eq!(float_from_text("0.0e-400"), FloatText::Value(0.0));
        // A subnormal keeps its magnitude, so it is a value, not an underflow.
        match float_from_text("1.0e-320") {
            FloatText::Value(v) => assert!(v > 0.0 && v < f64::MIN_POSITIVE),
            other => panic!("a subnormal is representable, got {other:?}"),
        }
    }

    #[test]
    fn the_two_silent_failures_are_named_separately() {
        assert_eq!(float_from_text("1.0e400"), FloatText::Overflow);
        assert_eq!(float_from_text("-9.9e999"), FloatText::Overflow);
        assert_eq!(float_from_text("1.0e-400"), FloatText::Underflow);
        assert_eq!(float_from_text(&format!("0.{}1", "0".repeat(400))), FloatText::Underflow);
    }

    /// The words are the reason this cannot be a bare `is_finite` check on the parsed value: they
    /// carry no digits, so calling them an "overflow" would be a lie about what the text said.
    /// They matter most for `parse_float`, where the text is program INPUT — without this, a data
    /// file could hand a program infinity that no line of its source ever wrote.
    #[test]
    fn the_non_finite_words_are_malformed_not_overflow() {
        for s in ["inf", "-inf", "infinity", "Infinity", "NaN", "nan", "-NaN"] {
            assert_eq!(float_from_text(s), FloatText::Malformed, "{s}");
        }
    }

    #[test]
    fn text_that_is_not_a_number_is_malformed() {
        for s in ["", "abc", "1.2.3", "--1", "1_000.5", " 1.5"] {
            assert_eq!(float_from_text(s), FloatText::Malformed, "{s}");
        }
    }
}
