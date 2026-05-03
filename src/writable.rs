#[derive(Debug, Clone, serde::Serialize)]
pub struct Writable {
    pub path: String,
    pub filename: String,
    pub content: String,
}
