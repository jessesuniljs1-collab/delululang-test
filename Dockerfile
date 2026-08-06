# Build the DeluluLang toolchain in a container, and ship only the binary.
#
# **STATUS: WRITTEN, NOT BUILT.** No `docker build` of this file has been executed — the Docker CLI
# is present on the authoring machine but its daemon was not running. It is therefore *prepared*,
# not *verified*, and is recorded that way in docs/design/CROSS_PLATFORM_VERIFICATION.md rather than
# counted as a passing platform. Treat the first `docker build` as an experiment, not a formality.
#
# Two stages, because the build needs a Rust toolchain and the result does not. `delulu` links no
# Python and embeds no interpreter (see INSTALL.md), so the runtime layer is genuinely small.

# ---- build ------------------------------------------------------------------------------------
# Pinned by digest-free tag deliberately: `rust-toolchain.toml` in the repository pins the exact
# toolchain, and rustup honours it, so the base image's own Rust version is not what decides the
# build. Pinning both would mean two places to update and one of them silently losing.
FROM rust:1-bookworm AS build

WORKDIR /src

# Copy the manifests first so dependency compilation caches independently of source edits.
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/

# `--locked` refuses to update Cargo.lock. In a container build that is the difference between
# reproducing the tested dependency set and quietly resolving a newer one.
RUN cargo build --release --locked -p delulu

# ---- runtime ----------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

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
