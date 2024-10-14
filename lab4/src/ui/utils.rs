use std::io;

use crossterm::{cursor, execute, terminal};

pub fn setup() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)
}

pub fn teardown() -> io::Result<()> {
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), terminal::LeaveAlternateScreen, cursor::Show)
}

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        _ = teardown();
        println!("{info}")
    }));
}

pub fn reset_panic_hook() {
    _ = std::panic::take_hook();
}
