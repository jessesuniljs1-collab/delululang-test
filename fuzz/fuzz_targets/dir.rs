//! RW 5.4 (V2 P7). A `.dpx` plugin's compiled DIR: hostile CBOR, never a panic, canonical re-encoding. The property is `delulu_check::fuzz::fuzz_one_dir` — the
//! ordinary suite replays the same function over a seeded corpus on every commit, so there is one rule.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    delulu_check::fuzz::fuzz_one_dir(data);
});
