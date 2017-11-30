use docopt::Docopt;
use Command;

const USAGE: &'static str = "
Help on each command.

Usage:
    pack help <command>
    pack help -h

Options:
    -h, --help      Display this message
";

#[derive(Debug, RustcDecodable)]
struct HelpArgs {
    arg_command: Command,
}

pub fn execute(args: &[String]) -> Command {
    let mut argv = vec!["pack".to_string(), "help".to_string()];
    argv.extend_from_slice(args);
    println!("{:?}", argv);

    let args: HelpArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());

    args.arg_command
}
