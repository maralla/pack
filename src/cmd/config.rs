use std::fs::{self, File};
use std::io::{Read, Write, ErrorKind};

use {Result, Error};
use docopt::Docopt;
use utils;
use package;

const LOAD_FUNC: &'static str = "
function! s:do_cmd(cmd, bang, start, end, args)
    exec printf('%s%s%s %s', (a:start == a:end ? '' : (a:start.','.a:end)), a:cmd, a:bang, a:args)
endfunction
";

const USAGE: &'static str = "
Config a plugin.

Usage:
    pack config <plugin>
    pack config -h | --help

Options:
    -h, --help              Display this message
";

#[derive(Debug, RustcDecodable)]
struct ConfigArgs {
    arg_plugin: String,
}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "config".to_string()];
    argv.extend_from_slice(args);

    let args: ConfigArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());
    if let Err(e) = config_plugin(&args.arg_plugin) {
        die!("{}", e);
    }
}

fn config_plugin(name: &str) -> Result<()> {
    let packs = package::fetch().unwrap_or(vec![]);
    let pack = packs.iter().filter(|x| name == x.name).next().ok_or(Error::PluginNotInstalled)?;

    let path = pack.config_path();

    let modified = match fs::metadata(&path) {
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                None
            } else {
                return Err(Error::Io(e));
            }
        }
        Ok(meta) => Some(meta.modified()?),
    };

    utils::open_editor(&path)?;

    let meta = match fs::metadata(&path) {
        Err(e) => {
            if e.kind() == ErrorKind::NotFound {
                return Ok(());
            } else {
                return Err(Error::Io(e));
            }
        }
        Ok(m) => m,
    };

    if meta.len() <= 0 {
        fs::remove_file(&path)?;
        if modified.is_some() {
            update_pack_plugin(&packs)?;
        }
    } else if modified.is_none() || meta.modified()? > modified.unwrap() {
        update_pack_plugin(&packs)?;
    }
    Ok(())
}

pub fn update_pack_plugin(packs: &[package::Package]) -> Result<()> {
    let mut f = File::create(&*package::PACK_PLUGIN_FILE)?;
    f.write_all(format!("{}\n\n", LOAD_FUNC).as_bytes())?;

    let mut buf = String::new();
    for (p, path) in packs.iter().map(|x| (x, x.config_path())) {
        buf.clear();
        if let Some(ref c) = p.load_command {
            let (_, repo) = p.repo();
            let msg = format!("\" {name}\ncommand! -nargs=* -range -bang {cmd} packadd {repo} | \
                               call s:do_cmd('{cmd}', \"<bang>\", <line1>, <line2>, <q-args>)\n\n",
                              name = &p.name,
                              cmd = c,
                              repo = repo);
            f.write_all(msg.as_bytes())?;
        }
        if path.is_file() {
            File::open(&path)?.read_to_string(&mut buf)?;
            if p.load_command.is_none() {
                f.write_all(format!("\" {}\n", &p.name).as_bytes())?;
            }
            f.write_all(format!("{}\n", &buf).as_bytes())?;
        }
    }
    Ok(())
}
