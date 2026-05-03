use crate::writable::Writable;
use anyhow::Result;
use serde_json::Value;

pub trait Reader {
    fn read(&self) -> Result<String>;
}

pub trait Deserializer {
    fn deserialize(&self, input: &str) -> Result<Value>;
}

pub trait Renderer {
    fn render(&self, data: &Value) -> Result<Vec<Writable>>;
}

pub trait Writer {
    fn write(&self, outputs: Vec<Writable>) -> Result<Vec<Writable>>;
}
