//! Stage 10 phase 10j — the deploy plan authority gate (Track H, spec §9.2, invariant 53).
//!
//! `delulu deploy plan` computes a whole-deployment authority answer — effects per service —
//! before anything launches, and checks it against an environment profile's authority ceiling
//! (`envs/*.toml`, the SAME `[authority]` manifest shape `delulu.toml` uses). A plan whose services
//! exceed that ceiling refuses with DL1909. "What can this deployment do to my cloud account?"
//! is supposed to get the same mechanical answer *before launch* that `delulu authority` already
//! gives for a single package — never a partial answer, never a silent widening when the profile
//! could not be read.
//!
//! Three laws are tested here, and the second is the one worth the file:
//!
//! 1. **The ceiling is exact, not a strict inequality.** A service whose effects equal the
//!    profile's ceiling — not a subset, identical — must be approved. This is the case a naive
//!    "strictly less than" comparison gets wrong first.
//! 2. **"No ceiling declared" must mean "grants nothing", never "nothing is checked".** An
//!    environment profile with an absent, empty, or unparseable `[authority]` section is the
//!    STRICTEST possible ceiling, not the most permissive one — and the direction of that mistake
//!    matters: getting it backwards means every deploy passes trivially, which is the one failure
//!    this whole mechanism exists to prevent (mirrors this project's skip-branch rule).
//! 3. **"Does not check" and "exceeds the profile" are different failures with different codes.**
//!    A service with a syntax error is a plain error, never DL1909 — conflating the two would send
//!    an operator to edit an environment profile that was never the problem.
//!
//! Every refusal test below is paired with a control that approves, so a build that refuses
//! everything cannot pass this file by accident (compute_cli.rs's rule, applied here).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

use delulu_runtime::parse_manifest;

// ----- process + fixture plumbing -----------------------------------------------------------

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(repo_root())
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}
/// Both streams concatenated, for substring checks that should not care which one carries the
/// message — the contract pins exit codes and JSON fields precisely, but not which stream a given
/// human-readable line lands on.
fn combined(o: &Output) -> String {
    format!("{}{}", stdout(o), stderr(o))
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-deploy-plan-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Write a minimal, cleanly-checking package at `dir` whose `main` actually PERFORMS exactly
/// `effects` (not merely declares them) — one of `[]` (pure), `["Write"]`, `["Net"]`, or
/// `["Net","Write"]`, the only combinations this file's tests need. The declared
/// `[authority] effects` line in `delulu.toml` matches `main`'s real row, so every fixture this
/// function builds is truthful, matching the shape used across this codebase's existing package
/// fixtures (see e.g. `signing_cli.rs::scaffold_pkg`, `provenance.rs`).
fn write_service(dir: &Path, module: &str, effects: &[&str]) {
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let mut set: Vec<&str> = effects.to_vec();
    set.sort_unstable();
    let (row, body): (&str, String) = match set.as_slice() {
        [] => ("", String::new()),
        ["Write"] => ("! {Write} ", "  let c = root.console()\n  c.println(\"up\")\n".to_string()),
        ["Net"] => (
            "! {Net} ",
            "  let h = root.http([\"example.com\"])\n  \
             match h.get(\"http://example.com\") { Ok(_) => {}, Err(_) => {} }\n"
                .to_string(),
        ),
        ["Net", "Write"] => (
            "! {Write, Net} ",
            "  let c = root.console()\n  c.println(\"up\")\n  \
             let h = root.http([\"example.com\"])\n  \
             match h.get(\"http://example.com\") { Ok(_) => {}, Err(_) => {} }\n"
                .to_string(),
        ),
        other => panic!("write_service: unsupported effect combination {other:?}"),
    };
    let src = format!("module {module}\nfn main(root: Root) {row}{{\n{body}}}\n");
    std::fs::write(dir.join("src/main.delulu"), src).unwrap();

    let declared: Vec<String> = set.iter().map(|e| format!("\"{e}\"")).collect();
    std::fs::write(
        dir.join("delulu.toml"),
        format!(
            "[package]\nname = \"{module}\"\nversion = \"0.1.0\"\n\n[authority]\neffects = [{}]\n",
            declared.join(", ")
        ),
    )
    .unwrap();
}

fn write_env(dir: &Path, filename: &str, contents: &str) -> PathBuf {
    let p = dir.join(filename);
    std::fs::write(&p, contents).unwrap();
    p
}

/// Run `delulu deploy plan --service NAME=DIR [--service ...] --env ENVFILE [--json]`, exactly the
/// contract's flag shape.
fn plan(services: &[(&str, PathBuf)], env: &Path, json: bool) -> Output {
    let mut args: Vec<String> = vec!["deploy".to_string(), "plan".to_string()];
    for (name, dir) in services {
        args.push("--service".to_string());
        args.push(format!("{name}={}", dir.display()));
    }
    args.push("--env".to_string());
    args.push(env.display().to_string());
    if json {
        args.push("--json".to_string());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    delulu(&refs)
}

/// A service's sorted effect list from a `deploy plan --json` `services` array, found by name.
fn service_effects(services: &[Value], name: &str) -> Vec<String> {
    let s = services
        .iter()
        .find(|s| s["name"] == name)
        .unwrap_or_else(|| panic!("missing service `{name}` among: {services:?}"));
    let mut es: Vec<String> = s["effects"]
        .as_array()
        .unwrap_or_else(|| panic!("service `{name}` has no effects array: {s}"))
        .iter()
        .filter_map(|x| x.as_str().map(str::to_string))
        .collect();
    es.sort();
    es
}

// ----- 0. pins the assumption the malformed-env tests below rely on ---------------------------

/// PINS AN ASSUMPTION the malformed-env CLI tests below depend on: `parse_manifest` never errors,
/// and text matching none of its `[section]` / `key = value` patterns degrades to a `Manifest`
/// with an EMPTY effects list — never to one read as "unconstrained". If a future rewrite of the
/// parser ever changed that (say, by treating unparseable content as "no ceiling declared, so
/// nothing is checked"), the CLI tests below would start failing for a reason worth knowing
/// immediately from THIS test, not from re-deriving it after sixty lines of process-spawning.
#[test]
fn parse_manifest_degrades_unparseable_prose_to_an_empty_ceiling_not_an_unbounded_one() {
    let m = parse_manifest(
        "this file is not shaped like an environment profile at all\n\
         someone meant to write toml here and did not\n\
         please read the docs before deploying anything\n",
    );
    assert!(m.effects.is_empty(), "garbled prose must not be read as declaring ANY effect: {:?}", m.effects);
}

// ----- 1. the boundary case ---------------------------------------------------------------------

/// THE BOUNDARY CASE. A service whose effects are EXACTLY the profile's ceiling — not a subset,
/// not over it, identical — must be approved. This is the case a naive "strictly less than" or an
/// off-by-one "subset-but-not-equal" check gets wrong first: a profile is a ceiling a plan may
/// touch, not a strict upper bound it must stay under.
#[test]
fn a_service_whose_effects_exactly_equal_the_ceiling_is_approved() {
    let root = scratch("boundary");
    let svc = root.join("edge");
    write_service(&svc, "edge", &["Write", "Net"]);
    let env = write_env(&root, "env.toml", "[authority]\neffects = [\"Write\", \"Net\"]\n");

    let o = plan(&[("edge", svc)], &env, false);
    let text = combined(&o);
    assert_eq!(o.status.code(), Some(0), "effects == ceiling must be APPROVED, not refused:\n{text}");
    assert!(text.contains("edge"), "the approved report must list the service:\n{text}");
}

// ----- 2. an empty/absent ceiling grants nothing, not everything --------------------------------

/// A profile with NO `[authority]` section at all still means "grants nothing" — not "no ceiling
/// declared, so nothing is checked". THE SKIP BRANCH: a profile a human forgot to fill in must
/// refuse everything non-trivial, never wave everything through because the ceiling was never
/// expressed as a concrete list.
#[test]
fn an_absent_authority_section_grants_nothing_not_everything() {
    let root = scratch("absent-authority");
    let env = write_env(&root, "env.toml", "# an environment profile someone forgot to fill in\n");

    let pure_dir = root.join("puresvc");
    write_service(&pure_dir, "puresvc", &[]);
    let o = plan(&[("puresvc", pure_dir)], &env, false);
    assert_eq!(
        o.status.code(),
        Some(0),
        "a PURE service performs no effect, so even a nothing-ceiling must still approve it:\n{}",
        combined(&o)
    );

    let write_dir = root.join("writersvc");
    write_service(&write_dir, "writersvc", &["Write"]);
    let o = plan(&[("writersvc", write_dir)], &env, false);
    let text = combined(&o);
    assert_eq!(o.status.code(), Some(1), "a real effect must be refused against a NOTHING ceiling:\n{text}");
    assert!(text.contains("DL1909"), "the refusal is the authority-ceiling code: {text}");
}

/// The same law, spelled differently: `effects = []` is not textually ABSENT, so this catches an
/// implementation that only special-cased the missing-section form and treated a present-but-empty
/// list some other way (e.g. "unspecified", defaulting open instead of defaulting shut).
#[test]
fn an_explicitly_empty_effects_list_grants_nothing_the_same_as_absent() {
    let root = scratch("empty-list");
    let env = write_env(&root, "env.toml", "[authority]\neffects = []\n");

    let pure_dir = root.join("puresvc");
    write_service(&pure_dir, "puresvc", &[]);
    let o = plan(&[("puresvc", pure_dir)], &env, false);
    assert_eq!(o.status.code(), Some(0), "a PURE service must still pass an empty ceiling:\n{}", combined(&o));

    let write_dir = root.join("writersvc");
    write_service(&write_dir, "writersvc", &["Write"]);
    let o = plan(&[("writersvc", write_dir)], &env, false);
    let text = combined(&o);
    assert_eq!(o.status.code(), Some(1), "a real effect must be refused against an EMPTY ceiling:\n{text}");
    assert!(text.contains("DL1909"), "the refusal is the authority-ceiling code: {text}");
}

// ----- 3. multiple services, only one exceeding --------------------------------------------------

/// Three services, one over the ceiling. The refusal must name THAT service and THAT effect — not
/// a vague "a service exceeded", and not the wrong one — and the WHOLE plan must refuse: spec §9.2
/// is a whole-deployment answer, not a per-service pass/fail that quietly drops the offender and
/// approves the rest.
#[test]
fn only_the_exceeding_service_is_named_and_the_whole_plan_refuses() {
    let root = scratch("multi-refuse");
    let env = write_env(&root, "env.toml", "[authority]\neffects = [\"Write\"]\n");

    let a = root.join("webfront");
    write_service(&a, "webfront", &["Write"]);
    let b = root.join("leakynet");
    write_service(&b, "leakynet", &["Net"]);
    let c = root.join("cachetool");
    write_service(&c, "cachetool", &["Write"]);

    let o = plan(&[("webfront", a), ("leakynet", b), ("cachetool", c)], &env, false);
    let err = stderr(&o);
    assert_eq!(o.status.code(), Some(1), "one service over the ceiling refuses the WHOLE plan:\n{err}");
    assert!(err.contains("DL1909"), "{err}");
    assert!(err.contains("leakynet"), "the refusal must name the exceeding SERVICE:\n{err}");
    assert!(err.contains("Net"), "the refusal must name the exceeding EFFECT:\n{err}");
    assert!(
        !err.contains("webfront") && !err.contains("cachetool"),
        "compliant services must not be blamed for the one that exceeded:\n{err}"
    );
}

// ----- 4. a malformed environment profile refuses, it does not silently widen -------------------

/// A malformed environment profile — text not shaped like TOML/manifest syntax at all — must never
/// be read as "no ceiling declared, so nothing is checked". `parse_manifest` (pinned above)
/// degrades unparseable content to an EMPTY effects list, which the ordinary DL1909 rule already
/// refuses any real-effect service against — so a correct implementation needs no special case
/// here, only that nothing upstream turns "couldn't parse" into "nothing to enforce".
#[test]
fn a_malformed_env_file_refuses_a_real_service_rather_than_approving_it() {
    let root = scratch("malformed-prose");
    let env = write_env(
        &root,
        "broken.toml",
        "this file is not shaped like an environment profile at all\n\
         someone meant to write toml here and did not\n\
         please read the docs before deploying anything\n",
    );
    let svc = root.join("writersvc");
    write_service(&svc, "writersvc", &["Write"]);

    let o = plan(&[("writersvc", svc)], &env, true);
    let text = combined(&o);
    // THE HARD SAFETY PROPERTY: whatever code path this hits, it must never look like approval.
    assert_ne!(
        o.status.code(),
        Some(0),
        "THE DANGEROUS DIRECTION: a profile the parser cannot make sense of must never let a \
         real-effect service through as approved — that is every deploy passing trivially, the \
         exact failure this mechanism exists to prevent:\n{text}"
    );
    if let Ok(v) = serde_json::from_slice::<Value>(&o.stdout) {
        assert_ne!(v["approved"], true, "the JSON verdict must not claim approval either: {v}");
    }
    // The contract-consistent PREDICTION, given parse_manifest's traced behaviour: unparseable
    // content degrades to an empty ceiling, refused by the SAME DL1909 rule as the empty/absent
    // profile tests above — not a distinct code path. (Judgment call — see report.)
    assert_eq!(o.status.code(), Some(1), "expected the ordinary DL1909 refusal path:\n{text}");
    assert!(text.contains("DL1909"), "{text}");
}

/// The same danger, forced through a different door: an env file that is not even valid UTF-8, so
/// `std::fs::read_to_string` fails outright before `parse_manifest` ever runs. An implementation
/// that maps "could not read the file" to "no profile, so nothing to check" would approve
/// everything — the same wrong-direction failure as the prose case, reached a different way. Only
/// the hard safety property is asserted here; the exact exit code for an unreadable file (as
/// opposed to a readable-but-meaningless one) is not pinned by the contract.
#[test]
fn an_unreadable_env_file_refuses_a_real_service_rather_than_approving_it() {
    let root = scratch("malformed-binary");
    let env = root.join("binary.toml");
    std::fs::write(&env, [0xFFu8, 0xFEu8, 0x00u8, 0xC0u8, 0xC1u8, 0x80u8]).unwrap();
    let svc = root.join("writersvc");
    write_service(&svc, "writersvc", &["Write"]);

    let o = plan(&[("writersvc", svc)], &env, false);
    let text = combined(&o);
    assert_ne!(
        o.status.code(),
        Some(0),
        "an unreadable env file must never be treated as an unconstrained profile:\n{text}"
    );
}

// ----- 5. a service that fails its own check is a plain error, not DL1909 -----------------------

/// A service directory that does not check cleanly is a PLAIN error — never DL1909. Conflating
/// "does not check" with "exceeds the profile" would misdiagnose a broken service as an authority
/// problem and send an operator to edit an environment profile that was never the issue. Mixed
/// with a perfectly fine service in the same plan, so the broken one cannot hide.
#[test]
fn a_service_that_fails_its_own_check_is_a_plain_error_not_dl1909() {
    let root = scratch("bad-check");
    let env = write_env(&root, "env.toml", "[authority]\neffects = [\"Write\"]\n");

    let broken = root.join("brokensvc");
    std::fs::create_dir_all(broken.join("src")).unwrap();
    std::fs::write(
        broken.join("src/main.delulu"),
        // Deliberately unbalanced parens — a guaranteed parse error, not a subtler type error.
        "module brokensvc\nfn main(root: Root) ! {Write} {\n  let c = root.console(\n  c.println(\"never gets here\")\n}\n",
    )
    .unwrap();
    std::fs::write(
        broken.join("delulu.toml"),
        "[package]\nname = \"brokensvc\"\nversion = \"0.1.0\"\n\n[authority]\neffects = [\"Write\"]\n",
    )
    .unwrap();

    let fine = root.join("finesvc");
    write_service(&fine, "finesvc", &["Write"]);

    let o = plan(&[("finesvc", fine), ("brokensvc", broken)], &env, false);
    let text = combined(&o);
    assert_eq!(o.status.code(), Some(1), "a broken service must fail the plan, plainly:\n{text}");
    assert!(
        !text.contains("DL1909"),
        "does-not-check is NOT an authority-ceiling failure — conflating the two misdiagnoses \
         every broken service as a profile problem:\n{text}"
    );
    assert!(text.contains("brokensvc"), "the broken service must be identifiable in the error:\n{text}");
}

// ----- 6. the control: multiple clean services within a real (non-trivial) ceiling --------------

/// THE CONTROL. Three services — pure, one real effect, and exactly-at-ceiling — all check cleanly
/// and all stay within a real, deliberately non-maximal profile (`{Write, Net}`, not "every effect
/// there is"). Without this, every refusal test above could be "explained" by a build that refuses
/// everything; this is the passing case that proves the gate isn't just shut.
#[test]
fn multiple_clean_services_within_a_real_ceiling_all_approve_together() {
    let root = scratch("control");
    let env = write_env(&root, "env.toml", "[authority]\neffects = [\"Write\", \"Net\"]\n");

    let a = root.join("alpha");
    write_service(&a, "alpha", &[]);
    let b = root.join("bravo");
    write_service(&b, "bravo", &["Write"]);
    let c = root.join("charlie");
    write_service(&c, "charlie", &["Write", "Net"]);

    let o = plan(&[("alpha", a), ("bravo", b), ("charlie", c)], &env, true);
    let text = combined(&o);
    assert_eq!(o.status.code(), Some(0), "three clean, in-envelope services must approve:\n{text}");

    let v: Value =
        serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("deploy plan --json must be valid JSON: {e}\n{text}"));
    assert_eq!(v["command"], "deploy");
    assert_eq!(v["subcommand"], "plan");
    assert_eq!(v["approved"], true);

    let services = v["services"].as_array().expect("services array");
    assert_eq!(services.len(), 3, "every service must be listed, none dropped: {v}");
    assert_eq!(service_effects(services, "alpha"), Vec::<String>::new(), "{v}");
    assert_eq!(service_effects(services, "bravo"), vec!["Write".to_string()], "{v}");
    assert_eq!(service_effects(services, "charlie"), vec!["Net".to_string(), "Write".to_string()], "{v}");
}

// ----- human rendering: lists services + effects, ends with an approved line --------------------

/// The contract's own words for success: "human-readable output lists each service + its effects
/// and ends with an 'approved' line". Checked directly — an operator reading a terminal, not
/// parsing JSON, still needs to see what was approved, and that it in fact WAS.
#[test]
fn human_output_lists_services_and_effects_and_ends_with_an_approved_line() {
    let root = scratch("human-render");
    let env = write_env(&root, "env.toml", "[authority]\neffects = [\"Write\", \"Net\"]\n");
    let a = root.join("frontdoor");
    write_service(&a, "frontdoor", &["Write"]);
    let b = root.join("backend");
    write_service(&b, "backend", &["Write", "Net"]);

    let o = plan(&[("frontdoor", a), ("backend", b)], &env, false);
    let text = combined(&o);
    assert_eq!(o.status.code(), Some(0), "{text}");
    assert!(text.contains("frontdoor") && text.contains("backend"), "every service must be listed:\n{text}");
    assert!(text.contains("Write") && text.contains("Net"), "each service's effects must be listed:\n{text}");

    let last = text.lines().map(str::trim).rfind(|l| !l.is_empty()).unwrap_or("");
    assert!(
        last.to_lowercase().contains("approved"),
        "the human report must END with an approved line, got last non-empty line {last:?}\nfull text:\n{text}"
    );
}

// ----- 7. --json and human text agree on both a refusal and a success ---------------------------

/// THE DRILL-001 LESSON (see `signing_cli.rs`), applied here: a caller gating on the exit code and
/// a caller parsing `--json` must reach the SAME verdict, for both a refusal and a success. A
/// build where the human line says one thing and the JSON envelope says another is a bypass
/// waiting for whichever caller trusted the wrong channel.
#[test]
fn json_and_human_text_agree_on_a_refusal_and_a_success() {
    let root = scratch("json-agree");
    let narrow = write_env(&root, "narrow.toml", "[authority]\neffects = [\"Write\"]\n");
    let wide = write_env(&root, "wide.toml", "[authority]\neffects = [\"Write\", \"Net\"]\n");
    let leaky = root.join("leaky");
    write_service(&leaky, "leaky", &["Net"]);

    // ----- refusal: a Net-only service against a Write-only ceiling -----
    let human = plan(&[("leaky", leaky.clone())], &narrow, false);
    let json = plan(&[("leaky", leaky.clone())], &narrow, true);
    assert_eq!(human.status.code(), json.status.code(), "human and --json must exit IDENTICALLY on refusal");
    assert_ne!(human.status.code(), Some(0), "the refusal case must not exit 0");
    assert!(combined(&human).contains("DL1909"), "{}", combined(&human));
    let jv: Value = serde_json::from_slice(&json.stdout)
        .unwrap_or_else(|e| panic!("refusal --json must still be valid JSON: {e}\n{}", combined(&json)));
    assert_ne!(jv["approved"], true, "the JSON verdict must not say approved either: {jv}");

    // ----- success: the SAME service, against a ceiling that actually covers it -----
    let human_ok = plan(&[("leaky", leaky.clone())], &wide, false);
    let json_ok = plan(&[("leaky", leaky)], &wide, true);
    assert_eq!(human_ok.status.code(), json_ok.status.code(), "human and --json must exit IDENTICALLY on success");
    assert_eq!(human_ok.status.code(), Some(0), "a service within its ceiling must approve:\n{}", combined(&human_ok));
    let jv_ok: Value = serde_json::from_slice(&json_ok.stdout).expect("success --json is valid JSON");
    assert_eq!(jv_ok["approved"], true, "{jv_ok}");
}

// ----- usage errors: zero services, missing --env -----------------------------------------------

/// Zero `--service` flags is a USAGE error (exit 2) — never a vacuously-approved empty plan. Also
/// guards against the CLI's unrecognized-subcommand fallback (`unknown command \`deploy\``, exit 2
/// today) accidentally satisfying this assertion for the wrong reason before `deploy` even exists.
#[test]
fn zero_services_is_a_usage_error_not_an_empty_approved_plan() {
    let root = scratch("usage-zero");
    let env = write_env(&root, "env.toml", "[authority]\neffects = [\"Write\"]\n");
    let o = delulu(&["deploy", "plan", "--env", &env.display().to_string()]);
    let text = combined(&o);
    assert_eq!(o.status.code(), Some(2), "no services named must be a usage error:\n{text}");
    assert!(
        !text.to_lowercase().contains("unknown command"),
        "this must be `deploy plan`'s OWN usage check, not the CLI's unrecognized-subcommand \
         fallback — which would spuriously satisfy the exit-2 assertion above before `deploy` is \
         even wired up:\n{text}"
    );
}

/// A missing `--env` is a usage error (exit 2) — there is no default profile a plan can fall back
/// to, because a default-open profile is exactly the "no ceiling" failure this file spends most of
/// its lines checking for. It must refuse to guess, not silently plan against an implicit ceiling.
#[test]
fn a_missing_env_flag_is_a_usage_error() {
    let root = scratch("usage-noenv");
    let svc = root.join("writersvc");
    write_service(&svc, "writersvc", &["Write"]);
    let o = delulu(&["deploy", "plan", "--service", &format!("writersvc={}", svc.display())]);
    let text = combined(&o);
    assert_eq!(o.status.code(), Some(2), "a missing --env must be a usage error:\n{text}");
    assert!(
        !text.to_lowercase().contains("unknown command"),
        "this must be `deploy plan`'s OWN usage check, not the CLI's unrecognized-subcommand \
         fallback:\n{text}"
    );
}

// ----- cross-check: the contract's own words, verified rather than trusted ----------------------

/// The contract's OWN words: a service's authority is "computed the same way `delulu authority
/// <package-dir>` computes it today". This does not trust that sentence — it CHECKS it, by running
/// both commands independently over the SAME directory and comparing effect lists. If a future
/// change makes deploy plan compute authority some other way (a different reachability walk, a
/// different manifest reading), this fails even if every other test in this file still passes.
#[test]
fn a_services_reported_effects_match_what_delulu_authority_reports_directly() {
    let root = scratch("cross-check");
    let svc = root.join("crosscheck");
    write_service(&svc, "crosscheck", &["Write", "Net"]);
    let env = write_env(&root, "env.toml", "[authority]\neffects = [\"Write\", \"Net\"]\n");

    let auth = delulu(&["authority", &svc.display().to_string(), "--json"]);
    assert!(auth.status.success(), "the fixture itself must check cleanly: {}", stderr(&auth));
    let av: Value = serde_json::from_slice(&auth.stdout).expect("authority --json is valid JSON");
    let mut authority_effects: Vec<String> = av["authority"]["effects"]
        .as_array()
        .expect("authority effects array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    authority_effects.sort();

    let o = plan(&[("crosscheck", svc)], &env, true);
    assert!(o.status.success(), "the plan itself must approve: {}", combined(&o));
    let pv: Value = serde_json::from_slice(&o.stdout).expect("deploy plan --json is valid JSON");
    let plan_effects = service_effects(pv["services"].as_array().expect("services array"), "crosscheck");

    assert_eq!(
        authority_effects, plan_effects,
        "deploy plan's per-service effects must match `delulu authority` independently — not just \
         happen to agree on this one fixture's expectations"
    );
}
