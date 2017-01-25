use {Error, Result};
use docopt::Docopt;
use package::{self, Package};
use git;
use utils::Spinner;

const USAGE: &'static str = "
Install a plugin.

Usage:
    pack install <plugin> [options]
    pack install -h | --help

Options:
    -o, --opt               Install this plugin as opt
    -c, --category CAT      Install this plugin to category CAT [default: default]
    -h, --help              Display this message
";

#[derive(Debug, RustcDecodable)]
struct InstallArgs {
    arg_plugin: String,
    flag_opt: bool,
    flag_category: String,
}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "install".to_string()];
    argv.extend_from_slice(args);

    let args: InstallArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());

    print!("Installing plugin '{}' ", &args.arg_plugin);
    let res = install_plugin(args.arg_plugin, args.flag_category, args.flag_opt);
    println!("");
    if let Err(e) = res {
        die!("{}", e);
    }
}

fn install_plugin(name: String, category: String, opt: bool) -> Result<()> {
    if !name.contains("/") {
        return Err(Error::RepoName);
    }

    let p = Package::new(&name, &category, opt);
    let path = p.path().ok_or(Error::RepoName)?;
    if path.is_dir() {
        return Err(Error::plugin_installed(&path));
    }

    let mut packs = package::fetch().unwrap_or(vec![]);
    let has_entry = {
        if let Some(p) = packs.iter_mut().filter(|p| p.name == name).next() {
            if p.is_installed() {
                return Err(Error::plugin_installed(p.path().unwrap()));
            }
            p.set_category(&category as &str);
            p.set_opt(opt);
            true
        } else {
            false
        }
    };
    if !has_entry {
        packs.push(p);
    }

    let repo = package::get_repo(&name).ok_or(Error::RepoName)?;
    let user = package::get_user(&name).ok_or(Error::RepoName)?;

    let spinner = Spinner::spin();
    git::clone(user, repo, &path)?;
    package::save(packs)?;
    spinner.stop();
    Ok(())
}
