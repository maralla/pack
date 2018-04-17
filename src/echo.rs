use std::sync::Mutex;
use std::io::{self, Write};

use termion::{clear, color, cursor};

pub fn async_print(line: u16, right: u16, msg: &str) {
    lazy_static! {
        static ref MUTEX: Mutex<i32> = Mutex::new(0);
    }

    let _ = MUTEX.lock().unwrap();
    print!("{}", cursor::Hide);
    print!(
        "{}{}{}{}",
        cursor::Up(line),
        msg,
        cursor::Left(right),
        cursor::Down(line)
    );
    print!("{}", cursor::Show);
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle.flush().unwrap();
}

pub fn character<C: color::Color>(line: u16, offset: u16, c: char, char_color: C) {
    async_print(
        line,
        offset + 1,
        &format!(
            "{}{}{}{}",
            cursor::Right(offset),
            color::Fg(char_color),
            c,
            color::Fg(color::Reset)
        ),
    );
}

pub fn inline_message(line: u16, offset: u16, msg: &str) {
    async_print(
        line,
        offset + msg.len() as u16,
        &format!("{}{}{}", cursor::Right(offset), clear::UntilNewline, msg),
    );
}

pub fn message(line: u16, offset: u16, msg: &str) {
    async_print(line, offset + msg.len() as u16, msg);
}
