use std::fs;

use {Result, Error};
use package;
use utils;
use docopt::Docopt;

const USAGE: &str = "
Move a plugin to a different category.

Usage:
    pack move <plugin> <category> [-o | --opt]
    pack move -h | --help

Options:
    -o, --opt         Move this plugin as opt
    -h, --help        Display this message
";

#[derive(Debug, RustcDecodable)]
struct MoveArgs {
    arg_plugin: String,
    arg_category: String,
    flag_opt: bool,
}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "move".to_string()];
    argv.extend_from_slice(args);

    let args: MoveArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());
    if let Err(e) = move_plugin(
        &args.arg_plugin,
        &args.arg_category,
        args.flag_opt) {
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
