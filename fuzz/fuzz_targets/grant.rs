//! RW 5.4 (V2 P7). The `--grant` parser: no input panics, and the verdict is a function of the input. The property is `delulu_runtime::broker::fuzz_one_grant` — the
//! ordinary suite replays the same function over a seeded corpus on every commit, so there is one rule.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    delulu_runtime::broker::fuzz_one_grant(data);
});
