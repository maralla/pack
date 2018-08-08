use clap::ArgMatches;
use git;
use num_cpus;
use package::{self, Package};
use task::TaskManager;
use {Error, Result};

#[derive(Debug)]
struct UpdateArgs {
    plugins: Vec<String>,
    skip: Vec<String>,
    threads: Option<usize>,
    packplugin: bool,
    reference: Option<String>,
}

impl UpdateArgs {
    fn from_matches(m: &ArgMatches) -> UpdateArgs {
        UpdateArgs {
            plugins: m.values_of_lossy("package").unwrap_or_else(|| vec![]),
            skip: m.values_of_lossy("skip").unwrap_or_else(|| vec![]),
            threads: value_t!(m, "threads", usize).ok(),
            packplugin: m.is_present("packplugin"),
            reference: value_t!(m, "reference", String).ok(),
        }
    }
}

pub fn exec(matches: &ArgMatches) {
    let args = UpdateArgs::from_matches(matches);

    if args.packplugin {
        if let Err(e) = update_packplugin() {
            die!("Err: {}", e);
        }
        return;
    }

    let threads = args.threads.unwrap_or_else(num_cpus::get);
    if threads < 1 {
        die!("Threads should be greater than 0");
    }

    if let Err(e) = update_plugins(args.plugins, threads, args.skip, args.reference) {
        die!("Err: {}", e);
    }
}

fn update_packplugin() -> Result<()> {
    println!("Update _pack file for all plugins.");
    let mut packs = package::fetch()?;

    packs.sort_by(|a, b| a.name.cmp(&b.name));
    package::update_pack_plugin(&packs)?;

    Ok(())
}

/// Update specified plugins.
///
/// The specified plugin can be updated to a different git commit or branch.
/// If the updating failed pack will not try to fix or rollback or delete
/// the plugin. If successed packfile will be updated.
fn update_plugins(
    plugins: Vec<String>,
    threads: usize,
    skip: Vec<String>,
    reference: Option<String>,
) -> Result<()> {
    let mut packs = package::fetch()?;
    let mut manager = TaskManager::new(threads);
    manager.set_commit(reference);

    {
        let pack_iter = packs
            .iter()
            .filter(|p| !skip.iter().any(|x| p.name.contains(x)));
        if !plugins.is_empty() {
            for pack in pack_iter.filter(|x| plugins.contains(&x.name)) {
                manager.add(pack.clone());
            }
        } else {
            for pack in pack_iter {
                manager.add(pack.clone());
            }
        }
    }

    let _failed = manager.run(update_plugin);
    // for pack in packs {}
    // package::save(packs);
    Ok(())
}

fn update_plugin(pack: &Package, _commit: &Option<String>) -> (Result<()>, bool) {
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
        git::update(&pack.name, &path, &pack.reference)
    }
}
