use crate::package;
use crate::utils;
use crate::{Error, Result};

use clap::{value_t, ArgMatches};
use std::fs;

#[derive(Debug)]
struct MoveArgs {
    plugin: String,
    category: String,
    opt: bool,
}

impl MoveArgs {
    fn from_matches(m: &ArgMatches) -> MoveArgs {
        MoveArgs {
            plugin: value_t!(m, "package", String).unwrap_or_default(),
            category: value_t!(m, "category", String).unwrap_or_default(),
            opt: m.is_present("opt"),
        }
    }
}

pub fn exec(matches: &ArgMatches) {
    let args = MoveArgs::from_matches(matches);

    if let Err(e) = move_plugin(&args.plugin, &args.category, args.opt) {
        die!("{}", e);
    }
}

fn move_plugin(plugin: &str, category: &str, opt: bool) -> Result<()> {
    let mut packs = package::fetch()?;
    let changed = {
        let pack = match packs.iter_mut().find(|p| p.name == plugin) {
            Some(p) => p,
            None => return Err(Error::PluginNotInstalled),
        };

        let origin_path = pack.path();
        if !origin_path.is_dir() {
            return Err(Error::PluginNotInstalled);
        }

        let path = package::Package::new(plugin, category, opt).path();
        if origin_path != path {
            utils::copy_directory(&origin_path, &path)?;
            fs::remove_dir_all(&origin_path)?;
            pack.set_category(category as &str);
            pack.set_opt(opt);
            true
        } else {
            false
        }
    };

    if changed {
        packs.sort_by(|a, b| a.name.cmp(&b.name));
        package::save(packs)?;
    }
    Ok(())
}
