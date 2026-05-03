use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub output: String,
    #[serde(default)]
    pub modules: Vec<ModuleConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ModuleConfig {
    pub path: String,
    #[serde(default)]
    pub config: HashMap<String, Value>,
}
