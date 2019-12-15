use clap::ArgMatches;
use std::env;
use std::io;

#[macro_use]
mod utils;

mod cli;
mod cmd;
mod echo;
mod error;
mod git;
mod package;
mod task;

pub use error::{Error, Result};

fn main() {
    let _ = env::var("PACK_LOG_FILE").and_then(|x| {
        simple_logging::log_to_file(x, log::LevelFilter::Info).expect("fail to init logging");
        Ok(())
    });

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
