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

    let filter = |x: &&Package| -> bool {
        let mut status = true;
        if let Some(ref c) = args.category {
            status &= &x.category == c;
        }
        if args.start {
            status &= !x.opt;
        }
        if args.opt {
            status &= x.opt;
        }
        status
    };

    for p in packs.iter().filter(filter) {
        println!("{}", p);
    }
    Ok(())
}
