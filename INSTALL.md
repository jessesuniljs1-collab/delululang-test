# Installing DeluluLang

Three ways to get the toolchain, and one that deliberately does not exist. Each is stated with what
it costs you, because an install page that only lists the happy path is how people end up debugging
a build at the exact moment they were deciding whether to bother.

## 1. From a release archive — no Rust toolchain needed

The shortest path. Unpack, put the binary on your `PATH`, done.

```
tar -xzf delulu-<version>-<target>.tar.gz
cd delulu-<version>-<target>
sha256sum -c SHA256SUMS          # verify before you trust it
./bin/delulu --version
```

Then the first two minutes with the language, which are the same two minutes in `INSTALL.txt`
inside the archive:

```
delulu check     examples/hello_wasm.delulu     # does it type- and effect-check?
delulu authority examples/hello_wasm.delulu     # what may it do to my machine?
delulu run       examples/hello_wasm.delulu     # REFUSED: exits 1 with DL0703
delulu run       examples/hello_wasm.delulu --grant console
```

**The third command is meant to fail.** It exits 1, reports `DL0703`, and names the exact flag that
would allow it. That is the language working: a program receives no authority it was not handed, and
the refusal arrives before anything happens rather than after.

**What this build cannot do:** it carries no embedded Python. `root.python(...)` returns `DL1307`
(unavailable) — the same behaviour as a machine with no interpreter, reported rather than crashed.
That is deliberate, and §3 explains why.

Build the archive yourself with `scripts/package-toolchain.sh`; it is the same script that produces
the published one.

## 2. From source — needs Rust (see `rust-toolchain.toml` for the pinned version)

```
git clone <repository> DeluluLang && cd DeluluLang
cargo install --path crates/delulu        # installs to ~/.cargo/bin
```

This gives you the **Python-capable** build, because `default = ["python"]`. Use it if you want
`root.python(...)` to work. For the portable configuration instead:

```
cargo install --path crates/delulu --no-default-features
```

## 3. Not available: `cargo install delulu` from crates.io

This is a decision, not an oversight, and it is unlikely to change.

`delulu` depends on nine sibling crates by path. Publishing it to crates.io would require publishing
all of them — and `docs/design/STABILITY.md` §2 promises the opposite: *the Rust crates are an
implementation detail; the stable interface is the language, the CLI and the machine schemas.* Every
crate but the CLI carries `publish = false` to make that promise mechanical rather than aspirational.

Uploading them would mint a semver contract over roughly seventeen public Rust modules that the
project has explicitly told you not to depend on, in exchange for one shorter command. The archive in
§1 is the trade the project makes instead.

## Why the shipped binary has no Python, in one paragraph

A default `cargo build --release` embeds CPython through `pyo3`, and the resulting executable imports
a **specific** interpreter — `python313.dll` on the machine this was written on. Not "Python", not
"Python 3": that build. Anyone without that exact version cannot start it, and the failure arrives as
a loader error before `main`, where no diagnostic of ours can help. Measured, not assumed: the
default binary carries that import and the `--no-default-features` binary carries none.

So the download is the portable one. Python capability is a from-source choice, made by someone who
knows which interpreter their machine has.

## Verifying an archive you did not build

`SHA256SUMS` covers every file in the archive and is generated from the staged tree, so it describes
what is actually inside rather than what was intended to be. Check it before running anything:

```
sha256sum -c SHA256SUMS                 # Linux/macOS
Get-FileHash bin\delulu.exe             # Windows PowerShell, compare by eye
```

## Platform honesty

**Windows and Linux** are built, tested, and verified — the suite, the gates and a hand-driven
CLI/compiler sweep all run green on both. **macOS has never been executed**: the code is written for
it and its conditional-compilation branches were audited, but the one attempt so far — CI's macOS
runner, 2026-09-14 — stopped while building a dependency's bundled C code, before any DeluluLang code
ran. A path that should work and a path that has been run are different claims.
`docs/design/CROSS_PLATFORM_VERIFICATION.md` carries the detail.
