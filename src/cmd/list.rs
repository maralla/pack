use Result;
use package::{self, Package};
use clap::ArgMatches;

#[derive(Debug)]
struct ListArgs {
    start: bool,
    opt: bool,
    category: Option<String>,
}

impl ListArgs {
    fn from_matches(m: &ArgMatches) -> ListArgs {
        ListArgs {
            start: m.is_present("start"),
            opt: m.is_present("opt"),
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
    let packs = package::fetch()?;

    let ps = if let Some(c) = args.category {
        packs
            .iter()
            .filter(|x| x.category == c)
            .collect::<Vec<&Package>>()
    } else {
        packs.iter().collect::<Vec<&Package>>()
    };

    let res = if args.start {
        ps.into_iter().filter(|x| !x.opt).collect::<Vec<&Package>>()
    } else if args.opt {
        ps.into_iter().filter(|x| x.opt).collect::<Vec<&Package>>()
    } else {
        ps.into_iter().collect::<Vec<&Package>>()
    };

    if res.is_empty() {
        println!("No such packages installed.");
    } else {
        for p in &res {
            println!("{}", p);
        }
    }
    Ok(())
}
