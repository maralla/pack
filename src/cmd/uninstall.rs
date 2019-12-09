use crate::package::{self, Package};
use crate::Result;

use clap::ArgMatches;
use std::fs;

#[derive(Debug)]
struct UninstallArgs {
    plugins: Vec<String>,
    all: bool,
}

impl UninstallArgs {
    fn from_matches(m: &ArgMatches) -> UninstallArgs {
        UninstallArgs {
            plugins: m.values_of_lossy("package").unwrap_or_else(|| vec![]),
            all: m.is_present("all"),
        }
    }
}

pub fn exec(matches: &ArgMatches) {
    let args = UninstallArgs::from_matches(matches);

    if let Err(e) = uninstall_plugins(&args.plugins, args.all) {
        die!("{}", e);
    }
}

fn uninstall_plugins(plugins: &[String], all: bool) -> Result<()> {
    let mut packs = package::fetch()?;

    for pack in packs.iter().filter(|p| plugins.contains(&p.name)) {
        uninstall_plugin(pack, all)?;
    }

    packs.retain(|x| !plugins.contains(&x.name));
    packs.sort_by(|a, b| a.name.cmp(&b.name));
    package::update_pack_plugin(&packs)?;
    package::save(packs)?;
    Ok(())
}

fn uninstall_plugin(plugin: &Package, all: bool) -> Result<()> {
    let config_file = plugin.config_path();
    let plugin_path = plugin.path();

    if config_file.is_file() && all {
        fs::remove_file(&config_file)?;
    }

    if plugin_path.is_dir() {
        fs::remove_dir_all(&plugin_path)?;
    }

    Ok(())
}
