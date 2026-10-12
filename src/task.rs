use crate::echo;
use crate::package::Package;
use crate::utils::Spinner;
use crate::Error;
use crate::Result;

use crossbeam_channel::bounded;
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;
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
    packs: Vec<Package>,
    thread_num: usize,
    /// Show the changelog overview of updates (opt-in: default false).
    pub changelog: bool,
}

impl TaskManager {
    pub fn new(_task_type: TaskType, thread_num: usize) -> TaskManager {
        TaskManager {
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
                        if let Err(e) = pack.try_build().map_err(|e| Error::build(format!("{}", e)))
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
        let name_prefix = format!("    {}", msg);
        echo::message(line, 0, &format!("{} syncing", name_prefix));

        const MSG_MARGIN: u16 = 5;
        const SIGN_MARGIN: u16 = 3;

        macro_rules! print_err {
            ($err:expr) => {
                let msg = format!("{}", $err);
                echo::result_line(
                    line,
                    &name_prefix,
                    SIGN_MARGIN,
                    '✗',
                    color::Red,
                    MSG_MARGIN + pos,
                    &msg,
                );
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
            echo::result_line(
                line,
                &name_prefix,
                SIGN_MARGIN,
                '○',
                color::LightBlack,
                MSG_MARGIN + pos,
                "up to date",
            );
        } else if let (Err(e), status) = &result {
            spinner.stop();
            let msg = format!("{}", e);
            echo::result_line(
                line,
                &name_prefix,
                SIGN_MARGIN,
                '✗',
                color::Red,
                MSG_MARGIN + pos,
                &msg,
            );
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
                echo::result_line(
                    line,
                    &name_prefix,
                    SIGN_MARGIN,
                    '✓',
                    color::Green,
                    MSG_MARGIN + pos,
                    "done",
                );
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

    pub fn run<F>(self, func: F) -> Result<Vec<String>>
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

        // One flag, no quit channel: the signal sets it (see setup_signal),
        // the workers read it at their stop points, and the feeder stops
        // feeding and releases them with sentinels. The only thing that
        // runs to completion is the in-flight git call in the worker
        // thread; the reclone it would have started is skipped in
        // git::update, so the finish is quick and nothing new starts.
        setup_signal().map_err(|e| Error::Io(e))?;

        let threads = self.thread_num;
        let (tx, rx) = bounded::<Option<Package>>(threads);
        let failures = Arc::new(Mutex::new(vec![]));
        let changelogs = Arc::new(Mutex::new(Vec::<(String, Vec<String>)>::new()));

        let mut workers = Vec::with_capacity(threads);
        for _ in 0..threads {
            let rx = rx.clone();
            let failures = failures.clone();
            let changelogs = changelogs.clone();
            workers.push(thread::spawn(move || {
                // One level of threads: this thread does the job. The
                // take blocks only while the queue is empty, and the
                // sentinels release it; the job blocks only on the
                // in-flight git call, and the quit check right after it
                // stops everything else.
                while let Ok(Some(pack)) = rx.recv() {
                    let index = if echo::plain() { 0 } else { echo::line() };
                    let ok = Self::update(&pack, index, func, &changelogs);
                    if quitting() {
                        // Graceful shutdown, the part every earlier design
                        // missed: drain the queue. The workers exiting
                        // leave it full of packages nobody will collect;
                        // the sentinels cannot enter a full queue, the
                        // feeder sticks on them, and the run hangs. Drain
                        // everything (discarding it), stop at the sentinel,
                        // and exit: the queue empties, the sentinels get
                        // through, every blocked take releases, and the
                        // joins complete.
                        while let Ok(Some(_)) = rx.recv() {}
                        break;
                    }
                    if !ok {
                        let mut f = failures.lock().unwrap();
                        f.push(pack.name);
                    }
                }
            }));
        }
        if !self.packs.is_empty() && !echo::plain() {
            println!();
        }

        // The feeder: non-blocking sends with a quit-checked retry. A
        // blocking send here is quit-invisible: the queue fills, the
        // feeder sticks in send(), the sentinels never go out, and the
        // workers block on the empty queue — a deadlock the flag cannot
        // wake. try_send + retry keeps every wait observable.
        for pack in self.packs.iter() {
            let item = Some(pack.clone());
            loop {
                if quitting() {
                    break;
                }
                match tx.try_send(item.clone()) {
                    Ok(()) => break,
                    Err(crossbeam_channel::TrySendError::Full(_)) => {
                        thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Err(_) => break,
                }
            }
            if quitting() {
                break;
            }
        }
        // Always release the workers: the sentinels are the wake-up.
        for _ in 0..threads {
            let _ = tx.send(None);
        }
        for w in workers {
            let _ = w.join();
        }

        if quitting() {
            // Reached the end: the in-flight git calls finished (the
            // reclones were skipped in git::update), the workers exited,
            // the joins completed. Print the message and abort the run;
            // the caller skips the regeneration, and main returns.
            return Err(Error::Aborted);
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

        let failures = failures.lock().unwrap();
        Ok(failures.clone())
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

/// Set up the quit channel: a SIGINT/SIGTERM disconnects it, every worker
/// observes the disconnection and winds down, and the run aborts before the
/// regeneration (which is what actually held the shell hostage: the run
/// itself returned, then the regeneration of `_pack.vim` and the packfile
/// ran for minutes with nothing able to interrupt it).
/// The process-wide quit flag. A SIGINT/SIGTERM sets it; every thread
/// that can stop reads it at its stop points. No channel, no
/// disconnection semantics, no select multiplexing: a flag read is a
/// flag read, and the sentinels in `run` do the waking.
static QUITTING: std::sync::OnceLock<std::sync::atomic::AtomicBool> = std::sync::OnceLock::new();

pub fn quitting() -> bool {
    QUITTING
        .get_or_init(Default::default)
        .load(std::sync::atomic::Ordering::Relaxed)
}

fn set_quitting() {
    QUITTING
        .get_or_init(Default::default)
        .store(true, std::sync::atomic::Ordering::Relaxed);
}

fn setup_signal() -> io::Result<()> {
    let mut signals = Signals::new([SIGTERM, SIGINT])?;

    thread::spawn(move || {
        for _ in &mut signals {
            set_quitting();
            return;
        }
    });
    Ok(())
}
