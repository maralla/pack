use {Error, Result};
use docopt::Docopt;
use package::{self, Package};
use git;
use utils::Spinner;
use ansi_term::Colour::{Red, Green};
use cmd::config::update_pack_plugin;

const USAGE: &'static str = "
Install plugins.

Usage:
    pack install
    pack install <plugin>... [options]
    pack install -h | --help

Options:
    -o, --opt               Install this plugin as opt
    -c, --category CAT      Install this plugin to category CAT [default: default]
    --on CMD                Command for loading this plugin
    --for TYPES             Load this plugin for TYPES
    --build BUILD           Build command for this plugin
    -h, --help              Display this message
";

#[derive(Debug, RustcDecodable)]
struct InstallArgs {
    arg_plugin: Vec<String>,
    flag_on: Option<String>,
    flag_for: Option<String>,
    flag_opt: bool,
    flag_category: String,
    flag_build: Option<String>,
}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "install".to_string()];
    argv.extend_from_slice(args);

    let args: InstallArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());

    let types = args.flag_for
        .map(|e| e.split(',').map(|e| e.to_string()).collect::<Vec<String>>());
    install_plugins(args.arg_plugin,
                    args.flag_category,
                    args.flag_opt,
                    args.flag_on,
                    types,
                    args.flag_build);
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

        if pack.build_command.is_some() {
            if let Err(e) = pack.build().map_err(|e| Error::build(format!("{}", e))) {
                die!("{}", e);
            }
        }
        if let Err(_) = pack.try_build_help() {
            println!("Warning: fail to build doc");
        }
    }
}

fn install_plugins(name: Vec<String>,
                   category: String,
                   opt: bool,
                   on: Option<String>,
                   types: Option<Vec<String>>,
                   build: Option<String>) {
    let mut packs = package::fetch().unwrap_or(vec![]);

    // If has load command opt is always true.
    let opt = if on.is_some() || types.is_some() {
        true
    } else {
        opt
    };

    if name.is_empty() {
        for pack in packs.iter() {
            report_install(pack, install_plugin);
        }
    } else {
        let targets = name.into_iter().map(|ref n| {
            let mut p = Package::new(n, &category, opt);
            if let Some(ref c) = on {
                p.set_load_command(c);
            }
            if let Some(ref t) = types {
                p.set_types(t.clone());
            }
            if let Some(ref c) = build {
                p.set_build_command(c);
            }
            p
        });
        for ref pack in targets {
            report_install(pack, |p| {
                let having = match packs.iter_mut().filter(|x| x.name == p.name).next() {
                    Some(x) => {
                        if x.is_installed() {
                            return Err(Error::plugin_installed(x.path()));
                        }
                        x.set_category(p.category.as_str());
                        x.set_opt(p.opt);
                        x.set_types(p.for_types.clone());
                        if let Some(ref c) = p.load_command {
                            x.set_load_command(c);
                        }
                        if let Some(ref c) = p.build_command {
                            x.set_build_command(c);
                        }
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
    }

    packs.sort_by(|a, b| a.name.cmp(&b.name));

    if let Err(e) = update_pack_plugin(&packs) {
        die!("Fail to update pack plugin file: {}", e);
    }

    if let Err(e) = package::save(packs) {
        die!("Fail to save packfile: {}", e);
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
