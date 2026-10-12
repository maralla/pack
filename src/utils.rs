use crate::echo;
use crate::{Error, Result};

use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time;
use termion::color;
use walkdir::WalkDir;

const SPINNER_CHARS: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
const DEFAULT_EDITOR: &str = "vi";

macro_rules! die {
    ($($arg:tt)*) => ({
        use std::io::Write;
        (writeln!(&mut ::std::io::stderr(), $($arg)*)).expect("stderr");
        ::std::process::exit(1)
    })
}

pub struct Spinner {
    tx: Option<Sender<bool>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn spin(x: u16, y: u16) -> Spinner {
        if echo::plain() {
            // Non-interactive sink: no spinner (and no spinner thread whose
            // death could take pack down with it).
            return Spinner {
                tx: None,
                handle: None,
            };
        }
        let (tx, rx) = channel();
        let handle = thread::spawn(move || {
            for &c in SPINNER_CHARS.iter().cycle() {
                if rx.try_recv().is_ok() {
                    break;
                }

                echo::character(x, y, c, color::Reset);
                thread::sleep(time::Duration::from_millis(100));
            }
        });
        Spinner {
            tx: Some(tx),
            handle: Some(handle),
        }
    }

    /// Best effort: a spinner thread that already died (for example the
    /// WSL `nanosleep` EINVAL panic, issue #45) must not take pack down
    /// with a panic in `unwrap()` here.
    pub fn stop(self) {
        if let Some(tx) = self.tx {
            let _ = tx.send(true);
        }
        if let Some(handle) = self.handle {
            let _ = handle.join();
        }
    }
}

pub fn copy_directory<P: AsRef<Path>>(src: P, dst: P) -> Result<()> {
    let wd = WalkDir::new(&src);
    for entry in wd {
        let e = entry?;
        let path = e.path();
        let stem = path.strip_prefix(&src)?;
        let new_path = dst.as_ref().join(stem);
        if path.is_dir() {
            fs::create_dir_all(new_path)?;
        } else if path.is_file() {
            fs::copy(path, new_path)?;
        }
    }
    Ok(())
}

fn get_editor() -> Option<String> {
    let term = env::var("TERM");
    if term.map(|t| t == "dumb").unwrap_or(true) {
        None
    } else {
        Some(
            env::var("PACK_EDITOR")
                .into_iter()
                .chain(env::var("EDITOR"))
                .next()
                .unwrap_or_else(|| String::from(DEFAULT_EDITOR)),
        )
    }
}

pub fn open_editor<P: AsRef<Path>>(path: P) -> Result<()> {
    let editor = get_editor().ok_or(Error::Editor)?;
    process::Command::new(editor)
        .arg(path.as_ref().as_os_str())
        .spawn()?
        .wait()?;
    Ok(())
}
