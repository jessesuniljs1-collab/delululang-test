//! A pragmatic Stage-1 REPL (§9.5, acceptance criterion 1). Declarations accumulate; an
//! expression is evaluated under a fully-granted root and its value printed. Entering a bare
//! function name prints its type — including its effect row — so authority is always visible.

use std::io::{BufRead, Write};
use std::rc::Rc;

use delulu_check::check_source;
use delulu_diag::{render_human, SourceMap};
use delulu_runtime::value::RootVal;
use delulu_runtime::{Grants, Interp, Value};
use delulu_syntax::ast::{Expr, Item, Stmt};

pub fn run(grants: Grants) -> i32 {
    let stdin = std::io::stdin();
    let mut decls: Vec<String> = Vec::new();
    eprintln!("DeluluLang REPL — declarations accumulate; type an expression to evaluate.");
    eprintln!("Enter a function name to see its type (with effect row). `:q` to quit.\n");
    loop {
        eprint!("delulu> ");
        let _ = std::io::stderr().flush();
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if input == ":q" || input == ":quit" {
            break;
        }
        handle(input, &mut decls, &grants);
    }
    0
}

fn is_declaration(input: &str) -> bool {
    ["fn ", "type ", "effect ", "pub ", "let "].iter().any(|k| input.starts_with(k))
}

fn is_bare_name(input: &str) -> bool {
    let mut chars = input.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && input.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn module_source(decls: &[String], tail: &str) -> String {
    format!("module repl\n{}\n{tail}\n", decls.join("\n"))
}

fn handle(input: &str, decls: &mut Vec<String>, grants: &Grants) {
    if is_declaration(input) {
        let candidate = module_source(decls, input);
        let checked = check_source(0, &candidate);
        let errs: Vec<_> = checked.diagnostics.iter().filter(|d| d.is_error()).collect();
        if !errs.is_empty() {
            print_errors(&checked.diagnostics, &candidate);
            return;
        }
        decls.push(input.to_string());
        // If it was a function, echo its type (authority included).
        if let Some(name) = input.strip_prefix("fn ").or_else(|| input.strip_prefix("pub fn ")) {
            let fname = name.split(['(', '[', ' ']).next().unwrap_or("").to_string();
            if let Some(ty) = checked.result.fn_types.get(&fname) {
                eprintln!("  {fname}: {ty}");
            }
        } else {
            eprintln!("  ok");
        }
        return;
    }

    // A bare function name: print its type.
    if is_bare_name(input) {
        let probe = module_source(decls, &format!("fn __probe(root: Root) {{ let __v = {input}\n }}"));
        let checked = check_source(0, &probe);
        if checked.result.fn_types.contains_key(input) {
            if let Some(ty) = checked.result.fn_types.get(input) {
                eprintln!("  {input}: {ty}");
                return;
            }
        }
    }

    // Otherwise, evaluate the expression under a fully-granted root.
    let wrapper = format!("fn __repl(root: Root) ! {{Read, Write, Net, Clock, Rand, Declassify}} {{ let __v = {input}\n }}");
    let src = module_source(decls, &wrapper);
    let checked = check_source(0, &src);
    let errs: Vec<_> = checked.diagnostics.iter().filter(|d| d.is_error()).collect();
    if !errs.is_empty() {
        print_errors(&checked.diagnostics, &src);
        return;
    }
    // Extract the expression AST (the value of `let __v = <expr>`).
    let Some(expr) = extract_repl_expr(&checked.module) else {
        eprintln!("  (nothing to evaluate)");
        return;
    };
    let interp = Interp::new(&checked.module);
    let root = Value::Root(Rc::new(full_root(grants)));
    match interp.eval_toplevel(&expr, Some(root)) {
        Ok(v) => eprintln!("  => {}", v.display()),
        Err(f) => eprintln!("  runtime {}: {}", f.code, f.message),
    }
}

fn extract_repl_expr(module: &delulu_syntax::ast::Module) -> Option<Expr> {
    for item in &module.items {
        if let Item::Fn(f) = item {
            if f.name.name == "__repl" {
                if let Some(Stmt::Let { value, .. }) = f.body.stmts.first() {
                    return Some(value.clone());
                }
            }
        }
    }
    None
}

fn full_root(grants: &Grants) -> RootVal {
    RootVal {
        console: true,
        fs_read: vec![std::env::current_dir().unwrap_or_default()],
        fs_write: vec![std::env::current_dir().unwrap_or_default()],
        net: grants.net.clone(),
        clock: true,
        rand: true,
        declassify: true,
        secrets: grants.secrets.clone(),
        foreign_load: !grants.foreign_c.is_empty() || !grants.foreign_python.is_empty(),
        python_allowlist: grants.foreign_python.clone(),
    }
}

fn print_errors(diags: &[delulu_diag::Diagnostic], src: &str) {
    let mut map = SourceMap::new();
    map.add_file("<repl>", src);
    for d in diags.iter().filter(|d| d.is_error()) {
        eprint!("{}", render_human(d, &map));
    }
}
