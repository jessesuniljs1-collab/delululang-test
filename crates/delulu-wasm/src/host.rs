//! The embedded Wasmtime host (Phase 3a). Pure functions have no imports, so instantiation grants
//! the guest ZERO ambient authority — the deny-by-default posture the sandbox floor is built on
//! (§4). Later phases add the `delulu:cap` import world; the guest never gets an OS handle.

use wasmtime::{Engine, Instance, Module, Store, Val};

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

/// Run an exported pure function of `wasm` with i64 arguments and return its i64 (or i32-as-i64)
/// result, under an empty import set (no ambient authority).
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
