#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Writable {
    pub path: String,
    pub filename: String,
    pub content: String,
}
