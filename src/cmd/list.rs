use Result;
use package::{self, Package};
use clap::ArgMatches;

#[derive(Debug)]
struct ListArgs {
    start: bool,
    opt: bool,
    detached: bool,
    category: Option<String>,
}

impl ListArgs {
    fn from_matches(m: &ArgMatches) -> ListArgs {
        ListArgs {
            start: m.is_present("start"),
            opt: m.is_present("opt"),
            detached: m.is_present("detached"),
            category: value_t!(m, "category", String).ok(),
        }
    }
}

pub fn exec(matches: &ArgMatches) {
    let args = ListArgs::from_matches(matches);

    if let Err(e) = list_packages(args) {
        die!("Err: {}", e);
    }
}

fn list_packages(args: ListArgs) -> Result<()> {
    if args.detached {
        list_detached()
    } else {
        list_installed(&args.category, args.start, args.opt)
    }
}

fn list_installed(category: &Option<String>, start: bool, opt: bool) -> Result<()> {
    let packs = package::fetch()?;

    let filter = |x: &Package| -> bool {
        let mut status = true;
        if let Some(ref c) = *category {
            status &= &x.category == c;
        }
        if start {
            status &= !x.opt;
        }
        if opt {
            status &= x.opt;
        }
        status
    };

    for p in packs.into_iter().filter(filter) {
        println!("{}", p);
    }
    Ok(())
}

fn list_detached() -> Result<()> {
    let available_packs = package::walk_packs()?;
    let installed = package::fetch()?;
    let pack_names: Vec<&str> = installed.iter().map(|p| p.repo().1).collect();

    let detached: Vec<String> = available_packs
        .into_iter()
        .filter(|&(_, _, ref name)| !pack_names.contains(&&*name.as_str()))
        .map(|(cat, opt, name)| format!("{}/{}/{}", cat, opt, name))
        .collect();

    for p in detached {
        println!("{}", p);
    }
    Ok(())
}
