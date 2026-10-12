use crate::echo;
use crate::package::Package;
use crate::utils::Spinner;
use crate::Error;
use crate::Result;

use crossbeam_channel::{bounded, select, Receiver};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
use std::fs;
use std::io;
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use termion::{color, terminal_size};

pub enum TaskType {
    Install,
    Update,
}

pub struct TaskManager {
    task_type: TaskType,
    packs: Vec<Package>,
    thread_num: usize,
    /// Show the changelog overview of updates (opt-in: default false).
    pub changelog: bool,
}

impl TaskManager {
    pub fn new(task_type: TaskType, thread_num: usize) -> TaskManager {
        TaskManager {
            task_type,
            packs: Vec::new(),
            thread_num,
            changelog: false,
        }
    }

    pub fn add(&mut self, pack: Package) {
        self.packs.push(pack);
    }

    /// returns true on success otherwise false
    fn update<F>(
        pack: &Package,
        line: u16,
        func: F,
        changelog_sink: &Mutex<Vec<(String, Vec<String>)>>,
    ) -> bool
    where
        F: Fn(&Package) -> (Result<Vec<String>>, bool),
    {
        if echo::plain() {
            // Non-interactive sink (pipe, file, CI — issue #33): the cursor
            // UI is unusable there, so print one plain result line per
            // package instead of driving the cursor.
            let result = func(pack);
            // The same markers as the interactive UI (hollow circle for
            // skipped, cross for failure, check for done) so the two modes
            // read identically; only the color is missing here.
            return match result {
                (Err(Error::UpToDate), _) => {
                    println!("○ {} up to date", pack.name);
                    true
                }
                (Err(e), status) => {
                    println!("✗ {}: {}", pack.name, e);
                    status
                }
                (Ok(changelog), _) => {
                    if pack.build_command.is_some() {
                        if let Err(e) =
                            pack.try_build().map_err(|e| Error::build(format!("{}", e)))
                        {
                            println!("✗ {}: {}", pack.name, e);
                            return false;
                        }
                    }
                    println!("✓ {} updated", pack.name);
                    if !changelog.is_empty() {
                        changelog_sink
                            .lock()
                            .unwrap()
                            .push((pack.name.clone(), changelog.clone()));
                    }
                    true
                }
            };
        }

        let msg = format!(" [{}]", pack.name);
        let pos = msg.len() as u16;
        echo::message(line, 0, &format!("    {} syncing", msg));

        const MSG_MARGIN: u16 = 5;
        const SIGN_MARGIN: u16 = 3;

        macro_rules! print_err {
            ($err:expr) => {
                let msg = format!("{}", $err);
                echo::character(line, SIGN_MARGIN, '✗', color::Red);
                echo::inline_message(line, MSG_MARGIN + pos, &msg);
            };
        }

        let mut successful = true;
        let spinner = Spinner::spin(line, SIGN_MARGIN);
        let result = func(pack);
        if let (Err(Error::UpToDate), _) = &result {
            // Already at the upstream tip: ignored, not a failure.
            // Overwrite the spinner glyph with the industry "skipped / no
            // change" marker (hollow circle, gray — as GitHub Actions and
            // GitLab CI render skipped jobs) so the line does not keep
            // spinning after the spinner stopped.
            spinner.stop();
            echo::character(line, SIGN_MARGIN, '○', color::LightBlack);
            echo::inline_message(line, MSG_MARGIN + pos, "up to date");
        } else if let (Err(e), status) = &result {
            spinner.stop();
            let msg = format!("{}", e);
            echo::character(line, SIGN_MARGIN, '✗', color::Red);
            echo::inline_message(line, MSG_MARGIN + pos, &msg);
            successful = *status;
        } else {
            if pack.build_command.is_some() {
                echo::inline_message(line, MSG_MARGIN + pos, "building");
                if let Err(e) = pack.try_build().map_err(|e| Error::build(format!("{}", e))) {
                    print_err!(e);
                }
            }

            spinner.stop();
            if successful {
                echo::character(line, SIGN_MARGIN, '✓', color::Green);
                echo::inline_message(line, MSG_MARGIN + pos, "done");
                // Issue #38: collect the update changelog for the
                // end-of-run overview.
                if let (Ok(changelog), _) = &result {
                    if !changelog.is_empty() {
                        changelog_sink
                            .lock()
                            .unwrap()
                            .push((pack.name.clone(), changelog.clone()));
                    }
                }
            }
        }
        successful
    }

    pub fn run<F>(self, func: F) -> Vec<String>
    where
        F: Fn(&Package) -> (Result<Vec<String>>, bool) + Send + 'static + Copy,
    {
        if self.packs.is_empty() {
            die!("No plugins to sync");
        }

        if !echo::plain() {
            let y = match terminal_size() {
                Err(e) => die!("Fail to get terminal size. {}", e),
                Ok((_, y)) => y,
            };

            if y <= 2 {
                die!("Terminal size too small.");
            }
        }

        let quit_notifier = match setup_signal() {
            Err(e) => die!("Fail to set up signal. {}", e),
            Ok(r) => r,
        };

        let threads = self.thread_num;

        let (tx, rx) = bounded::<Option<Package>>(threads);

        let failures = Arc::new(Mutex::new(vec![]));
        let pending = Arc::new(Mutex::new(vec![]));
        let changelogs = Arc::new(Mutex::new(Vec::<(String, Vec<String>)>::new()));

        let mut workers = Vec::with_capacity(threads);
        for _ in 0..threads {
            let rx = rx.clone();
            let failures = failures.clone();
            let pending = pending.clone();
            let quit_notifier = quit_notifier.clone();
            let changelogs = changelogs.clone();
            workers.push(thread::spawn(move || {
                while let Ok(Some(pack)) = rx.recv() {
                    log::info!("pack {}", &pack.name);
                    {
                        let mut p = pending.lock().unwrap();
                        log::info!("add to pending:{}", &pack.name);
                        p.push(pack.clone());
                    }

                    let name = pack.name.clone();
                    let failures = failures.clone();
                    let changelogs = changelogs.clone();

                    let (wtx, wrx) = bounded(0);
                    thread::spawn(move || {
                        let index = if echo::plain() { 0 } else { echo::line() };
                        if !Self::update(&pack, index, func, &changelogs) {
                            let mut f = failures.lock().unwrap();
                            f.push(pack.name);
                        }
                        let _ = wtx.send(());
                    });
                    select! {
                        recv(wrx) -> _ => {},
                        recv(quit_notifier) -> _ => {
                            log::info!("quit received {}", &name);
                            return;
                        }
                    }
                    {
                        let mut p = pending.lock().unwrap();
                        log::info!("remove from pending: {}", &name);
                        p.retain(|x| x.name != name);
                    }
                }
            }));
        }
        if !self.packs.is_empty() && !echo::plain() {
            println!();
        }

        for pack in self.packs.iter() {
            let _ = tx.send(Some(pack.clone()));
        }

        for _ in 0..threads {
            let _ = tx.send(None);
        }
        for w in workers {
            let _ = w.join();
        }

        if !self.packs.is_empty() && !echo::plain() {
            println!();
        }

        // Issue #38: changelog overview of everything updated in this run,
        // after the per-package UI is done. Opt-in: hidden by default.
        {
            let changelogs = changelogs.lock().unwrap();
            if self.changelog && !changelogs.is_empty() {
                println!("Changelogs of updated packages:");
                println!();
                for (name, log) in changelogs.iter() {
                    println!("{}", name);
                    for entry in log {
                        println!("  {}", entry);
                    }
                    println!();
                }
            }
        }

        log::info!("quit");

        helptags();

        if let TaskType::Install = self.task_type {
            for p in pending.lock().unwrap().iter() {
                log::info!("delete {:?}", p.path());
                let _ = fs::remove_dir_all(p.path());
            }
        }

        let failures = failures.lock().unwrap();
        failures.clone()
    }
}

fn helptags() {
    // Detach the editor completely: it stays alive after the command
    // (normal mode), so an inherited stdin/stderr keeps it holding the
    // caller's terminal or pipe. Through a pipe that is a hang (the
    // pipeline never EOFs while the child lives); on a terminal it is an
    // input-rate collapse (the child races the shell for input). Waiting
    // for it would reintroduce both.
    #[allow(clippy::zombie_processes)] // deliberate: the editor must outlive pack
    match process::Command::new("vim")
        .arg("--not-a-term")
        .arg("-c")
        .arg("silent! helptags ALL")
        .stdin(process::Stdio::null())
        .stdout(process::Stdio::null())
        .stderr(process::Stdio::null())
        .spawn()
    {
        Ok(_) => (),
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                process::Command::new("nvim")
                    .arg("--headless")
                    .arg("-c")
                    .arg("silent! helptags ALL")
                    .stdin(process::Stdio::null())
                    .stdout(process::Stdio::null())
                    .stderr(process::Stdio::null())
                    .spawn()
                    .expect("Error opening nvim");
            } else {
                panic!("Somthing happened when calling vim!")
            }
        }
    }
}

fn setup_signal() -> io::Result<Receiver<()>> {
    let (s, r) = bounded(10);
    let mut signals = Signals::new([SIGTERM, SIGINT])?;

    thread::spawn(move || {
        // One signal exits. The graceful path (disconnect the quit channel,
        // the workers observe it, the run winds down) works — but what
        // follows the run is the regeneration (_pack.vim, the packfile)
        // written through a slow overlay, and that is outside the quit
        // chain's reach: observed live, the run returned and the
        // regeneration kept the shell hostage for many minutes. A single
        // signal now exits the whole process instead: the in-flight work
        // dies with it and the regeneration is never reached. The channel
        // stays connected so the workers keep working until then.
        let _keep = s;
        for _ in &mut signals {
            process::exit(130);
        }
    });
    Ok(r)
}
