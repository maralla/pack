#[macro_use]
extern crate lazy_static;

extern crate rustc_serialize;
extern crate docopt;
extern crate yaml_rust;
extern crate git2;
extern crate termion;
extern crate walkdir;
extern crate num_cpus;
extern crate chan;

#[macro_use]
mod utils;

mod cmd;
mod error;
mod package;
mod git;
mod task;
mod echo;

pub use error::{Result, Error};
use docopt::Docopt;

const USAGE: &str = "
Usage:
    pack <command> [<args>...]
    pack [options]

Commands:
    help
    list
    install
    uninstall
    config
    move
    update
    version

Options:
    -h, --help      Display this message

See pack help <command> for help on each command.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_command: Option<Command>,
    arg_args: Vec<String>,
}

#[derive(Debug, RustcDecodable)]
pub enum Command {
    Version,
    Help,
    List,
    Install,
    Uninstall,
    Config,
    Move,
    Update,
}

fn execute(cmd: &Command, argv: &[String]) {
    match *cmd {
        Command::Version => cmd::version::execute(argv),
        Command::Help => {
            let cmd = cmd::help::execute(argv);
            let args = vec!["-h".to_string()];
            execute(&cmd, &args);
        }
        Command::List => cmd::list::execute(argv),
        Command::Install => cmd::install::execute(argv),
        Command::Uninstall => cmd::uninstall::execute(argv),
        Command::Config => cmd::config::execute(argv),
        Command::Move => cmd::move_cmd::execute(argv),
        Command::Update => cmd::update::execute(argv),
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.options_first(true).decode())
        .unwrap_or_else(|e| e.exit());

    match args.arg_command {
        Some(ref c) => execute(c, &args.arg_args),
        _ => println!("{}", USAGE)
    }
}
