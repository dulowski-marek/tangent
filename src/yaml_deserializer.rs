use anyhow::Result;
use serde_json::Value;
use crate::traits::Deserializer;

pub struct YamlDeserializer;

impl Deserializer for YamlDeserializer {
    fn deserialize(&self, input: &str) -> Result<Value> {
        let value: Value = serde_yaml::from_str(input)?;
        Ok(value)
    }
}
