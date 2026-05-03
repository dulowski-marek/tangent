use crate::{
    config::Config, filesystem_writer::FilesystemWriter, traits::Writer, wasm_runner::WasmRunner,
};
use anyhow::Result;
use wasmtime::Engine;

pub fn run(config: Config) -> Result<()> {
    let engine = Engine::default();
    let writer = FilesystemWriter::new(&config.output);

    let mut outputs = Vec::new();
    for module in &config.modules {
        let runner = WasmRunner::load(engine.clone(), &module.path)?;
        let module_config = serde_json::to_value(&module.config)?;
        outputs.extend(runner.execute(&module_config)?);
    }

    writer.write(outputs)?;
    Ok(())
}
