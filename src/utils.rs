use std::env;
use std::fs;
use std::thread;
use std::time;
use std::process;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};

use termion::color;
use {Result, Error};
use walkdir::WalkDir;
use echo;

const SPINNER_CHARS: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇',
                                   '⠏'];
const DEFAULT_EDITOR: &'static str = "vi";

macro_rules! die {
    ($($arg:tt)*) => {
        use std::io::Write;
        (writeln!(&mut ::std::io::stderr(), $($arg)*)).expect("stderr");
        ::std::process::exit(1);
    }
}

pub struct Spinner {
    tx: Sender<bool>,
    handle: thread::JoinHandle<()>,
}

impl Spinner {
    pub fn spin(x: u16, y: u16) -> Spinner {
        let (tx, rx) = channel();
        let handle = thread::spawn(move || for &c in SPINNER_CHARS.iter().cycle() {
            if let Ok(_) = rx.try_recv() {
                break;
            }

            echo::character(x, y, c, color::Reset);
            thread::sleep(time::Duration::from_millis(100));
        });
        Spinner {
            tx: tx,
            handle: handle,
        }
    }

    pub fn stop(self) {
        self.tx.send(true).unwrap();
        self.handle.join().unwrap();
    }
}

pub fn copy_directory<P: AsRef<Path>>(src: P, dst: P) -> Result<()> {
    for entry in WalkDir::new(&src).into_iter() {
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
        Some(env::var("PACK_EDITOR")
            .into_iter()
            .chain(env::var("EDITOR"))
            .next()
            .unwrap_or(DEFAULT_EDITOR.to_string()))
    }
}

pub fn open_editor<P: AsRef<Path>>(path: P) -> Result<()> {
    let editor = get_editor().ok_or(Error::Editor)?;
    process::Command::new(editor).arg(path.as_ref().as_os_str()).spawn()?.wait()?;
    Ok(())
}
