# Installing DeluluLang

Three ways to get the toolchain, and one that deliberately does not exist. Each is stated with what
it costs you, because an install page that only lists the happy path is how people end up debugging
a build at the exact moment they were deciding whether to bother.

## 1. From a release archive — no Rust toolchain needed

The shortest path. An archive is built for four targets — `x86_64-unknown-linux-gnu`,
`aarch64-unknown-linux-gnu`, `aarch64-apple-darwin` and `x86_64-pc-windows-msvc` — and comes with a
`.sha256` beside it. Download both, then, in a POSIX shell (on Windows, Git Bash):

```sh install-gate
sha256sum -c delulu-<version>-<target>.tar.gz.sha256   # the download is what was built
tar -xzf delulu-<version>-<target>.tar.gz
cd delulu-<version>-<target>
sha256sum -c SHA256SUMS                                # and so is every file inside it
export PATH="$PWD/bin:$PATH"                           # put this in your shell profile to keep it
cd ..
delulu --version
delulu new hello
cd hello
delulu test .                                          # one test, holding no authority at all
delulu authority . --grants                            # what to type to run it: --grant console
delulu run . --grant console
delulu run . --grant console --sandbox                 # the same program, as a confined guest
```

**This block is a test, not a recollection.** `scripts/check-install.sh` reads it out of this page
and runs it, as written, against a real archive; the release workflow does that on all four targets,
so a command here that stopped working would fail a build before it failed a reader. On macOS without
`sha256sum`, use `shasum -a 256` with the same arguments; in PowerShell, compare
`Get-FileHash <archive>` with the `.sha256` by eye.

The last line is the sandbox: the program runs as a guest that holds no authority of its own, and the
host performs each thing it asks for under the same grants. `delulu sandbox probe` says what this
machine can confine.

Then the examples in the archive, which are the same two minutes as `INSTALL.txt` inside it:

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

Build the archive yourself with `scripts/package-toolchain.sh`; the release workflow
(`.github/workflows/release.yml`) runs the same script.

## 2. From source — needs Rust (see `rust-toolchain.toml` for the pinned version)

```
git clone <repository> DeluluLang && cd DeluluLang
cargo install --path crates/delulu        # installs to ~/.cargo/bin
```

This gives you the **Python-capable** build, because `default = ["python", "net"]`. Use it if you
want `root.python(...)` to work. For the portable configuration instead — the one the archive ships:

```
cargo install --path crates/delulu --no-default-features --features net
```

`--features net` keeps the network client (`http.get` over verified HTTPS). `--no-default-features`
alone builds a delulu with neither Python nor a network client, in which every `http.get` answers
`Err(Refused)`; `delulu doctor` says which one you have, on its `network client` line.

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

A checksum says the bytes are the bytes that were packed; it does not say who packed them. For that,
the release workflow attests each archive's provenance — the repository, commit and workflow run that
built it — and the binary names its own build:

```
gh attestation verify delulu-<version>-<target>.tar.gz -R <repository>
delulu --version --json                 # "target" and "commit": what it was built for, and from
```

A binary built from a tree with uncommitted changes says so: its commit ends in `-dirty`. One built
without being told its commit says `null`, never a guess.

## Platform honesty

**Windows and Linux** are built, tested, and verified — the suite, the gates and a hand-driven
CLI/compiler sweep all run green on both. **macOS is verified by CI**: on 2026-09-14 the macOS runner
(Apple Silicon) passed the whole suite — 1,657 tests, 0 failed — and every gate after it, the CLI
sweep and the 50,000-program fuzz campaign included. Nothing has run on a Mac outside CI: no
developer's machine, and not the VS Code extension. On a Mac, keep a custom state directory short —
the broker's socket lives inside it, and macOS allows 103 bytes of socket path.
`docs/design/CROSS_PLATFORM_VERIFICATION.md` carries the detail.
