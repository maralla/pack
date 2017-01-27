use {Error, Result};
use docopt::Docopt;
use package::{self, Package};
use git;
use utils::Spinner;
use ansi_term::Colour::{Red, Green};

const USAGE: &'static str = "
Install plugins.

Usage:
    pack install
    pack install <plugin>... [options]
    pack install -h | --help

Options:
    -o, --opt               Install this plugin as opt
    -c, --category CAT      Install this plugin to category CAT [default: default]
    -h, --help              Display this message
";

#[derive(Debug, RustcDecodable)]
struct InstallArgs {
    arg_plugin: Vec<String>,
    flag_opt: bool,
    flag_category: String,
}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "install".to_string()];
    argv.extend_from_slice(args);

    let args: InstallArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());

    install_plugins(args.arg_plugin, args.flag_category, args.flag_opt);
}

fn report_install<F>(pack: &Package, mut install_func: F)
    where F: FnMut(&Package) -> Result<()>
{
    print!("Installing plugin '{}' ", &pack.name);
    if let Err(e) = install_func(pack) {
        println!("{}", Red.paint("✗"));
        println!("{}", e);
    } else {
        println!("{}", Green.paint("✓"));
    }
}

fn install_plugins(name: Vec<String>, category: String, opt: bool) {
    let mut packs = package::fetch().unwrap_or(vec![]);

    if name.is_empty() {
        for pack in packs.iter() {
            report_install(pack, install_plugin);
        }
    } else {
        for ref pack in name.into_iter().map(|ref n| Package::new(n, &category, opt)) {
            report_install(pack, |p| {
                let having = match packs.iter_mut().filter(|x| x.name == p.name).next() {
                    Some(x) => {
                        if x.is_installed() {
                            return Err(Error::plugin_installed(p.path()));
                        }
                        x.set_category(p.category.as_str());
                        x.set_opt(p.opt);
                        true
                    }
                    None => false,
                };
                if !having {
                    packs.push(p.clone());
                }
                install_plugin(p)
            });
        }
        if let Err(e) = package::save(packs) {
            die!("Fail to save packfile: {}", e);
        }
    }

}

fn install_plugin(pack: &Package) -> Result<()> {
    let path = pack.path();
    if path.is_dir() {
        Err(Error::plugin_installed(&path))
    } else {
        let (user, repo) = pack.repo();

        let spinner = Spinner::spin();
        git::clone(user, repo, &pack.path())?;
        spinner.stop();
        Ok(())
    }
}
