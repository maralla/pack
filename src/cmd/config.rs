use std::fs::{self, File};
use std::io::{Read, Write, ErrorKind};

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
    let packs = package::fetch().unwrap_or(vec![]);
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
            update_pack_plugin(&packs)?;
        }
    } else if modified.is_none() || meta.modified()? > modified.unwrap() {
        update_pack_plugin(&packs)?;
    }
    Ok(())
}

pub fn update_pack_plugin(packs: &[package::Package]) -> Result<()> {
    let mut f = File::create(&*package::PACK_PLUGIN_FILE)?;
    let mut buf = String::new();
    for (p, path) in packs.iter().map(|x| (x, x.config_path())) {
        buf.clear();
        if path.is_file() {
            File::open(&path)?.read_to_string(&mut buf)?;
            f.write_all(format!("\" {}\n{}\n", &p.name, &buf).as_bytes())?;
        }
    }
    Ok(())
}
