//! The Stage-1 diagnostic code registry (spec §10.3).
//!
//! Codes are allocated within domain ranges and NEVER reused. Every diagnostic the compiler
//! produces must carry a code registered here — the conformance suite's meta-test asserts it.

pub struct CodeInfo {
    pub code: &'static str,
    pub title: &'static str,
}

macro_rules! registry {
    ( $( $code:literal => $title:literal ),* $(,)? ) => {
        pub const REGISTRY: &[CodeInfo] = &[
            $( CodeInfo { code: $code, title: $title } ),*
        ];
    };
}

registry! {
    // DL01xx — lexical
    "DL0101" => "unexpected character",
    "DL0102" => "unterminated string literal",
    "DL0103" => "invalid escape sequence",
    "DL0104" => "invalid numeric literal",
    "DL0105" => "unterminated block comment",
    "DL0106" => "reserved word used as a declared name",

    // DL02xx — parse
    "DL0201" => "expected a different token",
    "DL0202" => "expected an expression",
    "DL0203" => "expected a type",
    "DL0204" => "file must begin with a `module` declaration",
    "DL0205" => "expected a pattern",
    "DL0206" => "comparison operators are non-associative",
    "DL0207" => "invalid assignment target",
    "DL0208" => "expected an item",
    "DL0209" => "expected a statement terminator",

    // DL03xx — names / modules
    "DL0301" => "unknown name",
    "DL0302" => "duplicate definition",
    "DL0303" => "unknown module in import",
    "DL0304" => "import cycle",
    "DL0305" => "module-level mutable state is forbidden",
    "DL0306" => "unknown effect name",
    "DL0307" => "not a capability resource kind",

    // DL04xx — types
    "DL0401" => "type mismatch",
    "DL0402" => "mixed numeric types (no implicit coercion)",
    "DL0403" => "wrong number of arguments",
    "DL0404" => "not callable",
    "DL0405" => "unknown field or method",
    "DL0406" => "wrong number of type arguments",
    "DL0407" => "non-exhaustive match",
    "DL0408" => "condition must be Bool",
    "DL0409" => "`?` requires Result in a Result-returning function",
    "DL0410" => "generic variable used as both type and effect row",

    // DL05xx — effects
    "DL0501" => "function performs an effect not declared in its row",
    "DL0502" => "declared effect never performed",
    "DL0503" => "more than one row variable per signature",
    "DL0504" => "conflicting bindings for row variable (rows never union-merge)",

    // DL06xx — capabilities & secrets
    "DL0601" => "capability type cannot be constructed or forged",
    "DL0602" => "secret value cannot flow here (Secret[T] is not T)",
    "DL0603" => "Secret.map requires a pure function",
    "DL0604" => "opaque type cannot be stringified or serialized",
    "DL0605" => "opaque type has no structural equality",

    // DL07xx — manifest / authority
    "DL0701" => "main's effect row exceeds the authority manifest",
    "DL0702" => "authority grant refused",
    "DL0703" => "root slice not granted",

    // DL08xx — plugins & grants (types reserved in Stage 1; runtime lands Stage 6)
    "DL0801" => "call through revoked plugin reference",
    "DL0802" => "grant exceeds holder's grant (attenuation violation)",
    "DL0803" => "function-typed argument to Contained plugin export",

    // DL10xx — packages / authority versioning (Stage 2)
    "DL1001" => "dependency authority exceeds its pin",
    "DL1002" => "locked authority hash mismatch (same version, different authority)",
    "DL1003" => "semver-authority violation: authority widened without a major version bump",
    "DL1004" => "malformed package manifest",
    "DL1005" => "package or re-export cycle",
    "DL1006" => "import is ambiguous between a local module and a dependency",
    "DL1007" => "git dependency without a pinned rev or tag",
    "DL1008" => "version conflict for one package name in the graph",
    "DL1009" => "package performs an effect not permitted by its own authority manifest",
    "DL1010" => "content hash mismatch (source changed under a locked version)",
    "DL1011" => "locked build requires resolution not present in delulu.lock",

    // DL11xx — trace / fuzz harness (Stage 2)
    "DL1101" => "effect-trace assertion violation (a runtime effect not in the static row)",
    "DL1102" => "repair did not produce an accepting program (fuzz harness)",

    // DL12xx — WASM backend / .dwx artifact (Stage 3)
    "DL1201" => "construct not supported by the WASM backend (runs on the interpreter instead)",
    "DL1202" => "artifact missing or invalid `delulu:authority` section",
    "DL1204" => "delulu:cap interface version unsupported by this toolchain",
    "DL1205" => "guest closure over secret contents in `--target wasm`",
    "DL1206" => "engine parity self-check failure (compiler-bug class)",

    // DL09xx — runtime
    "DL0901" => "integer overflow",
    "DL0902" => "division by zero",
    "DL0903" => "index out of bounds",
    "DL0904" => "capability scope violation",
    "DL0905" => "recursion depth exceeded",
    "DL0906" => "explicit panic",
    "DL0907" => "match reached no arm (checker bug if ever seen)",
}

pub fn is_registered(code: &str) -> bool {
    REGISTRY.iter().any(|c| c.code == code)
}

pub fn code_title(code: &str) -> Option<&'static str> {
    REGISTRY.iter().find(|c| c.code == code).map(|c| c.title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_unique_and_well_formed() {
        for (i, a) in REGISTRY.iter().enumerate() {
            assert!(a.code.starts_with("DL") && a.code.len() == 6, "bad code {}", a.code);
            assert!(a.code[2..].chars().all(|c| c.is_ascii_digit()));
            for b in &REGISTRY[i + 1..] {
                assert_ne!(a.code, b.code, "duplicate code {}", a.code);
            }
        }
    }

    #[test]
    fn lookup_works() {
        assert!(is_registered("DL0501"));
        assert!(!is_registered("DL9999"));
        assert_eq!(code_title("DL0305"), Some("module-level mutable state is forbidden"));
    }
}
