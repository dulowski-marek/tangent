use crate::{traits::Writer, writable::Writable};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, io::ErrorKind, path::PathBuf};

const LOCKFILE_VERSION: u32 = 1;
const LOCKFILE_NAME: &str = ".tangent.lock";

#[derive(Serialize, Deserialize)]
struct Lockfile {
    version: u32,
    generated: Vec<String>,
}

pub struct FilesystemWriter {
    root_dir: String,
}

impl FilesystemWriter {
    pub fn new(root_dir: impl Into<String>) -> Self {
        Self {
            root_dir: root_dir.into(),
        }
    }

    fn file_path(&self, output: &Writable) -> PathBuf {
        PathBuf::from(&self.root_dir)
            .join(&output.path)
            .join(&output.filename)
    }

    fn dir(&self, output: &Writable) -> PathBuf {
        PathBuf::from(&self.root_dir).join(&output.path)
    }

    fn lockfile_path(&self) -> PathBuf {
        PathBuf::from(&self.root_dir).join(LOCKFILE_NAME)
    }

    fn previously_generated(&self) -> Result<HashSet<String>> {
        let path = self.lockfile_path();
        match fs::read_to_string(&path) {
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(HashSet::new()),
            Err(e) => Err(e.into()),
            Ok(data) => {
                let lf: Lockfile = serde_json::from_str(&data)
                    .with_context(|| format!("parsing {}", path.display()))?;
                Ok(lf.generated.into_iter().collect())
            }
        }
    }
}

impl Writer for FilesystemWriter {
    fn write(&self, outputs: Vec<Writable>) -> Result<Vec<Writable>> {
        let prev = self.previously_generated()?;

        let next: HashSet<String> = outputs
            .iter()
            .map(|o| self.file_path(o).to_string_lossy().into_owned())
            .collect();

        for file in &prev {
            if !next.contains(file) {
                if let Err(e) = fs::remove_file(file) {
                    if e.kind() != ErrorKind::NotFound {
                        return Err(e.into());
                    }
                }
            }
        }

        let mut written = Vec::new();
        for output in &outputs {
            let dir = self.dir(output);
            let file = self.file_path(output);
            fs::create_dir_all(&dir)?;
            fs::write(&file, &output.content)?;
            written.push(file.to_string_lossy().into_owned());
        }

        let lf = Lockfile {
            version: LOCKFILE_VERSION,
            generated: written,
        };
        let data = serde_json::to_string_pretty(&lf)?;
        fs::write(self.lockfile_path(), data)?;

        Ok(outputs)
    }
}
