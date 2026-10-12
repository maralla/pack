
use std::io::{self, IsTerminal, Write};
use std::sync::{Mutex, OnceLock};
use termion::{clear, color, cursor};

static MUTEX: OnceLock<Mutex<u16>> = OnceLock::new();
static PLAIN: OnceLock<bool> = OnceLock::new();

/// Whether output goes to a non-interactive sink (a pipe, a file, CI).
///
/// The cursor-addressed UI needs a real terminal (terminal size, cursor
/// movement); on a non-terminal it is unusable and `terminal_size()` fails
/// outright. In plain mode callers print one plain line per item instead of
/// driving the cursor.
pub fn plain() -> bool {
    *PLAIN.get_or_init(|| !io::stdout().is_terminal())
}

pub fn line() -> u16 {
    if plain() {
        // No line reservation in plain mode: nothing cursor-addresses.
        return 0;
    }
    let index = next_line_index();
    println!();
    index
}

/// The interactive line counter (see [`line`]).
fn next_line_index() -> u16 {
    let mut v = MUTEX.get_or_init(|| Mutex::new(0)).lock().unwrap();
    let current = *v;
    *v += 1;
    current
}

pub fn async_print(line: u16, right: u16, msg: &str) {
    let current = MUTEX.get_or_init(|| Mutex::new(0)).lock().unwrap();
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
        // The counter itself, not the plain-mode gate around it (plain mode
        // depends on whether stdout is a terminal, which the test harness
        // cannot control).
        next_line_index();
        next_line_index();
        assert_eq!(next_line_index(), 2);
    }
}
