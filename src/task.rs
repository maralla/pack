use std::thread;

use Result;
use Error;
use package::Package;
use chan;
use echo;
use termion::{terminal_size, color};
use utils::Spinner;
use std::process;

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
    where
        F: Fn(&Package) -> Result<()>,
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

            spinner.stop();
            if !failed {
                echo::character(line, 3, '✓', color::Green);
                echo::inline_message(line, 5 + pos, "done");
            }
        }
    }

    pub fn run<F>(self, func: F)
    where
        F: Fn(&Package) -> Result<()> + Send + 'static + Copy,
    {
        if self.packs.is_empty() {
            die!("No plugins to syncing");
        }

        let y = match terminal_size() {
            Err(e) => die!("Fail to get terminal size. {}", e),
            Ok((_, y)) => y,
        };

        if y <= 2 {
            die!("Terminal size too small.");
        }

        let wg = chan::WaitGroup::new();
        let jobs = chan::WaitGroup::new();
        let (tx, rx) = chan::sync(0);

        for _ in 0..self.thread_num {
            wg.add(1);
            let rx = rx.clone();
            let wg = wg.clone();
            let jobs = jobs.clone();
            thread::spawn(move || while let Some(Some((index, pack))) = rx.recv() {
                jobs.add(1);
                Self::update(&pack, index, func);
                jobs.done();
            });
            wg.done();
        }

        if !self.packs.is_empty() {
            println!();
        }
        for chunk in self.packs.chunks(y as usize - 2) {
            let offset = chunk.len();
            for _ in 0..offset {
                println!();
            }

            for (j, pack) in chunk.into_iter().enumerate() {
                let o = offset - j;
                tx.send(Some((o as u16, pack.clone())));
            }
            jobs.wait();
        }
        if !self.packs.is_empty() {
            println!();
        }

        for _ in 0..self.thread_num {
            tx.send(None);
        }
        wg.wait();

        process::Command::new("vim")
            .arg("--not-a-term")
            .arg("-c")
            .arg("silent! helptags ALL")
            .stdout(process::Stdio::null())
            .spawn()
            .expect("Something went wrong!");
    }
}
