#!/usr/bin/env python3
"""FABLE audit research runner — session 1 fixtures on a GitHub runner.

Lives only on the throwaway branch `laptop/fable-research-runner` (never merged). It runs the audit's
own benign fixtures against the `delulu` binary built from this checkout and prints one JSON object of
OBSERVATIONS. It asserts nothing about the product: a surprising result is data, not a failed job.
Only a broken harness (no binary, an exception) fails the job.
"""
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = Path(os.environ.get("FABLE_REPO") or HERE.parent.parent)
EXE = Path(os.environ.get("FABLE_EXE") or REPO / "target" / "debug" / ("delulu.exe" if os.name == "nt" else "delulu"))
sys.path.insert(0, str(HERE))
import oracle  # noqa: E402

WORK = Path(tempfile.mkdtemp(prefix="fable-"))
os.environ["DELULU_HOME"] = str(WORK / "state" / "home")
os.environ["DELULU_STATE_DIR"] = str(WORK / "state" / "state")
os.environ["DELULU_NO_FIRST_RUN"] = "1"
(WORK / "state" / "home").mkdir(parents=True)
(WORK / "state" / "state").mkdir(parents=True)

GRANTS = ["--grant", "console", "--grant", "fs.read=./input", "--grant", "fs.write=./output", "--no-prompt"]


def clip(text, n=400):
    text = text.replace("\r\n", "\n")
    return text if len(text) <= n else text[:n] + f"...[{len(text)} chars]"


def run(args, cwd, timeout=180):
    start = time.perf_counter()
    try:
        p = subprocess.run([str(EXE)] + [str(a) for a in args], cwd=str(cwd), capture_output=True, timeout=timeout)
        code, out, err = p.returncode, p.stdout, p.stderr
    except subprocess.TimeoutExpired as e:
        code, out, err = "timeout", e.stdout or b"", e.stderr or b""
    return {
        "exit": code,
        "out": out.decode("utf-8", "replace").replace("\r\n", "\n"),
        "err": err.decode("utf-8", "replace").replace("\r\n", "\n"),
        "ms": round((time.perf_counter() - start) * 1000),
    }


def brief(r, n=300):
    return {"exit": r["exit"], "out": clip(r["out"], n), "err": clip(r["err"], n), "ms": r["ms"]}


def read_report(path):
    try:
        d = json.loads(Path(path).read_text(encoding="utf-8"))
        return {"outcome": d.get("outcome"), "diagnostic_codes": [x.get("code") for x in d.get("diagnostics", [])]}
    except FileNotFoundError:
        return "absent"
    except Exception as e:  # a present but unreadable report is itself an observation
        return f"unreadable: {e}"


# ---------------------------------------------------------------------------------------------
def identity():
    v = run(["--version", "--json"], REPO)
    return {
        "platform": platform.platform(),
        "machine": platform.machine(),
        "python": platform.python_version(),
        "runner_os": os.environ.get("RUNNER_OS"),
        "runner_arch": os.environ.get("RUNNER_ARCH"),
        "image": os.environ.get("ImageOS"),
        "exe_sha256": hashlib.sha256(EXE.read_bytes()).hexdigest(),
        "exe_bytes": EXE.stat().st_size,
        "version": json.loads(v["out"]) if v["exit"] == 0 else brief(v),
        "uid": os.getuid() if hasattr(os, "getuid") else None,
    }


def doctor():
    r = run(["doctor", "--check"], REPO)
    keep = ("delulu ", "embedded Python", "network client", "root issuance", "level available", "KVM", "OS primitives",
            "identity separation", "filesystem enforcement", "survey freshness", "backend")
    lines = [ln.strip() for ln in r["out"].splitlines() if any(k in ln[:40] for k in keep)]
    last = r["out"].strip().splitlines()[-1] if r["out"].strip() else ""
    return {"exit": r["exit"], "last": last, "lines": [clip(x, 260) for x in lines]}


def sandbox_properties():
    d = WORK / "props"
    d.mkdir()
    (d / "h.delulu").write_text('module h\n\nfn main(root: Root) ! {Write} {\n    root.console().println("hi")\n}\n', encoding="utf-8", newline="\n")
    r = run(["run", "h.delulu", "--sandbox", "--grant", "console", "--no-prompt", "--report-out", "r.json"], d)
    out = {"run": brief(r, 500)}
    try:
        rep = json.loads((d / "r.json").read_text(encoding="utf-8"))
        sb = rep.get("sandbox", {})
        out["level"] = sb.get("level")
        out["backend"] = sb.get("backend")
        out["granted"] = sb.get("granted")
        out["properties"] = {k: (v.get("state") if isinstance(v, dict) else v) for k, v in (sb.get("properties") or {}).items()}
    except Exception as e:
        out["report"] = f"unreadable: {e}"
    return out


# ---------------------------------------------------------------------------------------------
def json_task():
    prog = HERE / "jsontask.delulu"
    root = WORK / "json"
    modes = [("interp", []), ("wasm", ["--engine", "wasm"]), ("sandbox", ["--sandbox"])]
    result = {"check": brief(run(["check", prog], HERE)), "modes": {}, "fixtures": len(oracle.fixtures())}
    actual = {}
    for mode, extra in modes:
        passed, failures, slowest = 0, [], 0
        for name, raw in oracle.fixtures().items():
            d = root / mode / name
            (d / "input").mkdir(parents=True)
            (d / "output").mkdir()
            (d / "input" / "doc.json").write_bytes(raw)
            want = oracle.expected(raw)
            r = run(["run", prog] + extra + GRANTS, d)
            out_file = d / "output" / "out.json"
            file_state = "absent"
            if out_file.exists():
                file_state = "expected" if out_file.read_bytes().decode("utf-8", "replace").replace("\r\n", "\n") == want + "\n" else "different"
            want_file = "absent" if want.startswith('{"error"') else "expected"
            ok = r["exit"] == 0 and r["out"] == want + "\n" and file_state == want_file
            actual.setdefault(name, {})[mode] = r["out"]
            slowest = max(slowest, r["ms"])
            if ok:
                passed += 1
            else:
                failures.append({"fixture": name, "exit": r["exit"], "file": file_state, "want": clip(want, 120), "got": clip(r["out"], 120),
                                 "err": clip("\n".join(x for x in r["err"].splitlines() if not x.startswith(("sandbox:", "guest: sandbox:"))), 300)})
        result["modes"][mode] = {"passed": passed, "failed": len(failures), "slowest_ms": slowest, "failures": failures}
    result["modes_disagree_on"] = sorted(n for n, by in actual.items() if len(set(by.values())) > 1)
    denied = WORK / "json" / "denied"
    (denied / "input").mkdir(parents=True)
    (denied / "output").mkdir()
    (denied / "input" / "doc.json").write_bytes(oracle.fixtures()["f01_basic"])
    r = run(["run", prog, "--grant", "console", "--grant", "fs.read=./input", "--no-prompt"], denied)
    result["denied_write"] = {"exit": r["exit"], "first_error_line": (r["err"].splitlines() or [""])[0][:160], "files_written": len(list((denied / "output").iterdir()))}
    return result


# ---------------------------------------------------------------------------------------------
UTILITY = '''module utility

fn main(root: Root) ! {Read, Write} {
    let out = root.console()
    let reader = root.fs_read("./input")
    let writer = root.fs_write("./output")
    match reader.read_text("task.txt") {
        Ok(text) => {
            let n = len(text)
            match writer.write_text("result.txt", f"bytes={n}\\n") {
                Ok(done) => out.println(f"wrote result for {n} bytes")
                Err(e) => out.println("write failed")
            }
        }
        Err(e) => out.println("read failed")
    }
}
'''
EFFECT_MISSING = 'module effect_missing\n\nfn main(root: Root) {\n    root.console().println("should never print")\n}\n'
SPIN = 'module spin\n\nfn main(root: Root) ! {Write} {\n    let out = root.console()\n    out.println("started")\n    var n = 0\n    while true {\n        n += 1\n    }\n    out.println("unreachable")\n}\n'
MANY = 'module many_lines\n\nfn main(root: Root) ! {Write} {\n    let out = root.console()\n    for i in 0..50000 {\n        out.println(f"line {i}")\n    }\n}\n'
MAIN_ERR = 'module main_result\n\nfn main(root: Root) -> Result[Unit, Str] ! {Write} {\n    root.console().println("about to fail")\n    Err("application failure")\n}\n'


def outcomes():
    t = WORK / "t2"
    (t / "input").mkdir(parents=True)
    (t / "output").mkdir()
    (t / "input" / "task.txt").write_bytes(b"value=10\nvalue=11\n")
    for name, text in (("utility", UTILITY), ("effect_missing", EFFECT_MISSING), ("spin", SPIN), ("many_lines", MANY), ("main_result", MAIN_ERR)):
        (t / f"{name}.delulu").write_text(text, encoding="utf-8", newline="\n")
    sentinel = t / "output" / "result.txt"
    res = {}

    def case(key, args, report=None):
        if sentinel.exists():
            sentinel.unlink()
        shutil.rmtree(t / "reports", ignore_errors=True)
        (t / "reports").mkdir()
        if key == "b":
            (t / "reports" / "isdir").mkdir()
        r = run(args, t)
        res[key] = {**brief(r, 260), "sentinel": sentinel.exists(), "report": read_report(t / report) if report else None}

    case("a_success", ["run", "utility.delulu"] + GRANTS + ["--json", "--report-out", "reports/a.json"], "reports/a.json")
    case("b", ["run", "utility.delulu"] + GRANTS + ["--json", "--report-out", "reports/isdir"])
    case("d_missing_parent", ["run", "utility.delulu"] + GRANTS + ["--json", "--report-out", "reports/nope/deeper/d.json"])
    case("c_inside_grant", ["run", "utility.delulu"] + GRANTS + ["--json", "--report-out", "output/report.json"])
    case("e1_checker_refusal", ["run", "effect_missing.delulu", "--grant", "console", "--no-prompt", "--json", "--report-out", "reports/e1.json"], "reports/e1.json")
    case("e2_checker_refusal_sandbox", ["run", "effect_missing.delulu", "--sandbox", "--grant", "console", "--no-prompt", "--json", "--report-out", "reports/e2.json"], "reports/e2.json")
    case("f1_missing_grant", ["run", "utility.delulu", "--grant", "console", "--no-prompt", "--json", "--report-out", "reports/f1.json"], "reports/f1.json")
    case("g_sandbox_success", ["run", "utility.delulu", "--sandbox"] + GRANTS + ["--json", "--report-out", "reports/g.json"], "reports/g.json")
    case("m_wall_budget", ["run", "spin.delulu", "--grant", "console", "--no-prompt", "--limits", "wall=1", "--json", "--report-out", "reports/m.json"], "reports/m.json")
    case("n_main_returns_err", ["run", "main_result.delulu", "--grant", "console", "--no-prompt", "--report-out", "reports/n.json"], "reports/n.json")
    shutil.copy(t / "effect_missing.delulu", t / "fixme.delulu")
    r = run(["fix", "fixme.delulu", "--dry-run", "--json"], t)
    try:
        d = json.loads(r["out"])
        res["h_fix_dry_run"] = {"exit": r["exit"], "summary": d.get("summary"), "errors_after": d.get("fix", {}).get("errors_after")}
    except Exception:
        res["h_fix_dry_run"] = brief(r)
    r = run(["new", "demo"], t)
    res["new_demo"] = {"exit": r["exit"]}
    demo = t / "demo"
    r = run(["test", str(demo / "does-not-exist.delulu"), "--json"], demo)
    try:
        d = json.loads(r["out"])
        res["i_test_missing_selector"] = {"exit": r["exit"], "summary": d.get("summary"), "tests": len(d.get("tests", [])), "ms": r["ms"]}
    except Exception:
        res["i_test_missing_selector"] = brief(r)
    r = run(["test", ".", "--json"], demo)
    try:
        res["i_test_control"] = {"exit": r["exit"], "summary": json.loads(r["out"]).get("summary"), "ms": r["ms"]}
    except Exception:
        res["i_test_control"] = brief(r)
    (t / "bad-envelope.json").write_text("{}", encoding="utf-8")
    r = run(["schema", "validate", "envelope", "bad-envelope.json", "--json"], t)
    try:
        d = json.loads(r["out"])
        res["j_schema_validate"] = {"exit": r["exit"], "summary": d.get("summary"), "valid": d.get("validate", {}).get("valid")}
    except Exception:
        res["j_schema_validate"] = brief(r)
    res["k1_authority_bogus_isolation"] = {"exit": run(["authority", "utility.delulu", "--isolation", "bogus", "--json"], t)["exit"]}
    res["k3_check_unknown_option"] = {"exit": run(["check", "utility.delulu", "--no-such-option", "--json"], t)["exit"]}

    for key, extra in (("l_closed_pipe_interp", []), ("l_closed_pipe_wasm", ["--engine", "wasm"])):
        shutil.rmtree(t / "reports", ignore_errors=True)
        (t / "reports").mkdir()
        p = subprocess.Popen([str(EXE), "run", "many_lines.delulu"] + extra + ["--grant", "console", "--no-prompt", "--report-out", "reports/l.json"],
                             cwd=str(t), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        first = p.stdout.readline()
        p.stdout.close()
        try:
            code = p.wait(timeout=120)
        except subprocess.TimeoutExpired:
            p.kill()
            code = "timeout"
        err = p.stderr.read().decode("utf-8", "replace")
        p.stderr.close()
        res[key] = {"first_line": first.decode("utf-8", "replace").strip(), "exit": code, "err": clip(err, 300), "report": read_report(t / "reports" / "l.json")}
    return res


# ---------------------------------------------------------------------------------------------
TOML = '[package]\nname = "pkgmix"\nversion = "0.1.0"\nkind = "bin"\n\n[authority]\neffects  = ["Write"]\nfs.read  = []\nfs.write = []\nnet      = []\nsecrets  = []\n'
MAIN = 'module pkgmix\nimport pkgmix.helper\nfn main(root: Root) ! {Write} { root.console().println(str(answer())) }\n'
HELPER = 'module pkgmix.helper\npub fn answer() -> Int { 42 }\n'
HELLO = 'module hello\nfn main(root: Root) ! {Write} { root.console().println("hello from solulu") }\n'


def extension():
    t = WORK / "t4"
    t.mkdir()
    (t / "hello.solulu").write_text(HELLO, encoding="utf-8", newline="\n")
    res = {
        "loose_check": brief(run(["check", "hello.solulu"], t), 160),
        "loose_run": brief(run(["run", "hello.solulu", "--grant", "console", "--no-prompt"], t), 160),
        "loose_run_wasm": brief(run(["run", "hello.solulu", "--engine", "wasm", "--grant", "console", "--no-prompt"], t), 160),
        "loose_fmt_check": brief(run(["fmt", "--check", "hello.solulu"], t), 200),
    }

    def pkg(name, main_ext, helper_ext):
        d = t / name / "src"
        d.mkdir(parents=True)
        (t / name / "delulu.toml").write_text(TOML, encoding="utf-8", newline="\n")
        (d / f"main.{main_ext}").write_text(MAIN, encoding="utf-8", newline="\n")
        (d / f"helper.{helper_ext}").write_text(HELPER, encoding="utf-8", newline="\n")
        return t / name

    old = pkg("old", "delulu", "delulu")
    res["old_check"] = brief(run(["check", "."], old), 200)
    res["old_run"] = brief(run(["run", ".", "--grant", "console", "--no-prompt"], old), 120)
    res["old_run_wasm"] = brief(run(["run", ".", "--engine", "wasm", "--grant", "console", "--no-prompt"], old), 120)
    res["old_entry_file_run"] = brief(run(["run", "src/main.delulu", "--grant", "console", "--no-prompt"], old), 200)
    a = pkg("mix_a", "delulu", "solulu")
    res["mix_a_check"] = brief(run(["check", "."], a), 200)
    b = pkg("mix_b", "solulu", "delulu")
    res["mix_b_check"] = brief(run(["check", "."], b), 200)
    res["mix_b_run"] = brief(run(["run", ".", "--grant", "console", "--no-prompt"], b), 200)
    return res


# ---------------------------------------------------------------------------------------------
READ_ONE = '''module readone
fn main(root: Root) ! {{Read, Write}} {{
    let out = root.console()
    let r = root.fs_read("./input")
    match r.read_text("{name}") {{
        Ok(text) => out.println(f"read {{len(text)}} bytes")
        Err(_) => out.println("refused or missing")
    }}
}}
'''


def filesystem():
    t = WORK / "fs"
    (t / "input").mkdir(parents=True)
    (t / "outside").mkdir()
    (t / "input" / "doc.txt").write_bytes(b"0123456789")
    (t / "outside" / "secret.txt").write_bytes(b"outside-the-scope")
    res = {"case_insensitive_fs": (t / "input" / "DOC.TXT").exists()}
    try:
        os.symlink(os.path.join("..", "outside", "secret.txt"), t / "input" / "link.txt")
        res["symlink_created"] = True
    except (OSError, NotImplementedError) as e:
        res["symlink_created"] = f"no: {e}"
    names = ["doc.txt", "DOC.TXT", "../outside/secret.txt", "sub/../doc.txt"]
    if res["symlink_created"] is True:
        names.append("link.txt")
    for name in names:
        (t / "one.delulu").write_text(READ_ONE.format(name=name), encoding="utf-8", newline="\n")
        r = run(["run", "one.delulu", "--grant", "console", "--grant", "fs.read=./input", "--no-prompt"], t)
        res[name] = {"exit": r["exit"], "out": r["out"].strip(), "first_error_line": (r["err"].splitlines() or [""])[0][:140]}
    return res


def main():
    if not EXE.exists():
        print(f"harness error: no binary at {EXE}")
        return 1
    results = {"harness_errors": []}
    for key, fn in (("identity", identity), ("doctor", doctor), ("sandbox_properties", sandbox_properties), ("json_task", json_task),
                    ("outcomes", outcomes), ("extension", extension), ("filesystem", filesystem)):
        try:
            results[key] = fn()
        except Exception as e:  # keep going: one broken section must not hide the others
            results["harness_errors"].append(f"{key}: {type(e).__name__}: {e}")
    text = json.dumps(results, indent=1, ensure_ascii=True)
    print("===FABLE-RESULTS-BEGIN===")
    print(text)
    print("===FABLE-RESULTS-END===")
    jt = results.get("json_task", {}).get("modes", {})
    summary = "; ".join(f"{m} {v['passed']}/{v['passed'] + v['failed']}" for m, v in jt.items())
    line = f"json task: {summary} | harness errors: {len(results['harness_errors'])}"
    print(line)
    gs = os.environ.get("GITHUB_STEP_SUMMARY")
    if gs:
        with open(gs, "a", encoding="utf-8") as f:
            f.write(f"**{os.environ.get('RUNNER_OS')} {os.environ.get('RUNNER_ARCH')}** — {line}\n")
    shutil.rmtree(WORK, ignore_errors=True)
    return 1 if results["harness_errors"] else 0


if __name__ == "__main__":
    sys.exit(main())
