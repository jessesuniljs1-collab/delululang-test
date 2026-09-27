//! The fuzz properties for the checker's parsers of untrusted bytes (RW 5.4, V2 P7).
//!
//! Each `fuzz_one_*` is the ONE copy of its rule: the `cargo-fuzz` target in `fuzz/fuzz_targets/`
//! calls it coverage-guided under AddressSanitizer on every push, and the ordinary suite below replays
//! it over a seeded, deterministic mutation corpus on every commit — the arrangement PS-A-02 chose for
//! the channel, so a coverage-guided run and a per-commit run cannot be testing different things.
//!
//! Chosen because an attacker controls the bytes: a program an agent hands over (the source), the
//! `delulu.toml` / plugin manifest / lockfile of a package someone cloned, and a compiled DIR carried
//! inside a `.dpx` plugin (whose decoder promises "never a panic" on hostile input).

/// Program text, all the way through the checker: lexing, parsing (whose four nesting classes are
/// bounded, DL0210–DL0213), resolution and inference. No input may panic or hang.
pub fn fuzz_one_source(data: &[u8]) {
    if let Ok(src) = std::str::from_utf8(data) {
        let _ = crate::check_source(0, src);
    }
}

/// The package manifest, a plugin's manifest, and a lockfile: every one read from a directory
/// somebody else wrote. No input may panic.
pub fn fuzz_one_manifest(data: &[u8]) {
    if let Ok(src) = std::str::from_utf8(data) {
        let _ = crate::manifest::Manifest::parse(src, 0);
        let _ = crate::plugin::PluginManifest::parse(src, 0);
        let _ = crate::lockfile::Lockfile::parse(src);
    }
}

/// A compiled DIR from a `.dpx`: hostile CBOR must be an `Err`, never a panic — and anything that
/// DOES decode must re-encode to bytes that decode to the same thing (canonical, so a signature over
/// it means one program).
pub fn fuzz_one_dir(data: &[u8]) {
    if let Ok(dir) = crate::dir::Dir::decode(data) {
        let once = dir.encode();
        let again = crate::dir::Dir::decode(&once).expect("a DIR that decoded re-encodes to one that decodes");
        assert_eq!(again.encode(), once, "DIR re-encoding is stable");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic byte mutator: flips, inserts, deletes and splices over the seeds, so the
    /// per-commit replay reaches past the seeds' own shapes without any randomness in the suite.
    pub(crate) fn corpus(seeds: &[Vec<u8>], n: usize, mut property: impl FnMut(&[u8])) {
        let mut s: u64 = 0x5EED_F022_D11A_7E57;
        let mut next = move || {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        };
        for seed in seeds {
            property(seed);
        }
        for _ in 0..n {
            let mut b = seeds[(next() % seeds.len() as u64) as usize].clone();
            for _ in 0..1 + next() % 4 {
                let at = if b.is_empty() { 0 } else { (next() % b.len() as u64) as usize };
                match next() % 5 {
                    0 if !b.is_empty() => b[at] ^= 1 << (next() % 8),
                    1 => b.insert(at, next() as u8),
                    2 if !b.is_empty() => {
                        b.remove(at);
                    }
                    3 => {
                        let other = &seeds[(next() % seeds.len() as u64) as usize];
                        if !other.is_empty() {
                            let from = (next() % other.len() as u64) as usize;
                            let take = (next() % 16) as usize;
                            let piece: Vec<u8> = other[from..(from + take).min(other.len())].to_vec();
                            b.splice(at..at, piece);
                        }
                    }
                    _ => b.push(b"{}()[]\"=\n.:;,!#"[(next() % 15) as usize]),
                }
            }
            property(&b);
        }
    }

    fn n() -> usize {
        if cfg!(miri) {
            16
        } else {
            1_500
        }
    }

    #[test]
    fn the_source_property_holds_over_a_seeded_corpus() {
        let seeds: Vec<Vec<u8>> = [
            "module a\n\nfn main(root: Root) ! {Write} {\n    let out = root.console()\n    out.println(\"hi\")\n}\n",
            "module b\n\ntype P[T] = { x: T, y: List[T] }\n\nfn f(p: P[Int]) -> Int {\n    match p.y.len() {\n        0 => p.x,\n        n => n + p.x\n    }\n}\n",
            "module c\n\nenum E { A(Int), B(Str) }\n\nfn g(e: E) -> Str {\n    match e { A(i) => str(i), B(s) => s }\n}\n",
        ]
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect();
        corpus(&seeds, n(), fuzz_one_source);
    }

    #[test]
    fn the_manifest_property_holds_over_a_seeded_corpus() {
        let seeds: Vec<Vec<u8>> = [
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n\n[authority]\neffects = [\"Write\"]\n\n[dependencies]\nutil = { path = \"../util\" }\n",
            "[plugin]\nname = \"p\"\nversion = \"1.0.0\"\nclass = \"verified\"\napi = 1\n\n[ceiling]\neffects = [\"Read\"]\n",
            "version = 1\n\n[[package]]\nname = \"util\"\nsource = \"path+../util\"\nhash = \"blake3:00\"\n",
        ]
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect();
        corpus(&seeds, n(), fuzz_one_manifest);
    }

    #[test]
    fn the_dir_property_holds_over_a_seeded_corpus() {
        let src = "module d\n\nfn add(a: Int, b: Int) -> Int {\n    a + b\n}\n";
        let checked = crate::check_source(0, src);
        let seed = crate::dir::Dir::from_checked(&checked.module, &checked.result).encode();
        assert!(crate::dir::Dir::decode(&seed).is_ok(), "the seed itself decodes");
        corpus(&[seed, vec![0xa1, 0x00, 0x00], vec![]], n(), fuzz_one_dir);
    }
}
