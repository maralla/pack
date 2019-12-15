use crate::package;
use crate::Result;
use clap::ArgMatches;

pub fn exec(_matches: &ArgMatches) {
    let _ = update_packfile();
}

fn update_packfile() -> Result<()> {
    let mut packs = package::fetch()?;

    packs.sort_by(|a, b| a.name.cmp(&b.name));
    package::update_pack_plugin(&packs)?;

    Ok(())
}
