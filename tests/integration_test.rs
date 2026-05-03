use anyhow::Result;
use serde_json::Value;
use std::fs;
use tangent_core::{
    filesystem_reader::FilesystemReader,
    filesystem_writer::FilesystemWriter,
    generate::GenerateUsecase,
    traits::{Deserializer, Reader, Renderer, Writer},
    writable::Writable,
    yaml_deserializer::YamlDeserializer,
};

// --- test doubles ---

struct ConstReader(String);
impl Reader for ConstReader {
    fn read(&self) -> Result<String> {
        Ok(self.0.clone())
    }
}

struct JsonDeserializer;
impl Deserializer for JsonDeserializer {
    fn deserialize(&self, input: &str) -> Result<Value> {
        Ok(serde_json::from_str(input)?)
    }
}

struct FixedRenderer(Vec<Writable>);
impl Renderer for FixedRenderer {
    fn render(&self, _data: &Value) -> Result<Vec<Writable>> {
        Ok(self.0.clone())
    }
}

struct NullWriter;
impl Writer for NullWriter {
    fn write(&self, outputs: Vec<Writable>) -> Result<Vec<Writable>> {
        Ok(outputs)
    }
}

// --- FilesystemReader ---

#[test]
fn filesystem_reader_reads_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.txt");
    fs::write(&path, "hello world").unwrap();

    let reader = FilesystemReader::new(path.to_str().unwrap());
    assert_eq!(reader.read().unwrap(), "hello world");
}

#[test]
fn filesystem_reader_errors_on_missing_file() {
    let reader = FilesystemReader::new("/nonexistent/path/file.txt");
    assert!(reader.read().is_err());
}

// --- YamlDeserializer ---

#[test]
fn yaml_deserializer_parses_to_json_value() {
    let yaml = "name: alice\nage: 30\n";
    let value = YamlDeserializer.deserialize(yaml).unwrap();
    assert_eq!(value["name"], "alice");
    assert_eq!(value["age"], 30);
}

#[test]
fn yaml_deserializer_errors_on_invalid_input() {
    assert!(YamlDeserializer.deserialize(": : :").is_err());
}

// --- FilesystemWriter ---

#[test]
fn filesystem_writer_creates_files_and_lockfile() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_str().unwrap();

    let writer = FilesystemWriter::new(root);
    writer.write(vec![
        Writable { path: "a".into(), filename: "foo.ts".into(), content: "// foo".into() },
    ]).unwrap();

    assert_eq!(fs::read_to_string(dir.path().join("a/foo.ts")).unwrap(), "// foo");

    let lock: Value = serde_json::from_str(
        &fs::read_to_string(dir.path().join(".tangent.lock")).unwrap()
    ).unwrap();
    assert_eq!(lock["version"], 1);
    assert_eq!(lock["generated"].as_array().unwrap().len(), 1);
}

#[test]
fn filesystem_writer_deletes_stale_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_str().unwrap();
    let writer = FilesystemWriter::new(root);

    writer.write(vec![
        Writable { path: "".into(), filename: "old.ts".into(), content: "old".into() },
    ]).unwrap();
    assert!(dir.path().join("old.ts").exists());

    writer.write(vec![
        Writable { path: "".into(), filename: "new.ts".into(), content: "new".into() },
    ]).unwrap();
    assert!(!dir.path().join("old.ts").exists());
    assert!(dir.path().join("new.ts").exists());
}

#[test]
fn filesystem_writer_fails_on_corrupt_lockfile() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_str().unwrap();

    fs::write(dir.path().join(".tangent.lock"), "this is not json").unwrap();

    let writer = FilesystemWriter::new(root);
    let err = writer.write(vec![]).unwrap_err();
    assert!(err.to_string().contains("parsing"));
}

// --- GenerateUsecase ---

#[test]
fn generate_usecase_chains_reader_deserializer_renderer_writer() {
    let gen = GenerateUsecase::new(
        ConstReader(r#"{"key": "value"}"#.into()),
        JsonDeserializer,
        vec![Box::new(FixedRenderer(vec![
            Writable { path: "src".into(), filename: "Foo.ts".into(), content: "export class Foo {}".into() },
        ]))],
        NullWriter,
    );

    let outputs = gen.execute().unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].filename, "Foo.ts");
    assert_eq!(outputs[0].content, "export class Foo {}");
}
