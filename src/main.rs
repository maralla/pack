#[macro_use]
extern crate lazy_static;

extern crate chan;
#[macro_use]
extern crate clap;
extern crate git2;
extern crate num_cpus;
extern crate termion;
extern crate walkdir;
extern crate yaml_rust;
extern crate dirs;

use std::io;
use clap::ArgMatches;

#[macro_use]
mod utils;

mod cli;
mod cmd;
mod error;
mod package;
mod git;
mod task;
mod echo;

pub use error::{Error, Result};
fn main() {
    let app_m = cli::build_cli().get_matches();

    match app_m.subcommand() {
        ("list", Some(m)) => cmd::list::exec(m),
        ("install", Some(m)) => cmd::install::exec(m),
        ("uninstall", Some(m)) => cmd::uninstall::exec(m),
        ("config", Some(m)) => cmd::config::exec(m),
        ("move", Some(m)) => cmd::move_cmd::exec(m),
        ("update", Some(m)) => cmd::update::exec(m),
        ("generate", Some(m)) => cmd::generate::exec(m),
        ("completions", Some(m)) => {
            let shell = m.value_of("SHELL").unwrap();
            cli::build_cli().gen_completions_to("pack", shell.parse().unwrap(), &mut io::stdout());
        }
        _ => cmd::list::exec(&ArgMatches::default()),
    }
}
