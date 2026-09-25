# The egress client's dependency cost (PS-B-02, owner ruling D-V2-30)

**What this measures.** D-NE-28 reserved one thing for the owner: the TLS dependency, because it "is
the largest this project would take and needs the owner and a `cargo deny` pass." This record is that
pass, taken **before and after** the manifest change so the number is a measurement rather than an
estimate. The owner ruled `reqwest` + `rustls` with a minimal, explicitly-named feature set on
2026-09-20; this is what it cost.

**Method.** Same machine, same `Cargo.lock` lineage, same toolchain, nothing else changed between the
two readings. The manifest edit is one block in `crates/delulu-runtime/Cargo.toml` adding
`reqwest = { version = "0.12", default-features = false, features = ["blocking",
"rustls-tls-native-roots"], optional = true }` behind a new default-on `net` feature.

The crate count is the DISTINCT crate names in `cargo deny list` — the whole resolved graph, every
target included. (This sentence said "and dev-dependencies included" until 2026-09-25, and was
vacuously true: the workspace HAD no dev-dependencies. When PS-B-02 added the first ones, `cargo deny
list` did not show them — its listing omits them — while `cargo deny check`, the actual gate, does
see them: a temporary `[bans] deny` of `rcgen` failed the bans check at once. The count method is
therefore stated precisely now, and the test-only crates are counted separately below.) That is deliberately the largest honest number: a
`cargo tree --edges normal` reading of the workspace is smaller (189 → some larger figure) because it
omits dev-dependencies and target-gated edges, and a dependency that only appears on another platform
is still a dependency in the lockfile and still something a reviewer has to read.

    $ cargo deny check                      # before and after
    $ cargo deny list -f json               # before and after, diffed

## Result

| | before | after | delta |
|---|---|---|---|
| distinct crates in the resolved graph | 222 | **303** | **+81 (+36.5%)** |
| distinct licenses | 14 | 15 | +1 |
| unlicensed crates | 0 | 0 | 0 |
| crates removed | — | — | **none** |
| `cargo deny check advisories` | ok | **ok** | no new advisory |
| `cargo deny check bans` | ok | **ok** | no new ban, no duplicate flagged |
| `cargo deny check licenses` | ok | **ok** | every new crate satisfies the allowlist |
| `cargo deny check sources` | ok | **ok** | every new crate from crates.io |

`cargo check --workspace --all-targets` passes with the dependency present.

### The one new license

`BSL-1.0` (Boost), arriving through `ryu` (float formatting, reached via `serde_urlencoded`). `ryu` is
`Apache-2.0 OR BSL-1.0`, so `Apache-2.0` is selectable and the project's own licence is unaffected;
`cargo deny` records the disjunction rather than a new obligation. No copyleft license entered the
graph: the `GPL-2.0-only` and `LGPL-2.1-or-later` entries visible in both readings predate this change
and are unchanged by it.

### The 81 crates, and the three parts worth naming

The full list is in `new-crates.txt`, one per line. Three groups deserve a reviewer's attention rather
than a count:

1. **`ring` + `untrusted` + `rustls-webpki`** — the cryptographic core under `rustls`. `ring` carries
   assembly and C. This is the dependency the owner was actually being asked to accept, and it is the
   reason the alternative (writing TLS) was never a real option: house rule 5 forbids hand-rolled
   cryptography, and this is what not hand-rolling it looks like.
2. **`wasm-bindgen`, `js-sys`, `web-sys`, `wasm-bindgen-futures`** — reqwest supports `wasm32` targets.
   These compile on **no platform this project builds** (they are target-gated), but they are in the
   graph and in the lockfile, so they are counted here. Counting them is the point: a reviewer reading
   the lockfile meets them, and a record that quietly excluded them would be measuring the flattering
   subset.
3. **The ICU stack — `icu_collections`, `icu_locale_core`, `icu_normalizer`, `icu_normalizer_data`,
   `icu_properties`, `icu_properties_data`, `icu_provider`, `litemap`, `potential_utf`, `tinystr`,
   `writeable`, `yoke`, `yoke-derive`, `zerofrom`, `zerofrom-derive`, `zerotrie`, `zerovec`,
   `zerovec-derive`, `utf8_iter` — nineteen of the eighty-one)** — reached through `idna`, which `url` uses for
   internationalized domain names. **This is the group with a security consequence, not just a size
   consequence.** IDNA is a NORMALIZATION performed on a host name, and a host name is a string this
   project compares to make a security decision. Four of campaign P22's defects were security decisions
   taken on an unnormalized or differently-spelled string; this tree introduces a normalizer that runs
   *inside the HTTP client*, after the allowlist check has already been made on the string the program
   wrote.

   So PS-B-02 must not assume the host it checked is the host `reqwest` connects to. Resolving the name
   host-side and **pinning the resolved address** (`ClientBuilder::resolve`) is what makes that
   answerable rather than a matter of trust: the client is handed an address, not a name to re-derive.
   A conformance case comparing the checked host against the connected host belongs in the network
   family, and a URL whose authority carries userinfo should be refused outright rather than parsed.

### What this record did not contain when it was written, and does now

When the dependency landed (`d0ae0f9`) **no line of code used `reqwest`** — on purpose, so the number
above could be taken against a tree where nothing else had changed. That made it the dependency's
cost, not evidence that the client is correct. The client arrived with PS-B-02 (2026-09-25):

**The shipped graph did not move.** `cargo deny list` still reads **303** distinct crates and **15**
licenses. The two new direct dependencies of `delulu-runtime` — `rustls` and `rustls-native-certs`,
named so the TLS configuration is built in this repository and a TLS failure is recognised by type —
were already in the graph through `reqwest`, at the same versions.

**Test-only: eight crates.** `rcgen` (a certificate generated at test time, so no private key is ever
committed) and the server half of `rustls` are dev-dependencies of `delulu-runtime`. Measured as the
difference between `cargo tree --workspace -e normal,build,dev --target all` and the same without `dev`:
`rcgen`, `pem`, `yasna`, `time`, `time-core`, `deranged`, `powerfmt`, `num-conv` — every one
`MIT OR Apache-2.0` or `MIT`, none reaching a release build. `Cargo.lock` gains more lines than that
(24 names), because a lockfile records optional dependencies nothing enables — `x509-parser` and its
family among them.

    $ cargo deny check      # after PS-B-02
    advisories ok, bans ok, licenses ok, sources ok

**Feature accounting is a gate now, not a comment.** `crates/delulu-runtime/tests/egress_features.rs`
asks `cargo metadata` — the same resolution a build makes — which reqwest features are ENABLED, and
fails on any refused one (`gzip`, `brotli`, `zstd`, `deflate`, `cookies`, `charset`, `http2`, `json`,
`default` and the rest) and on any feature D-V2-30 does not account for. It reads the resolved graph,
not the manifest line, because Cargo unifies features: another crate asking for `reqwest/gzip` would
turn it on while the manifest still read virtuously. Falsified: adding `gzip` fails both of its tests,
naming the feature.
