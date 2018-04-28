use {Error, Result};
use package::{self, Package};
use num_cpus;
use git;
use task::TaskManager;
use clap::ArgMatches;

#[derive(Debug)]
struct UpdateArgs {
    plugins: Vec<String>,
    skip: Vec<String>,
    threads: Option<usize>,
    packfile: bool,
}

impl UpdateArgs {
    fn from_matches(m: &ArgMatches) -> UpdateArgs {
        UpdateArgs {
            plugins: m.values_of_lossy("package").unwrap_or_else(|| vec![]),
            skip: m.values_of_lossy("skip").unwrap_or_else(|| vec![]),
            threads: value_t!(m, "threads", usize).ok(),
            packfile: m.is_present("packfile"),
        }
    }
}

pub fn exec(matches: &ArgMatches) {
    let args = UpdateArgs::from_matches(matches);

    if args.packfile {
        if let Err(e) = update_packfile() {
            die!("Err: {}", e);
        }
        return;
    }

    let threads = args.threads.unwrap_or_else(num_cpus::get);
    if threads < 1 {
        die!("Threads should be greater than 0");
    }

    if let Err(e) = update_plugins(&args.plugins, threads, &args.skip) {
        die!("Err: {}", e);
    }
}

fn update_packfile() -> Result<()> {
    println!("Update _pack file for all plugins.");
    let mut packs = package::fetch()?;

    packs.sort_by(|a, b| a.name.cmp(&b.name));
    package::update_pack_plugin(&packs)?;

    Ok(())
}

fn update_plugins(plugins: &[String], threads: usize, skip: &[String]) -> Result<()> {
    let mut packs = package::fetch()?;

    let mut manager = TaskManager::new(threads);
    if plugins.is_empty() {
        for pack in &packs {
            if skip.iter().any(|x| pack.name.contains(x)) {
                println!("Skip {}", pack.name);
                continue;
            }
            manager.add(pack.clone());
        }
    } else {
        for pack in packs.iter().filter(|x| plugins.contains(&x.name)) {
            manager.add(pack.clone());
        }
    }

    for fail in manager.run(update_plugin) {
        packs.retain(|e| e.name != fail);
    }

    packs.sort_by(|a, b| a.name.cmp(&b.name));

    package::update_pack_plugin(&packs)?;

    Ok(())
}

fn update_plugin(pack: &Package) -> (Result<()>, bool) {
    let res = do_update(pack);
    let status = match res {
        Err(Error::SkipLocal) | Err(Error::Git(_)) => true,
        Err(_) => false,
        _ => true,
    };
    (res, status)
}

fn do_update(pack: &Package) -> Result<()> {
    let path = pack.path();
    if !path.is_dir() {
        Err(Error::PluginNotInstalled)
    } else if pack.local {
        Err(Error::SkipLocal)
    } else {
        git::update(&pack.name, &path)
    }
}
