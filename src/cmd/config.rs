use std::fs;
use std::io::ErrorKind;

use {Error, Result};
use clap::ArgMatches;
use utils;
use package;

#[derive(Debug)]
struct ConfigArgs {
    plugin: String,
    delete: bool,
}

impl ConfigArgs {
    fn from_matches(m: &ArgMatches) -> ConfigArgs {
        ConfigArgs {
            plugin: value_t!(m, "package", String).unwrap_or_default(),
            delete: m.is_present("delete"),
        }
    }
}

pub fn exec(matches: &ArgMatches) {
    let args = ConfigArgs::from_matches(matches);

    if let Err(e) = config_plugin(&args.plugin, args.delete) {
        die!("{}", e);
    }
}

fn config_plugin(name: &str, delete: bool) -> Result<()> {
    let packs = package::fetch()?;
    let temp_pack = package::Package::new(name, "temp", true);
    let pack = packs.iter().find(|x| name == x.name).unwrap_or(&temp_pack);

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

    if modified.is_some() && delete {
        fs::remove_file(&path)?;
        return Ok(());
    }

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

    if meta.len() == 0 {
        fs::remove_file(&path)?;
        if modified.is_some() {
            package::update_pack_plugin(&packs)?;
        }
    } else if modified.is_none() || meta.modified()? > modified.unwrap() {
        package::update_pack_plugin(&packs)?;
    }
    Ok(())
}
