use std::fs;
use std::io::ErrorKind;

use {Result, Error};
use docopt::Docopt;
use utils;
use package;

const USAGE: &'static str = "
Config a plugin.

Usage:
    pack config <plugin>
    pack config -h | --help

Options:
    -h, --help              Display this message
";

#[derive(Debug, RustcDecodable)]
struct ConfigArgs {
    arg_plugin: String,
}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "config".to_string()];
    argv.extend_from_slice(args);

    let args: ConfigArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());
    if let Err(e) = config_plugin(&args.arg_plugin) {
        die!("{}", e);
    }
}

fn config_plugin(name: &str) -> Result<()> {
    let packs = package::fetch()?;
    let pack = packs.iter().filter(|x| name == x.name).next().ok_or(Error::PluginNotInstalled)?;

    let path = pack.config_path();

    let modified = match fs::metadata(&path) {
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                None
            } else {
                return Err(Error::Io(e));
            }
        }
        Ok(meta) => Some(meta.modified()?),
    };

    utils::open_editor(&path)?;

    let meta = match fs::metadata(&path) {
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                return Ok(());
            } else {
                return Err(Error::Io(e));
            }
        }
        Ok(m) => m,
    };

    if meta.len() <= 0 {
        fs::remove_file(&path)?;
        if modified.is_some() {
            package::update_pack_plugin(&packs)?;
        }
    } else if modified.is_none() || meta.modified()? > modified.unwrap() {
        package::update_pack_plugin(&packs)?;
    }
    Ok(())
}
