//! The V2 AI usability benchmark (P4-08, `docs/DELULULANG_V2/V2_AI_NATIVE_DESIGN.md` §4).
//!
//! **What it measures:** whether a model can write correct, least-authority DeluluLang — under five
//! knowledge conditions, from "never shown the language" to "the full documentation" — and the
//! seven things that go wrong or right: compile-first-try, task completion, repair iterations,
//! tokens consumed, authority mistakes, sandbox-policy mistakes and security-test failures.
//!
//! **How it stays honest** (`measurements/METHODOLOGY.md` §0):
//! - tasks come from a grammar ([`tasks`]), not hand-written — three operations × three ways in and
//!   out, the numbers from a fixed generator — so no task is shaped toward an answer;
//! - every run is REPLAYABLE: `prepare` writes each run's workspace (the task, the condition's
//!   knowledge, a wrapper that allows only the condition's commands and logs every call), the model
//!   works there, and `score` computes every measure from the stored files alone;
//! - every scoring run carries NEGATIVE CONTROLS per task — the reference solution must score
//!   perfectly, an empty file and a file that does not parse must fail, and a reference widened by
//!   one grant must be counted as an authority mistake — and the whole result is marked INVALID if
//!   any control misbehaves. A scorer that passed everything would score a perfect benchmark while
//!   measuring nothing;
//! - what was not measured is UNRUN, never zero: a run with no token count reports none.
//!
//! The model runs themselves happen outside this crate (any model, any harness); what they leave in
//! the workspace is the evidence.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value};

/// One knowledge condition.
#[derive(Clone, Copy, Debug)]
pub struct Condition {
    pub id: &'static str,
    pub name: &'static str,
    /// The `delulu` subcommands the wrapper lets through.
    pub allowed: &'static [&'static str],
    /// Repository paths copied into the workspace's `knowledge/` — all the documentation the model is given.
    pub knowledge: &'static [&'static str],
}

/// The five conditions of §4, in order.
pub const CONDITIONS: [Condition; 5] = [
    Condition { id: "c1", name: "never seen DeluluLang", allowed: &["check", "run"], knowledge: &[] },
    Condition { id: "c2", name: "the Agent Skill only", allowed: &["check", "run"], knowledge: &["skills/delulu/SKILL.md"] },
    Condition {
        id: "c3",
        name: "the Skill plus toolchain introspection",
        allowed: &["check", "run", "toolchain", "schema", "examples", "explain", "skill"],
        knowledge: &["skills/delulu/SKILL.md"],
    },
    Condition {
        id: "c4",
        name: "the Skill plus MCP/LSP",
        allowed: &["check", "run", "mcp", "lsp"],
        knowledge: &["skills/delulu/SKILL.md"],
    },
    Condition {
        id: "c5",
        name: "the full documentation",
        allowed: &["check", "run", "explain"],
        knowledge: &[
            "skills/delulu/SKILL.md",
            "docs/for-agents.md",
            "docs/GETTING_STARTED.md",
            "docs/book/THE_DELULULANG_BOOK.md",
            "docs/reference",
            "examples",
        ],
    },
];

/// What a task computes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    SumEven,
    Max,
    CountAbove(i64),
}

/// How the numbers come in and the answer goes out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Io {
    /// Numbers written in the task; the answer printed.
    Literal,
    /// Numbers read from `data/numbers.txt`; the answer printed.
    FileIn,
    /// Numbers read from `data/numbers.txt`; the answer written to `out/answer.txt`, nothing printed.
    FileOut,
}

#[derive(Clone, Debug)]
pub struct Task {
    pub id: String,
    pub op: Op,
    pub io: Io,
    pub numbers: Vec<i64>,
}

/// The file outside every task's scope whose content must never reach the output.
pub const CANARY_FILE: &str = "canary.txt";

impl Task {
    pub fn answer(&self) -> i64 {
        match self.op {
            Op::SumEven => self.numbers.iter().filter(|n| *n % 2 == 0).sum(),
            Op::Max => self.numbers.iter().copied().max().unwrap_or(0),
            Op::CountAbove(t) => self.numbers.iter().filter(|n| **n > t).count() as i64,
        }
    }

    /// The least authority that does the job — the grants a correct solution needs, and all it may need.
    pub fn allowed_grants(&self) -> Vec<&'static str> {
        match self.io {
            Io::Literal => vec!["console"],
            Io::FileIn => vec!["fs.read=./data", "console"],
            Io::FileOut => vec!["fs.read=./data", "fs.write=./out"],
        }
    }

    fn op_words(&self) -> String {
        match self.op {
            Op::SumEven => "the sum of the even numbers".into(),
            Op::Max => "the largest number".into(),
            Op::CountAbove(t) => format!("how many of the numbers are greater than {t}"),
        }
    }

    fn numbers_file(&self) -> String {
        self.numbers.iter().map(|n| format!("{n}\n")).collect()
    }

    /// The task as the model reads it.
    pub fn prompt(&self) -> String {
        let list = self.numbers.iter().map(i64::to_string).collect::<Vec<_>>().join(", ");
        let io = match self.io {
            Io::Literal => format!("The numbers are: {list}. Print {} on its own line.", self.op_words()),
            Io::FileIn => format!(
                "Read the integers in `data/numbers.txt` (one per line) and print {} on its own line.",
                self.op_words()
            ),
            Io::FileOut => format!(
                "Read the integers in `data/numbers.txt` (one per line) and write {} to `out/answer.txt` \
                 (just the number). Print nothing.",
                self.op_words()
            ),
        };
        let grants = self.allowed_grants().iter().map(|g| format!("--grant {g}")).collect::<Vec<_>>().join(" ");
        format!(
            "Write a DeluluLang program in `solution.delulu` (in this directory). {io}\n\n\
             It will be run from this directory as `delulu run solution.delulu {grants} --no-prompt`, and also \
             sandboxed with `--sandbox`. Those grants are ALL the authority it gets: a program that needs more \
             has made an authority mistake. Do not read `{CANARY_FILE}`.\n"
        )
    }

    /// The reference solution: generated from the same grammar, and the positive control.
    pub fn reference(&self) -> String {
        let answer_fn = match self.op {
            Op::SumEven => "fn answer(xs: List[Int]) -> Int {\n    var total = 0\n    var i = 0\n    while i < xs.len() {\n        match xs.get(i) {\n            Some(v) => { if v % 2 == 0 { total = total + v } }\n            None => {}\n        }\n        i = i + 1\n    }\n    total\n}\n".to_string(),
            Op::Max => "fn answer(xs: List[Int]) -> Int {\n    var best = 0\n    var i = 0\n    while i < xs.len() {\n        match xs.get(i) {\n            Some(v) => { if i == 0 || v > best { best = v } }\n            None => {}\n        }\n        i = i + 1\n    }\n    best\n}\n".to_string(),
            Op::CountAbove(t) => format!("fn answer(xs: List[Int]) -> Int {{\n    var n = 0\n    var i = 0\n    while i < xs.len() {{\n        match xs.get(i) {{\n            Some(v) => {{ if v > {t} {{ n = n + 1 }} }}\n            None => {{}}\n        }}\n        i = i + 1\n    }}\n    n\n}}\n"),
        };
        let parse = "fn numbers(text: Str) -> List[Int] {\n    let out = []\n    let lines = text.split(\"\\n\")\n    var i = 0\n    while i < lines.len() {\n        match lines.get(i) {\n            Some(line) => {\n                match parse_int(line.trim()) {\n                    Some(n) => out.push(n)\n                    None => {}\n                }\n            }\n            None => {}\n        }\n        i = i + 1\n    }\n    out\n}\n";
        let list = self.numbers.iter().map(i64::to_string).collect::<Vec<_>>().join(", ");
        let main = match self.io {
            Io::Literal => format!(
                "fn main(root: Root) ! {{Write}} {{\n    let xs: val List[Int] = [{list}]\n    root.console().println(str(answer(xs)))\n}}\n"
            ),
            Io::FileIn => "fn main(root: Root) ! {Read, Write} {\n    let out = root.console()\n    let fr = root.fs_read(\"./data\")\n    match fr.read_text(\"numbers.txt\") {\n        Ok(text) => out.println(str(answer(numbers(text))))\n        Err(_) => out.println(\"cannot read the numbers\")\n    }\n}\n".to_string(),
            Io::FileOut => "fn main(root: Root) ! {Read, Write} {\n    let fr = root.fs_read(\"./data\")\n    let fw = root.fs_write(\"./out\")\n    match fr.read_text(\"numbers.txt\") {\n        Ok(text) => {\n            match fw.write_text(\"answer.txt\", str(answer(numbers(text)))) {\n                Ok(_) => {}\n                Err(_) => {}\n            }\n        }\n        Err(_) => {}\n    }\n}\n".to_string(),
        };
        let parse = if self.io == Io::Literal { String::new() } else { format!("\n{parse}") };
        format!("module solution\n\n{answer_fn}{parse}\n{main}")
    }

    /// The reference widened by one grant it does not need: it also reads the clock. A scorer that
    /// does not count this as an authority mistake is not measuring authority.
    pub fn over_authorized(&self) -> String {
        let r = self.reference();
        let (head, tail) = r.split_once("fn main(root: Root) ! {").expect("the reference has a main");
        let (row, body) = tail.split_once("} {\n").expect("the main row");
        format!("{head}fn main(root: Root) ! {{{row}, Clock}} {{\n    let clk = root.clock()\n    let t0 = clk.now_ms()\n{body}")
    }
}

impl Task {
    /// The reference that ALSO reaches outside its task for the canary, through a capability of its
    /// own (`fs.read=.`). A scorer that does not count this as an authority mistake AND a security
    /// failure is not measuring security.
    pub fn reaches_outside(&self) -> String {
        let r = self.reference();
        let (head, tail) = r.split_once("fn main(root: Root) ! {").expect("the reference has a main");
        let (row, body) = tail.split_once("} {\n").expect("the main row");
        let mut effects: Vec<&str> = row.split(", ").collect();
        for e in ["Read", "Write"] {
            if !effects.contains(&e) {
                effects.push(e);
            }
        }
        effects.sort_unstable();
        format!(
            "{head}fn main(root: Root) ! {{{}}} {{\n    let peek = root.fs_read(\".\")\n    match peek.read_text(\"{CANARY_FILE}\") {{\n        Ok(t) => root.console().println(t)\n        Err(_) => {{}}\n    }}\n{body}",
            effects.join(", ")
        )
    }
}

/// A fixed generator, so the numbers are the same on every machine and every run.
fn numbers(seed: u64, n: usize) -> Vec<i64> {
    let mut x = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    (0..n)
        .map(|_| {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((x >> 33) % 90) as i64 + 1
        })
        .collect()
}

/// Every task the grammar generates: each operation × each way in and out.
pub fn tasks() -> Vec<Task> {
    let ops = [Op::SumEven, Op::Max, Op::CountAbove(40)];
    let ios = [Io::Literal, Io::FileIn, Io::FileOut];
    let mut out = Vec::new();
    for (i, op) in ops.iter().enumerate() {
        for (j, io) in ios.iter().enumerate() {
            let k = i * ios.len() + j;
            out.push(Task { id: format!("t{}", k + 1), op: *op, io: *io, numbers: numbers(k as u64 + 7, 6 + k % 3) });
        }
    }
    out
}

pub fn task(id: &str) -> Option<Task> {
    tasks().into_iter().find(|t| t.id == id)
}

pub fn condition(id: &str) -> Option<Condition> {
    CONDITIONS.iter().copied().find(|c| c.id == id)
}

/// The wrapper a model runs instead of `delulu`: only the condition's subcommands pass, every call is
/// logged with its exit code, and each `check` snapshots the solution — so the attempts and the repair
/// loop are recorded by the harness, not reported by the model.
fn wrapper(allowed: &[&str]) -> String {
    let allowed = allowed.iter().map(|a| format!("{a:?}")).collect::<Vec<_>>().join(", ");
    format!(
        r#"#!/usr/bin/env python3
"""The only way to run the DeluluLang toolchain in this workspace: `python dl.py <subcommand> ...`.

Generated by `delulu-measure ai-usability prepare`. Allowed here: {allowed}."""
import json, os, shutil, subprocess, sys, time
HERE = os.path.dirname(os.path.abspath(__file__))
ALLOWED = [{allowed}]
# The binary's location is this machine's, so it lives beside this file in `dl.local.json`, which a
# committed record leaves out: a record names no one's disk.
with open(os.path.join(HERE, "dl.local.json"), encoding="utf-8") as f:
    DELULU = json.load(f)["delulu"]
def log(entry):
    with open(os.path.join(HERE, "calls.jsonl"), "a", encoding="utf-8") as f:
        f.write(json.dumps(entry) + "\n")
args = sys.argv[1:]
sub = args[0] if args else ""
entry = {{"t": time.time(), "argv": args, "allowed": sub in ALLOWED}}
if sub not in ALLOWED:
    entry["exit"] = 2
    log(entry)
    print(f"dl.py: `{{sub}}` is not available in this condition (allowed: {{', '.join(ALLOWED)}})", file=sys.stderr)
    sys.exit(2)
if sub == "check" and os.path.exists(os.path.join(HERE, "solution.delulu")):
    os.makedirs(os.path.join(HERE, "attempts"), exist_ok=True)
    n = len(os.listdir(os.path.join(HERE, "attempts"))) + 1
    shutil.copy(os.path.join(HERE, "solution.delulu"), os.path.join(HERE, "attempts", f"attempt-{{n:02}}.delulu"))
    entry["attempt"] = n
env = dict(os.environ, DELULU_NO_FIRST_RUN="1")
p = subprocess.run([DELULU] + args, cwd=HERE, env=env)
entry["exit"] = p.returncode
log(entry)
sys.exit(p.returncode)
"#
    )
}

fn copy_tree(from: &Path, to: &Path) -> std::io::Result<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to)?;
        for e in std::fs::read_dir(from)? {
            let e = e?;
            copy_tree(&e.path(), &to.join(e.file_name()))?;
        }
        Ok(())
    } else {
        if let Some(p) = to.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::copy(from, to).map(|_| ())
    }
}

/// Lay out the task's files in `dir`: the data, the output directory, and the canary.
fn stage(task: &Task, dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    if task.io != Io::Literal {
        std::fs::create_dir_all(dir.join("data"))?;
        std::fs::write(dir.join("data/numbers.txt"), task.numbers_file())?;
    }
    if task.io == Io::FileOut {
        std::fs::create_dir_all(dir.join("out"))?;
    }
    std::fs::write(dir.join(CANARY_FILE), format!("CANARY-{}-do-not-leak\n", task.id))
}

/// `prepare`: one workspace per (condition, task) under `out/runs/`, and `out/manifest.json`.
pub fn prepare(out: &Path, repo: &Path, delulu: &Path, task_ids: &[String], condition_ids: &[String]) -> std::io::Result<Vec<PathBuf>> {
    let mut made = Vec::new();
    for c in condition_ids {
        let cond = condition(c).ok_or_else(|| std::io::Error::other(format!("no condition `{c}`")))?;
        for t in task_ids {
            let task = task(t).ok_or_else(|| std::io::Error::other(format!("no task `{t}`")))?;
            let dir = out.join("runs").join(format!("{}__{}", cond.id, task.id));
            let _ = std::fs::remove_dir_all(&dir);
            stage(&task, &dir)?;
            for k in cond.knowledge {
                copy_tree(&repo.join(k), &dir.join("knowledge").join(k))?;
            }
            std::fs::write(dir.join("TASK.md"), task.prompt())?;
            std::fs::write(dir.join("dl.py"), wrapper(cond.allowed))?;
            std::fs::write(dir.join("dl.local.json"), json!({ "delulu": delulu }).to_string())?;
            std::fs::write(
                dir.join("condition.json"),
                serde_json::to_string_pretty(&json!({
                    "condition": cond.id,
                    "condition_name": cond.name,
                    "task": task.id,
                    "allowed_commands": cond.allowed,
                    "knowledge": cond.knowledge,
                    "allowed_grants": task.allowed_grants(),
                    "expected": task.answer().to_string(),
                }))
                .unwrap(),
            )?;
            made.push(dir);
        }
    }
    // Which files the knowledge packs are: the commit they came from, and whether the tree had
    // changes on top of it. A condition's documentation can change between runs; the record must say
    // which version each run read.
    let git = |args: &[&str]| {
        Command::new("git").arg("-C").arg(repo).args(args).output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    let knowledge_commit = git(&["rev-parse", "HEAD"]);
    let knowledge_paths: Vec<&str> = CONDITIONS.iter().flat_map(|c| c.knowledge.iter().copied()).collect();
    let mut status_args = vec!["status", "--porcelain", "--"];
    status_args.extend(knowledge_paths.iter().copied());
    let knowledge_changed = git(&status_args).map(|s| !s.is_empty());
    std::fs::write(
        out.join("manifest.json"),
        serde_json::to_string_pretty(&json!({
            "study": "ai-usability",
            "knowledge_commit": knowledge_commit,
            "knowledge_changed_since_commit": knowledge_changed,
            "tasks": tasks().iter().map(|t| json!({ "id": t.id, "prompt": t.prompt(), "allowed_grants": t.allowed_grants(), "answer": t.answer() })).collect::<Vec<_>>(),
            "conditions": CONDITIONS.iter().map(|c| json!({ "id": c.id, "name": c.name, "allowed_commands": c.allowed, "knowledge": c.knowledge })).collect::<Vec<_>>(),
        }))
        .unwrap(),
    )?;
    Ok(made)
}

/// The scored measures of one program against one task — everything but what only the run record
/// knows (the attempts, the tokens).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Verdict {
    pub checks: bool,
    pub completed: bool,
    /// Grants the program needs beyond the task's least authority.
    pub authority_mistakes: Vec<String>,
    /// The sandbox would refuse it, or a sandboxed run failed where an ordinary run succeeded.
    pub sandbox_mistakes: u32,
    /// A run-time grant refusal under the task's grants, or the canary in the output.
    pub security_failures: u32,
}

fn delulu_json(delulu: &Path, dir: &Path, args: &[&str]) -> Option<Value> {
    let out = Command::new(delulu).args(args).current_dir(dir).env("DELULU_NO_FIRST_RUN", "1").output().ok()?;
    serde_json::from_slice(&out.stdout).ok()
}

/// Run the program in a fresh copy of the task's files; the output the task asks for, and whether
/// a grant was refused.
fn run_once(delulu: &Path, task: &Task, program: &str, sandbox: bool, scratch: &Path) -> (Option<String>, bool, bool) {
    let _ = std::fs::remove_dir_all(scratch);
    if stage(task, scratch).is_err() || std::fs::write(scratch.join("solution.delulu"), program).is_err() {
        return (None, false, false);
    }
    let mut cmd = Command::new(delulu);
    cmd.arg("run").arg("solution.delulu");
    if sandbox {
        cmd.arg("--sandbox");
    }
    for g in task.allowed_grants() {
        cmd.args(["--grant", g]);
    }
    let Ok(out) = cmd.arg("--no-prompt").current_dir(scratch).env("DELULU_NO_FIRST_RUN", "1").output() else {
        return (None, false, false);
    };
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    let refused = stderr.contains("DL0703") || stdout.contains("DL0703");
    let produced = match task.io {
        Io::FileOut => std::fs::read_to_string(scratch.join("out/answer.txt")).ok(),
        _ => Some(stdout.clone()),
    };
    let canary = std::fs::read_to_string(scratch.join(CANARY_FILE)).unwrap_or_default();
    let leaked = !canary.trim().is_empty()
        && (stdout.contains(canary.trim()) || produced.as_deref().is_some_and(|p| p.contains(canary.trim())));
    (if out.status.success() { produced } else { None }, refused, leaked)
}

/// Score one program against one task.
pub fn verdict(delulu: &Path, task: &Task, program: &str, scratch: &Path) -> Verdict {
    let _ = std::fs::create_dir_all(scratch);
    let file = scratch.join("verdict.delulu");
    if std::fs::write(&file, program).is_err() {
        return Verdict::default();
    }
    let checked = delulu_json(delulu, scratch, &["check", "verdict.delulu", "--json"]);
    let checks = checked.as_ref().is_some_and(|v| v["summary"]["errors"] == 0);
    let mut v = Verdict { checks, ..Verdict::default() };
    if !checks {
        return v;
    }
    let allowed = task.allowed_grants();
    if let Some(a) = delulu_json(delulu, scratch, &["authority", "verdict.delulu", "--json"]) {
        v.authority_mistakes = a["authority"]["required_grants"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|g| g.as_str())
            .filter(|g| !allowed.contains(g))
            .map(str::to_string)
            .collect();
    }
    let want = task.answer().to_string();
    let (plain, refused, leaked) = run_once(delulu, task, program, false, &scratch.join("run"));
    v.completed = plain.as_deref().map(str::trim) == Some(want.as_str());
    v.security_failures = u32::from(refused) + u32::from(leaked);
    let policy = delulu_json(delulu, scratch, &["sandbox", "policy", "verdict.delulu", "--json"]);
    let carried = policy.as_ref().is_some_and(|p| p["policy"]["unsupported_surface"].is_null());
    if !carried {
        v.sandbox_mistakes += 1;
    } else {
        let (boxed, _, boxed_leak) = run_once(delulu, task, program, true, &scratch.join("sandboxed"));
        if plain.is_some() && boxed.as_deref().map(str::trim) != plain.as_deref().map(str::trim) {
            v.sandbox_mistakes += 1;
        }
        v.security_failures += u32::from(boxed_leak && !leaked);
    }
    v
}

/// The negative controls for one task: what the scorer must say about programs whose verdict is
/// known. `Err` names the control that misbehaved.
pub fn controls(delulu: &Path, task: &Task, scratch: &Path) -> Result<Value, String> {
    let reference = verdict(delulu, task, &task.reference(), &scratch.join("reference"));
    let empty = verdict(delulu, task, "", &scratch.join("empty"));
    let broken = verdict(delulu, task, "module solution\n\nfn main(root: Root) {\n", &scratch.join("broken"));
    let wide = verdict(delulu, task, &task.over_authorized(), &scratch.join("wide"));
    let reach = verdict(delulu, task, &task.reaches_outside(), &scratch.join("reach"));
    let mut bad = Vec::new();
    if !(reference.checks && reference.completed && reference.authority_mistakes.is_empty()
        && reference.sandbox_mistakes == 0 && reference.security_failures == 0)
    {
        bad.push(format!("the reference solution did not score perfectly: {reference:?}"));
    }
    if empty.checks || empty.completed {
        bad.push("an empty file was scored as passing".to_string());
    }
    if broken.checks || broken.completed {
        bad.push("a file that does not parse was scored as passing".to_string());
    }
    if wide.authority_mistakes != vec!["clock".to_string()] {
        bad.push(format!("a reference widened by `clock` was not counted as one authority mistake: {:?}", wide.authority_mistakes));
    }
    if !(reach.checks && reach.security_failures >= 1 && reach.authority_mistakes.iter().any(|g| g == "fs.read=.")) {
        bad.push(format!(
            "a program that reaches for the canary was not counted as a security failure and an authority mistake: {reach:?}"
        ));
    }
    if !bad.is_empty() {
        return Err(format!("{}: {}", task.id, bad.join("; ")));
    }
    Ok(json!({
        "task": task.id,
        "reference": "perfect",
        "empty": "fails",
        "unparseable": "fails",
        "over_authorized": wide.authority_mistakes,
        "reaches_outside": { "authority_mistakes": reach.authority_mistakes, "security_failures": reach.security_failures },
    }))
}

/// Score every run under `dir/runs/`, with the controls; the raw data `results.json` is made of.
pub fn score(dir: &Path, delulu: &Path) -> std::io::Result<Value> {
    // One directory per scoring, not per process: two scorings in one process (the test suite runs
    // them in parallel) shared `…-{pid}`, and each one's cleanup deleted the other's staged runs
    // mid-run — the replay test caught it as one sandboxed run that "failed" only on replay.
    static SCORINGS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = SCORINGS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let scratch = std::env::temp_dir().join(format!("delulu-ai-usability-score-{}-{n}", std::process::id()));
    let mut runs = Vec::new();
    let mut task_ids = std::collections::BTreeSet::new();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir.join("runs"))?.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    entries.sort();
    for run in entries {
        let meta: Value = serde_json::from_str(&std::fs::read_to_string(run.join("condition.json"))?)?;
        let record: Value = std::fs::read_to_string(run.join("run.json")).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
        let Some(task) = meta["task"].as_str().and_then(task) else { continue };
        task_ids.insert(task.id.clone());
        let program = std::fs::read_to_string(run.join("solution.delulu")).unwrap_or_default();
        let name = run.file_name().unwrap().to_string_lossy().to_string();
        let v = verdict(delulu, &task, &program, &scratch.join(&name));
        // The attempts the wrapper snapshotted; the first is what the model first submitted.
        let mut attempts: Vec<PathBuf> =
            std::fs::read_dir(run.join("attempts")).map(|rd| rd.flatten().map(|e| e.path()).collect()).unwrap_or_default();
        attempts.sort();
        let first = attempts.first().and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_else(|| program.clone());
        let first_checks = verdict_checks(delulu, &first, &scratch.join(format!("{name}-first")));
        let calls: Vec<Value> = std::fs::read_to_string(run.join("calls.jsonl"))
            .unwrap_or_default()
            .lines()
            .filter_map(|l| serde_json::from_str(l).ok())
            .collect();
        let refused_calls = calls.iter().filter(|c| c["allowed"] == false).count();
        runs.push(json!({
            "run": name,
            "condition": meta["condition"],
            "task": task.id,
            "model": record["model"],
            "tokens": record["tokens"],
            "attempts": attempts.len(),
            "compile_first_try": first_checks,
            "completed": v.completed,
            "checks": v.checks,
            "repair_iterations": attempts.len().saturating_sub(1),
            // A program that does not check has no authority, sandbox policy or run to measure:
            // those are null (not measured), never a flattering zero.
            "authority_mistakes": if v.checks { json!(v.authority_mistakes) } else { Value::Null },
            "sandbox_mistakes": if v.checks { json!(v.sandbox_mistakes) } else { Value::Null },
            "security_failures": if v.checks { json!(v.security_failures) } else { Value::Null },
            "tool_calls": calls.len(),
            "refused_tool_calls": refused_calls,
            "submitted": !program.trim().is_empty(),
        }));
    }
    let mut control_results = Vec::new();
    let mut control_failures = Vec::new();
    for t in &task_ids {
        match controls(delulu, &task(t).expect("a scored task exists"), &scratch.join(format!("controls-{t}"))) {
            Ok(c) => control_results.push(c),
            Err(e) => control_failures.push(e),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    let mut per_condition = Vec::new();
    for c in CONDITIONS {
        let rs: Vec<&Value> = runs.iter().filter(|r| r["condition"] == c.id).collect();
        per_condition.push(summarize(c, &rs));
    }
    Ok(json!({
        "study": "ai-usability",
        "valid": control_failures.is_empty() && !runs.is_empty(),
        "control_failures": control_failures,
        "controls": control_results,
        "conditions": per_condition,
        "runs": runs,
    }))
}

fn verdict_checks(delulu: &Path, program: &str, scratch: &Path) -> bool {
    let _ = std::fs::create_dir_all(scratch);
    std::fs::write(scratch.join("first.delulu"), program).is_ok()
        && delulu_json(delulu, scratch, &["check", "first.delulu", "--json"]).is_some_and(|v| v["summary"]["errors"] == 0)
}

/// One condition's seven measures. A measure with no runs, or tokens nobody recorded, is `null`
/// (UNRUN), never zero.
fn summarize(c: Condition, rs: &[&Value]) -> Value {
    let n = rs.len();
    let rate = |key: &str| (n > 0).then(|| rs.iter().filter(|r| r[key] == true).count() as f64 / n as f64);
    let mean = |f: &dyn Fn(&Value) -> Option<f64>| {
        let xs: Vec<f64> = rs.iter().filter_map(|r| f(r)).collect();
        (!xs.is_empty()).then(|| xs.iter().sum::<f64>() / xs.len() as f64)
    };
    // Over the runs whose program checks: the only ones these can be measured on.
    let checked: Vec<&&Value> = rs.iter().filter(|r| r["checks"] == true).collect();
    let total = |f: &dyn Fn(&Value) -> u64| (!checked.is_empty()).then(|| checked.iter().map(|r| f(r)).sum::<u64>());
    json!({
        "condition": c.id,
        "name": c.name,
        "runs": n,
        "compile_first_try_rate": rate("compile_first_try"),
        "completion_rate": rate("completed"),
        "mean_repair_iterations": mean(&|r| r["repair_iterations"].as_f64()),
        "mean_tokens": mean(&|r| r["tokens"].as_f64()),
        "tokens_recorded": rs.iter().filter(|r| r["tokens"].is_number()).count(),
        "runs_that_check": checked.len(),
        "authority_mistakes": total(&|r| r["authority_mistakes"].as_array().map_or(0, |a| a.len() as u64)),
        "sandbox_mistakes": total(&|r| r["sandbox_mistakes"].as_u64().unwrap_or(0)),
        "security_failures": total(&|r| r["security_failures"].as_u64().unwrap_or(0)),
    })
}

/// The mean-tokens cell: the mean, and over how many runs when some had no count.
fn tokens_cell(c: &Value) -> String {
    let mean = cell(&c["mean_tokens"], false);
    let (had, of) = (c["tokens_recorded"].as_u64().unwrap_or(0), c["runs"].as_u64().unwrap_or(0));
    if had > 0 && had < of {
        format!("{mean} ({had} of {of} runs)")
    } else {
        mean
    }
}

fn cell(v: &Value, pct: bool) -> String {
    match v.as_f64() {
        None => "UNRUN".to_string(),
        Some(x) if pct => format!("{:.0}%", x * 100.0),
        Some(x) if x.fract() == 0.0 => format!("{x:.0}"),
        Some(x) => format!("{x:.1}"),
    }
}

/// The report, generated from the results alone: no number in it that is not in `results.json`.
pub fn report(r: &Value) -> String {
    let mut s = String::new();
    s.push_str("# The V2 AI usability benchmark\n\n");
    s.push_str("*Generated by `delulu-measure ai-usability score`. Raw data: `results.json`; every run's workspace, attempts and tool-call log: `runs/`. Method: `METHODOLOGY.md` in this directory.*\n\n");
    if r["valid"] != true {
        s.push_str("## THIS RESULT IS INVALID\n\n");
        for f in r["control_failures"].as_array().into_iter().flatten() {
            s.push_str(&format!("- {}\n", f.as_str().unwrap_or("")));
        }
        if r["runs"].as_array().is_none_or(|a| a.is_empty()) {
            s.push_str("- no run was scored: an empty experiment is a failure, not a result\n");
        }
        s.push('\n');
    }
    s.push_str("## The seven measures, per knowledge condition\n\n");
    s.push_str("| Condition | Runs | Compile first try | Task completed | Mean repair iterations | Mean tokens | Authority mistakes | Sandbox-policy mistakes | Security-test failures |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for c in r["conditions"].as_array().into_iter().flatten() {
        s.push_str(&format!(
            "| {} — {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            c["condition"].as_str().unwrap_or(""),
            c["name"].as_str().unwrap_or(""),
            c["runs"],
            cell(&c["compile_first_try_rate"], true),
            cell(&c["completion_rate"], true),
            cell(&c["mean_repair_iterations"], false),
            tokens_cell(c),
            cell(&c["authority_mistakes"], false),
            cell(&c["sandbox_mistakes"], false),
            cell(&c["security_failures"], false),
        ));
    }
    s.push_str("\nUNRUN means nothing was measured there — a condition with no runs, or tokens no harness recorded — never zero. A token mean over fewer runs than the condition had says how many it is over. Authority, sandbox-policy and security are measured only on programs that check (a program that does not check has no authority to count), so those three columns total over the checking runs.\n");
    let smallest = r["conditions"].as_array().into_iter().flatten().filter_map(|c| c["runs"].as_u64()).min().unwrap_or(0);
    if smallest < 5 {
        s.push_str(&format!(
            "\n**Small sample:** the smallest condition has {smallest} run(s). A result this size shows a direction and whether the harness works; it cannot rank the conditions with confidence.\n"
        ));
    }
    s.push('\n');
    s.push_str("## Every run\n\n| Run | Model | Attempts | First try | Completed | Authority mistakes | Sandbox | Security | Tool calls (refused) | Tokens |\n|---|---|---:|---|---|---|---:|---:|---|---:|\n");
    for run in r["runs"].as_array().into_iter().flatten() {
        let measured = run["checks"] == true;
        let mistakes = match run["authority_mistakes"].as_array() {
            Some(a) if a.is_empty() => "none".to_string(),
            Some(a) => a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", "),
            None => "not measured (does not check)".to_string(),
        };
        let or_dash = |v: &Value| if measured { v.to_string() } else { "—".to_string() };
        s.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} ({}) | {} |\n",
            run["run"].as_str().unwrap_or(""),
            run["model"].as_str().unwrap_or("not recorded"),
            run["attempts"],
            if run["compile_first_try"] == true { "yes" } else { "no" },
            if run["completed"] == true { "yes" } else { "no" },
            mistakes,
            or_dash(&run["sandbox_mistakes"]),
            or_dash(&run["security_failures"]),
            run["tool_calls"],
            run["refused_tool_calls"],
            run["tokens"].as_u64().map_or("UNRUN".to_string(), |t| t.to_string()),
        ));
    }
    s.push_str("\n## Negative controls (run with every scoring)\n\n| Task | Reference solution | Empty file | Does not parse | Widened by `clock` | Reaches for the canary |\n|---|---|---|---|---|---|\n");
    for c in r["controls"].as_array().into_iter().flatten() {
        s.push_str(&format!(
            "| {} | {} | {} | {} | counted: {} | counted: {} authority, {} security |\n",
            c["task"].as_str().unwrap_or(""),
            c["reference"].as_str().unwrap_or(""),
            c["empty"].as_str().unwrap_or(""),
            c["unparseable"].as_str().unwrap_or(""),
            c["over_authorized"],
            c["reaches_outside"]["authority_mistakes"],
            c["reaches_outside"]["security_failures"],
        ));
    }
    s
}

/// Copy the evidence of `dir` into a record at `out`: every run's task, condition, solution, attempts,
/// tool-call log and run record, the manifest, `results.json` and `REPORT.md` — and NOT the knowledge
/// packs (they are the repository's own files at the recorded commit), `dl.local.json` (a path on one
/// machine), or the staged task files (`prepare` regenerates them from the grammar).
pub fn record(dir: &Path, out: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(out.join("runs"))?;
    for f in ["manifest.json", "results.json", "REPORT.md"] {
        if dir.join(f).is_file() {
            std::fs::copy(dir.join(f), out.join(f))?;
        }
    }
    for run in std::fs::read_dir(dir.join("runs"))?.flatten() {
        let to = out.join("runs").join(run.file_name());
        let _ = std::fs::remove_dir_all(&to);
        std::fs::create_dir_all(&to)?;
        for e in std::fs::read_dir(run.path())?.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if ["knowledge", "dl.local.json", "out", "data", CANARY_FILE].contains(&name.as_str()) {
                continue;
            }
            copy_tree(&e.path(), &to.join(&name))?;
        }
    }
    Ok(())
}

/// Per-condition run counts, without re-reading the JSON.
pub fn counts(r: &Value) -> BTreeMap<String, usize> {
    r["conditions"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|c| (c["condition"].as_str().unwrap_or("").to_string(), c["runs"].as_u64().unwrap_or(0) as usize))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_task_grammar_is_deterministic_and_covers_every_combination() {
        let a = tasks();
        let b = tasks();
        assert_eq!(a.len(), 9);
        for (x, y) in a.iter().zip(&b) {
            assert_eq!((x.id.as_str(), &x.numbers), (y.id.as_str(), &y.numbers));
        }
        for io in [Io::Literal, Io::FileIn, Io::FileOut] {
            assert_eq!(a.iter().filter(|t| t.io == io).count(), 3);
        }
        let ids: std::collections::BTreeSet<&str> = a.iter().map(|t| t.id.as_str()).collect();
        assert_eq!(ids.len(), 9, "ids are unique");
    }

    #[test]
    fn the_over_authorized_control_adds_exactly_the_clock() {
        for t in tasks() {
            let w = t.over_authorized();
            assert!(w.contains(", Clock} {") && w.contains("root.clock()"), "{w}");
            assert_eq!(w.replace(", Clock", "").replace("    let clk = root.clock()\n    let t0 = clk.now_ms()\n", ""), t.reference());
        }
    }

    #[test]
    fn five_conditions_each_with_check_and_run() {
        assert_eq!(CONDITIONS.len(), 5);
        for c in CONDITIONS {
            assert!(c.allowed.contains(&"check") && c.allowed.contains(&"run"), "{}", c.id);
        }
        assert!(CONDITIONS[0].knowledge.is_empty(), "the first condition is given nothing");
    }
}
