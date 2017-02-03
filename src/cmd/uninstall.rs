use std::fs;

use Result;
use package::{self, Package};
use docopt::Docopt;

const USAGE: &'static str = "
Uninstall plugins.

Usage:
    pack uninstall <plugin>...
    pack uninstall -h | --help

Options:
    -h, --help              Display this message
";

#[derive(Debug, RustcDecodable)]
struct UninstallArgs {
    arg_plugin: Vec<String>,
}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "uninstall".to_string()];
    argv.extend_from_slice(args);

    let args: UninstallArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());

    if let Err(e) = uninstall_plugins(args.arg_plugin) {
        die!("{}", e);
    }
}

fn uninstall_plugins(plugins: Vec<String>) -> Result<()> {
    let mut packs = package::fetch()?;

    for pack in packs.iter().filter(|p| plugins.contains(&p.name)) {
        uninstall_plugin(&pack)?;
    }

    packs.retain(|x| !plugins.contains(&x.name));
    packs.sort_by(|a, b| a.name.cmp(&b.name));
    package::update_pack_plugin(&packs)?;
    package::save(packs)?;
    Ok(())
}

fn uninstall_plugin(plugin: &Package) -> Result<()> {
    let config_file = plugin.config_path();
    let plugin_path = plugin.path();

    if config_file.is_file() {
        fs::remove_file(&config_file)?;
    }

    if plugin_path.is_dir() {
        fs::remove_dir_all(&plugin_path)?;
    }

    Ok(())
}
