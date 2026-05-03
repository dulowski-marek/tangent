use anyhow::Result;
use serde_json::{json, Value};
use std::fs;
use tangent_core::{
    config::{Config, ModuleConfig},
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
    writer
        .write(vec![Writable {
            path: "a".into(),
            filename: "foo.ts".into(),
            content: "// foo".into(),
        }])
        .unwrap();

    assert_eq!(
        fs::read_to_string(dir.path().join("a/foo.ts")).unwrap(),
        "// foo"
    );

    let lock: Value =
        serde_json::from_str(&fs::read_to_string(dir.path().join(".tangent.lock")).unwrap())
            .unwrap();
    assert_eq!(lock["version"], 1);
    assert_eq!(lock["generated"].as_array().unwrap().len(), 1);
}

#[test]
fn filesystem_writer_deletes_stale_files() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_str().unwrap();
    let writer = FilesystemWriter::new(root);

    writer
        .write(vec![Writable {
            path: "".into(),
            filename: "old.ts".into(),
            content: "old".into(),
        }])
        .unwrap();
    assert!(dir.path().join("old.ts").exists());

    writer
        .write(vec![Writable {
            path: "".into(),
            filename: "new.ts".into(),
            content: "new".into(),
        }])
        .unwrap();
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
        vec![Box::new(FixedRenderer(vec![Writable {
            path: "src".into(),
            filename: "Foo.ts".into(),
            content: "export class Foo {}".into(),
        }]))],
        NullWriter,
    );

    let outputs = gen.execute().unwrap();
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].filename, "Foo.ts");
    assert_eq!(outputs[0].content, "export class Foo {}");
}

// --- End-to-end: real WASM + tangent binary ---

#[test]
fn e2e_example_module_receives_config_and_writes_output() {
    let module_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/example-module");
    let wasm_path =
        format!("{module_dir}/target/wasm32-unknown-unknown/release/tangent_example.wasm");
    assert!(
        std::path::Path::new(&wasm_path).exists(),
        "Example WASM not built.\nRun `cargo build --manifest-path example-module/Cargo.toml --target wasm32-unknown-unknown` --release"
    );

    let dir = tempfile::tempdir().unwrap();
    let output_dir = dir.path().join("out");

    let config = Config {
        output: output_dir.to_str().unwrap().into(),
        modules: vec![ModuleConfig {
            path: wasm_path.clone(),
            config: [("greeting".into(), json!("hello-from-test"))].into_iter().collect(),
        }],
    };
    let config_path = dir.path().join("tangent.yaml");
    fs::write(&config_path, serde_yaml::to_string(&config).unwrap()).unwrap();

    let tangent_bin = env!("CARGO_BIN_EXE_tangent");
    let output = std::process::Command::new(tangent_bin)
        .arg("generate")
        .current_dir(dir.path())
        .output()
        .expect("failed to run tangent binary");

    assert!(
        output.status.success(),
        "tangent generate failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    // The example module echoes config into the file content
    let generated_file = output_dir.join("src/Generated.ts");
    assert!(generated_file.exists(), "Generated.ts was not created");
    let content = fs::read_to_string(&generated_file).unwrap();
    assert!(
        content.contains("hello-from-test"),
        "generated file should contain config value, got:\n{content}"
    );

    // Lockfile records the path
    let lock: Value =
        serde_json::from_str(&fs::read_to_string(output_dir.join(".tangent.lock")).unwrap())
            .unwrap();
    let generated = lock["generated"].as_array().unwrap();
    assert!(generated
        .iter()
        .any(|p| p.as_str().unwrap().ends_with("Generated.ts")));
}
