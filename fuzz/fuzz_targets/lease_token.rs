//! RW 5.4 (V2 P7). The string `run --lease` is handed: a broker that minted nothing redeems nothing. The property is `delulu_broker::lease::fuzz_one_lease_token` — the
//! ordinary suite replays the same function over a seeded corpus on every commit, so there is one rule.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    delulu_broker::lease::fuzz_one_lease_token(data);
});
