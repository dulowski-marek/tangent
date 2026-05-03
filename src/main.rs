const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let arg = std::env::args().nth(1);

    match arg.as_deref() {
        Some("-V") | Some("--version") => println!("tangent {VERSION}"),
        Some("-h") | Some("--help") | None => print_help(),
        Some(arg) => {
            eprintln!("error: unknown argument '{arg}'");
            eprintln!("Run 'tangent --help' for usage.");
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!("tangent {VERSION}");
    println!();
    println!("Usage: tangent [OPTIONS] <SPEC>");
    println!();
    println!("Arguments:");
    println!("  <SPEC>  Path to an OpenAPI spec file (YAML)");
    println!();
    println!("Options:");
    println!("  -h, --help     Print help");
    println!("  -V, --version  Print version");
}
