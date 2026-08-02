# DeluluLang

> **Authority and effects are part of the type of every program — total, verifiable, and
> enforced across all code, all dependencies, and all runtime-loaded plugins.**

DeluluLang (`.delulu`) is a programming language where every function, module, and plugin
carries its **authority and effects in its type**, so the compiler can answer — mechanically —
the question no mainstream toolchain can: *"what can this program actually do to my system?"*

```delulu
fn fib(n: Int) -> Int {                       // provably pure: no row means !{}
  if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn greet(out: Cap[Console], name: Str) ! {Write} {   // the row is what it does;
  out.println("hello, " + name)                      // the capability is its permission
}
```

Primary users: AI agents, LLMs, and future AI systems — the population for whom unrestricted
authority is most dangerous. Humans use it to learn and to **review AI-written code**. No
discrimination between holders: the only asymmetry is the grant relation — what you grant
cannot exceed what you hold, and what you granted cannot break you.

## Status

**v1.0.0.** Stages 1–10 are built and closed, from the skeleton through the "Industrial" stage.
The language core has been change-frozen since Stage 9; `docs/design/STABILITY.md` is the contract.

What that does and does not mean:

| | |
|---|---|
| **Built and tested** | 12 crates, ~94,000 lines of Rust. 111 test suites, 1,449 tests passing. Conformance coverage is a hard per-commit gate at 100%, and [`tests/core-invariance/SNAPSHOT.txt`](tests/core-invariance/SNAPSHOT.txt) records the exact bytes the toolchain answers with for all 108 programs the repository ships — so work on the tooling cannot quietly move the language. The size figures are recounted from the tree by [the Survey](docs/survey/SURVEY.md) and a test fails when they drift. |
| **Mapped** | [`docs/survey/`](docs/survey/) — a map of this repository generated from this repository, where every edge cites the file and line it was read from. Start there before changing anything. |
| **Verified on** | Windows (native) and Linux (WSL), every gate green on both. |
| **Never executed on** | **macOS.** No Apple hardware is available to the project. The Unix code path is the same one Linux runs green, which is an argument, not an execution. |
| **Not distributed** | There is no release binary, no package-manager entry, and no public repository. You build from source. See [Installing](#installing). |
| **Licensed** | Code under **Apache-2.0**; the **DeluluLang** name is a trademark. Free to use, modify, and sell. See [License](#license). |
| **Not certified** | Under any regime, for any domain, including the autonomy domains Stage 10 addresses. |

An in-progress hardening campaign — testing every stage to failure and fixing what breaks — is
tracked in [`docs/design/HARDENING_CAMPAIGN.md`](docs/design/HARDENING_CAMPAIGN.md), including the
defects it has found so far and the ones still open.

## Installing

There is no download. Build it from this source tree.

**Prerequisites:** [rustup](https://rustup.rs). Nothing else — the toolchain version is pinned by
`rust-toolchain.toml` and rustup installs it automatically on first build. Do **not** run
`rustup default stable`; the pin exists so that every build is reproducible.

```sh
git clone <this repository>          # or use the tree you already have
cd DeluluLang
cargo build --release                # first build fetches and compiles dependencies
./target/release/delulu --help
```

Put it on your `PATH` with either of:

```sh
cargo install --path crates/delulu   # installs to ~/.cargo/bin
# or copy ./target/release/delulu (delulu.exe on Windows) wherever you keep binaries
```

`cargo test --workspace` runs the full suite if you want to verify the build yourself.

**Optional: building without Python.** The default build embeds CPython for the `foreign.python`
boundary. If you do not need it, or Python is awkward on your platform, build with
`--no-default-features` — a first-class, gate-tested configuration:

```sh
cargo build --release -p delulu --no-default-features
```

## Your first program

```sh
delulu new hello        # a package that already checks, tests and runs
cd hello
delulu run . --grant console
```

The generated package declares a ceiling equal to **exactly what its code does** — one effect —
because tightening that line is the habit worth forming. `delulu new hello --lib` starts a library,
whose ceiling is empty.

Or write it by hand. Save this as `hello.delulu`:

```delulu
module hello

fn main(root: Root) ! {Write} {
    let out = root.console()
    out.println("Hello, Delulu")
}
```

Check it, ask what it can do, then run it:

```sh
delulu check hello.delulu
delulu authority hello.delulu
delulu run hello.delulu --grant console
```

**The `--grant console` is not boilerplate — it is the whole language.** Leave it off and the
program fails with `DL0703`, because DeluluLang programs hold **zero ambient authority**: the right
to write to your terminal is something a human hands over, once, explicitly. Every capability works
this way — files, network, secrets, clocks, and actuators. `delulu authority` prints the complete
list for any program before you run it.

Next: [`docs/GETTING_STARTED.md`](docs/GETTING_STARTED.md) walks from here to writing real programs
— types, effects, errors, packages, and the parts of the language the samples do not cover.

## The commands that matter

```sh
delulu new <name> [--lib]            # a package that already checks, tests and runs
delulu add --path <dir>              # a dependency, pinned at exactly the authority it needs
delulu check <file>... | <package>   # diagnostics with typed, machine-applicable repairs
delulu fix <file>                    # apply them — never one that widens authority unless named
delulu authority <file|package>      # everything this program can do, computed from the code
delulu why <Effect> <file|package>   # why it can do that, at function granularity
delulu run <file> --grant K[=V]      # run it, under exactly the authority you hand over
delulu test                          # tests, each holding only its own declared row
delulu explain DL0501                # long-form docs for any diagnostic code
delulu fmt <path>                    # one canonical style, zero options
```

Add `--json` to any of them for the machine-readable envelope. `delulu --help` is the complete
reference and is kept in sync with the code by a build gate.

**Pass `check` all your files at once.** On Windows 82% of a small `check` is the operating system
creating a process — the compiler's own work on a 35-line file is under 1.5 ms — so twenty separate
invocations cost 711 ms where one costs 48. The numbers, the controls that produced them, and what
they rule out are in [`measurements/agent-loop/RECORD.md`](measurements/agent-loop/RECORD.md).

Shell completion is generated from that same command list, so it can never offer a command that
does not exist or miss one that does:

```sh
delulu completions bash        # also zsh, fish, powershell — see `delulu completions --help`
```

## Documentation

| Where | What |
|---|---|
| [`docs/GETTING_STARTED.md`](docs/GETTING_STARTED.md) | Install → first program → real programs. Start here. |
| [`docs/book/THE_DELULULANG_BOOK.md`](docs/book/THE_DELULULANG_BOOK.md) | The complete guide, 20 chapters. Read Ch. 6 if you review AI-written code. |
| [`docs/for-agents.md`](docs/for-agents.md) | The one page an agent harness should pin. |
| [`docs/reference/`](docs/reference/) | Generated reference: grammar, tokens, primitives, diagnostics, CLI contracts, coverage. |
| [`docs/design/CONSTITUTION.md`](docs/design/CONSTITUTION.md) | The v1.0 constitution — identity, semantics, honesty clauses. |
| [`docs/design/SOUNDNESS_AUDIT.md`](docs/design/SOUNDNESS_AUDIT.md) | Rules R-1…R-8 that keep authority in the type, and the five holes they close. |
| [`docs/design/STABILITY.md`](docs/design/STABILITY.md) | What is promised to stay put, and what is not. |
| [`SECURITY.md`](SECURITY.md) · [`CONTRIBUTING.md`](CONTRIBUTING.md) · [`rfcs/`](rfcs/) | Reporting, contributing, and changing the language. |

## Honesty

This project never claims "faster than C," "lowest tokens," or "unbreakable." Guarantees are
stated relative to a named threat model; strength comes from defense in depth (type-system proof
→ WASM/WASI floor → microVM containment → human-held broker keys), and every trust assumption
(compiler, hardware, hypervisor, side channels) is named. See the constitution, §5.14 and §9.

Three things worth knowing before you evaluate it:

- **Performance is measured, never promised.** v1.0 is **not competitive with C** on the measured
  workloads (2.0×–60.5× slower), and [`measurements/study-c/REPORT.md`](measurements/study-c/REPORT.md)
  says so in those words.
- **Foreign code is outside the proof.** A `ForeignCall` is a hole in the guarantee. It is
  *enumerated* in the authority report rather than hidden, which is the honest version, not a fix.
- **The guarantee is about the authority boundary, not intent.** A dependency that was always
  granted `Net` and starts using it differently is not caught by this, and nothing here claims
  otherwise.

## License

**Code: [Apache-2.0](LICENSE).** Free to use, modify, distribute, and sell — by anyone, human or
AI, for any purpose, including commercially. The license carries an explicit patent grant and, via
its `NOTICE` mechanism, keeps the authorship attribution attached through redistribution.

**Name: a trademark of Jesse Sunil** ([TRADEMARK.md](TRADEMARK.md)). The Apache-2.0 license covers
the *code*; it grants no rights in the *name*. You may build on, sell, and fork DeluluLang freely —
but a modified or derivative language must ship under a **different name** and must not present
itself as the original DeluluLang. This is the same separation Rust, Python, and Mozilla use: a
permissive code license plus a trademark policy so no one can be misled about what the original is
or who created it.

**Creator:** Jesse Sunil, permanently recorded in [`NOTICE`](NOTICE). Project governance is in
[`GOVERNANCE.md`](GOVERNANCE.md).

---

*Be delulu: write code as if no program can ever exceed its authority — then make the compiler
make it true.* 🐦‍🔥
