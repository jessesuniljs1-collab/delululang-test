//! The host reads these bytes from a guest it has assumed is compromised. Everything this target
//! checks lives in `delulu-runtime`, in `channel::fuzz_one_frame`, which the ordinary test suite
//! also calls over a seeded corpus: one rule, one place, so a coverage-guided run and a per-commit
//! run can never be testing different things.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    delulu_runtime::channel::fuzz_one_frame(data);
});
