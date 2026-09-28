# Build the DeluluLang toolchain in a container, and ship only the binary.
#
# **STATUS: built by hand on GitHub** — `.github/workflows/container.yml` (workflow_dispatch). Written
# 2026-08-07 and never built until 2026-09-28, when the first build (run 36399908461) FAILED: this file
# copied only `crates/`, and `delulu` embeds `skills/` and `examples/` at compile time (10 errors,
# `couldn't read …`). It also built the default, Python-embedding binary while saying below that it
# does not. Both are fixed here; the workflow's next run is the first expected to pass, and
# docs/DELULULANG_V2/V2_LOG.md records what it said. Dockerfiles fail for reasons invisible by reading.
#
# Two stages, because the build needs a Rust toolchain and the result does not. The binary is the
# portable one the release ships (`--no-default-features --features net`, `scripts/package-toolchain.sh`):
# no embedded Python — a default build imports one specific libpython, which this runtime image does
# not have (INSTALL.md) — and the network client, so the runtime layer is genuinely small.

# ---- build ------------------------------------------------------------------------------------
# Pinned by digest-free tag deliberately: `rust-toolchain.toml` in the repository pins the exact
# toolchain, and rustup honours it, so the base image's own Rust version is not what decides the
# build. Pinning both would mean two places to update and one of them silently losing.
FROM rust:1-bookworm AS build

WORKDIR /src

# Copy the manifests first so dependency compilation caches independently of source edits.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
# Embedded at compile time (`include_str!`): `delulu skill` prints the Agent Skill and `delulu examples`
# the example programs, from the binary itself.
COPY skills/ skills/
COPY examples/ examples/

# `--locked` refuses to update Cargo.lock. In a container build that is the difference between
# reproducing the tested dependency set and quietly resolving a newer one.
RUN cargo build --release --locked -p delulu --no-default-features --features net

# ---- runtime ----------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# The network client verifies TLS against the platform's trust roots (D-V2-30), and a slim image has none.
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# `delulu run` executes user programs and needs no privileges to do it: the language's whole model is
# that a program holds only what it is granted on the command line. Running as a non-root user costs
# nothing here and removes the most common container footgun.
RUN useradd --create-home --shell /bin/bash delulu
USER delulu
WORKDIR /home/delulu/work

COPY --from=build /src/target/release/delulu /usr/local/bin/delulu
COPY --chown=delulu:delulu LICENSE NOTICE /usr/local/share/delulu/

# A container that cannot answer `--version` is a container that failed silently at COPY time.
RUN delulu --version

ENTRYPOINT ["delulu"]
CMD ["--help"]
