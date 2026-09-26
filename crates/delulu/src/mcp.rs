//! P4-03: `delulu mcp` — a Model Context Protocol server over stdio, READ-ONLY by construction
//! (D-NE-6, taken as D-V2-38).
//!
//! An agent host that speaks MCP can point this at a workspace and ask what the compiler, the
//! authority analysis, the Atlas, the sandbox and — inside the source tree — the Survey and `doctor`
//! say, as tools. It is the LSP's sibling: stdio, hand-written JSON-RPC (newline-delimited, as MCP's
//! stdio transport is), no SDK dependency.
//!
//! **The door rule.** The server that reads must never be an effector, because that is what makes it
//! safe to point at a hostile workspace. So no tool runs a program, tests one, grants anything, loads
//! code, starts a broker, signs, publishes, formats, fixes or writes. Every tool is annotated
//! `readOnlyHint: true`, and that annotation is not a promise the code makes — it is a property
//! a test checks: each tool's command line is built from a fixed table, and the table is held against
//! the list of commands that act (`every_tool_is_read_only`).
//!
//! **The answers are the CLI's.** A tool runs this same binary's `--json` subcommand with a fixed
//! argument vector (never a shell), so what an agent gets over MCP is byte-for-byte what `delulu check
//! --json` prints — there is no second implementation to disagree. An argument that looks like an
//! option (`-…`) is refused, so a caller cannot smuggle a flag into the fixed vector. Each call is
//! bounded in time. The Survey tools answer in-process from `delulu_survey::answers`, the functions
//! `delulu-survey --json` prints.
//!
//! **Stateless.** `tools/list` and `tools/call` answer whether or not `initialize` came first, and
//! `tools/list` is the same list, in the same order, every time.

use std::io::{BufRead, Write};

use serde_json::{json, Value};

/// The protocol revisions this server speaks, newest first. `initialize` answers with the client's
/// revision when it is one of these, and with the newest otherwise, as the protocol asks.
const PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2025-03-26", "2024-11-05"];

/// How long one tool call may run before it is stopped and reported as an error.
const CALL_DEADLINE: std::time::Duration = std::time::Duration::from_secs(120);

/// Where a tool's answer comes from.
enum Source {
    /// This binary, run with the argument vector the function builds from the call's arguments.
    Cli(fn(&Value) -> Result<Vec<String>, String>),
    /// The Survey, answered in-process (inside the source tree only).
    Survey(fn(&std::path::Path, &Value) -> Result<Value, String>),
}

struct Tool {
    name: &'static str,
    description: &'static str,
    /// The input schema's `properties` and `required`, as JSON Schema.
    input: fn() -> Value,
    source: Source,
    /// Offered only inside the DeluluLang source tree.
    tree_only: bool,
}

/// A string argument that becomes one element of a command line: present, not empty, and not
/// something the command would read as an option.
fn word(args: &Value, key: &str) -> Result<String, String> {
    let s = args.get(key).and_then(Value::as_str).ok_or_else(|| format!("`{key}` is required and must be a string"))?;
    if s.is_empty() {
        return Err(format!("`{key}` is empty"));
    }
    if s.starts_with('-') {
        return Err(format!("`{key}` may not begin with `-`: an option cannot be passed through a tool argument"));
    }
    if s.contains('\0') {
        return Err(format!("`{key}` contains a NUL byte"));
    }
    Ok(s.to_string())
}

fn optional_word(args: &Value, key: &str) -> Result<Option<String>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => word(args, key).map(Some),
    }
}

fn props(p: Value, required: &[&str]) -> Value {
    json!({ "type": "object", "properties": p, "required": required, "additionalProperties": false })
}

fn string(what: &str) -> Value {
    json!({ "type": "string", "description": what })
}

fn tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "atlas",
            description: "The typed code-and-authority graph of a program (`delulu atlas <path> --json`): modules, functions, types, effects, resources and the edges between them, with the authority report.",
            input: || props(json!({ "path": string("a .delulu file or a package directory") }), &["path"]),
            source: Source::Cli(|a| Ok(vec!["atlas".into(), word(a, "path")?, "--json".into()])),
            tree_only: false,
        },
        Tool {
            name: "atlas_query",
            description: "One structural question to the Atlas (`delulu atlas <verb> …`): `node` (a node by name or id), `callers`/`calls` (of a function), `why` (why a program can perform an effect or reach a resource), `path` (how one node reaches another).",
            input: || {
                props(
                    json!({
                        "verb": { "type": "string", "enum": ["node", "callers", "calls", "why", "path"] },
                        "subject": string("the node, function, effect or resource asked about"),
                        "to": string("for `path` only: the node to reach"),
                        "path": string("the .delulu file or package directory to ask about"),
                    }),
                    &["verb", "subject", "path"],
                )
            },
            source: Source::Cli(|a| {
                let verb = word(a, "verb")?;
                if !["node", "callers", "calls", "why", "path"].contains(&verb.as_str()) {
                    return Err(format!("`verb` must be node, callers, calls, why or path (got `{verb}`)"));
                }
                let mut v = vec!["atlas".to_string(), verb.clone(), word(a, "subject")?];
                if verb == "path" {
                    v.push(word(a, "to")?);
                }
                v.push(word(a, "path")?);
                v.push("--json".into());
                Ok(v)
            }),
            tree_only: false,
        },
        Tool {
            name: "authority",
            description: "What a program can do to the machine (`delulu authority <path> --json`): its effects, capabilities and scopes, secrets, foreign calls, and the exact `--grant` flags it needs to run. A requirement, never a permission.",
            input: || props(json!({ "path": string("a .delulu file or a package directory") }), &["path"]),
            source: Source::Cli(|a| Ok(vec!["authority".into(), word(a, "path")?, "--json".into()])),
            tree_only: false,
        },
        Tool {
            name: "check",
            description: "Type- and effect-check one or more files, or a package (`delulu check … --json`): every diagnostic with its code, byte spans and typed repairs. Never runs anything.",
            input: || {
                props(
                    json!({ "paths": { "type": "array", "items": { "type": "string" }, "minItems": 1, "description": ".delulu files, or one package directory" } }),
                    &["paths"],
                )
            },
            source: Source::Cli(|a| {
                let paths = a.get("paths").and_then(Value::as_array).ok_or("`paths` is required and must be a list")?;
                if paths.is_empty() {
                    return Err("`paths` is empty".into());
                }
                let mut v = vec!["check".to_string()];
                for (i, p) in paths.iter().enumerate() {
                    v.push(word(&json!({ "path": p }), "path").map_err(|e| format!("paths[{i}]: {e}"))?);
                }
                v.push("--json".into());
                Ok(v)
            }),
            tree_only: false,
        },
        Tool {
            name: "doctor_check",
            description: "Is this machine healthy, and inside the source tree is the repository map current (`delulu doctor --check --json`)? The read-only form: it never regenerates anything.",
            input: || props(json!({}), &[]),
            source: Source::Cli(|_| Ok(vec!["doctor".into(), "--check".into(), "--json".into()])),
            tree_only: true,
        },
        Tool {
            name: "examples",
            description: "The shipped example programs (`delulu examples --json`), each known to check, with its source, its authority report and the `run` line its grants spell.",
            input: || props(json!({}), &[]),
            source: Source::Cli(|_| Ok(vec!["examples".into(), "--json".into()])),
            tree_only: false,
        },
        Tool {
            name: "explain",
            description: "Long-form documentation for a diagnostic code (`DL0501`) or a named topic (`E-SANDBOX`, `E-PLUGIN`, …) — `delulu explain <code> --json`.",
            input: || props(json!({ "code": string("a DLxxxx code or an E-TOPIC") }), &["code"]),
            source: Source::Cli(|a| Ok(vec!["explain".into(), word(a, "code")?, "--json".into()])),
            tree_only: false,
        },
        Tool {
            name: "sandbox_policy",
            description: "What confinement a sandboxed run of a program WOULD have, without running it (`delulu sandbox policy <path> --json`): the limits, the mode, the policy hash, and whether the sandbox can carry the program at all.",
            input: || {
                props(
                    json!({
                        "path": string("a .delulu file"),
                        "profile": { "type": "string", "enum": ["dev", "contained", "hostile-agent"] },
                    }),
                    &["path"],
                )
            },
            source: Source::Cli(|a| {
                let mut v = vec!["sandbox".to_string(), "policy".into(), word(a, "path")?];
                if let Some(p) = optional_word(a, "profile")? {
                    if crate::policy::Profile::parse(&p).is_none() {
                        return Err(format!("`profile` must be dev, contained or hostile-agent (got `{p}`)"));
                    }
                    v.push("--sandbox-profile".into());
                    v.push(p);
                }
                v.push("--json".into());
                Ok(v)
            }),
            tree_only: false,
        },
        Tool {
            name: "sandbox_probe",
            description: "Which isolation levels this host can give a program, L0 to L4, each from an attempt rather than a version string (`delulu sandbox probe --json`). The L1 attempt starts a guest that runs nothing and writes nothing.",
            input: || props(json!({}), &[]),
            source: Source::Cli(|_| Ok(vec!["sandbox".into(), "probe".into(), "--json".into()])),
            tree_only: false,
        },
        Tool {
            name: "schema",
            description: "The closed JSON Schema of one of this toolchain's outputs (`delulu schema <name> --json`), or the list of them when no name is given.",
            input: || props(json!({ "name": string("envelope, diagnostic, repair, authority, atlas, sandbox, policy, run-report or toolchain") }), &[]),
            source: Source::Cli(|a| {
                let mut v = vec!["schema".to_string()];
                if let Some(n) = optional_word(a, "name")? {
                    v.push(n);
                }
                v.push("--json".into());
                Ok(v)
            }),
            tree_only: false,
        },
        Tool {
            name: "survey_diff",
            description: "Inside the DeluluLang source tree: what a CHANGE breaks — the files git says changed since a revision (the working tree and untracked files included, or a range A..B), the Survey node each maps to, the entrenched ones named with their owner, and the union of their impacts, every hop cited and traced to the file it came from.",
            input: || {
                props(
                    json!({
                        "rev": string("a git revision (`HEAD`, `origin/master`) or a range (`HEAD~3..HEAD`)"),
                        "depth": { "type": "integer", "minimum": 1 },
                    }),
                    &["rev"],
                )
            },
            source: Source::Survey(|root, a| {
                let rev = word(a, "rev")?;
                let depth = a
                    .get("depth")
                    .and_then(Value::as_u64)
                    .map_or(delulu_survey::MAX_WALK_DEPTH, |d| (d as u32).clamp(1, delulu_survey::MAX_WALK_DEPTH));
                // `git diff` and `git ls-files` only: read-only, and `word` has already refused a
                // revision that begins with `-` (`git_changes` refuses it again).
                let changes = delulu_survey::diff::git_changes(root, &rev)?;
                let survey = delulu_survey::Survey::build(root);
                Ok(delulu_survey::diff::diff_json(&survey, &rev, &changes, depth))
            }),
            tree_only: true,
        },
        Tool {
            name: "survey_impact",
            description: "Inside the DeluluLang source tree: everything that breaks if a node changes (or, with `direction: rests_on`, everything it rests on), every hop cited with the file and line it was read from.",
            input: || {
                props(
                    json!({
                        "id": string("a Survey node id, e.g. `crate:delulu-check` or `mod:crates/delulu-check/src/ty.rs`"),
                        "direction": { "type": "string", "enum": ["breaks", "rests_on"] },
                        "depth": { "type": "integer", "minimum": 1 },
                    }),
                    &["id"],
                )
            },
            source: Source::Survey(|root, a| {
                let id = word(a, "id")?;
                let reverse = a.get("direction").and_then(Value::as_str) != Some("rests_on");
                let depth = a
                    .get("depth")
                    .and_then(Value::as_u64)
                    .map_or(delulu_survey::MAX_WALK_DEPTH, |d| (d as u32).clamp(1, delulu_survey::MAX_WALK_DEPTH));
                let survey = delulu_survey::Survey::build(root);
                delulu_survey::answers::walk_json(&survey, &id, reverse, depth).ok_or_else(|| not_in_map(&survey, &id))
            }),
            tree_only: true,
        },
        Tool {
            name: "survey_query",
            description: "Inside the DeluluLang source tree: one node of the repository's map — what it is, what points at it, what it points at, and whether it is entrenched — every edge cited.",
            input: || props(json!({ "id": string("a Survey node id, e.g. `crate:delulu-check`, `code:DL0501`, `doc:README.md`") }), &["id"]),
            source: Source::Survey(|root, a| {
                let id = word(a, "id")?;
                let survey = delulu_survey::Survey::build(root);
                delulu_survey::answers::query_json(&survey, &id, false).ok_or_else(|| not_in_map(&survey, &id))
            }),
            tree_only: true,
        },
        Tool {
            name: "toolchain",
            description: "This toolchain as data (`delulu toolchain --json`): every command with the options its help documents, the ten effects, every --grant form with an example, the primitive table, budgets, sandbox levels, diagnostic codes.",
            input: || props(json!({}), &[]),
            source: Source::Cli(|_| Ok(vec!["toolchain".into(), "--json".into()])),
            tree_only: false,
        },
        Tool {
            name: "why",
            description: "Why a program can perform an effect (`delulu why <Effect> <path> --json`): the shortest chain of calls from `main` to the function that performs it.",
            input: || {
                props(
                    json!({ "effect": string("Read, Write, Net, Clock, Rand, Declassify, ForeignCall, Load, Async or Actuate"), "path": string("a .delulu file or package directory") }),
                    &["effect", "path"],
                )
            },
            source: Source::Cli(|a| Ok(vec!["why".into(), word(a, "effect")?, word(a, "path")?, "--json".into()])),
            tree_only: false,
        },
    ]
}

fn not_in_map(survey: &delulu_survey::Survey, id: &str) -> String {
    let near = delulu_survey::answers::near(survey, id);
    if near.is_empty() {
        format!("no node `{id}` in the map")
    } else {
        format!("no node `{id}` in the map — did you mean: {}", near.join(", "))
    }
}

/// The tools this server offers here, in name order: the source-tree ones only inside it.
fn offered(in_tree: bool) -> Vec<Tool> {
    let mut t: Vec<Tool> = tools().into_iter().filter(|t| in_tree || !t.tree_only).collect();
    t.sort_by_key(|t| t.name);
    t
}

fn list_json(in_tree: bool) -> Value {
    let tools: Vec<Value> = offered(in_tree)
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": (t.input)(),
                "annotations": {
                    "readOnlyHint": true,
                    "destructiveHint": false,
                    "idempotentHint": true,
                    "openWorldHint": false,
                },
            })
        })
        .collect();
    json!({ "tools": tools })
}

/// Run this binary with `argv`, bounded by [`CALL_DEADLINE`], and return its standard output as JSON.
fn run_self(argv: &[String]) -> Result<Value, String> {
    let exe = std::env::current_exe().map_err(|e| format!("this executable cannot be located: {e}"))?;
    let mut child = std::process::Command::new(exe)
        .args(argv)
        .env("DELULU_NO_FIRST_RUN", "1")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not start `delulu {}`: {e}", argv.join(" ")))?;
    // Read both pipes on their own threads, so a large answer cannot fill a pipe and stall the child
    // while this thread waits on it.
    let mut out = child.stdout.take().expect("piped");
    let mut err = child.stderr.take().expect("piped");
    let out_t = std::thread::spawn(move || {
        let mut s = Vec::new();
        let _ = std::io::Read::read_to_end(&mut out, &mut s);
        s
    });
    let err_t = std::thread::spawn(move || {
        let mut s = Vec::new();
        let _ = std::io::Read::read_to_end(&mut err, &mut s);
        s
    });
    let started = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() > CALL_DEADLINE => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("`delulu {}` did not finish within {CALL_DEADLINE:?}", argv.join(" ")));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(5)),
            Err(e) => return Err(format!("waiting for `delulu {}`: {e}", argv.join(" "))),
        }
    }
    let stdout = out_t.join().unwrap_or_default();
    let stderr = err_t.join().unwrap_or_default();
    serde_json::from_slice(&stdout).map_err(|_| {
        let said = String::from_utf8_lossy(&stderr);
        format!("`delulu {}` did not answer in JSON: {}", argv.join(" "), said.trim())
    })
}

/// Answer one `tools/call`. `Err` is a protocol error (no such tool); a tool that ran and refused, or
/// whose command reported errors, is an ordinary result with `isError` set.
fn call(name: &str, args: &Value, tree: Option<&std::path::Path>) -> Result<Value, (i64, String)> {
    let Some(tool) = offered(tree.is_some()).into_iter().find(|t| t.name == name) else {
        return Err((-32602, format!("no tool named `{name}`")));
    };
    let args = if args.is_null() { json!({}) } else { args.clone() };
    let answer = match (&tool.source, tree) {
        (Source::Cli(build), _) => build(&args).and_then(|argv| run_self(&argv)),
        (Source::Survey(answer), Some(root)) => answer(root, &args),
        (Source::Survey(_), None) => Err("this tool answers only inside the DeluluLang source tree".into()),
    };
    Ok(match answer {
        Ok(v) => {
            // A command that ran and reported errors (a program that does not check) is a
            // successful CALL whose answer says so; the envelope's summary is how a caller tells.
            let failed = v["summary"]["errors"].as_u64().is_some_and(|n| n > 0) && name != "check";
            json!({
                "content": [{ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }],
                "structuredContent": v,
                "isError": failed,
            })
        }
        Err(why) => json!({ "content": [{ "type": "text", "text": why }], "isError": true }),
    })
}

fn initialize(params: &Value, in_tree: bool) -> Value {
    let asked = params["protocolVersion"].as_str().unwrap_or("");
    let version = if PROTOCOL_VERSIONS.contains(&asked) { asked } else { PROTOCOL_VERSIONS[0] };
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": "delulu", "version": env!("CARGO_PKG_VERSION") },
        "instructions": format!(
            "DeluluLang's compiler, authority analysis, Atlas and sandbox preview as read-only tools{}. \
             Nothing here runs a program, grants authority or loads code: to run one, use the `delulu` CLI, \
             where the grants are a person's decision.",
            if in_tree { ", and the repository's Survey and doctor (you are inside the source tree)" } else { "" }
        ),
    })
}

pub fn run_mcp(args: &[String]) -> i32 {
    if let Some(a) = args.first() {
        eprintln!("error: `mcp` takes no arguments (got `{a}`) — it is a server on standard input and output");
        return 2;
    }
    let tree = delulu_survey::find_source_tree();
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Err(e) => Some(json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": format!("not JSON: {e}") } })),
            Ok(msg) => handle(&msg, tree.as_deref()),
        };
        if let Some(r) = reply {
            let mut out = stdout.lock();
            // One message per line: serde_json never writes a raw newline inside a value.
            if writeln!(out, "{r}").and_then(|()| out.flush()).is_err() {
                break;
            }
        }
    }
    0
}

/// One message in, at most one reply out. A notification (no `id`) is never answered.
fn handle(msg: &Value, tree: Option<&std::path::Path>) -> Option<Value> {
    let id = msg.get("id").cloned();
    let method = msg["method"].as_str().unwrap_or("");
    let result = match method {
        "initialize" => Ok(initialize(&msg["params"], tree.is_some())),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(list_json(tree.is_some())),
        "tools/call" => call(msg["params"]["name"].as_str().unwrap_or(""), &msg["params"]["arguments"], tree),
        m if m.starts_with("notifications/") => return None,
        "" => Err((-32600, "not a request: no `method`".to_string())),
        other => Err((-32601, format!("`{other}` is not a method this server offers"))),
    };
    let id = id?;
    Some(match result {
        Ok(r) => json!({ "jsonrpc": "2.0", "id": id, "result": r }),
        Err((code, message)) => json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every command a tool can run, read from the tool table by building each tool's command line.
    fn commands_run_by_tools() -> Vec<Vec<String>> {
        let sample = json!({
            "path": "x.delulu", "paths": ["x.delulu"], "code": "DL0501", "effect": "Write", "verb": "path",
            "subject": "main", "to": "helper", "name": "diagnostic", "profile": "dev", "id": "crate:delulu",
        });
        tools()
            .iter()
            .filter_map(|t| match &t.source {
                Source::Cli(build) => Some(build(&sample).unwrap_or_else(|e| panic!("{}: {e}", t.name))),
                Source::Survey(_) => None,
            })
            .collect()
    }

    /// The commands that act: run or test a program, grant, start or stop a broker, touch the Guard,
    /// store a secret, make a key, sign, publish, log in, deploy, command a fleet, fix, format,
    /// scaffold, add, lock, build, build a plugin, install a locale, EDIT a file, write an audit
    /// bundle, or serve a language or MCP. `sandbox` and `doctor` act too, outside the read-only
    /// forms `every_tool_is_read_only` holds them to.
    const EFFECTORS: &[&str] = &[
        "run", "test", "repl", "grants", "broker", "guard", "secrets", "keygen", "sign", "publish",
        "login", "deploy", "fleet", "fix", "fmt", "new", "add", "lock", "build", "plugin", "locale", "lsp",
        "mcp", "edit", "audit",
    ];
    /// The commands that only read and answer. A tool may run one of these and nothing else.
    const READ_ONLY: &[&str] = &[
        "check", "authority", "why", "atlas", "explain", "verify-sig", "morph", "completions",
        "skill", "toolchain", "schema", "examples", "sandbox", "doctor",
    ];

    /// Every command is one or the other, so a command added later cannot reach a tool unreviewed:
    /// the door rule was first written as a list of effectors only, and `edit` — a command that
    /// writes — would have walked past it, because nothing asked about a name the list did not have.
    #[test]
    fn every_command_is_declared_read_only_or_acting() {
        for cmd in crate::cli::SUBCOMMANDS {
            let (r, e) = (READ_ONLY.contains(cmd), EFFECTORS.contains(cmd));
            assert!(r != e, "`{cmd}` must be in exactly one of READ_ONLY and EFFECTORS (read-only: {r}, acting: {e})");
        }
        for cmd in READ_ONLY.iter().chain(EFFECTORS) {
            assert!(crate::cli::SUBCOMMANDS.contains(cmd), "`{cmd}` is not a delulu command");
        }
    }

    /// THE DOOR RULE, as a test. A tool's command line starts only a command declared read-only —
    /// `sandbox` only as `probe` and `policy`, `doctor` only with `--check` — and every tool is
    /// annotated read-only in the list a client sees.
    #[test]
    fn every_tool_is_read_only() {
        for argv in commands_run_by_tools() {
            let cmd = argv[0].as_str();
            assert!(READ_ONLY.contains(&cmd) && !EFFECTORS.contains(&cmd), "a tool runs `delulu {}`", argv.join(" "));
            if cmd == "sandbox" {
                assert!(["probe", "policy"].contains(&argv[1].as_str()), "`sandbox {}` is not read-only", argv[1]);
            }
            if cmd == "doctor" {
                assert!(argv.contains(&"--check".to_string()), "`doctor` without --check writes");
            }
            assert!(argv.contains(&"--json".to_string()), "`{}` must answer in JSON", argv.join(" "));
        }
        for t in list_json(true)["tools"].as_array().unwrap() {
            assert_eq!(t["annotations"]["readOnlyHint"], true, "{}", t["name"]);
            assert_eq!(t["annotations"]["destructiveHint"], false, "{}", t["name"]);
        }
    }

    /// An argument cannot become an option: a value beginning with `-` is refused before any command
    /// line is built, in every string position.
    #[test]
    fn an_argument_that_looks_like_an_option_is_refused() {
        for t in tools() {
            let Source::Cli(build) = &t.source else { continue };
            for key in ["path", "code", "effect", "subject", "name"] {
                let hostile = json!({ key: "--grant=net=evil.example", "paths": ["--sandbox=off"], "verb": "node", "path": "x", "code": "x", "effect": "x", "subject": "x" });
                let mut a = hostile.clone();
                a[key] = json!("--grant=net=evil.example");
                if let Ok(argv) = build(&a) {
                    assert!(
                        !argv.iter().any(|w| w.starts_with("--grant") || w.starts_with("--sandbox=")),
                        "{}: `{}` smuggled an option",
                        t.name,
                        argv.join(" ")
                    );
                }
            }
        }
        assert!(word(&json!({ "p": "-x" }), "p").is_err());
        assert!(word(&json!({ "p": "" }), "p").is_err());
        assert!(word(&json!({ "p": "ok.delulu" }), "p").is_ok());
    }

    /// The list is deterministic — the same tools in the same order every time — and the source-tree
    /// tools are offered only inside it.
    #[test]
    fn the_tool_list_is_deterministic_and_the_tree_tools_are_offered_only_in_the_tree() {
        assert_eq!(list_json(true), list_json(true));
        let names = |v: Value| -> Vec<String> {
            v["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap().to_string()).collect()
        };
        let inside = names(list_json(true));
        let outside = names(list_json(false));
        let mut sorted = inside.clone();
        sorted.sort();
        assert_eq!(inside, sorted, "name order");
        for t in ["survey_query", "survey_impact", "doctor_check"] {
            assert!(inside.contains(&t.to_string()) && !outside.contains(&t.to_string()), "{t}");
        }
        for t in ["check", "authority", "why", "explain", "atlas_query", "toolchain", "schema", "sandbox_probe"] {
            assert!(outside.contains(&t.to_string()), "the roadmap names `{t}`");
        }
    }
}
