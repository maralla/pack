use clap::ArgMatches;
use package::{self, Package};
use Result;

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
    let f = if args.detached {
        list_detached
    } else {
        list_installed
    };
    f(&args.category, args.start, args.opt)
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

fn list_detached(category: &Option<String>, start: bool, opt: bool) -> Result<()> {
    let installed = package::fetch()?;
    let pack_names: Vec<&str> = installed.iter().map(|p| p.repo().1).collect();

    package::walk_packs(category, start, opt, |cate, option, name| {
        if !pack_names.contains(&name) {
            println!("{}/{}/{}", cate, option, name);
        }
    })
}
