use std::cmp;
use Result;
use Error;
use package::Package;
use chan;
use echo;
use termion::{color, terminal_size};
use utils::Spinner;
use std::process;
use std::thread;
use std::sync::{Arc, Mutex};

pub struct TaskManager {
    packs: Vec<Package>,
    thread_num: usize,
}

impl TaskManager {
    pub fn new(thread_num: usize) -> TaskManager {
        TaskManager {
            packs: Vec::new(),
            thread_num,
        }
    }

    pub fn add(&mut self, pack: Package) {
        self.packs.push(pack);
    }

    /// returns true on success otherwise false
    fn update<F>(pack: &Package, line: u16, func: F) -> bool
    where
        F: Fn(&Package) -> (Result<()>, bool),
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

        let mut successful = true;
        let spinner = Spinner::spin(line, 3);
        if let (Err(e), status) = func(pack) {
            spinner.stop();
            print_err!(e);
            successful = status;
        } else {
            if pack.build_command.is_some() {
                echo::inline_message(line, 5 + pos, "building");
                if let Err(e) = pack.try_build().map_err(|e| Error::build(format!("{}", e))) {
                    print_err!(e);
                }
            }

            spinner.stop();
            if successful {
                echo::character(line, 3, '✓', color::Green);
                echo::inline_message(line, 5 + pos, "done");
            }
        }
        successful
    }

    pub fn run<F>(self, func: F) -> Vec<String>
    where
        F: Fn(&Package) -> (Result<()>, bool) + Send + 'static + Copy,
    {
        if self.packs.is_empty() {
            die!("No plugins to sync");
        }

        let y = match terminal_size() {
            Err(e) => die!("Fail to get terminal size. {}", e),
            Ok((_, y)) => y,
        };

        if y <= 2 {
            die!("Terminal size too small.");
        }

        let jobs = chan::WaitGroup::new();
        let (tx, rx) = chan::sync(0);

        let failures = Arc::new(Mutex::new(vec![]));

        for _ in 0..self.thread_num {
            let rx = rx.clone();
            let jobs = jobs.clone();
            let failures = failures.clone();
            thread::spawn(move || {
                while let Some(Some((index, pack))) = rx.recv() {
                    jobs.add(1);
                    if !Self::update(&pack, index, func) {
                        let mut f = failures.lock().unwrap();
                        f.push(pack.name);
                    }
                    jobs.done();
                }
            });
        }

        if !self.packs.is_empty() {
            println!();
        }
        for chunk in self.packs.chunks(cmp::min(y as usize - 2, self.thread_num)) {
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
        jobs.wait();

        process::Command::new("vim")
            .arg("--not-a-term")
            .arg("-c")
            .arg("silent! helptags ALL")
            .stdout(process::Stdio::null())
            .spawn()
            .expect("Something went wrong!");

        let failures = failures.lock().unwrap();
        failures.clone()
    }
}
