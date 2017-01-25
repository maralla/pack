use docopt::Docopt;
use package::{self, Package};

const USAGE: &'static str = "
List installed packages.

Usage:
    pack list [(--start | --opt) --category CAT]

Options:
    -s, --start          Only list start packages
    -o, --opt            Only list option packages
    -c, --category CAT   Only list packages under this category
    -h, --help           Display this message
";

#[derive(Debug, RustcDecodable)]
struct ListArgs {
    flag_start: bool,
    flag_opt: bool,
    flag_category: Option<String>,
}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "list".to_string()];
    argv.extend_from_slice(args);

    let args: ListArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());

    list_packages(args.flag_start, args.flag_opt, args.flag_category);
}

fn list_packages(start: bool, opt: bool, cat: Option<String>) {
    let packs = package::fetch().unwrap_or(vec![]);

    let ps = if let Some(c) = cat {
        packs.iter().filter(|x| x.category == c).collect::<Vec<&Package>>()
    } else {
        packs.iter().collect::<Vec<&Package>>()
    };

    let res = if start {
        ps.into_iter().filter(|x| !x.opt).collect::<Vec<&Package>>()
    } else if opt {
        ps.into_iter().filter(|x| x.opt).collect::<Vec<&Package>>()
    } else {
        ps.into_iter().collect::<Vec<&Package>>()
    };

    if res.is_empty() {
        println!("No plugin installed.");
    } else {
        for p in res.iter() {
            println!("{}", p);
        }
    }
}
