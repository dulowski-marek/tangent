use anyhow::Result;
use std::fs;
use crate::traits::Reader;

pub struct FilesystemReader {
    path: String,
}

impl FilesystemReader {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl Reader for FilesystemReader {
    fn read(&self) -> Result<String> {
        Ok(fs::read_to_string(&self.path)?)
    }
}
