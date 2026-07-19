//! Study A — whole-program authority verification at scale (Stage 9d, spec §3.1).
//!
//! ## The claim under test
//! *A patch release that adds an effect anywhere in a dependency graph is caught, every time.*
//!
//! This is a claim of **mechanism**, not of heuristic, so the required catch rate is 100% and
//! nothing less counts. A heuristic at 97% is a good heuristic; a mechanism at 97% has a hole, and
//! the correct response is to find the hole rather than to soften the number.
//!
//! ## The injection, precisely
//! For each library in each chain, a variant is built where that library:
//! 1. gains an effect it did not have (it writes to the console), and
//! 2. declares that effect in its own manifest, and
//! 3. bumps its PATCH version (0.1.0 → 0.1.1).
//!
//! Step 2 matters. A naive injection that changes only the source is caught by the package's own
//! authority check (DL1009) — the package would be lying about itself, which is a much easier thing
//! to catch and not the scenario anyone fears. The real xz-shaped attack is a dependency that is
//! *honest about its new authority* and simply expects nobody to look. Step 3 matters for the same
//! reason: a patch bump is what a maintainer would actually ship, and semver says patches are safe.
//!
//! The consumer's lockfile pinned the old authority. The build must refuse.
//!
//! ## What is NOT claimed
//! That the corpus is representative of real-world code, that 25 packages is a large sample, or
//! that catching an authority change is the same as catching malice. A dependency that was always
//! allowed `Net` and starts using it differently is not caught by this mechanism, and nothing here
//! claims otherwise. See METHODOLOGY.md, threats to validity.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use crate::corpus::{self, Chain};

/// The outcome of one injection.
pub struct Injection {
    pub chain: String,
    pub archetype: String,
    /// The library that gained an effect.
    pub victim: String,
    /// How deep beneath the application it sits (1 = direct dependency).
    pub depth: usize,
    /// Whether the build refused.
    pub caught: bool,
    /// **The validity fence.** Whether the injected package compiles cleanly ON ITS OWN. If it
    /// does not, the mutation was broken code and any refusal downstream is a compile error the
    /// authority mechanism did not earn. An injection that fails this is not counted as caught.
    pub injection_valid: bool,
    /// The diagnostic codes the refusal carried.
    pub codes: Vec<String>,
    pub millis: u128,
}

/// A clean (uninjected) verification, for the time-to-verify numbers.
pub struct Baseline {
    pub chain: String,
    pub archetype: String,
    pub packages: usize,
    pub depth: usize,
    pub lock_millis: u128,
    pub verify_millis: u128,
    pub clean: bool,
}

/// A NEGATIVE CONTROL: the whole injection pipeline run with the mutation replaced by a no-op.
///
/// Without this, a build that refused *everything* would score a perfect 100% catch rate and look
/// like a triumph. The control proves the pipeline can produce a non-catch — that the experiment
/// is capable of failing to detect, and therefore that detecting means something.
pub struct Control {
    pub chain: String,
    /// The build must SUCCEED here. If it does not, the campaign's catches prove nothing.
    pub built_clean: bool,
    pub codes: Vec<String>,
}

pub struct Results {
    pub baselines: Vec<Baseline>,
    pub injections: Vec<Injection>,
    pub controls: Vec<Control>,
}

impl Results {
    /// Caught **and** earned: the injected package compiled, so the refusal came from the
    /// authority mechanism rather than from broken syntax.
    pub fn caught(&self) -> usize {
        self.injections.iter().filter(|i| i.caught && i.injection_valid).count()
    }
    /// Injections whose mutation did not compile. These are experiment defects, not results.
    pub fn invalid(&self) -> Vec<&Injection> {
        self.injections.iter().filter(|i| !i.injection_valid).collect()
    }
    pub fn catch_rate(&self) -> f64 {
        if self.injections.is_empty() {
            // No injections is NOT a 100% catch rate. An empty experiment proves nothing, and
            // reporting it as success is how a measurement program lies without saying anything
            // false.
            return 0.0;
        }
        100.0 * self.caught() as f64 / self.injections.len() as f64
    }
    /// Whether every negative control built clean — i.e. whether the pipeline is capable of NOT
    /// catching. A campaign that catches everything, including the no-ops, has measured nothing.
    pub fn controls_pass(&self) -> bool {
        !self.controls.is_empty() && self.controls.iter().all(|c| c.built_clean)
    }
    /// The mechanism claim holds only at exactly 100%, over a non-empty set, with **every**
    /// injection valid **and** every negative control clean. A run containing a broken mutation,
    /// or one where even the no-op was refused, is an invalid experiment rather than a pass.
    pub fn mechanism_holds(&self) -> bool {
        !self.injections.is_empty()
            && self.invalid().is_empty()
            && self.controls_pass()
            && self.caught() == self.injections.len()
    }
    pub fn missed(&self) -> Vec<&Injection> {
        self.injections.iter().filter(|i| !i.caught && i.injection_valid).collect()
    }
}

fn delulu_bin() -> PathBuf {
    // The binary built alongside this crate. `CARGO_BIN_EXE_delulu` is only set for the `delulu`
    // crate's own tests, so locate it beside our own executable instead.
    let exe = std::env::current_exe().expect("current exe");
    let dir = exe.parent().and_then(|d| if d.ends_with("deps") { d.parent() } else { Some(d) })
        .expect("target dir");
    dir.join(if cfg!(windows) { "delulu.exe" } else { "delulu" })
}

fn run(args: &[&str]) -> (bool, String, u128) {
    let t = Instant::now();
    let out = Command::new(delulu_bin())
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .expect("failed to run delulu");
    let ms = t.elapsed().as_millis();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text, ms)
}

/// Every `DLxxxx` code mentioned in output, in order of first appearance.
fn codes_in(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let bytes: Vec<char> = text.chars().collect();
    for i in 0..bytes.len().saturating_sub(5) {
        if bytes[i] == 'D' && bytes[i + 1] == 'L' && bytes[i + 2..i + 6].iter().all(|c| c.is_ascii_digit()) {
            let code: String = bytes[i..i + 6].iter().collect();
            if !out.contains(&code) {
                out.push(code);
            }
        }
    }
    out
}

/// Run the whole study against a working directory (which is created and populated).
pub fn run_study(work: &Path) -> std::io::Result<Results> {
    let chains = corpus::build();
    let mut baselines = Vec::new();
    let mut injections = Vec::new();
    let mut controls = Vec::new();

    for chain in &chains {
        let base = work.join("baseline");
        std::fs::create_dir_all(&base)?;
        corpus::write_chain(&base, chain)?;
        let app_dir = base.join(&chain.name).join(&chain.app().dir);
        let app_s = app_dir.to_string_lossy().to_string();

        let (lock_ok, lock_out, lock_ms) = run(&["lock", &app_s]);
        let (build_ok, build_out, verify_ms) = run(&["build", &app_s, "--locked"]);

        baselines.push(Baseline {
            chain: chain.name.clone(),
            archetype: chain.archetype.clone(),
            packages: chain.packages.len(),
            depth: chain.depth(),
            lock_millis: lock_ms,
            verify_millis: verify_ms,
            clean: lock_ok && build_ok,
        });
        if !lock_ok || !build_ok {
            // A corpus that does not build clean would make every "catch" meaningless — the build
            // would be failing for reasons that have nothing to do with the injection. Surface it
            // rather than proceeding to collect a fake 100%.
            eprintln!(
                "corpus chain `{}` does not verify clean before injection:\nlock: {lock_out}\nbuild: {build_out}",
                chain.name
            );
            continue;
        }

        // THE NEGATIVE CONTROL: the identical pipeline — fresh copy, lock, rebuild — with no
        // mutation at all. It must build CLEAN. If it refuses, every "catch" below is suspect,
        // because the campaign would be refusing regardless of what was injected.
        {
            let ctl_root = work.join(format!("control-{}", chain.name));
            std::fs::create_dir_all(&ctl_root)?;
            corpus::write_chain(&ctl_root, chain)?;
            let ctl_app = ctl_root.join(&chain.name).join(&chain.app().dir);
            let ctl_s = ctl_app.to_string_lossy().to_string();
            let (lock_ok, _, _) = run(&["lock", &ctl_s]);
            let (ok, out, _) = run(&["build", &ctl_s, "--locked"]);
            controls.push(Control {
                chain: chain.name.clone(),
                built_clean: lock_ok && ok,
                codes: codes_in(&out),
            });
        }

        // One injection per library in the chain.
        for (i, victim) in chain.libs().iter().enumerate() {
            let inj_root = work.join(format!("inject-{}-{}", chain.name, victim.name));
            std::fs::create_dir_all(&inj_root)?;
            corpus::write_chain(&inj_root, chain)?;

            let app_dir = inj_root.join(&chain.name).join(&chain.app().dir);
            let app_s = app_dir.to_string_lossy().to_string();
            // Lock against the HONEST corpus first: this is the consumer's pin, taken before the
            // malicious release exists.
            let (ok, out, _) = run(&["lock", &app_s]);
            assert!(ok, "pre-injection lock must succeed: {out}");

            inject(&inj_root, chain, victim)?;

            // THE VALIDITY FENCE: does the mutated package compile on its own? If it does not,
            // the downstream refusal is a compile error and proves nothing about authority
            // verification. Checked before the result is recorded, not asserted in prose.
            let victim_dir = inj_root.join(&chain.name).join(&victim.dir);
            let (valid, _, _) = run(&["check", &victim_dir.to_string_lossy()]);

            let (build_ok, build_out, ms) = run(&["build", &app_s, "--locked"]);
            injections.push(Injection {
                chain: chain.name.clone(),
                archetype: chain.archetype.clone(),
                victim: victim.name.clone(),
                depth: i + 1,
                caught: !build_ok,
                injection_valid: valid,
                codes: codes_in(&build_out),
                millis: ms,
            });
        }
    }

    Ok(Results { baselines, injections, controls })
}

/// The xz-shaped mutation: the victim library gains an effect, declares it honestly, and ships it
/// as a PATCH release.
fn inject(root: &Path, chain: &Chain, victim: &corpus::Package) -> std::io::Result<()> {
    let dir = root.join(&chain.name).join(&victim.dir);

    // 1 + 2. The library now writes, and its manifest says so. An attacker publishing this is not
    // lying about their own package — that is exactly what makes it dangerous.
    // The victim already receives `root` (it passes it down). A patch release starts *using* it:
    // it derives a console and writes. The public signature is unchanged apart from the effect
    // row — nothing at the call site looks different to a human skimming the diff.
    //
    // The injected code is deliberately VALID, compiling DeluluLang. A broken mutation would fail
    // the build for reasons that have nothing to do with authority, and would be scored as a catch
    // the mechanism did not earn. `verify_injection_compiles` fences that.
    let marker = "(root: Root, x: Str) -> Str {";
    let src = victim.source.replacen(
        marker,
        "(root: Root, x: Str) -> Str ! {Write} {\n  \
         let phone_home = root.console()\n  \
         phone_home.println(x)\n",
        1,
    );
    assert_ne!(
        src, victim.source,
        "the injection did not apply to `{}` — a no-op mutation would be scored as an uncaught \
         attack that never happened",
        victim.name
    );
    std::fs::write(dir.join(&victim.source_path), src)?;

    let manifest = victim
        .manifest
        .replace("version = \"0.1.0\"", "version = \"0.1.1\"")
        .replace("effects = []", "effects = [\"Write\"]");
    std::fs::write(dir.join("delulu.toml"), manifest)?;
    Ok(())
}

/// The machine-readable results (raw data, spec §3).
pub fn json(results: &Results) -> serde_json::Value {
    use serde_json::json;
    json!({
        "study": "A",
        "title": "whole-program authority verification at scale",
        "schema": "study-a/1",
        "corpus": {
            "chains": results.baselines.len(),
            "packages": results.baselines.iter().map(|b| b.packages).sum::<usize>(),
            "min_depth": results.baselines.iter().map(|b| b.depth).min().unwrap_or(0),
        },
        "injection": {
            "sites": results.injections.len(),
            "caught": results.caught(),
            "catch_rate_pct": results.catch_rate(),
            "mechanism_holds": results.mechanism_holds(),
            "invalid_mutations": results.invalid().iter().map(|m| json!({
                "chain": m.chain, "victim": m.victim
            })).collect::<Vec<_>>(),
        },
        "negative_controls": {
            "runs": results.controls.len(),
            "all_clean": results.controls_pass(),
            "detail": results.controls.iter().map(|c| json!({
                "chain": c.chain, "built_clean": c.built_clean, "codes": c.codes
            })).collect::<Vec<_>>(),
            "missed": results.missed().iter().map(|m| json!({
                "chain": m.chain, "victim": m.victim, "depth": m.depth
            })).collect::<Vec<_>>(),
        },
        "baselines": results.baselines.iter().map(|b| json!({
            "chain": b.chain,
            "archetype": b.archetype,
            "packages": b.packages,
            "depth": b.depth,
            "lock_ms": b.lock_millis,
            "verify_ms": b.verify_millis,
            "clean": b.clean,
        })).collect::<Vec<_>>(),
        "injections": results.injections.iter().map(|i| json!({
            "chain": i.chain,
            "archetype": i.archetype,
            "victim": i.victim,
            "depth": i.depth,
            "caught": i.caught,
            "injection_valid": i.injection_valid,
            "codes": i.codes,
            "ms": i.millis,
        })).collect::<Vec<_>>(),
    })
}

/// The human-readable report.
pub fn report(results: &Results) -> String {
    let mut s = String::new();
    s.push_str("# Study A — whole-program authority verification at scale\n\n");
    s.push_str("*Generated by `delulu-measure study-a`. Raw data: `results.json`. Method and \
                threats to validity: `METHODOLOGY.md`.*\n\n");

    let pkgs: usize = results.baselines.iter().map(|b| b.packages).sum();
    let min_depth = results.baselines.iter().map(|b| b.depth).min().unwrap_or(0);
    s.push_str(&format!(
        "## Corpus\n\n{} chains, {pkgs} packages, minimum dependency depth {min_depth}.\n\n",
        results.baselines.len()
    ));
    s.push_str("| Chain | Shaped like | Packages | Depth | Lock (ms) | Verify (ms) | Clean |\n\
                |---|---|---|---|---|---|---|\n");
    for b in &results.baselines {
        s.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} |\n",
            b.chain, b.archetype, b.packages, b.depth, b.lock_millis, b.verify_millis,
            if b.clean { "yes" } else { "**NO**" }
        ));
    }

    s.push_str(&format!(
        "\n## The injection campaign\n\n\
         An effect was injected at **every library position in every chain** — {} sites. Each \
         injection adds an effect, declares it honestly in the victim's own manifest, and ships it \
         as a PATCH version bump, which is what a real supply-chain attack looks like: not a package \
         lying about itself, but one being truthful about a change nobody reads.\n\n\
         **Caught: {} of {} ({:.1}%).**\n\n",
        results.injections.len(),
        results.caught(),
        results.injections.len(),
        results.catch_rate()
    ));

    s.push_str(&format!(
        "Every mutation was checked to COMPILE on its own before its result was recorded \
         ({} of {} valid). This fence matters: an earlier version of this study injected code that \
         did not parse, and scored a clean 100% that the authority mechanism had not earned — the \
         builds were failing on syntax. An injection that does not compile is an experiment defect, \
         not a catch.\n\n",
        results.injections.len() - results.invalid().len(),
        results.injections.len()
    ));

    if !results.invalid().is_empty() {
        s.push_str("**INVALID EXPERIMENT.** These mutations did not compile, so their results \
                    prove nothing:\n\n");
        for m in results.invalid() {
            s.push_str(&format!("- `{}` in chain `{}`\n", m.victim, m.chain));
        }
        s.push('\n');
    }

    s.push_str(&format!(
        "### The negative control\n\nThe identical pipeline — fresh copy, lock, rebuild — run with \
         **no mutation at all**, once per chain. Every one must build clean, otherwise the campaign \
         is refusing regardless of what was injected and a 100% catch rate would mean nothing.\n\n\
         **{} of {} controls built clean.**\n\n",
        results.controls.iter().filter(|c| c.built_clean).count(),
        results.controls.len()
    ));
    if !results.controls_pass() {
        s.push_str("**The control FAILED.** The catch rate above is not evidence of anything.\n\n");
    }

    if results.mechanism_holds() {
        s.push_str("The claim under test is a claim of *mechanism*, so the required rate is 100% \
                    and it is met — over valid mutations, against a control that proves the \
                    pipeline can decline to catch. A mechanism that caught 97% would have a hole to \
                    find, not a number to round.\n\n");
    } else if results.injections.is_empty() {
        s.push_str("**No injections ran.** This is a failed experiment, not a passing one.\n\n");
    } else {
        s.push_str("**The mechanism claim does NOT hold.** The misses below are holes to fix:\n\n");
        for m in results.missed() {
            s.push_str(&format!("- `{}` in chain `{}` at depth {}\n", m.victim, m.chain, m.depth));
        }
        s.push('\n');
    }

    s.push_str("### By depth\n\nDepth 1 is a direct dependency; depth 4 is the leaf, four levels \
                below the application — the position an attacker would prefer.\n\n\
                | Depth | Sites | Caught |\n|---|---|---|\n");
    for d in 1..=4 {
        let at: Vec<&Injection> = results.injections.iter().filter(|i| i.depth == d).collect();
        if at.is_empty() {
            continue;
        }
        let c = at.iter().filter(|i| i.caught).count();
        s.push_str(&format!("| {d} | {} | {c} |\n", at.len()));
    }

    let mut codes: std::collections::BTreeMap<String, usize> = Default::default();
    for i in &results.injections {
        for c in &i.codes {
            *codes.entry(c.clone()).or_default() += 1;
        }
    }
    s.push_str("\n### The refusals\n\nWhich diagnostics did the catching:\n\n| Code | Injections |\n|---|---|\n");
    for (c, n) in &codes {
        s.push_str(&format!("| `{c}` | {n} |\n"));
    }

    let total_verify: u128 = results.baselines.iter().map(|b| b.verify_millis).sum();
    s.push_str(&format!(
        "\n## Time to verify\n\nVerifying all {} packages across {} graphs took **{} ms** in total \
         ({} ms mean per graph). These are wall-clock numbers from one machine and one run; they \
         are reported as measured and are not a performance claim (see the stability contract: \
         performance is measured, never promised).\n",
        pkgs,
        results.baselines.len(),
        total_verify,
        if results.baselines.is_empty() { 0 } else { total_verify / results.baselines.len() as u128 }
    ));
    s
}
