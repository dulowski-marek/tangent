use crate::writable::Writable;
use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use wasmtime::{Engine, Instance, Linker, Module, Store};

const TIMEOUT_SECS: u64 = 60;

pub struct WasmRunner {
    engine: Engine,
    module: Module,
}

impl WasmRunner {
    pub fn load(engine: Engine, path: &str) -> Result<Self> {
        let module = Module::from_file(&engine, path)
            .map_err(|e| anyhow!("loading WASM module {path}: {e}"))?;
        Ok(Self { engine, module })
    }

    pub fn execute(&self, config: &Value) -> Result<Vec<Writable>> {
        let config_json = serde_json::to_string(config)?;
        let config_bytes = config_json.as_bytes();
        let config_len =
            i32::try_from(config_bytes.len()).map_err(|_| anyhow!("config JSON exceeds 2 GB"))?;

        let done = Arc::new(AtomicBool::new(false));
        let ticker = {
            let done = done.clone();
            let engine = self.engine.clone();
            std::thread::spawn(move || {
                for _ in 0..=TIMEOUT_SECS {
                    if done.load(Ordering::Relaxed) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    engine.increment_epoch();
                }
            })
        };

        let result = self.run(config_bytes, config_len);

        done.store(true, Ordering::Relaxed);
        let _ = ticker.join();

        result
    }

    fn run(&self, config_bytes: &[u8], config_len: i32) -> Result<Vec<Writable>> {
        let mut store = Store::new(&self.engine, ());
        store.set_epoch_deadline(TIMEOUT_SECS);
        store.epoch_deadline_trap();

        let linker = Linker::<()>::new(&self.engine);
        let instance = linker
            .instantiate(&mut store, &self.module)
            .map_err(|e| anyhow!("instantiating WASM module: {e}"))?;

        let memory = get_memory(&instance, &mut store)?;

        let config_buf_ptr = instance
            .get_typed_func::<(), i32>(&mut store, "config_ptr")
            .map_err(|e| anyhow!("WASM module must export 'config_ptr() -> i32': {e}"))?
            .call(&mut store, ())
            .map_err(|e| anyhow!("calling config_ptr: {e}"))?;
        let config_offset = usize::try_from(config_buf_ptr)
            .map_err(|_| anyhow!("WASM returned negative config_ptr: {config_buf_ptr}"))?;

        memory
            .write(&mut store, config_offset, config_bytes)
            .map_err(|e| anyhow!("writing config to WASM memory: {e}"))?;

        let render = instance
            .get_typed_func::<(i32, i32), ()>(&mut store, "render")
            .map_err(|e| anyhow!("WASM module must export 'render(i32, i32)': {e}"))?;
        render
            .call(&mut store, (config_buf_ptr, config_len))
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

        let ptr = usize::try_from(result_ptr)
            .map_err(|_| anyhow!("WASM returned negative result_ptr: {result_ptr}"))?;
        let len = usize::try_from(result_len)
            .map_err(|_| anyhow!("WASM returned negative result_len: {result_len}"))?;

        let mem_size = memory.data_size(&store);
        if ptr.saturating_add(len) > mem_size {
            return Err(anyhow!(
                "WASM result [{ptr}..{}] exceeds memory size {mem_size}",
                ptr + len
            ));
        }

        let mut buf = vec![0u8; len];
        memory
            .read(&store, ptr, &mut buf)
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
