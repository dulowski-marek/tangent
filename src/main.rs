use std::{fs, path::Path, process};
use tangent_core::{config::Config, runner};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const CONFIG_FILENAME: &str = "tangent.yaml";

const CONFIG_TEMPLATE: &str = "# Tangent configuration

# Output directory for generated files
output: src/generated

# Modules to run
modules: []
  # - path: ./modules/my-module.wasm
  #   config:
  #     key: value
";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("-V") | Some("--version") => println!("tangent {VERSION}"),
        Some("-h") | Some("--help") | None => print_help(),
        Some("init") => cmd_init(),
        Some("generate") => cmd_generate(),
        Some(arg) => {
            eprintln!("error: unknown command '{arg}'");
            eprintln!("Run 'tangent --help' for usage.");
            process::exit(1);
        }
    }
}

fn cmd_init() {
    if Path::new(CONFIG_FILENAME).exists() {
        eprintln!("error: {CONFIG_FILENAME} already exists");
        process::exit(1);
    }
    if let Err(e) = fs::write(CONFIG_FILENAME, CONFIG_TEMPLATE) {
        eprintln!("error: {e}");
        process::exit(1);
    }
    println!("Created {CONFIG_FILENAME}");
}

fn cmd_generate() {
    let config_str = match fs::read_to_string(CONFIG_FILENAME) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error reading {CONFIG_FILENAME}: {e}");
            process::exit(1);
        }
    };
    let config: Config = match serde_yaml::from_str(&config_str) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error parsing {CONFIG_FILENAME}: {e}");
            process::exit(1);
        }
    };
    if let Err(e) = runner::run(config) {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn print_help() {
    println!("tangent {VERSION}");
    println!();
    println!("Usage: tangent [OPTIONS] <COMMAND>");
    println!();
    println!("Commands:");
    println!("  init      Create a tangent.yaml config file in the current directory");
    println!("  generate  Run all configured modules and write output");
    println!();
    println!("Options:");
    println!("  -h, --help     Print help");
    println!("  -V, --version  Print version");
}
