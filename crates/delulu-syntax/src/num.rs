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

/// RW 2.37: the text of a `Float`, one rule for the formatter's literal and the runtime's `str`. It is the shortest
/// digits that read back as `v` (Rust's `{:?}`), with an exponent below 1e-4 and from 1e16 up in magnitude — Python's
/// `repr` thresholds — written as DeluluLang spells a Float literal, with a fraction before the exponent (`1.0e300`,
/// `2.6881171418161356e43`), so the text a program prints is a literal that reads back as the same number. Before RW
/// 2.37 the runtime printed every digit (`str(1.0e300)` was 301 of them) and the formatter had its own copy of this rule
/// (FMT-FLOAT-EXP-1). A NaN is `NaN` and the infinities `inf` and `-inf`, as before.
pub fn float_text(v: f64) -> String {
    let s = format!("{v:?}");
    if !v.is_finite() {
        return s;
    }
    match s.split_once('e') {
        Some((mant, exp)) if !mant.contains('.') => format!("{mant}.0e{exp}"),
        Some(_) => s,
        None if s.contains('.') => s,
        None => format!("{s}.0"),
    }
}

/// RW 2.37: `float_text`, the one text of a `Float`.
#[cfg(test)]
mod float_text_tests {
    use super::*;

    #[test]
    fn the_exponent_starts_where_pythons_repr_starts_it() {
        for (v, want) in [
            (0.0001, "0.0001"),
            (0.00009999, "9.999e-5"),
            (1.0e16, "1.0e16"),
            (9999999999999998.0, "9999999999999998.0"),
            (123.0, "123.0"),
            (2.5, "2.5"),
            (-0.0, "-0.0"),
            (-2.5e-10, "-2.5e-10"),
            (1.0e300, "1.0e300"),
            (5e-324, "5.0e-324"),
            (f64::MAX, "1.7976931348623157e308"),
            (f64::NAN, "NaN"),
            (f64::INFINITY, "inf"),
            (f64::NEG_INFINITY, "-inf"),
        ] {
            assert_eq!(float_text(v), want, "{v:e}");
        }
    }

    /// Every finite `Float` prints as a literal that reads back as itself, through the rule the lexer and `parse_float`
    /// share — over generated bit patterns, every exponent and sign, not a hand-picked list.
    #[test]
    fn every_finite_float_prints_as_a_literal_that_reads_back_as_itself() {
        let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut checked = 0;
        for _ in 0..200_000 {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let v = f64::from_bits(seed);
            if !v.is_finite() {
                continue;
            }
            let text = float_text(v);
            assert!(text.contains('.'), "{text}: a Float literal has a fraction");
            // Subnormals included: the shared rule keeps them (`FloatText::Value`).
            match float_from_text(&text) {
                FloatText::Value(back) => assert_eq!(back.to_bits(), v.to_bits(), "{text}"),
                other => panic!("{text} read back as {other:?}"),
            }
            checked += 1;
        }
        assert!(checked > 190_000, "the generator must give finite floats ({checked})");
    }
}
