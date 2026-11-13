#!/usr/bin/env python3
"""FABLE audit T5 — two principals, separate keys, strict custody. Expectations are written here BEFORE the run.

Principal A (the issuer) holds the anchor key and nothing else. Principal B (the holder) runs the broker in
strict mode (roots only from A's anchor), adopts A's certificate, delegates narrower leases and runs programs.
With FABLE_T5_USER_A and FABLE_T5_USER_B set (Linux, passwordless sudo), A and B are two real OS accounts
with their own homes; otherwise both run as the current user and the result is labelled same-user.

Prints one JSON object. Each step records what was expected, what was observed, and whether they match.
Tokens and owner codes never leave this process: outputs are redacted before printing.
"""
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.dont_write_bytecode = True
EXE = os.environ.get("FABLE_EXE") or str(Path(__file__).resolve().parent.parent.parent / "target" / "debug" / ("delulu.exe" if os.name == "nt" else "delulu"))
UA, UB = os.environ.get("FABLE_T5_USER_A"), os.environ.get("FABLE_T5_USER_B")
TWO_USERS = bool(UA and UB)
BASE = Path(os.environ.get("FABLE_T5_BASE") or tempfile.mkdtemp(prefix="fable-t5-"))
HOME_A = Path(f"/home/{UA}") if TWO_USERS else BASE / "homeA"
HOME_B = Path(f"/home/{UB}") if TWO_USERS else BASE / "homeB"
STATE_B = HOME_B / "dstate"
SECRET = re.compile(r"(dlt1_|gow1_)[A-Za-z0-9_.\-]+")
steps = []


def redact(text):
    return SECRET.sub(lambda m: m.group(1) + "<redacted>", text)


def env_for(home, state):
    return {"DELULU_NO_FIRST_RUN": "1", "DELULU_HOME": str(home / "dhome"), "DELULU_STATE_DIR": str(state), "HOME": str(home)}


def call(who, args, cwd=None, timeout=60, raw=False):
    """Run a command as principal 'A' or 'B'. args[0] == EXE unless raw."""
    home, state, user = (HOME_A, HOME_A / "dstate", UA) if who == "A" else (HOME_B, STATE_B, UB)
    env = env_for(home, state)
    cmd = list(map(str, args))
    if TWO_USERS:
        cmd = ["sudo", "-n", "-u", user, "env"] + [f"{k}={v}" for k, v in env.items()] + (["--chdir=" + str(cwd)] if False else []) + cmd
        full_env = None
    else:
        full_env = {**os.environ, **env}
    try:
        p = subprocess.run(cmd, cwd=str(cwd or BASE), env=full_env, capture_output=True, timeout=timeout)
        code, out, err = p.returncode, p.stdout.decode("utf-8", "replace"), p.stderr.decode("utf-8", "replace")
    except subprocess.TimeoutExpired:
        code, out, err = "timeout", "", ""
    return code, out.replace("\r\n", "\n"), err.replace("\r\n", "\n")


def dl(who, *args, **kw):
    return call(who, [EXE, *args], **kw)


def step(sid, what, expect, observed_ok, detail):
    steps.append({"id": sid, "what": what, "expect": expect, "matches": bool(observed_ok), "detail": redact(detail)[:330]})
    return observed_ok


def first_line(text):
    for ln in text.splitlines():
        if ln.strip() and not ln.startswith(("guard:", "lease:", "custody:", "delulu guard")):
            return ln.strip()
    return ""


def refused(code, out, err, needle=None):
    blob = out + err
    return code not in (0, "timeout") and (needle is None or needle in blob)


def lease(parent, effects, extra, ttl="5m"):
    code, out, err = dl("B", "grants", "delegate", "--parent", parent, "--effects", effects, *extra, "--ttl", ttl, "--json")
    try:
        d = json.loads(out)
        return code, d.get("token"), d.get("node"), out + err
    except Exception:
        return code, None, None, out + err


PROGS = {
    "read_sub": 'module read_sub\nfn main(root: Root) ! {Read, Write} {\n    let out = root.console()\n    let r = root.fs_read("./data/in/sub")\n    match r.read_text("c.txt") {\n        Ok(t) => out.println(f"read: {t.trim()}")\n        Err(_) => out.println("read refused or missing")\n    }\n}\n',
    "read_in": 'module read_in\nfn main(root: Root) ! {Read, Write} {\n    let out = root.console()\n    let r = root.fs_read("./data/in")\n    match r.read_text("a.txt") {\n        Ok(t) => out.println(f"read: {t.trim()}")\n        Err(_) => out.println("read refused or missing")\n    }\n}\n',
    "read_sibling": 'module read_sibling\nfn main(root: Root) ! {Read, Write} {\n    let out = root.console()\n    let r = root.fs_read("./data/other")\n    match r.read_text("s.txt") {\n        Ok(t) => out.println(f"read: {t.trim()}")\n        Err(_) => out.println("read refused or missing")\n    }\n}\n',
    "write_out": 'module write_out\nfn main(root: Root) ! {Read, Write} {\n    let out = root.console()\n    let w = root.fs_write("./data/out")\n    match w.write_text("o.txt", "written") {\n        Ok(_) => out.println("wrote")\n        Err(_) => out.println("write refused")\n    }\n}\n',
    "spin": 'module spin\nfn main(root: Root) ! {Read, Write} {\n    let out = root.console()\n    let r = root.fs_read("./data/in/sub")\n    out.println("started")\n    var n = 0\n    while true {\n        n += 1\n    }\n}\n',
}


def setup():
    for d in ("data/in/sub", "data/out", "data/other", "progs"):
        (BASE / d).mkdir(parents=True, exist_ok=True)
    (BASE / "data/in/a.txt").write_text("alpha\n")
    (BASE / "data/in/sub/c.txt").write_text("child-ok\n")
    (BASE / "data/other/s.txt").write_text("sibling-secret\n")
    for name, text in PROGS.items():
        (BASE / "progs" / f"{name}.delulu").write_text(text, encoding="utf-8", newline="\n")
    if TWO_USERS:
        subprocess.run(["sudo", "-n", "chmod", "-R", "a+rX", str(BASE)], check=True)
        subprocess.run(["sudo", "-n", "chown", "-R", UB, str(BASE / "data")], check=True)
        subprocess.run(["sudo", "-n", "chmod", "a+rx", EXE], check=False)
    else:
        for h in (HOME_A, HOME_B):
            h.mkdir(parents=True, exist_ok=True)


def main():
    setup()
    absd = (BASE / "data").as_posix()
    ids = {"two_os_users": TWO_USERS}
    if TWO_USERS:
        ids["uid_A"] = call("A", ["id", "-u"], raw=True)[1].strip()
        ids["uid_B"] = call("B", ["id", "-u"], raw=True)[1].strip()
        ids["uid_harness"] = str(os.getuid())

    # --- keys and the strict broker -------------------------------------------------------------
    c, o, e = dl("A", "grants", "pubkey", "--key", HOME_A / "anchor.key")
    anchor = o.strip()
    step("s01", "A mints an anchor key and prints its public half", "64 hex characters", c == 0 and re.fullmatch(r"[0-9a-f]{64}", anchor or ""), f"exit {c}")
    c, o, e = dl("B", "broker", "start", "--require-anchored-roots", anchor)
    step("s02", "B starts its broker in strict mode under A's anchor", "started", c == 0 and "broker started" in o + e, first_line(o + e))
    time.sleep(1.0)
    c, o, e = dl("B", "grants", "pubkey")
    holder = o.strip()
    step("s03", "B prints its holder key", "64 hex characters, not the anchor", c == 0 and re.fullmatch(r"[0-9a-f]{64}", holder or "") and holder != anchor, f"exit {c}")
    c, o, e = dl("B", "run", "progs/read_sub.delulu", "--broker", "daemon", "--grant", "console", "--grant", "fs.read=./data/in/sub", "--no-prompt")
    step("s04", "B asks for an unsigned root under strict mode", "refused DL1421, nothing read", refused(c, o, e, "DL1421") and "read:" not in o, first_line(e or o))

    # --- A certifies, B adopts ------------------------------------------------------------------
    cert = BASE / "root.dlcert"
    c, o, e = dl("A", "grants", "certify", "--subject", holder, "--effects", "Read,Write", "--fs-read", f"{absd}/in", "--ttl", "20m", "--key", HOME_A / "anchor.key", "--out", HOME_A / "root.dlcert")
    if TWO_USERS:
        subprocess.run(["sudo", "-n", "cp", str(HOME_A / "root.dlcert"), str(cert)], check=False)
        subprocess.run(["sudo", "-n", "chmod", "a+r", str(cert)], check=False)
    else:
        shutil.copy(HOME_A / "root.dlcert", cert)
    step("s05", "A certifies Read and Write effects with one scope, fs.read on data/in, for B's key (20 minutes)", "certificate written", c == 0 and cert.exists(), first_line(o + e))
    c, o, e = dl("B", "grants", "adopt", cert, "--anchor", anchor)
    node = next((ln.strip() for ln in o.splitlines() if ln.startswith("g_")), None)
    step("s06", "B adopts A's certificate", "a live node", c == 0 and node is not None, first_line(o + e))
    if not node:
        return finish(ids)

    def run_with(token, prog):
        return dl("B", "run", f"progs/{prog}.delulu", "--lease", token or "missing", "--no-prompt")

    # --- the allowed task, then every refusal ---------------------------------------------------
    c, tok, n1, blob = lease(node, "Read,Write", ["--fs-read", f"{absd}/in/sub"])
    step("s07", "B delegates a narrower lease: fs.read on data/in/sub", "a token", c == 0 and bool(tok), first_line(blob) if not tok else "token issued")
    c, o, e = run_with(tok, "read_sub")
    step("s08", "the delegate reads data/in/sub/c.txt under the lease", "prints read: child-ok, exit 0", c == 0 and "read: child-ok" in o, f"exit {c}: {first_line(o)}")
    c, o, e = run_with(tok, "read_sub")
    step("s09", "the same token is presented again (replay)", "refused, nothing read", refused(c, o, e) and "read:" not in o, first_line(e or o))
    c, tok, _, _ = lease(node, "Read,Write", ["--fs-read", f"{absd}/in/sub"])
    c, o, e = run_with(tok, "read_in")
    step("s10", "lease on data/in/sub; the program asks for the wider data/in", "refused for the read scope, a.txt not read", refused(c, o, e) and "read: alpha" not in o and "`console`" not in first_line(e or o), first_line(e or o))
    c, tok, _, _ = lease(node, "Read,Write", ["--fs-read", f"{absd}/in/sub"])
    c, o, e = run_with(tok, "read_sibling")
    step("s11", "lease on data/in/sub; the program asks for the sibling data/other", "refused for the read scope, s.txt not read", refused(c, o, e) and "sibling-secret" not in o and "`console`" not in first_line(e or o), first_line(e or o))
    c, tok, _, _ = lease(node, "Read,Write", ["--fs-read", f"{absd}/in/sub"])
    c, o, e = run_with(tok, "write_out")
    step("s12", "no fs.write scope in the lease; the program writes data/out/o.txt", "refused for the write scope, no file", refused(c, o, e) and not (BASE / "data/out/o.txt").exists() and "`console`" not in first_line(e or o), first_line(e or o))
    c, tok, _, blob = lease(node, "Read", ["--fs-read", f"{absd}/other"])
    step("s13", "B delegates a scope outside the certificate (data/other)", "refused at delegation", c != 0 and not tok, first_line(blob))
    c, tok, _, blob = lease(node, "Read,Write,Net", ["--fs-read", f"{absd}/in/sub", "--net", "example.com"])
    step("s14", "B delegates an effect the certificate lacks (Net)", "refused at delegation", c != 0 and not tok, first_line(blob))
    c, tok, _, blob = lease(node, "Read,Write", ["--fs-read", f"{absd}/in/sub", "--fs-write", f"{absd}/out"])
    step("s14b", "B delegates a write scope the certificate lacks", "refused at delegation", c != 0 and not tok, first_line(blob))
    c, tok, n5, _ = lease(node, "Read,Write", ["--fs-read", f"{absd}/in/sub"])
    rc, ro, re_ = dl("B", "grants", "revoke", n5 or "g_missing")
    c, o, e = run_with(tok, "read_sub")
    step("s15", "a lease is revoked, then used", "revoke succeeds; the run is refused", rc == 0 and refused(c, o, e) and "read:" not in o, f"revoke exit {rc}; run: {first_line(e or o)}")
    c, tok, _, blob = lease(node, "Read,Write", ["--fs-read", f"{absd}/in/sub"], ttl="2s")
    time.sleep(3.5)
    c2, o, e = run_with(tok, "read_sub")
    step("s16", "a 2-second lease is used after 3.5 seconds", "refused as expired", bool(tok) and refused(c2, o, e) and "read:" not in o, first_line(e or o) if tok else first_line(blob))

    # --- certificates that must not be adopted --------------------------------------------------
    c, o, e = dl("A", "grants", "pubkey", "--key", HOME_A / "other.key")
    other = o.strip()
    c, o, e = dl("A", "grants", "certify", "--subject", other, "--effects", "Read,Write", "--fs-read", f"{absd}/other", "--ttl", "20m", "--key", HOME_A / "anchor.key", "--out", HOME_A / "wrong-subject.dlcert")
    pub = BASE / "wrong-subject.dlcert"
    copy_out(HOME_A / "wrong-subject.dlcert", pub)
    c, o, e = dl("B", "grants", "adopt", pub, "--anchor", anchor)
    foreign = next((ln.strip() for ln in o.splitlines() if ln.startswith("g_")), None)
    step("s17", "B adopts a certificate A issued to a different key", "refused (wrong holder)", refused(c, o, e), first_line(e or o))
    if foreign:
        c, tok, _, blob = lease(foreign, "Read,Write", ["--fs-read", f"{absd}/other"])
        c, o, e = run_with(tok, "read_sibling")
        step("s17b", "consequence of s17: B uses the authority that certificate carried", "nothing to use (s17 refused)", False, f"exit {c}: {first_line(o) or first_line(e)}")
    tampered = BASE / "tampered.dlcert"
    text = cert.read_text(encoding="utf-8")
    tampered.write_text(text.replace("/in", "/other", 1), encoding="utf-8", newline="\n")
    if TWO_USERS:
        subprocess.run(["sudo", "-n", "chmod", "a+r", str(tampered)], check=False)
    c, o, e = dl("B", "grants", "adopt", tampered, "--anchor", anchor)
    step("s18", "B adopts A's certificate with its scope edited", "refused (signature)", text != tampered.read_text(encoding="utf-8") and refused(c, o, e), first_line(e or o))
    c, o, e = dl("B", "grants", "pubkey", "--key", HOME_B / "rogue.key")
    rogue = o.strip()
    c, o, e = dl("B", "grants", "certify", "--subject", holder, "--effects", "Read,Write", "--fs-read", f"{absd}/other", "--ttl", "20m", "--key", HOME_B / "rogue.key", "--out", HOME_B / "rogue.dlcert")
    c, o, e = dl("B", "grants", "adopt", HOME_B / "rogue.dlcert", "--anchor", anchor)
    step("s19a", "B signs its own certificate with a key that is not the anchor, adopts it naming A's anchor", "refused", refused(c, o, e), first_line(e or o))
    c, o, e = dl("B", "grants", "adopt", HOME_B / "rogue.dlcert", "--anchor", rogue)
    step("s19b", "the same, naming its own key as the anchor on the command line", "refused: strict mode trusts the configured anchor only", refused(c, o, e), first_line(e or o))
    c, o, e = dl("B", "grants", "tree")
    step("s19c", "after s17 to s19b the tree holds no node on data/other", "no such node", c == 0 and "/other" not in o, f"{o.count('[live]')} live node(s); a node on data/other present: {'/other' in o}")

    # --- revoking the root ends everything under it ---------------------------------------------
    c, tok, _, _ = lease(node, "Read,Write", ["--fs-read", f"{absd}/in/sub"])
    rc, ro, re_ = dl("B", "grants", "revoke", node)
    c, o, e = run_with(tok, "read_sub")
    step("s20", "the adopted root is revoked; an unused lease under it is then used", "revoke succeeds; the run is refused", rc == 0 and refused(c, o, e) and "read:" not in o, f"revoke exit {rc}; run: {first_line(e or o)}")
    c, tok, _, blob = lease(node, "Read,Write", ["--fs-read", f"{absd}/in/sub"])
    step("s21", "B delegates under the revoked root", "refused", c != 0 and not tok, first_line(blob))
    c, o, e = dl("B", "audit", "verify")
    step("s22", "B's audit chain verifies after all of the above", "ok", c == 0 and "verified" in o + e, first_line(o + e))
    c, o, e = dl("B", "audit", "query", "--action", "redeem")
    ids["audit_redeem_lines"] = {"allow": o.count(" allow"), "deny": o.count(" deny")}

    # --- the OS boundary between the two accounts (two-user mode only) --------------------------
    if TWO_USERS:
        c, o, e = call("A", ["cat", str(STATE_B / "grant.key")], raw=True)
        step("o1", "A reads B's holder key file", "permission denied by the OS", c != 0 and len(o) == 0, (e or "").strip()[:120])
        c, o, e = call("A", ["ls", str(STATE_B)], raw=True)
        step("o2", "A lists B's state directory", "permission denied by the OS", c != 0, (e or "").strip()[:120])
        c, o, e = call("A", ["env", f"DELULU_STATE_DIR={STATE_B}", EXE, "grants", "list"], raw=True)
        step("o3", "A points the CLI at B's state directory", "refused", c != 0, first_line(e or o))
        modes = {}
        for name in ("", "grant.key", "broker.key", "root_policy.json", "audit"):
            c, o, e = call("B", ["stat", "-c", "%a %U", str(STATE_B / name)], raw=True)
            modes[name or "."] = o.strip()
        c, o, e = call("A", ["stat", "-c", "%a %U", str(HOME_A / "anchor.key")], raw=True)
        modes["A:anchor.key"] = o.strip()
        ids["modes"] = modes
        # a second root so that a lease exists to be shown on a command line
        c, o, e = dl("A", "grants", "certify", "--subject", holder, "--effects", "Read,Write", "--fs-read", f"{absd}/in", "--ttl", "20m", "--key", HOME_A / "anchor.key", "--out", HOME_A / "root2.dlcert")
        copy_out(HOME_A / "root2.dlcert", BASE / "root2.dlcert")
        c, o, e = dl("B", "grants", "adopt", BASE / "root2.dlcert", "--anchor", anchor)
        node2 = next((ln.strip() for ln in o.splitlines() if ln.startswith("g_")), None)
        c, tok, _, _ = lease(node2 or "g_missing", "Read", ["--fs-read", f"{absd}/in/sub"])
        if tok:
            env = env_for(HOME_B, STATE_B)
            cmd = ["sudo", "-n", "-u", UB, "env"] + [f"{k}={v}" for k, v in env.items()] + [EXE, "run", "progs/spin.delulu", "--lease", tok, "--no-prompt", "--limits", "wall=4"]
            p = subprocess.Popen(cmd, cwd=str(BASE), stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            time.sleep(1.5)
            c, o, e = call("A", ["ps", "-eo", "user:20,args"], raw=True)
            seen = any(tok in ln for ln in o.splitlines())
            p.wait(timeout=30)
            step("o4", "while B's leased run is alive, A lists every process's command line", "observation only: is B's lease token visible to A?", True, f"token visible to the other account: {seen}")
            ids["lease_token_visible_to_other_user"] = seen
            c, o, e = call("A", ["env", f"DELULU_STATE_DIR={HOME_A / 'dstate'}", EXE, "run", str(BASE / "progs/read_sub.delulu"), "--lease", tok, "--no-prompt"], raw=True)
            step("o5", "A presents B's (already redeemed) token to its own broker context", "refused", c != 0 and "read:" not in o, first_line(e or o))
    return finish(ids)


def copy_out(src, dst):
    if TWO_USERS:
        subprocess.run(["sudo", "-n", "cp", str(src), str(dst)], check=False)
        subprocess.run(["sudo", "-n", "chmod", "a+r", str(dst)], check=False)
    else:
        try:
            shutil.copy(src, dst)
        except OSError:
            pass


def finish(ids):
    c, o, e = dl("B", "broker", "stop")
    ids["broker_stop_exit"] = c
    time.sleep(0.5)
    c, o, e = dl("B", "broker", "status")
    ids["broker_running_after_stop"] = (c == 0)
    res = {"mode": "two OS users" if TWO_USERS else "same OS user (two keys, two homes)", "ids": ids, "steps": steps,
           "matched": sum(1 for s in steps if s["matches"]), "total": len(steps)}
    print("===FABLE-T5-BEGIN===")
    print(json.dumps(res, indent=1, ensure_ascii=True))
    print("===FABLE-T5-END===")
    for s in steps:
        print(("ok  " if s["matches"] else "DIFF"), s["id"], "-", s["what"], "=>", s["detail"][:150])
    if not TWO_USERS:
        shutil.rmtree(BASE, ignore_errors=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
