//! P4a: the Agent Skill (`skills/delulu/SKILL.md`), and the gates that keep it from drifting.
//!
//! A skill is instructions an agent acts on without a human reading them first, which makes a stale
//! one worse than a missing one: nobody is checking. So the format is validated in-tree, and — the
//! part that actually matters — every command the skill teaches must exist in the binary's own
//! `--help`. That is the gate the roadmap asked for, and it is the one no external format validator
//! could give us.
//!
//! Validated here rather than by the reference Node validator on purpose (ruling D-V2-28): adding an
//! npm dependency to CI to check four single-line fields is supply-chain surface for no gain, and the
//! four rules are cheaper to state than to import.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn delulu(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .args(args)
        .env("DELULU_NO_FIRST_RUN", "1")
        .env("DELULU_NO_COLOR", "1")
        .output()
        .expect("the delulu binary runs")
}

fn skill_text() -> String {
    std::fs::read_to_string(root().join("skills").join("delulu").join("SKILL.md"))
        .expect("skills/delulu/SKILL.md is committed")
}

/// The Agent Skills format, in the four rules this project depends on.
#[test]
fn the_skill_is_a_valid_agent_skills_document() {
    let text = skill_text();
    assert!(text.starts_with("---\n"), "the document must open with YAML frontmatter");
    let body_start = text[4..].find("\n---\n").expect("the frontmatter must be closed") + 4;
    let front = &text[4..body_start];

    // 1. `name` must equal the folder name — that is the format's rule, and a harness resolves the
    //    skill by folder.
    let name = front
        .lines()
        .find_map(|l| l.strip_prefix("name:"))
        .map(str::trim)
        .expect("the frontmatter declares `name`");
    assert_eq!(name, "delulu", "`name:` must equal the folder `skills/delulu/`");

    // 2. `description` carries the triggers. An agent decides whether to load a skill from this one
    //    line, so an empty or vague one means the skill is never loaded at all.
    let desc = front
        .lines()
        .find_map(|l| l.strip_prefix("description:"))
        .map(str::trim)
        .expect("the frontmatter declares `description`");
    assert!(desc.len() > 80, "the description must say enough to trigger on: {desc:?}");
    for trigger in [".delulu", "delulu.toml", "authority"] {
        assert!(desc.contains(trigger), "the description must name `{trigger}` as a trigger: {desc:?}");
    }

    // 3. Under 500 lines. The format's limit, and the reason for it — an agent reads the whole thing.
    let lines = text.lines().count();
    assert!(lines < 500, "the skill is {lines} lines; the format's limit is 500");

    // 4. It points at the pinned reference rather than restating it, which is what keeps the two from
    //    disagreeing.
    assert!(text.contains("docs/for-agents.md"), "the skill must point at the pinned reference");
}

/// **The gate that matters.** Every `delulu <verb>` the skill teaches must be a verb the binary has.
///
/// A skill that names a command the tool does not have sends an agent into a loop it cannot escape —
/// it will try, fail, and try again, because the instructions it was given are the authority. This is
/// derived from `--help`, so the two cannot drift.
#[test]
fn every_command_the_skill_teaches_exists_in_the_binary() {
    let help = String::from_utf8_lossy(&delulu(&["--help"]).stdout).into_owned();
    // The verbs `--help` documents, taken from its own lines rather than from a second list.
    let known: Vec<String> = help
        .lines()
        .filter_map(|l| l.trim().strip_prefix("delulu "))
        .filter_map(|l| l.split_whitespace().next())
        .map(str::to_string)
        .collect();
    assert!(known.len() > 20, "the help must list the commands: found {known:?}");

    // Only COMMAND contexts: a line inside a fenced block that begins with `delulu `, or an inline
    // `` `delulu …` ``. Scanning the prose too would read "Use when you see .delulu files" as the verb
    // `files`, which is how the first version of this test failed — a extractor that cannot tell an
    // instruction from a sentence reports the sentence.
    let text = skill_text();
    let mut taught: Vec<String> = Vec::new();
    let take = |s: &str, out: &mut Vec<String>| {
        if let Some(word) = s.split(|c: char| !c.is_ascii_alphanumeric() && c != '-').next() {
            if !word.is_empty() {
                out.push(word.to_string());
            }
        }
    };
    let mut in_fence = false;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            if let Some(rest) = line.trim_start().strip_prefix("delulu ") {
                take(rest, &mut taught);
            }
            continue;
        }
        // Inline code spans only.
        let mut rest = line;
        while let Some(i) = rest.find("`delulu ") {
            rest = &rest[i + 8..];
            take(rest, &mut taught);
        }
    }
    assert!(!taught.is_empty(), "the skill must teach some commands");
    let missing: Vec<&String> = taught
        .iter()
        // `--help`-style words and prose fragments are not verbs.
        .filter(|w| !w.starts_with('-'))
        .filter(|w| !known.contains(w))
        .collect();
    assert!(
        missing.is_empty(),
        "the skill teaches commands the binary does not have: {missing:?}\nthe binary has: {known:?}"
    );
}

/// Every stable anchor the skill sends a reader to must exist in `for-agents.md`. A reference that
/// points at a heading nobody kept is a dead end an agent cannot diagnose.
#[test]
fn every_anchor_the_skill_cites_exists_in_the_reference() {
    let reference = std::fs::read_to_string(root().join("docs").join("for-agents.md"))
        .expect("docs/for-agents.md is committed");
    let text = skill_text();
    let mut cited: Vec<String> = Vec::new();
    let mut rest = text.as_str();
    while let Some(i) = rest.find("[agents.") {
        rest = &rest[i..];
        if let Some(end) = rest.find(']') {
            cited.push(rest[..=end].to_string());
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }
    assert!(cited.len() >= 10, "the skill should cite the reference's anchors: {cited:?}");
    for a in &cited {
        assert!(reference.contains(a.as_str()), "`{a}` is cited by the skill and absent from for-agents.md");
    }
}

/// `delulu skill` prints the same bytes as the file, and `--json` carries the frontmatter apart from
/// the body. Two copies of agent instructions is two things to go stale; this asserts there is one.
#[test]
fn the_command_prints_exactly_the_committed_skill() {
    let printed = String::from_utf8_lossy(&delulu(&["skill"]).stdout).replace("\r\n", "\n");
    let file = skill_text().replace("\r\n", "\n");
    assert_eq!(printed, file, "`delulu skill` must print the committed file byte for byte");

    let o = delulu(&["skill", "--json"]);
    assert_eq!(o.status.code(), Some(0), "{}", String::from_utf8_lossy(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).expect("one envelope");
    assert_eq!(v["command"], "skill", "{v}");
    assert_eq!(v["skill"]["name"], "delulu", "{v}");
    assert!(
        v["skill"]["description"].as_str().is_some_and(|d| d.contains(".delulu")),
        "the description must survive the split: {v}"
    );
    assert!(
        v["skill"]["body"].as_str().is_some_and(|b| b.starts_with("# DeluluLang")),
        "the body must start after the frontmatter, not include it: {v}"
    );
}

/// The skill states the honesty caveats. These are the sentences a user hears from an agent, and an
/// agent only says them if it was told: that foreign code is a hole in the guarantee, that the sandbox
/// is not the account boundary, and that certification is none.
#[test]
fn the_skill_carries_the_honesty_caveats() {
    // Whitespace-collapsed, because prose WRAPS: the first version of this test looked for
    // "holes in that guarantee" against a document where those words span two lines, and failed on a
    // skill that said exactly the right thing.
    let text: String = skill_text().split_whitespace().collect::<Vec<_>>().join(" ");
    for must in [
        "holes in that guarantee",
        "under** the OS account boundary",
        "same user",
        "Certification is NONE",
        "authority_widening",
    ] {
        assert!(text.contains(must), "the skill must say `{must}` — an agent repeats what it was told");
    }
}
