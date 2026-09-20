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
target and dev-dependencies included. That is deliberately the largest honest number: a
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

### What this record does NOT yet contain

The measurement above is the dependency's cost. It is not yet evidence that the egress client is
correct, because at this commit **no line of code uses `reqwest`** — the dependency and its accounting
landed first, on purpose, so the number could be taken against a tree where nothing else had changed.
The client itself, its policy tests and the network conformance family are the rest of PS-B-02; see
`docs/DELULULANG_V2/V2_LOG.md` for the handoff that describes them.

**Feature accounting is owed a gate.** The manifest names every enabled feature and why every refused
one is absent. A comment explaining that decompression is off does not keep decompression off, so
PS-B-02 owes a test that reads the manifest and fails if `gzip`, `brotli`, `zstd`, `deflate`, `cookies`
or `charset` is ever enabled — the same shape as the gates P3 added for the arity column.
