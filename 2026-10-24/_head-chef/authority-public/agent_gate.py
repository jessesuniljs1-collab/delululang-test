"""PreToolUse gate: the head chef holds DeluluLang's authority; a test agent (a subagent) cannot change its access.

The main session passes untouched. A subagent's tool call is refused when it would reach the authority (the broker's
state, keys, leases, the owner code, this gate), run delulu other than through `dl`, name a grant or lease itself,
change accounts, permissions or Claude Code's settings, touch the repository, write outside the dogfood workdirs, or
spawn agents of its own. Every decision is logged to gate.log beside this file.

Pattern-based: it stops an agent that follows its brief from changing its access through the obvious spellings. The
boundary against a determined same-account process is a second OS account (docs/DEPLOYMENT.md, Tier 2).
"""
import json
import os
import re
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
LOG = os.path.join(HERE, "gate.log")
SCRATCH = "/c/users/jesse/appdata/local/temp/claude/d--nelan-delululang/eb34182b-835b-44af-b5ec-07064fcae52c/scratchpad"
AUTH = SCRATCH + "/authority"
DOGFOOD = SCRATCH + "/dogfood"
WRAPPER = AUTH + "/bin/dl"
WRAPPER_SHOWN = ("/c/Users/jesse/AppData/Local/Temp/claude/D--nelan-DeluluLang/eb34182b-835b-44af-b5ec-07064fcae52c/"
                 "scratchpad/authority/bin/dl")


def norm(s):
    s = s.replace("\\\\", "/").replace("\\", "/").lower()
    return re.sub(r"(?<![a-z0-9])([a-z]):/", lambda m: "/" + m.group(1) + "/", s)


def log(entry):
    try:
        with open(LOG, "a", encoding="utf-8") as f:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except Exception:
        pass


def deny(reason):
    print(json.dumps({"hookSpecificOutput": {
        "hookEventName": "PreToolUse", "permissionDecision": "deny",
        "permissionDecisionReason": "The head chef holds DeluluLang's authority for this test campaign; this call " + reason +
        ". Run programs only as `" + WRAPPER_SHOWN + " run <condition> <file>` (`... dl conditions` lists them); to ask "
        "for access you lack, use `dl grants propose` and record it in your report."}}))


AUTH_REF = re.compile(
    r"[/.]authority\b|\bauthority/"
    r"|\b(cd|ls|dir|cat|type|rm|cp|mv|pushd|tee|less|head|tail|more|find|get-content|gc|stat|file|xxd|od|strings)"
    r"\s+[\"']?authority\b"
    r"|owner\.code|anchor\.key|holder\.key|\.dlcert|broker\.key|root_policy|agent_gate|gate\.log|/conditions/|\.token\b")
COMMAND_DENY = [
    (re.compile(r"--grant|--lease|--break-glass|--dangerously|--owner\b|--test-authority|--accept-authority"
                r"|--require-anchored|--guard-policy|--state-dir"), "names a grant, lease or owner flag itself"),
    (re.compile(r"\bdelulu\.exe\b|target/(debug|release)/delulu|\bcargo\b|/bin/delulu\b"),
     "runs delulu other than through `dl`"),
    (re.compile(r"delulu_[a-z_]+\s*=|\$env:delulu_|\bsetx\b"), "sets a DELULU_ environment variable (dl sets them)"),
    (re.compile(r"\bwsl(\.exe)?\b|\brunas\b|\bicacls\b|\btakeown\b|\bnet(\.exe)?\s+(user|localgroup)\b|\bschtasks\b"
                r"|\breg(\.exe)?\s+(add|delete)\b|set-executionpolicy|\bsudo\b|\bsc(\.exe)?\s+(create|config)\b"),
     "changes accounts, permissions or the host"),
    (re.compile(r"\.claude/|settings(\.local)?\.json"), "touches Claude Code's settings"),
    (re.compile(r"/d/nelan|\bgit\b"), "touches the DeluluLang repository (read its docs with Read, Grep or Glob)"),
]
SCRIPT_EXT = re.compile(r"\.(sh|bash|py|pyw|ps1|psm1|bat|cmd|js|mjs|cjs|ts|rb|pl|vbs|lnk)$|/[^./]+$")
CONTENT_DENY = re.compile(r"delulu\.exe|--grant|--lease|delulu_[a-z_]+|[/.]authority\b|owner\.code|\bcargo\b"
                          r"|settings\.local\.json|\bwsl\b|\brunas\b")


AUTH_REF_READ = re.compile(r"scratchpad/authority|\.\./authority\b|owner\.code|anchor\.key|holder\.key|\.dlcert"
                           r"|broker\.key|root_policy|agent_gate|gate\.log|\.token\b")
STARTS_IN_DOGFOOD = re.compile(r"^\s*(cd|set-location|pushd)\s+[\"']?" + re.escape(DOGFOOD) + r"(/[^\s;&|\"']*)?[\"']?\s*(&&|;|$)")


def check_command(raw):
    c = norm(raw).replace(WRAPPER, " DLWRAP")
    if not STARTS_IN_DOGFOOD.search(c):
        return "does not start with `cd` into your dogfood workdir (every command must, so nothing lands in the repository)"
    if AUTH_REF.search(c):
        return "reaches the head chef's authority directory or its keys, leases or owner code"
    for rx, why in COMMAND_DENY:
        if rx.search(c):
            return why
    return None


def decide(data):
    tool = data.get("tool_name", "")
    ti = data.get("tool_input", {}) or {}
    entry = {"tool": tool}
    reason = None
    if tool in ("Bash", "PowerShell"):
        entry["command"] = ti.get("command", "")[:400]
        reason = check_command(ti.get("command", ""))
    elif tool in ("Write", "Edit", "NotebookEdit", "MultiEdit"):
        p = norm(ti.get("file_path", "") or ti.get("notebook_path", ""))
        entry["path"] = p
        if not p.startswith(DOGFOOD + "/") or p.startswith(DOGFOOD + "/_tool-state/"):
            reason = "writes outside your dogfood workdir"
        elif SCRIPT_EXT.search(p) and CONTENT_DENY.search(norm(json.dumps(ti, ensure_ascii=False)).replace(WRAPPER, "DLWRAP")):
            reason = "writes a script that would run delulu, name a grant, lease or the authority, outside `dl`"
    elif tool in ("Read", "Grep", "Glob", "LS"):
        text = norm(json.dumps(ti, ensure_ascii=False)).replace(WRAPPER, "DLWRAP")
        entry["input"] = text[:300]
        if AUTH_REF_READ.search(text):
            reason = "reads the head chef's authority directory"
    elif tool not in ("TodoWrite", "TaskCreate", "TaskUpdate", "TaskList", "TaskGet", "ToolSearch"):
        reason = "uses a tool (" + tool + ") that is not open to a test agent"
    return entry, reason


def main():
    raw = sys.stdin.read()
    is_agent = None
    try:
        data = json.loads(raw)
        is_agent = bool(data.get("agent_id") or data.get("agent_type"))
        head = {"t": time.strftime("%Y-%m-%dT%H:%M:%S"), "keys": sorted(data.keys()),
                "agent_id": data.get("agent_id"), "agent_type": data.get("agent_type")}
        if not is_agent:
            head.update({"tool": data.get("tool_name"), "decision": "main-session"})
            log(head)
            return
        entry, reason = decide(data)
        head.update(entry)
        head["decision"] = "deny" if reason else "allow"
        if reason:
            head["reason"] = reason
        log(head)
        if reason:
            deny(reason)
    except Exception as e:  # fail closed for an agent (or an unreadable call), open for the main session
        log({"t": time.strftime("%Y-%m-%dT%H:%M:%S"), "error": repr(e), "is_agent": is_agent})
        if is_agent is not False:
            deny("could not be judged by the authority gate (" + repr(e) + ")")


if __name__ == "__main__":
    main()
