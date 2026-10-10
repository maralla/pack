use clap::ArgMatches;
use clap_complete::Shell;
use log::{LevelFilter, Log, Metadata, Record};
use std::env;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::sync::Mutex;

struct FileLogger(Mutex<io::BufWriter<std::fs::File>>);

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut w = self.0.lock().unwrap();
            let _ = writeln!(w, "{} [{}] {}", timestamp(), record.level(), record.args());
            let _ = w.flush();
        }
    }

    fn flush(&self) {
        let _ = self.0.lock().unwrap().flush();
    }
}

fn timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default()
}

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
        let f = OpenOptions::new().create(true).append(true).open(&x);
        match f {
            Ok(f) => {
                let logger = FileLogger(Mutex::new(io::BufWriter::new(f)));
                let _ = log::set_boxed_logger(Box::new(logger));
                log::set_max_level(LevelFilter::Info);
            }
            Err(e) => eprintln!("fail to init logging: {}", e),
        }
        Ok(())
    });

    let app_m = cli::build_cli().get_matches();

    match app_m.subcommand() {
        Some(("list", m)) => cmd::list::exec(m),
        Some(("install", m)) => cmd::install::exec(m),
        Some(("uninstall", m)) => cmd::uninstall::exec(m),
        Some(("config", m)) => cmd::config::exec(m),
        Some(("move", m)) => cmd::move_cmd::exec(m),
        Some(("update", m)) => cmd::update::exec(m),
        Some(("generate", m)) => cmd::generate::exec(m),
        Some(("completions", m)) => {
            let shell = m.get_one::<String>("SHELL").unwrap();
            let shell: Shell = shell.parse().unwrap();
            clap_complete::generate(shell, &mut cli::build_cli(), "pack", &mut io::stdout());
        }
        _ => cmd::list::exec(&ArgMatches::default()),
    }
}
