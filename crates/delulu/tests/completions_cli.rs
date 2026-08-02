//! `delulu completions` — and the three-way binding that makes it safe to have.
//!
//! A completion script is a copy of the command list. The value of this suite is not that the
//! scripts parse; it is that **the dispatcher, the help text and the completion script are proved
//! to name the same set of commands**. `deploy` and `fleet` were once working commands `--help`
//! never mentioned, and that is exactly the shape of mistake this catches.

use std::process::{Command, Output};

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary must run")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

/// The commands the generated bash script offers — the observable form of the canonical list.
fn completed_commands() -> Vec<String> {
    let script = stdout(&delulu(&["completions", "bash"]));
    let start = script.find("compgen -W \"").expect("the bash script offers a word list") + 12;
    let rest = &script[start..];
    let end = rest.find('"').expect("the word list is quoted");
    rest[..end].split_whitespace().map(str::to_string).collect()
}

/// The commands `delulu --help` documents.
fn documented_commands() -> Vec<String> {
    let help = stdout(&delulu(&["--help"]));
    let mut out: Vec<String> = Vec::new();
    for line in help.lines() {
        // Only INDENTED invocation lines. The unindented first line is the title —
        // `delulu — the DeluluLang compiler and runtime` — and the trailing prose mentions
        // commands inside backticks, so neither is an invocation.
        if !line.starts_with(char::is_whitespace) {
            continue;
        }
        let Some(rest) = line.trim_start().strip_prefix("delulu ") else { continue };
        let Some(name) = rest.split_whitespace().next() else { continue };
        if name.starts_with('-') || name.starts_with('<') {
            continue;
        }
        if !out.iter().any(|c| c == name) {
            out.push(name.to_string());
        }
    }
    out
}

/// **The dispatcher, the help text and the completion script name the same commands.**
///
/// This is the whole reason a generated completion is worth more than a written one. Each of the
/// three can be edited without the others, and each is the thing some other part of the toolchain
/// trusts: the sweep in `json_contract` iterates the documented set, a user reads the help, and a
/// shell offers the script. When they disagree, a real command silently stops being swept, or
/// offered, or findable.
#[test]
fn the_command_list_agrees_with_the_dispatcher_and_the_usage_text() {
    let mut completed = completed_commands();
    let mut documented = documented_commands();
    assert!(completed.len() > 25, "sanity: the list is populated ({})", completed.len());

    completed.sort();
    documented.sort();
    assert_eq!(
        completed, documented,
        "the completion script and `delulu --help` disagree about which commands exist"
    );

    // And every one of them is a command the dispatcher actually accepts. `--help` is the safe
    // probe: rule D11 makes asking a command what it does never make it do the thing.
    for name in &completed {
        let o = delulu(&[name, "--help"]);
        let seen = format!("{}{}", stdout(&o), String::from_utf8_lossy(&o.stderr));
        assert!(
            !seen.contains("unknown command"),
            "`{name}` is offered by completion but the dispatcher does not know it"
        );
        assert_eq!(o.status.code(), Some(0), "`delulu {name} --help` must answer:\n{seen}");
    }
}

/// The internal foreign worker is never offered. It is spawned by the host, never typed.
#[test]
fn the_hidden_worker_subcommand_is_not_advertised() {
    let completed = completed_commands();
    for name in &completed {
        assert!(
            !name.contains("worker"),
            "`{name}` looks like the internal worker and must not be completable"
        );
    }
}

/// Each script has the shape its shell requires to load at all.
#[test]
fn each_shell_gets_a_script_its_shell_can_load() {
    let checks: [(&str, &[&str]); 4] = [
        ("bash", &["_delulu()", "complete -o default -F _delulu delulu"]),
        ("zsh", &["#compdef delulu", "_describe -t commands"]),
        ("fish", &["complete -c delulu -f", "__fish_use_subcommand"]),
        ("powershell", &["Register-ArgumentCompleter -Native -CommandName delulu", "CompletionResult"]),
    ];
    for (shell, needles) in checks {
        let o = delulu(&["completions", shell]);
        assert_eq!(o.status.code(), Some(0), "`completions {shell}` must succeed");
        let s = stdout(&o);
        for n in needles {
            assert!(s.contains(n), "the {shell} script must contain `{n}`:\n{s}");
        }
        // Every script carries the full command list, not a subset someone trimmed.
        for cmd in completed_commands() {
            assert!(s.contains(&cmd), "the {shell} script is missing `{cmd}`");
        }
    }
}

/// `#compdef` must be the first line of a zsh completion, or zsh will not autoload it.
#[test]
fn the_zsh_script_starts_with_its_directive() {
    let s = stdout(&delulu(&["completions", "zsh"]));
    assert!(s.starts_with("#compdef delulu\n"), "zsh reads this line first:\n{s}");
}

/// A shell nobody generates for is refused by name, with the ones that exist listed.
#[test]
fn an_unknown_shell_is_refused() {
    let o = delulu(&["completions", "tcsh"]);
    assert_eq!(o.status.code(), Some(2));
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("unknown shell `tcsh`"), "{err}");
    assert!(err.contains("bash") && err.contains("powershell"), "list what does exist:\n{err}");
    assert!(stdout(&o).is_empty(), "and emit no half-script");
}

/// There is no JSON form of a shell script, and saying so beats emitting something unusable.
#[test]
fn there_is_no_json_form() {
    let o = delulu(&["completions", "bash", "--json"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&o.stderr).contains("no `--json` form"),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
}

/// The negative half of the CLI contract.
#[test]
fn completions_refuses_an_option_it_does_not_know() {
    let o = delulu(&["completions", "bash", "--fancy"]);
    assert_eq!(o.status.code(), Some(2), "a bad invocation exits 2");
    assert!(String::from_utf8_lossy(&o.stderr).contains("unknown option"));

    let o2 = delulu(&["completions"]);
    assert_eq!(o2.status.code(), Some(2), "no shell exits 2");

    let o3 = delulu(&["completions", "bash", "zsh"]);
    assert_eq!(o3.status.code(), Some(2), "two shells exits 2");
}
