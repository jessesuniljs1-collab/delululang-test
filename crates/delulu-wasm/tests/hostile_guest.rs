//! Hostile-guest tests (Stage 3 §9.4): the sandbox floor must hold against WebAssembly the DeluluLang
//! compiler did NOT produce. These hand-craft adversarial modules that import `delulu:cap` and try to
//! abuse it, and prove the embedded deny-by-default host refuses each — WITHOUT performing the effect,
//! reading out-of-bounds memory, or aborting the process. The whole thesis (no code exceeds its
//! granted authority) is only real if it survives a guest that isn't playing by the rules.

use delulu_wasm::{run_console_fn, WasmError};
use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, EntityType, ExportKind, ExportSection, Function,
    FunctionSection, ImportSection, Instruction, MemorySection, MemoryType, Module, TypeSection,
    ValType,
};

fn one_page() -> MemoryType {
    MemoryType { minimum: 1, maximum: None, memory64: false, shared: false, page_size_log2: None }
}

/// A module importing `delulu:cap.console_println` whose `attack()` calls it with the given
/// (handle, ptr). Optionally seeds a length-prefixed string into linear memory at offset 0.
fn attacker(handle: i32, ptr: i32, seed_string: Option<&str>) -> Vec<u8> {
    let mut types = TypeSection::new();
    // console_println(cap, ptr, file, start, end) — the effect host fns carry a span triple since
    // Stage 4 phase 4g (the host records the effect's `TraceRecord` with it).
    types.ty().function([ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32], []); // 0
    types.ty().function::<[ValType; 0], [ValType; 0]>([], []); // 1: attack()

    let mut imports = ImportSection::new();
    imports.import("delulu:cap", "console_println", EntityType::Function(0));

    let mut funcs = FunctionSection::new();
    funcs.function(1); // attack : type 1

    let mut mems = MemorySection::new();
    mems.memory(one_page());

    let mut exports = ExportSection::new();
    exports.export("memory", ExportKind::Memory, 0);
    exports.export("attack", ExportKind::Func, 1); // func 0 is the imported println

    let mut code = CodeSection::new();
    let mut f = Function::new([]);
    f.instruction(&Instruction::I32Const(handle));
    f.instruction(&Instruction::I32Const(ptr));
    f.instruction(&Instruction::I32Const(0)); // span file
    f.instruction(&Instruction::I32Const(0)); // span start
    f.instruction(&Instruction::I32Const(0)); // span end
    f.instruction(&Instruction::Call(0)); // console_println (import index 0)
    f.instruction(&Instruction::End);
    code.function(&f);

    let mut datas = DataSection::new();
    if let Some(s) = seed_string {
        let mut bytes = (s.len() as u32).to_le_bytes().to_vec();
        bytes.extend_from_slice(s.as_bytes());
        datas.active(0, &ConstExpr::i32_const(0), bytes);
    }

    let mut m = Module::new();
    m.section(&types);
    m.section(&imports);
    m.section(&funcs);
    m.section(&mems);
    m.section(&exports);
    m.section(&code);
    m.section(&datas);
    m.finish()
}

/// A module that imports a `delulu:cap` function the host does NOT provide (`fs_open`).
fn imports_unprovided_capability() -> Vec<u8> {
    let mut types = TypeSection::new();
    types.ty().function::<[ValType; 0], [ValType; 0]>([], []); // 0: () -> ()

    let mut imports = ImportSection::new();
    imports.import("delulu:cap", "fs_open", EntityType::Function(0)); // not in the host's world

    let mut funcs = FunctionSection::new();
    funcs.function(0); // attack : type 0

    let mut exports = ExportSection::new();
    exports.export("attack", ExportKind::Func, 1); // func 0 is the (missing) import

    let mut code = CodeSection::new();
    let mut f = Function::new([]);
    f.instruction(&Instruction::Call(0)); // call fs_open
    f.instruction(&Instruction::End);
    code.function(&f);

    let mut m = Module::new();
    m.section(&types);
    m.section(&imports);
    m.section(&funcs);
    m.section(&exports);
    m.section(&code);
    m.finish()
}

#[test]
fn forged_capability_handle_is_refused_dl0904() {
    // The cap table (run_console_fn) grants exactly one Console at handle 0. The guest forges
    // handle 999 — not in the table — so the host refuses and never performs the Write.
    let wasm = attacker(999, 0, None);
    let r = run_console_fn(&wasm, "attack", &[]);
    match r {
        Err(WasmError::Trap(msg)) => assert!(msg.contains("DL0904"), "expected a scope violation, got: {msg}"),
        other => panic!("a forged handle must be refused (DL0904), got {other:?}"),
    }
}

#[test]
fn out_of_bounds_string_pointer_is_refused_dl0903() {
    // A VALID handle (0) but a pointer past the end of the 64 KiB memory: the host must bounds-check
    // host-side and refuse, not read out of bounds or panic.
    let wasm = attacker(0, 100_000, None);
    let r = run_console_fn(&wasm, "attack", &[]);
    match r {
        Err(WasmError::Trap(msg)) => assert!(msg.contains("DL0903"), "expected out-of-bounds, got: {msg}"),
        other => panic!("an out-of-bounds pointer must be refused (DL0903), got {other:?}"),
    }
}

#[test]
fn hugely_negative_pointer_does_not_panic_the_host() {
    // ptr = -1 sign-extends to a giant address; the host must treat it as an unsigned wasm offset
    // and refuse cleanly rather than overflow `usize` and abort the process.
    let wasm = attacker(0, -1, None);
    let r = run_console_fn(&wasm, "attack", &[]);
    assert!(matches!(r, Err(WasmError::Trap(_))), "a -1 pointer must be refused, not crash: {r:?}");
}

#[test]
fn importing_an_unprovided_capability_fails_to_instantiate() {
    // Deny-by-default: the host's import world is a strict whitelist. A guest that imports
    // `delulu:cap.fs_open` (which the host does not provide) cannot even instantiate.
    let wasm = imports_unprovided_capability();
    let r = run_console_fn(&wasm, "attack", &[]);
    assert!(
        matches!(r, Err(WasmError::Instantiate(_))),
        "a guest importing an unprovided capability must fail to instantiate, got {r:?}"
    );
}

#[test]
fn well_formed_hand_crafted_guest_performs_the_effect() {
    // Positive control: the SAME runner, given a well-formed guest (valid handle 0, a valid string
    // seeded at ptr 0), DOES perform the Write. This proves the refusals above are real refusals,
    // not the harness failing everything indiscriminately.
    let wasm = attacker(0, 0, Some("ok"));
    let out = run_console_fn(&wasm, "attack", &[]).expect("a well-formed guest should run");
    assert_eq!(out, "ok\n");
}
