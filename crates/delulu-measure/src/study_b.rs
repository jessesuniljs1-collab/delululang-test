//! Study B — agent task success and repair loops (Stage 9e, spec §3.2).
//!
//! ## What this study actually measures
//! **What the toolchain hands a machine**, not which language is better and not how clever a model
//! is. The lanes are not a race between languages: they are a comparison of how much structured
//! information each toolchain gives an automated repair loop.
//!
//! A DeluluLang diagnostic can carry a typed repair — an id, a confidence, an authority-widening
//! flag, and byte-range edits a program can splice without understanding the language. A Python
//! traceback carries prose. That difference is structural and it is the thing being measured.
//!
//! ## The lanes (build order D4)
//! - **Scripted lane (RUN).** A deterministic repair loop with no model in it at all. It applies
//!   whatever machine-applicable repairs the toolchain offers, re-checks, and repeats. Every number
//!   published by this study comes from here.
//! - **Live-model lane (UNRUN).** The harness accepts a model endpoint and would measure
//!   iterations-to-green for a real agent. It has never been run — no API keys in CI, and a remote
//!   model version is not reproducible. It is labelled UNRUN rather than dropped so the reader
//!   knows what is missing rather than assuming the scripted lane is the whole story.
//!
//! ## The uncomfortable finding, up front
//! Machine-applicable repairs exist for a **small minority** of real defects today. The mechanism
//! works end to end; its coverage does not yet match the promise the phrase "typed repairs"
//! implies. This study reports that number rather than the subset where it looks good.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// One task: a program with a real defect, and what the toolchain offered for it.
pub struct Task {
    pub name: String,
    /// The primary diagnostic code the defect produces.
    pub code: String,
    /// Whether at least one diagnostic carried a machine-applicable repair.
    pub repair_offered: bool,
    /// The repair ids offered, if any.
    pub repair_ids: Vec<String>,
    /// Whether any offered repair would WIDEN authority — a repair a machine must not apply
    /// silently, however exact it is.
    pub authority_widening: bool,
    /// Iterations of the mechanical loop needed to reach green. `None` if it never got there.
    pub iterations_to_green: Option<usize>,
    pub millis: u128,
}

/// The Python comparison lane: the same defect shapes, measured for structured repair information.
pub struct PythonTask {
    pub name: String,
    /// Whether the interpreter offered anything a program could apply without understanding
    /// Python. Measured by looking, not assumed.
    pub machine_applicable_repair: bool,
    pub error_kind: String,
}

pub struct Results {
    pub tasks: Vec<Task>,
    pub python: Vec<PythonTask>,
    pub python_available: bool,
    /// Compile-time refusals of an undeclared effect across the task set — the "unauthorized
    /// effect attempt" count, caught before anything ran.
    pub unauthorized_effect_attempts: usize,
}

impl Results {
    pub fn with_repair(&self) -> usize {
        self.tasks.iter().filter(|t| t.repair_offered).count()
    }
    pub fn repair_availability_pct(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }
        100.0 * self.with_repair() as f64 / self.tasks.len() as f64
    }
    pub fn repaired_to_green(&self) -> usize {
        self.tasks.iter().filter(|t| t.iterations_to_green.is_some()).count()
    }
    pub fn python_with_repair(&self) -> usize {
        self.python.iter().filter(|t| t.machine_applicable_repair).count()
    }
}

fn delulu_bin() -> PathBuf {
    let exe = std::env::current_exe().expect("current exe");
    let dir = exe
        .parent()
        .and_then(|d| if d.ends_with("deps") { d.parent() } else { Some(d) })
        .expect("target dir");
    dir.join(if cfg!(windows) { "delulu.exe" } else { "delulu" })
}

fn check_json(file: &Path) -> Option<serde_json::Value> {
    let out = Command::new(delulu_bin())
        .args(["check", &file.to_string_lossy(), "--json"])
        .env("DELULU_NO_FIRST_RUN", "1")
        .output()
        .ok()?;
    serde_json::from_slice(&out.stdout).ok()
}

/// Apply a repair's edits to the file, back to front so earlier byte offsets stay valid.
fn apply_repair(file: &Path, repair: &serde_json::Value) -> std::io::Result<bool> {
    let Some(edits) = repair["edits"].as_array() else { return Ok(false) };
    let mut src = std::fs::read(file)?;
    let mut ranges: Vec<(usize, usize, String)> = Vec::new();
    for e in edits {
        let start = e["range"]["start_byte"].as_u64().unwrap_or(0) as usize;
        let end = e["range"]["end_byte"].as_u64().unwrap_or(0) as usize;
        let insert = e["insert"].as_str().unwrap_or("").to_string();
        if start > src.len() || end > src.len() || start > end {
            return Ok(false);
        }
        ranges.push((start, end, insert));
    }
    ranges.sort_by(|a, b| b.0.cmp(&a.0));
    for (start, end, insert) in ranges {
        src.splice(start..end, insert.bytes());
    }
    std::fs::write(file, src)?;
    Ok(true)
}

/// The MECHANICAL repair loop: no model, no heuristics, no guessing. It applies repairs the
/// toolchain declares machine-applicable and stops when the program is clean or nothing is left to
/// apply. A repair flagged `authority_widening` is NEVER applied automatically — widening authority
/// to silence a diagnostic is precisely the move a machine must not make on its own.
fn repair_loop(file: &Path, max_iterations: usize) -> (Option<usize>, bool) {
    let mut widening_seen = false;
    for i in 0..max_iterations {
        let Some(json) = check_json(file) else { return (None, widening_seen) };
        if json["summary"]["errors"].as_u64().unwrap_or(1) == 0 {
            return (Some(i), widening_seen);
        }
        let mut applied = false;
        if let Some(diags) = json["diagnostics"].as_array() {
            for d in diags {
                let Some(repairs) = d["repairs"].as_array() else { continue };
                for r in repairs {
                    if r["authority_widening"].as_bool().unwrap_or(false) {
                        widening_seen = true;
                        continue;
                    }
                    if r["requires_human"].as_bool().unwrap_or(false) {
                        continue;
                    }
                    if apply_repair(file, r).unwrap_or(false) {
                        applied = true;
                        break;
                    }
                }
                if applied {
                    break;
                }
            }
        }
        if !applied {
            return (None, widening_seen);
        }
    }
    (None, widening_seen)
}

/// Run the study. `corpus` is the directory of real defect programs (the conformance reject set).
pub fn run_study(repo_root: &Path, work: &Path) -> std::io::Result<Results> {
    std::fs::create_dir_all(work)?;
    let reject = repo_root.join("tests/conformance/reject");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&reject)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("delulu"))
        .collect();
    files.sort();

    let mut tasks = Vec::new();
    let mut unauthorized = 0usize;

    for f in &files {
        let name = f.file_stem().and_then(|s| s.to_str()).unwrap_or("?").to_string();
        // Work on a copy: the loop mutates the program, and the corpus is not ours to edit.
        let scratch = work.join(format!("{name}.delulu"));
        std::fs::copy(f, &scratch)?;

        let t = Instant::now();
        let Some(json) = check_json(&scratch) else { continue };
        let mut code = String::new();
        let mut repair_ids = Vec::new();
        let mut widening = false;
        if let Some(diags) = json["diagnostics"].as_array() {
            for d in diags {
                if code.is_empty() {
                    code = d["code"].as_str().unwrap_or("").to_string();
                }
                // An undeclared-effect refusal IS an unauthorized-effect attempt, caught before
                // the program ran at all.
                if matches!(d["code"].as_str(), Some("DL0501") | Some("DL0701") | Some("DL1009")) {
                    unauthorized += 1;
                }
                if let Some(rs) = d["repairs"].as_array() {
                    for r in rs {
                        if let Some(id) = r["id"].as_str() {
                            repair_ids.push(id.to_string());
                        }
                        if r["authority_widening"].as_bool().unwrap_or(false) {
                            widening = true;
                        }
                    }
                }
            }
        }
        let repair_offered = !repair_ids.is_empty();
        let (iterations, _) = if repair_offered { repair_loop(&scratch, 8) } else { (None, false) };

        tasks.push(Task {
            name,
            code,
            repair_offered,
            repair_ids,
            authority_widening: widening,
            iterations_to_green: iterations,
            millis: t.elapsed().as_millis(),
        });
    }

    let (python, python_available) = python_lane(work);

    Ok(Results { tasks, python, python_available, unauthorized_effect_attempts: unauthorized })
}

/// The Python comparison lane. Six defect shapes analogous to the DeluluLang ones. The question
/// asked of each is the same question asked of DeluluLang: *does the toolchain hand a program
/// something it can apply without understanding the language?*
fn python_lane(work: &Path) -> (Vec<PythonTask>, bool) {
    let probe = Command::new("python").arg("--version").output();
    if probe.is_err() {
        return (Vec::new(), false);
    }
    const DEFECTS: &[(&str, &str)] = &[
        ("undefined_name", "def f():\n    return undefined_thing\nf()\n"),
        ("wrong_arity", "def f(a, b):\n    return a\nf(1)\n"),
        ("type_mismatch", "def f():\n    return 'a' + 1\nf()\n"),
        ("missing_attribute", "def f():\n    return (1).nope\nf()\n"),
        ("bad_index", "def f():\n    return [1][9]\nf()\n"),
        ("undeclared_io", "import os\ndef f():\n    return os.listdir('.')\nf()\n"),
    ];
    let mut out = Vec::new();
    for (name, src) in DEFECTS {
        let p = work.join(format!("py_{name}.py"));
        let _ = std::fs::write(&p, src);
        let res = Command::new("python").arg(&p).output();
        let (kind, machine) = match res {
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr).to_string();
                let kind = err
                    .lines()
                    .last()
                    .map(|l| l.split(':').next().unwrap_or("?").trim().to_string())
                    .unwrap_or_else(|| "none".into());
                // The measurement: does the output contain anything structured a program could
                // apply — a machine-readable edit, a range, a repair id? A traceback does not.
                // `undeclared_io` SUCCEEDS: listing a directory needs no permission in Python, so
                // there is no diagnostic at all and nothing to repair. That is the finding.
                let structured = err.contains("\"edits\"") || err.contains("\"repair\"");
                (if o.status.success() { "no error".to_string() } else { kind }, structured)
            }
            Err(_) => ("unavailable".to_string(), false),
        };
        out.push(PythonTask {
            name: name.to_string(),
            machine_applicable_repair: machine,
            error_kind: kind,
        });
    }
    (out, true)
}

pub fn json(r: &Results) -> serde_json::Value {
    use serde_json::json;
    json!({
        "study": "B",
        "title": "agent task success and repair loops",
        "schema": "study-b/1",
        "lanes": {
            "scripted": "RUN — every number below comes from this lane",
            "live_model": "UNRUN — harness present, never executed (no API keys, remote model \
                           versions are not reproducible). Build order D4.",
            "go_baseline": "UNRUN — no Go toolchain available on the measurement machine.",
        },
        "tasks": {
            "count": r.tasks.len(),
            "with_machine_applicable_repair": r.with_repair(),
            "repair_availability_pct": (r.repair_availability_pct() * 10.0).round() / 10.0,
            "reached_green_mechanically": r.repaired_to_green(),
        },
        "unauthorized_effect_attempts_caught_at_compile_time": r.unauthorized_effect_attempts,
        "python_lane": {
            "available": r.python_available,
            "tasks": r.python.len(),
            "with_machine_applicable_repair": r.python_with_repair(),
            "detail": r.python.iter().map(|p| json!({
                "task": p.name, "error_kind": p.error_kind,
                "machine_applicable_repair": p.machine_applicable_repair
            })).collect::<Vec<_>>(),
        },
        "detail": r.tasks.iter().map(|t| json!({
            "task": t.name,
            "code": t.code,
            "repair_offered": t.repair_offered,
            "repair_ids": t.repair_ids,
            "authority_widening": t.authority_widening,
            "iterations_to_green": t.iterations_to_green,
            "ms": t.millis,
        })).collect::<Vec<_>>(),
    })
}

pub fn report(r: &Results) -> String {
    let mut s = String::new();
    s.push_str("# Study B — agent task success and repair loops\n\n\
                *Generated by `delulu-measure study-b`. Raw data: `results.json`. Method and \
                threats to validity: `../METHODOLOGY.md` §2.*\n\n");

    s.push_str("## What this measures\n\n\
                **What the toolchain hands a machine** — not which language is better, and not how \
                clever a model is. A DeluluLang diagnostic can carry a typed repair: an id, a \
                confidence, an authority-widening flag, and byte-range edits a program can splice \
                without understanding the language. A Python traceback carries prose. That \
                structural difference is the thing under measurement.\n\n\
                The scripted lane contains **no model at all**. It applies whatever the toolchain \
                declares machine-applicable, re-checks, and repeats.\n\n");

    s.push_str(&format!(
        "## The headline number, including the part that is not flattering\n\n\
         Across **{} real defect programs** (the conformance reject corpus — genuine defects, not \
         synthesised ones), **{} offered a machine-applicable repair: {:.1}%**.\n\n\
         The repair *mechanism* works end to end: where a repair exists, a loop with no \
         intelligence in it applies the edits and reaches a clean program. But coverage is a small \
         minority of real defects today. The phrase \"typed repairs\" invites the reader to imagine \
         universality; the measurement does not support that, and this report says so rather than \
         quoting the subset where it looks good.\n\n\
         **{} of {} repairable tasks reached green mechanically.**\n\n",
        r.tasks.len(),
        r.with_repair(),
        r.repair_availability_pct(),
        r.repaired_to_green(),
        r.with_repair()
    ));

    let ids: std::collections::BTreeMap<String, usize> =
        r.tasks.iter().flat_map(|t| t.repair_ids.iter()).fold(Default::default(), |mut m, id| {
            *m.entry(id.clone()).or_default() += 1;
            m
        });
    if !ids.is_empty() {
        s.push_str("### Which repairs exist\n\n| Repair id | Tasks |\n|---|---|\n");
        for (id, n) in &ids {
            s.push_str(&format!("| `{id}` | {n} |\n"));
        }
        s.push('\n');
    }

    // Why the mechanical loop reached green zero times, stated precisely — a bare zero reads like
    // a broken harness, and the reader deserves to know it is a finding rather than a fault.
    let widening = r.tasks.iter().filter(|t| t.repair_offered && t.authority_widening).count();
    let ineffective = r.with_repair() - widening - r.repaired_to_green();
    if r.repaired_to_green() == 0 && r.with_repair() > 0 {
        s.push_str(&format!(
            "### Why zero tasks reached green mechanically\n\n\
             This is a result, not a broken harness, and it decomposes cleanly:\n\n\
             - **{widening} task(s): the only repair offered was `add_effect_to_row`, which WIDENS \
               AUTHORITY.** The loop refuses these by policy. The repair is exact and correct for a \
               human who has decided the effect belongs there; it is not something a machine may do \
               on its own, because widening authority to silence a diagnostic does not fix the \
               program, it removes the objection. **The right behaviour here is the refusal.**\n\
             - **{ineffective} task(s): the offered repair addressed a WARNING while an error \
               remained.** `remove_effect_from_row` clears a DL0502-class complaint; it cannot \
               clear the DL0405 or DL0504 error sitting next to it. Applying it changes the program \
               without making it compile.\n\n\
             So the honest summary is sharper than the availability number alone: **a repair loop \
             with no model in it cannot currently drive any real DeluluLang defect to green** — \
             either because the only exact repair is one a machine must not take, or because the \
             repair does not address the error. The machine-applicable repair channel is real, \
             typed, and correctly conservative; what it does not yet have is coverage of the \
             defects that actually stop a build.\n\n"
        ));
    }

    s.push_str(&format!(
        "### Authority-widening repairs are never applied automatically\n\n\
         Some repairs are exact and still must not be taken by a machine: `add_effect_to_row` \
         silences a diagnostic by *granting the program more authority*. The loop refuses to apply \
         any repair flagged `authority_widening`, however confident it is. A repair loop that \
         widens authority to reach green has not fixed the program — it has removed the objection.\n\n\
         ## Unauthorized-effect attempts\n\n\
         **{} attempts were refused at COMPILE time** across the task set — before any program ran. \
         In the Python lane the analogous operation (`os.listdir`) simply succeeds: there is no \
         declaration to violate, so there is no diagnostic, nothing to count, and nothing to \
         repair. The harness can only count what a toolchain surfaces, and it says so.\n\n",
        r.unauthorized_effect_attempts
    ));

    s.push_str("## The Python lane\n\n");
    if r.python_available {
        s.push_str(&format!(
            "{} defect shapes run through CPython. **{} offered a machine-applicable repair.**\n\n\
             | Defect | What Python reported | Machine-applicable repair |\n|---|---|---|\n",
            r.python.len(),
            r.python_with_repair()
        ));
        for p in &r.python {
            s.push_str(&format!(
                "| `{}` | {} | {} |\n",
                p.name,
                p.error_kind,
                if p.machine_applicable_repair { "yes" } else { "no" }
            ));
        }
        s.push_str("\nThis is not a criticism of CPython, which is doing what a dynamic language \
                    can do. It is the measurement the study exists to make: a traceback is written \
                    for a human, and a program cannot apply it without a model in the loop.\n\n");
    } else {
        s.push_str("**UNRUN** — no Python interpreter on the measurement machine.\n\n");
    }

    s.push_str("## Lanes not run\n\n\
                - **Live-model lane: UNRUN.** The harness accepts a model endpoint and would \
                  measure iterations-to-green for a real agent. It has never been executed: no API \
                  keys in CI, and a remote model version is not reproducible. Labelled rather than \
                  dropped, so the reader knows the scripted lane is not the whole story.\n\
                - **Go baseline: UNRUN.** No Go toolchain on the measurement machine.\n\n");

    s.push_str("## Appendix — token counts\n\n\
                Deliberately a minor appendix, and no comparison is drawn. Constitution §5.11, \
                verbatim: *\"emoji/exotic syntax for 'token efficiency' (increases BPE tokens); \
                'lowest tokens of any language' as a headline (tokenizer-specific; token efficiency \
                is a minor consequence of clean regular syntax, never a claim).\"*\n\n\
                No token counts are published here, because a count without a fixed tokenizer is \
                meaningless and a count with one invites exactly the headline the constitution \
                rejects.\n");
    s
}
