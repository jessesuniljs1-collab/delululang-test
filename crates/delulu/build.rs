// P5-04: what `delulu --version --json` says about the build itself — the target it was compiled for,
// and the commit it was built from when the builder said (`DELULU_BUILD_COMMIT`, which
// `scripts/package-toolchain.sh` and the release workflow set). Nothing is read from the environment
// otherwise: a build that was not told its commit reports `null`, never a guess, and two builds of the
// same tree told the same thing produce the same bytes (the microVM guest's reproducibility check
// depends on that).
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=DELULU_BUILD_COMMIT");
    let target = std::env::var("TARGET").unwrap_or_default();
    println!("cargo:rustc-env=DELULU_BUILD_TARGET={target}");
    if let Ok(commit) = std::env::var("DELULU_BUILD_COMMIT") {
        let commit = commit.trim();
        if !commit.is_empty() {
            println!("cargo:rustc-env=DELULU_BUILD_COMMIT={commit}");
        }
    }
}
