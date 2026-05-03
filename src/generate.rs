use anyhow::Result;
use crate::{
    traits::{Deserializer, Reader, Renderer, Writer},
    writable::Writable,
};

pub struct GenerateUsecase<R, D, W> {
    reader: R,
    deserializer: D,
    renderers: Vec<Box<dyn Renderer>>,
    writer: W,
}

impl<R: Reader, D: Deserializer, W: Writer> GenerateUsecase<R, D, W> {
    pub fn new(reader: R, deserializer: D, renderers: Vec<Box<dyn Renderer>>, writer: W) -> Self {
        Self { reader, deserializer, renderers, writer }
    }

    pub fn execute(&self) -> Result<Vec<Writable>> {
        let raw = self.reader.read()?;
        let data = self.deserializer.deserialize(&raw)?;

        let mut outputs = Vec::new();
        for renderer in &self.renderers {
            outputs.extend(renderer.render(&data)?);
        }

        self.writer.write(outputs)
    }
}
