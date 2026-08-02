//! `delulu new` — start a package that already works.
//!
//! Until this existed, the answer to "I built the compiler, now what?" was to hand-write
//! `delulu.toml` and guess the layout from an example. That is a poor first five minutes for a
//! language whose entire proposition has to be understood before anything else makes sense.
//!
//! **The scaffold is a teaching artifact, and its authority is the lesson.** The generated package
//! declares a ceiling equal to exactly what its code does — one effect for a binary, none at all
//! for a library — because the habit worth forming on day one is tightening that line, never
//! loosening it. A template that shipped `effects = ["Read", "Write", "Net"]` "to save you time"
//! would teach the opposite of the thing being taught, on every project anyone ever starts.
//!
//! The generated test holds **no authority at all** and there is no `[test-authority]` table,
//! which is the same lesson from the other side: an absent table grants nothing (invariant 41).

pub fn cmd_new(rest: &[String]) -> i32 {
    let mut target: Option<String> = None;
    let mut lib = false;
    let mut json = false;

    for a in rest {
        match a.as_str() {
            "--lib" => lib = true,
            "--json" => json = true,
            // Human reason to stderr, exit 2, nothing on stdout: the single JSON envelope for a
            // usage error belongs to the outer dispatch, and a second object here would break the
            // one-object contract `json_contract` exists to hold.
            other if other.starts_with('-') => {
                eprintln!("error: unknown option `{other}`\n\n{}", help());
                return 2;
            }
            other => {
                if target.is_some() {
                    eprintln!("error: `new` takes one path\n\n{}", help());
                    return 2;
                }
                target = Some(other.to_string());
            }
        }
    }

    let Some(target) = target else {
        eprintln!("error: `new` needs a name, e.g. `delulu new myapp`\n\n{}", help());
        return 2;
    };

    let dir = std::path::PathBuf::from(&target);
    // `delulu new path/to/myapp` names the directory; the package is `myapp`. This is what a
    // reader expects from every other toolchain, and it is also the only reading that lets the
    // package name stay a legal module identifier while the path stays a path.
    let name = dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| target.clone());

    if let Err(why) = legal_package_name(&name) {
        eprintln!("error: `{name}` cannot be a package name — {why}");
        eprintln!(
            "note: the package name becomes a module name in the source (`module {name}`), so it \
             has to be a legal identifier"
        );
        if let Some(fixed) = suggestion(&name) {
            eprintln!("note: try `delulu new {fixed}`");
        }
        return 2;
    }

    // Refusing a non-empty directory is not pedantry: this writes source files, and a scaffold
    // that quietly overwrote the `src/main.delulu` someone had been working on would be the worst
    // possible first impression.
    if dir.exists() {
        let occupied = std::fs::read_dir(&dir).map(|mut d| d.next().is_some()).unwrap_or(true);
        if occupied {
            eprintln!("error: `{}` already exists and is not empty", dir.display());
            eprintln!("note: `new` never writes into a directory that has something in it");
            return 2;
        }
    }

    let kind = if lib { "lib" } else { "bin" };
    let entry = if lib { format!("src/{name}.delulu") } else { "src/main.delulu".to_string() };
    let files: Vec<(String, String)> = vec![
        ("delulu.toml".to_string(), if lib { lib_manifest(&name) } else { bin_manifest(&name) }),
        (entry.clone(), if lib { lib_source(&name) } else { bin_source(&name) }),
        (".gitignore".to_string(), "/target\n".to_string()),
    ];

    if let Err(e) = std::fs::create_dir_all(dir.join("src")) {
        eprintln!("error: cannot create `{}`: {e}", dir.join("src").display());
        return 1;
    }
    for (rel, body) in &files {
        let p = dir.join(rel);
        if let Err(e) = std::fs::write(&p, body) {
            eprintln!("error: cannot write `{}`: {e}", p.display());
            return 1;
        }
    }

    if json {
        crate::cli::note_json_emitted();
        let map = delulu_diag::SourceMap::new();
        let mut env = delulu_diag::envelope("new", &[], None, &map);
        env["new"] = serde_json::json!({
            "name": name,
            "kind": kind,
            "path": dir.display().to_string(),
            "entry": entry,
            "files": files.iter().map(|(r, _)| r.clone()).collect::<Vec<_>>(),
            "authority": if lib { Vec::<String>::new() } else { vec!["Write".to_string()] },
        });
        println!("{}", serde_json::to_string_pretty(&env).expect("envelope serialization cannot fail"));
    } else {
        eprintln!("{}", next_steps(&target, &name, lib, files.len()));
    }
    0
}

/// Whether this can be both a directory name and a `module` name.
///
/// The two are the same string in the generated source, so a name that is legal for one and not
/// the other produces a package that does not parse — which is a far worse error to hand someone
/// than a refusal here.
fn legal_package_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() {
        return Err("it is empty");
    }
    if !name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_') {
        return Err("it has to start with a letter or `_`");
    }
    if let Some(bad) = name.chars().find(|c| !c.is_ascii_alphanumeric() && *c != '_') {
        return Err(match bad {
            '-' => "`-` is not allowed in an identifier",
            '.' => "`.` separates module paths and cannot be part of a name",
            _ => "it may only contain letters, digits and `_`",
        });
    }
    // BOTH keyword lists, because they mean different things and only one of them is obvious.
    // `is_reserved` covers words reserved for FUTURE use (`async`, `iso`, `val`, …); the active
    // keywords the lexer emits today live in `MORPHABLE_KEYWORDS`. Checking only the first accepts
    // `delulu new fn` and hands someone a package containing `module fn`, which does not parse —
    // caught by `a_name_that_cannot_be_a_module_is_refused`, not by reading the code.
    if delulu_syntax::token::is_reserved(name)
        || delulu_syntax::morph::MORPHABLE_KEYWORDS.contains(&name)
    {
        return Err("it is a keyword");
    }
    Ok(())
}

/// The nearest legal name, when there is an obvious one. `my-app` → `my_app`.
fn suggestion(name: &str) -> Option<String> {
    let fixed: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let fixed = fixed.trim_matches('_').to_string();
    (!fixed.is_empty() && legal_package_name(&fixed).is_ok() && fixed != name).then_some(fixed)
}

fn bin_manifest(name: &str) -> String {
    format!(
        "[package]\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         kind = \"bin\"\n\
         \n\
         # The CEILING, not a request. No function in this package may exceed it, and no\n\
         # dependency can widen it — `delulu authority --diff` treats a later widening of this\n\
         # line as the supply-chain event it is.\n\
         #\n\
         # It starts at exactly what src/main.delulu does. Tightening this line is the habit\n\
         # worth forming; a template that granted more \"to save time\" would teach the opposite.\n\
         [authority]\n\
         effects  = [\"Write\"]\n\
         fs.read  = []\n\
         fs.write = []\n\
         net      = []\n\
         secrets  = []\n\
         \n\
         # There is deliberately no [test-authority] table, and that means PURE: the test in\n\
         # src/main.delulu holds no authority at all. Add one only when a test genuinely needs an\n\
         # effect — an absent table grants nothing, rather than defaulting to something.\n"
    )
}

fn lib_manifest(name: &str) -> String {
    format!(
        "[package]\n\
         name = \"{name}\"\n\
         version = \"0.1.0\"\n\
         kind = \"lib\"\n\
         \n\
         # Nothing, and that is the point. A library that computes has no business touching the\n\
         # disk, the network or the clock, and this line is the ENFORCED claim that it does not —\n\
         # not a comment about intent. It is also the most any consumer can ever be asked to\n\
         # grant this package, which is what makes an empty ceiling worth keeping empty.\n\
         [authority]\n\
         effects = []\n"
    )
}

fn bin_source(name: &str) -> String {
    format!(
        "module {name}\n\
         \n\
         // `main` receives the only authority this program will ever have. Nothing is ambient:\n\
         // delete `root` from this signature and nothing below can reach the console, however\n\
         // much it asks. The `! {{Write}}` row is the compiler's record of what that permits.\n\
         fn main(root: Root) ! {{Write}} {{\n    \
             let out = root.console()\n    \
             out.println(greeting(\"world\"))\n\
         }}\n\
         \n\
         // No row means `!{{}}` — provably pure, and checked rather than promised. Give this\n\
         // function something effectful to do and the compiler refuses it until you say so above.\n\
         fn greeting(name: Str) -> Str {{\n    \
             \"hello, \" + name\n\
         }}\n\
         \n\
         test \"greeting builds the message\" {{\n    \
             assert(greeting(\"world\") == \"hello, world\")\n\
         }}\n"
    )
}

fn lib_source(name: &str) -> String {
    format!(
        "module {name}\n\
         \n\
         // `pub` is what crosses the package boundary. No row means `!{{}}` — provably pure, and\n\
         // proved ACROSS that boundary: a consumer does not have to trust this, and neither does\n\
         // a consumer's consumer.\n\
         pub fn greeting(name: Str) -> Str {{\n    \
             \"hello, \" + name\n\
         }}\n\
         \n\
         test \"greeting builds the message\" {{\n    \
             assert(greeting(\"world\") == \"hello, world\")\n\
         }}\n"
    )
}

fn next_steps(target: &str, name: &str, lib: bool, written: usize) -> String {
    let kind = if lib { "lib" } else { "bin" };
    let mut s = format!("Created `{name}` ({kind}) — {written} files.\n\n");
    s.push_str(&format!("  cd {target}\n"));
    s.push_str("  delulu check .            it already checks clean\n");
    // `delulu test .`, not `delulu test`. The bare form looks for a `./tests` directory and
    // refuses without one, and a scaffold whose very first instructions do not work is worse than
    // no scaffold. Every command printed here is executed by `the_generated_package_does_what_the
    // _message_promises`, which is what caught this.
    s.push_str("  delulu test .             one test, holding no authority at all\n");
    if lib {
        s.push_str("  delulu authority .        what this package can do: nothing\n\n");
        s.push_str(
            "An empty ceiling in delulu.toml is the strongest claim a package can make, and\n\
             the easiest one to keep. Widen it only when the code genuinely needs to.\n",
        );
    } else {
        s.push_str("  delulu authority .        what this program can do: {Write}\n");
        s.push_str("  delulu run . --grant console\n\n");
        s.push_str(
            "`--grant console` is not boilerplate — it is the whole language. Leave it off and\n\
             the program stops with DL0703, because a DeluluLang program holds no ambient\n\
             authority: the right to write to your terminal is something a person hands over,\n\
             once, explicitly.\n",
        );
    }
    s
}

fn help() -> String {
    "delulu new — start a package that already checks, tests and runs\n\n\
     USAGE:\n  \
       delulu new <name>          a binary package (src/main.delulu)\n  \
       delulu new <name> --lib    a library package (src/<name>.delulu)\n  \
       delulu new <name> --json   one JSON envelope describing what was written\n\n\
     The path may have directories in it (`delulu new apps/myapp`); the last component is the\n\
     package name and must be a legal identifier, because it becomes a module name.\n\n\
     The generated package declares a ceiling equal to exactly what its code does — one effect\n\
     for a binary, none for a library. Nothing is written into a directory that is not empty.\n\n\
     Exit 0 on success, 1 if a file could not be written, 2 on a bad invocation."
        .to_string()
}
