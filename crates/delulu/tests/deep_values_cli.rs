//! DATA-DEPTH-1 and DATA-DEPTH-2 (RW 4.74, 4.75), end to end: a value a loop builds is as deep as memory allows, and
//! `MAX_DEPTH` — which bounds the program's own recursion — says nothing about it. Printed with `str`, a linked list of
//! 300,000 records killed the CLI (`thread 'delulu-main' has overflowed its stack`, its 512 MiB stack included), and a
//! chain of 300,000 links sent to an actor did the same. Both now run to their answer, inside the default budget.

use std::path::PathBuf;
use std::process::{Command, Output};

fn delulu(args: &[&str], dir: &std::path::Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_delulu"))
        .current_dir(dir)
        .env("DELULU_NO_FIRST_RUN", "1")
        .args(args)
        .output()
        .expect("run delulu")
}

fn scratch(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("delulu-deep-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run(tag: &str, src: &str) -> String {
    let dir = scratch(tag);
    std::fs::write(dir.join("m.delulu"), src).unwrap();
    let o = delulu(&["run", "m.delulu", "--grant", "console"], &dir);
    let _ = std::fs::remove_dir_all(&dir);
    assert!(o.status.success(), "exit {:?}, stderr: {}", o.status.code(), String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn str_of_a_list_a_loop_built_300000_deep_prints_it() {
    let src = "module m\n\
        type Node {\n  v: Int,\n  next: Option[ref Node],\n}\n\
        fn main(root: Root) ! {Write} {\n\
        \x20 let out = root.console()\n\
        \x20 var head = Node { v: 0, next: None }\n\
        \x20 var i = 1\n\
        \x20 while i < 300000 {\n\
        \x20   head = Node { v: i, next: Some(head) }\n\
        \x20   i = i + 1\n\
        \x20 }\n\
        \x20 out.println(str(str(head).len()))\n\
        }\n";
    assert_eq!(run("str", src), "9488888\n");
}

#[test]
fn a_chain_a_loop_built_300000_deep_crosses_to_an_actor() {
    let src = "module m\n\
        type Chain = Nil | Link(Int, Chain)\n\
        fn links(c: Chain) -> Int {\n\
        \x20 var n = 0\n\
        \x20 var cur = c\n\
        \x20 var going = true\n\
        \x20 while going {\n\
        \x20   match cur {\n\
        \x20     Link(_, rest) => {\n\
        \x20       n = n + 1\n\
        \x20       cur = rest\n\
        \x20     }\n\
        \x20     Nil => { going = false }\n\
        \x20   }\n\
        \x20 }\n\
        \x20 n\n\
        }\n\
        actor Sink {\n\
        \x20 var got: Int\n\
        \x20 new() {\n    self.got = 0\n  }\n\
        \x20 be take(c: Chain, out: Cap[Console]) ! {Write} {\n    out.println(str(links(c)))\n  }\n\
        }\n\
        fn main(root: Root) ! {Async, Write} {\n\
        \x20 var c = Nil\n\
        \x20 var i = 0\n\
        \x20 while i < 300000 {\n\
        \x20   c = Link(i, c)\n\
        \x20   i = i + 1\n\
        \x20 }\n\
        \x20 let s = spawn Sink()\n\
        \x20 s.take(c, root.console())\n\
        }\n";
    assert_eq!(run("actor", src), "300000\n");
}
