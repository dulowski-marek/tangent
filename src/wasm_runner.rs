use crate::writable::Writable;
use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use wasmtime::{Engine, Instance, Linker, Module, Store};

pub struct WasmRunner {
    engine: Engine,
    module: Module,
}

impl WasmRunner {
    pub fn load(path: &str) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, path)
            .map_err(|e| anyhow!("loading WASM module {path}: {e}"))?;
        Ok(Self { engine, module })
    }

    pub fn execute(&self, config: &Value) -> Result<Vec<Writable>> {
        let config_json = serde_json::to_string(config)?;
        let config_bytes = config_json.as_bytes();

        let mut store = Store::new(&self.engine, ());
        let linker = Linker::<()>::new(&self.engine);
        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| anyhow!("instantiating WASM module: {e}"))?;

        let memory = get_memory(&instance, &mut store)?;

        memory
            .write(&mut store, 0, config_bytes)
            .map_err(|e| anyhow!("writing config to WASM memory: {e}"))?;

        let render = instance
            .get_typed_func::<(i32, i32), ()>(&mut store, "render")
            .map_err(|e| anyhow!("WASM module must export 'render(i32, i32)': {e}"))?;
        render
            .call(&mut store, (0, config_bytes.len() as i32))
            .map_err(|e| anyhow!("calling render: {e}"))?;

        let result_ptr = instance
            .get_typed_func::<(), i32>(&mut store, "result_ptr")
            .map_err(|e| anyhow!("WASM module must export 'result_ptr() -> i32': {e}"))?
            .call(&mut store, ())
            .map_err(|e| anyhow!("calling result_ptr: {e}"))?;

        let result_len = instance
            .get_typed_func::<(), i32>(&mut store, "result_len")
            .map_err(|e| anyhow!("WASM module must export 'result_len() -> i32': {e}"))?
            .call(&mut store, ())
            .map_err(|e| anyhow!("calling result_len: {e}"))?;

        if result_len == 0 {
            return Ok(Vec::new());
        }

        let mut buf = vec![0u8; result_len as usize];
        memory
            .read(&store, result_ptr as usize, &mut buf)
            .map_err(|e| anyhow!("reading result from WASM memory: {e}"))?;

        serde_json::from_slice(&buf).context("parsing WASM render result")
    }
}

fn get_memory(instance: &Instance, store: &mut Store<()>) -> Result<wasmtime::Memory> {
    instance
        .get_export(store, "memory")
        .and_then(|e| e.into_memory())
        .context("WASM module must export 'memory'")
}
