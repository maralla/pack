use Result;
use package::{self};
use docopt::Docopt;

const USAGE: &str = "
Generate the pack plugin file (_pack.vim) which combines all plugin configurations

Usage:
    pack generate [options]

Options:
    -h, --help              Display this message
";

#[derive(Debug, RustcDecodable)]
struct GenerateArgs { }

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "generate".to_string()];
    argv.extend_from_slice(args);

    let _args: GenerateArgs =
        Docopt::new(USAGE)
            .and_then(|d| d.argv(argv).decode())
            .unwrap_or_else(|e| e.exit());


    let _ = update_packfile();
}

fn update_packfile() -> Result<()> {
    let mut packs = package::fetch()?;

    packs.sort_by(|a, b| a.name.cmp(&b.name));
    package::update_pack_plugin(&packs)?;

    Ok(())
}
