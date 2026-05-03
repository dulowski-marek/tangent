use crate::traits::Deserializer;
use anyhow::Result;
use serde_json::Value;

pub struct YamlDeserializer;

impl Deserializer for YamlDeserializer {
    fn deserialize(&self, input: &str) -> Result<Value> {
        let value: Value = serde_yaml::from_str(input)?;
        Ok(value)
    }
}
