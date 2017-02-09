use std::path::Path;
use std::os::unix::fs::symlink;

use {Error, Result};
use docopt::Docopt;
use package::{self, Package};
use git;
use num_cpus;
use task::TaskManager;

const USAGE: &'static str = "
Install plugins.

Usage:
    pack install
    pack install <plugin>... [options]
    pack install -h | --help

Options:
    -o, --opt               Install this plugin as opt
    -c, --category CAT      Install this plugin to category CAT [default: default]
    -l, --local             Install a local plugin
    --on CMD                Command for loading this plugin
    --for TYPES             Load this plugin for TYPES
    --build BUILD           Build command for this plugin
    -j, --threads THREADS   Installing plugins concurrently
    -h, --help              Display this message
";

#[derive(Debug, RustcDecodable)]
struct InstallArgs {
    arg_plugin: Vec<String>,
    flag_local: bool,
    flag_on: Option<String>,
    flag_for: Option<String>,
    flag_threads: Option<usize>,
    flag_opt: bool,
    flag_category: String,
    flag_build: Option<String>,
}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "install".to_string()];
    argv.extend_from_slice(args);

    let args: InstallArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());

    let threads = args.flag_threads.unwrap_or(num_cpus::get());
    if threads < 1 {
        die!("Threads should be greater than 0");
    }

    let opt = args.flag_on.is_some() || args.flag_for.is_some() || args.flag_opt;
    let types = args.flag_for
        .map(|e| e.split(',').map(|e| e.to_string()).collect::<Vec<String>>());
    if let Err(e) = install_plugins(args.arg_plugin,
                                    args.flag_category,
                                    opt,
                                    args.flag_on,
                                    types,
                                    args.flag_build,
                                    threads,
                                    args.flag_local) {
        die!("Err: {}", e);
    }
}

fn install_plugins(name: Vec<String>,
                   category: String,
                   opt: bool,
                   on: Option<String>,
                   types: Option<Vec<String>>,
                   build: Option<String>,
                   threads: usize,
                   local: bool)
                   -> Result<()> {
    let mut packs = package::fetch()?;

    {
        let mut manager = TaskManager::new(threads);

        if name.is_empty() {
            for pack in packs.iter() {
                manager.add(pack.clone());
            }
        } else {
            let targets = name.into_iter().map(|ref n| {
                let mut p = Package::new(n, &category, opt);
                p.local = local;
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
            for mut pack in targets.into_iter() {
                let having = match packs.iter_mut().filter(|x| x.name == pack.name).next() {
                    Some(x) => {
                        if !x.is_installed() {
                            x.set_category(pack.category.as_str());
                            x.set_opt(pack.opt);
                            x.set_types(pack.for_types.clone());

                            x.load_command = pack.load_command.clone();
                            x.build_command = pack.build_command.clone();
                        } else {
                            pack.set_category(x.category.as_str());
                            pack.set_opt(x.opt);
                        }
                        true
                    }
                    None => false,
                };
                if !having {
                    packs.push(pack.clone());
                }
                manager.add(pack);
            }
        }
        manager.run(install_plugin);
    }

    packs.sort_by(|a, b| a.name.cmp(&b.name));

    package::update_pack_plugin(&packs)?;
    package::save(packs)
}

fn install_plugin(pack: &Package) -> Result<()> {
    let path = pack.path();
    if path.is_dir() {
        Err(Error::plugin_installed(&path))
    } else if pack.local {
        let src = Path::new(&pack.name);
        if !src.is_dir() {
            Err(Error::NoPlugin)
        } else {
            symlink(&src, &path)?;
            Ok(())
        }
    } else {
        git::clone(&pack.name, &pack.path())?;
        Ok(())
    }
}
