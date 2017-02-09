use {Error, Result};
use package::{self, Package};
use num_cpus;
use docopt::Docopt;
use git;
use task::TaskManager;

const USAGE: &'static str = "
Update plugins.

Usage:
    pack update
    pack update <plugin>... [options]
    pack update -h | --help

Options:
    -j, --threads THREADS   Update plugins concurrently
    -h, --help              Display this message
";

#[derive(Debug, RustcDecodable)]
struct UpdateArgs {
    arg_plugin: Vec<String>,
    flag_threads: Option<usize>,
}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "update".to_string()];
    argv.extend_from_slice(args);

    let args: UpdateArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());

    let threads = args.flag_threads.unwrap_or(num_cpus::get());
    if threads < 1 {
        die!("Threads should be greater than 0");
    }
    if let Err(e) = update_plugins(args.arg_plugin, threads) {
        die!("Err: {}", e);
    }
}

fn update_plugins(plugins: Vec<String>, threads: usize) -> Result<()> {
    let packs = package::fetch()?;

    let mut manager = TaskManager::new(threads);
    if plugins.is_empty() {
        for pack in packs.iter() {
            manager.add(pack.clone());
        }
    } else {
        for pack in packs.iter().filter(|x| plugins.contains(&x.name)) {
            manager.add(pack.clone());
        }
    }
    manager.run(update_plugin);
    Ok(())
}

fn update_plugin(pack: &Package) -> Result<()> {
    let path = pack.path();
    if !path.is_dir() {
        Err(Error::PluginNotInstalled)
    } else if pack.local {
        Err(Error::SkipLocal)
    } else {
        git::update(&pack.name, &path)
    }
}
