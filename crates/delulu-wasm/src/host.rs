//! The embedded Wasmtime host (Phase 3a/3b).
//!
//! Pure functions (Phase 3a) instantiate with an EMPTY import set — zero ambient authority.
//! Effectful functions (Phase 3b) get exactly the `delulu:cap` host functions their grant allows;
//! the host performs the effect and its scope check, and reads string bytes out of the guest's
//! exported linear memory. The guest never receives an OS handle — capabilities are opaque i32
//! handles into the host's cap table (§4).

use wasmtime::{Caller, Engine, Instance, Linker, Module, Store, Val};

#[derive(Clone, Debug)]
pub enum WasmError {
    Module(String),
    Instantiate(String),
    NoExport(String),
    Trap(String),
    BadResult,
}

impl WasmError {
    pub fn message(&self) -> String {
        match self {
            WasmError::Module(e) => format!("invalid WASM module: {e}"),
            WasmError::Instantiate(e) => format!("instantiation failed: {e}"),
            WasmError::NoExport(n) => format!("no exported function `{n}`"),
            WasmError::Trap(e) => format!("WASM trap: {e}"),
            WasmError::BadResult => "unexpected result type from WASM".to_string(),
        }
    }
}

/// Run an exported PURE function with i64 arguments (no imports, no ambient authority).
pub fn run_int_fn(wasm: &[u8], name: &str, args: &[i64]) -> Result<i64, WasmError> {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).map_err(|e| WasmError::Module(e.to_string()))?;
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).map_err(|e| WasmError::Instantiate(e.to_string()))?;
    let func = instance.get_func(&mut store, name).ok_or_else(|| WasmError::NoExport(name.to_string()))?;
    let params: Vec<Val> = args.iter().map(|&a| Val::I64(a)).collect();
    let mut results = vec![Val::I64(0)];
    func.call(&mut store, &params, &mut results).map_err(|e| WasmError::Trap(e.to_string()))?;
    match results.first() {
        Some(Val::I64(n)) => Ok(*n),
        Some(Val::I32(n)) => Ok(*n as i64),
        _ => Err(WasmError::BadResult),
    }
}

/// The host's capability table. A handle is an index into `caps`; index 0 is conventionally the
/// root (for `main`) or a directly-granted Console (for a bare cap function).
#[derive(Clone, Copy, Debug)]
pub enum CapKind {
    Root,
    Console,
}

struct HostState {
    caps: Vec<CapKind>,
    /// Whether the human/broker granted console output (the run-time authority grant).
    console_granted: bool,
    output: String,
    /// Set host-side when a capability check fails. We record it and return without trapping (an
    /// error returned across the wasm frame aborts on some platforms); the runner turns a set flag
    /// into a clean `WasmError` after the call.
    refused: Option<String>,
}

/// Build the `delulu:cap` host import world: `root_console` mints a Console handle from the root
/// (host-side grant check), and `console_println` performs the Write and reads the string from the
/// guest's exported memory. Neither ever traps from inside the callback.
fn build_linker(engine: &Engine) -> Result<Linker<HostState>, WasmError> {
    let mut linker = Linker::new(engine);
    linker
        .func_wrap("delulu:cap", "root_console", |mut caller: Caller<'_, HostState>, root: i32| -> i32 {
            if caller.data().refused.is_some() {
                return -1; // a prior refusal already poisoned this run; don't overwrite its cause
            }
            let is_root = caller.data().caps.get(root as usize).map(|c| matches!(c, CapKind::Root)).unwrap_or(false);
            if !is_root {
                caller.data_mut().refused = Some(format!("root handle {root} is not the root capability"));
                return -1;
            }
            if !caller.data().console_granted {
                caller.data_mut().refused = Some("DL0703: console was not granted to this program".into());
                return -1;
            }
            let st = caller.data_mut();
            st.caps.push(CapKind::Console);
            (st.caps.len() - 1) as i32
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    linker
        .func_wrap("delulu:cap", "console_println", |mut caller: Caller<'_, HostState>, cap: i32, ptr: i32| {
            if caller.data().refused.is_some() {
                return; // a prior refusal stands as the root cause; don't clobber it with a use-site error
            }
            let ok = caller.data().caps.get(cap as usize).map(|c| matches!(c, CapKind::Console)).unwrap_or(false);
            if !ok {
                caller.data_mut().refused = Some(format!("DL0904: handle {cap} is not a granted Console capability"));
                return;
            }
            let Some(mem) = caller.get_export("memory").and_then(|e| e.into_memory()) else {
                caller.data_mut().refused = Some("guest exports no `memory`".into());
                return;
            };
            let s = {
                let data = mem.data(&caller);
                let p = ptr as usize;
                if p + 4 > data.len() {
                    caller.data_mut().refused = Some("string header out of bounds".into());
                    return;
                }
                let len = u32::from_le_bytes([data[p], data[p + 1], data[p + 2], data[p + 3]]) as usize;
                if p + 4 + len > data.len() {
                    caller.data_mut().refused = Some("string body out of bounds".into());
                    return;
                }
                String::from_utf8_lossy(&data[p + 4..p + 4 + len]).to_string()
            };
            let out = &mut caller.data_mut().output;
            out.push_str(&s);
            out.push('\n');
        })
        .map_err(|e| WasmError::Instantiate(e.to_string()))?;
    Ok(linker)
}

fn finish(mut store: Store<HostState>, func: wasmtime::Func, params: &[Val]) -> Result<String, WasmError> {
    let mut results: [Val; 0] = [];
    func.call(&mut store, params, &mut results).map_err(|e| WasmError::Trap(e.to_string()))?;
    let state = store.into_data();
    if let Some(reason) = state.refused {
        return Err(WasmError::Trap(reason));
    }
    Ok(state.output)
}

/// Run an exported function that takes Console-capability handles directly (index 0 = a granted
/// Console). Returns the captured console output.
pub fn run_console_fn(wasm: &[u8], name: &str, cap_handles: &[usize]) -> Result<String, WasmError> {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).map_err(|e| WasmError::Module(e.to_string()))?;
    let state = HostState { caps: vec![CapKind::Console], console_granted: true, output: String::new(), refused: None };
    let mut store = Store::new(&engine, state);
    let linker = build_linker(&engine)?;
    let instance = linker.instantiate(&mut store, &module).map_err(|e| WasmError::Instantiate(e.to_string()))?;
    let func = instance.get_func(&mut store, name).ok_or_else(|| WasmError::NoExport(name.to_string()))?;
    let params: Vec<Val> = cap_handles.iter().map(|&h| Val::I32(h as i32)).collect();
    finish(store, func, &params)
}

/// Run `main(root: Root)` under Wasmtime with the given console grant. The root handle (index 0) is
/// passed in; `root.console()` mints a Console handle host-side iff `console_granted`. Returns the
/// captured console output (or a `WasmError` if a capability was refused).
pub fn run_main_console(wasm: &[u8], console_granted: bool) -> Result<String, WasmError> {
    let engine = Engine::default();
    let module = Module::new(&engine, wasm).map_err(|e| WasmError::Module(e.to_string()))?;
    let state = HostState { caps: vec![CapKind::Root], console_granted, output: String::new(), refused: None };
    let mut store = Store::new(&engine, state);
    let linker = build_linker(&engine)?;
    let instance = linker.instantiate(&mut store, &module).map_err(|e| WasmError::Instantiate(e.to_string()))?;
    let func = instance.get_func(&mut store, "main").ok_or_else(|| WasmError::NoExport("main".to_string()))?;
    finish(store, func, &[Val::I32(0)]) // root handle
}
