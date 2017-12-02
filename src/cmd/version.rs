use docopt::Docopt;

const USAGE: &'static str = "
Show version number.

Usage:
    pack version
    pack version -h | --help

Options:
    -h, --help      Display this message
";

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug, RustcDecodable)]
struct VersionArgs {}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "version".to_string()];
    argv.extend_from_slice(args);
    let _args: VersionArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());

    println!("{}", VERSION);

}
