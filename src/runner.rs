use anyhow::Result;
use crate::{config::Config, filesystem_writer::FilesystemWriter, traits::Writer, wasm_runner::WasmRunner};

pub fn run(config: Config) -> Result<()> {
    let writer = FilesystemWriter::new(&config.output);

    let mut outputs = Vec::new();
    for module in &config.modules {
        let runner = WasmRunner::load(&module.path)?;
        let module_config = serde_json::to_value(&module.config)?;
        outputs.extend(runner.execute(&module_config)?);
    }

    writer.write(outputs)?;
    Ok(())
}
