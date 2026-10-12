use crate::git;
use crate::package::{self, Package};
use crate::task::{TaskManager, TaskType};
use crate::{Error, Result};
use clap::ArgMatches;

#[derive(Debug)]
struct UpdateArgs {
    plugins: Vec<String>,
    skip: Vec<String>,
    threads: Option<usize>,
    packfile: bool,
    changelog: bool,
}

impl UpdateArgs {
    fn from_matches(m: &ArgMatches) -> UpdateArgs {
        UpdateArgs {
            plugins: m
                .get_many::<String>("package")
                .map(|v| v.cloned().collect())
                .unwrap_or_default(),
            skip: m
                .get_many::<String>("skip")
                .map(|v| v.cloned().collect())
                .unwrap_or_default(),
            threads: m.get_one::<usize>("threads").copied(),
            packfile: m.get_flag("packfile"),
            changelog: m.get_flag("changelog"),
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

    if let Err(e) = update_plugins(&args.plugins, threads, &args.skip, args.changelog) {
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

fn update_plugins(
    plugins: &[String],
    threads: usize,
    skip: &[String],
    changelog: bool,
) -> Result<()> {
    let mut packs = package::fetch()?;

    let mut manager = TaskManager::new(TaskType::Update, threads);
    manager.changelog = changelog;
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

    let fails = match manager.run(update_plugin) {
        // Aborted: skip the regeneration entirely — it cannot be
        // interrupted once it starts, and the user already asked to stop.
        // The newline: the last write left the cursor mid-row, and without
        // it the shell prompt lands on the output's last line.
        Err(Error::Aborted) => {
            println!();
            return Ok(());
        }
        Err(e) => return Err(e),
        Ok(fails) => fails,
    };
    for fail in fails {
        packs.retain(|e| e.name != fail);
    }

    packs.sort_by(|a, b| a.name.cmp(&b.name));

    package::update_pack_plugin(&packs)?;

    Ok(())
}

fn update_plugin(pack: &Package) -> (Result<Vec<String>>, bool) {
    let res = do_update(pack);
    let status = match res {
        Err(Error::SkipLocal) | Err(Error::Git(_)) => true,
        Err(_) => false,
        _ => true,
    };
    (res, status)
}

fn do_update(pack: &Package) -> Result<Vec<String>> {
    let path = pack.path();
    if !path.is_dir() {
        Err(Error::PluginNotInstalled)
    } else if pack.local {
        Err(Error::SkipLocal)
    } else {
        // UpToDate is reported as the sentinel error so the task runner can
        // ignore the package without counting it as a failure; the
        // changelog travels as the Ok payload (printed after the run).
        match git::update(&pack.name, &path)? {
            git::UpdateStatus::UpToDate => Err(Error::UpToDate),
            git::UpdateStatus::Updated(log) => Ok(log),
        }
    }
}
