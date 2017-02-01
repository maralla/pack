use std::thread;

use {Error, Result};
use docopt::Docopt;
use package::{self, Package};
use git;
use utils::{Spinner, async_print};
use termion::{color, cursor, clear};
use num_cpus;
use chan;

const USAGE: &'static str = "
Install plugins.

Usage:
    pack install
    pack install <plugin>... [options]
    pack install -h | --help

Options:
    -o, --opt               Install this plugin as opt
    -c, --category CAT      Install this plugin to category CAT [default: default]
    --on CMD                Command for loading this plugin
    --for TYPES             Load this plugin for TYPES
    --build BUILD           Build command for this plugin
    -j, --threads THREADS   Installing plugins concurrently
    -h, --help              Display this message
";

#[derive(Debug, RustcDecodable)]
struct InstallArgs {
    arg_plugin: Vec<String>,
    flag_on: Option<String>,
    flag_for: Option<String>,
    flag_threads: Option<usize>,
    flag_opt: bool,
    flag_category: String,
    flag_build: Option<String>,
}

pub fn execute(args: &[String]) {
    let mut argv = vec!["pack".to_string(), "install".to_string()];
    argv.extend_from_slice(args);

    let args: InstallArgs =
        Docopt::new(USAGE).and_then(|d| d.argv(argv).decode()).unwrap_or_else(|e| e.exit());

    let threads = args.flag_threads.unwrap_or(num_cpus::get());
    if threads < 1 {
        die!("Threads should be greater than 0");
    }

    let types = args.flag_for
        .map(|e| e.split(',').map(|e| e.to_string()).collect::<Vec<String>>());
    install_plugins(args.arg_plugin,
                    args.flag_category,
                    args.flag_opt,
                    args.flag_on,
                    types,
                    args.flag_build,
                    threads);
}

struct TaskManager {
    packs: Vec<Package>,
    thread_num: usize,
}

impl TaskManager {
    fn new(thread_num: usize) -> TaskManager {
        TaskManager {
            packs: Vec::new(),
            thread_num: thread_num,
        }
    }

    fn add(&mut self, pack: Package) {
        self.packs.push(pack);
    }

    fn run(self) {
        print!("{}", cursor::Hide);
        let (tx, rx) = chan::sync(0);
        let wg = chan::WaitGroup::new();

        for _ in 0..self.thread_num {
            wg.add(1);
            let rx = rx.clone();
            let wg = wg.clone();
            thread::spawn(move || {
                while let Some(Some((index, pack))) = rx.recv() {
                    report_install(index, &pack);
                }
                wg.done();
            });
        }

        for _ in 0..self.packs.len() + 1 {
            print!("\n");
        }
        let offset = self.packs.len() as u16 + 1;
        print!("{}", cursor::Up(offset));

        for (i, pack) in self.packs.into_iter().enumerate() {
            tx.send(Some((i as u16 + 1, pack)));
        }

        for _ in 0..self.thread_num {
            tx.send(None);
        }
        wg.wait();
        print!("{}{}", cursor::Down(offset), cursor::Show);
    }
}

fn report_install(line: u16, pack: &Package) {
    let msg = format!(" [{}]", &pack.name);
    let pos = msg.len() as u16;

    async_print(line, pos + 15, &format!("    {} installing", &msg));

    macro_rules! print_err {
        ($err:expr) => {
            let msg = format!("{}", $err);
            async_print(line,
                        4,
                        &format!("{}{}✗{}",
                                cursor::Right(3),
                                color::Fg(color::Red),
                                color::Fg(color::Reset)));
            async_print(line,
                        5 + pos + msg.len() as u16,
                        &format!("{}{}{}", cursor::Right(5 + pos), clear::UntilNewline, &msg));
        }
    }

    let spinner = Spinner::spin(line, 3);

    if pack.is_installed() {
        spinner.stop();
        print_err!(Error::plugin_installed(&pack.path()));
        return;
    }

    if let Err(e) = install_plugin(pack) {
        spinner.stop();
        print_err!(e);
    } else {
        let mut failed = false;
        if pack.build_command.is_some() {
            async_print(line,
                        13 + pos,
                        &format!("{}{}building", cursor::Right(5 + pos), clear::UntilNewline));
            if let Err(e) = pack.build().map_err(|e| Error::build(format!("{}", e))) {
                print_err!(e);
                failed = true;
            }
        }
        if pack.path().join("doc").is_dir() {
            async_print(line,
                        17 + pos,
                        &format!("{}{}building doc",
                                 cursor::Right(5 + pos),
                                 clear::UntilNewline));
            if let Err(_) = pack.try_build_help() {
                print_err!("Warning: fail to build doc");
                failed = true;
            }
        }

        spinner.stop();
        if !failed {
            async_print(line,
                        4,
                        &format!("{}{}✓{}",
                                 cursor::Right(3),
                                 color::Fg(color::Green),
                                 color::Fg(color::Reset)));
            async_print(line,
                        9 + pos,
                        &format!("{}{}done", cursor::Right(5 + pos), clear::UntilNewline));
        }
    }
}

fn install_plugins(name: Vec<String>,
                   category: String,
                   opt: bool,
                   on: Option<String>,
                   types: Option<Vec<String>>,
                   build: Option<String>,
                   threads: usize) {
    let mut packs = package::fetch().unwrap_or(vec![]);

    // If has load command opt is always true.
    let opt = if on.is_some() || types.is_some() {
        true
    } else {
        opt
    };

    {
        let mut manager = TaskManager::new(threads);

        if name.is_empty() {
            for pack in packs.iter() {
                manager.add(pack.clone());
            }
        } else {
            let targets = name.into_iter().map(|ref n| {
                let mut p = Package::new(n, &category, opt);
                if let Some(ref c) = on {
                    p.set_load_command(c);
                }
                if let Some(ref t) = types {
                    p.set_types(t.clone());
                }
                if let Some(ref c) = build {
                    p.set_build_command(c);
                }
                p
            });
            for mut pack in targets.into_iter() {
                let having = match packs.iter_mut().filter(|x| x.name == pack.name).next() {
                    Some(x) => {
                        if !x.is_installed() {
                            x.set_category(pack.category.as_str());
                            x.set_opt(pack.opt);
                            x.set_types(pack.for_types.clone());
                            if let Some(ref c) = pack.load_command {
                                x.set_load_command(c);
                            }
                            if let Some(ref c) = pack.build_command {
                                x.set_build_command(c);
                            }
                        } else {
                            pack.set_category(x.category.as_str());
                            pack.set_opt(x.opt);
                        }
                        true
                    }
                    None => false,
                };
                if !having {
                    packs.push(pack.clone());
                }
                manager.add(pack);
            }
        }
        manager.run();
    }

    packs.sort_by(|a, b| a.name.cmp(&b.name));

    if let Err(e) = package::update_pack_plugin(&packs) {
        die!("Fail to update pack plugin file: {}", e);
    }

    if let Err(e) = package::save(packs) {
        die!("Fail to save packfile: {}", e);
    }
}

fn install_plugin(pack: &Package) -> Result<()> {
    let path = pack.path();
    if path.is_dir() {
        Err(Error::plugin_installed(&path))
    } else {
        let (user, repo) = pack.repo();
        git::clone(user, repo, &pack.path())?;
        Ok(())
    }
}
