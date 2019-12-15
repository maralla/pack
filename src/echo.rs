use lazy_static::lazy_static;
use std::io::{self, Write};
use std::sync::Mutex;
use termion::{clear, color, cursor};

lazy_static! {
    static ref MUTEX: Mutex<u16> = Mutex::new(0);
}

pub fn line() -> u16 {
    let mut v = MUTEX.lock().unwrap();
    let current = *v;
    *v = *v + 1;
    println!();
    current
}

pub fn async_print(line: u16, right: u16, msg: &str) {
    let current = MUTEX.lock().unwrap();
    let offset = *current - line;
    print!("{}", cursor::Hide);
    print!(
        "{}{}{}{}",
        cursor::Up(offset),
        msg,
        cursor::Left(right),
        cursor::Down(offset)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line() {
        line();
        line();
        assert_eq!(line(), 2);
    }
}
