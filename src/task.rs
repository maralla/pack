use std::thread;

use Result;
use Error;
use package::Package;
use chan;
use echo;
use termion::color;
use utils::Spinner;

pub struct TaskManager {
    packs: Vec<Package>,
    thread_num: usize,
}

impl TaskManager {
    pub fn new(thread_num: usize) -> TaskManager {
        TaskManager {
            packs: Vec::new(),
            thread_num: thread_num,
        }
    }

    pub fn add(&mut self, pack: Package) {
        self.packs.push(pack);
    }

    fn update<F>(pack: &Package, line: u16, func: F)
        where F: Fn(&Package) -> Result<()>
    {
        let msg = format!(" [{}]", &pack.name);
        let pos = msg.len() as u16;
        echo::message(line, 0, &format!("    {} syncing", &msg));

        macro_rules! print_err {
            ($err:expr) => {
                let msg = format!("{}", $err);
                echo::character(line, 3, '✗', color::Red);
                echo::inline_message(line, 5 + pos, &msg);
            }
        }

        let spinner = Spinner::spin(line, 3);
        if let Err(e) = func(pack) {
            spinner.stop();
            print_err!(e);
        } else {
            let mut failed = false;
            if pack.build_command.is_some() {
                echo::inline_message(line, 5 + pos, "building");
                if let Err(e) = pack.try_build().map_err(|e| Error::build(format!("{}", e))) {
                    print_err!(e);
                    failed = true;
                }
            }
            if pack.path().join("doc").is_dir() {
                echo::inline_message(line, 5 + pos, "building doc");
                if let Err(_) = pack.try_build_help() {
                    print_err!("Warning: fail to build doc");
                    failed = true;
                }
            }

            spinner.stop();
            if !failed {
                echo::character(line, 3, '✓', color::Green);
                echo::inline_message(line, 5 + pos, "done");
            }
        }
    }

    pub fn run<F>(self, func: F)
        where F: Fn(&Package) -> Result<()> + Send + 'static + Copy
    {
        if self.packs.is_empty() {
            die!("No plugins to syncing");
        }

        let (tx, rx) = chan::sync(0);
        let wg = chan::WaitGroup::new();

        for _ in 0..self.thread_num {
            wg.add(1);
            let rx = rx.clone();
            let wg = wg.clone();
            thread::spawn(move || {
                while let Some(Some((index, pack))) = rx.recv() {
                    Self::update(&pack, index, func);
                }
                wg.done();
            });
        }

        let offset = self.packs.len() as u16 + 2;
        for _ in 0..offset {
            print!("\n");
        }
        for (i, pack) in self.packs.into_iter().enumerate() {
            tx.send(Some((offset - i as u16 - 1, pack)));
        }

        for _ in 0..self.thread_num {
            tx.send(None);
        }
        wg.wait();
    }
}
