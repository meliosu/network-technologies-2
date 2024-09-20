#![allow(dead_code)]

use std::io;
use std::time::Instant;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use layout::Layout;
use ratatui::widgets::{Block, StatefulWidget};
use ratatui::{prelude::*, widgets::Gauge};

fn main() -> io::Result<()> {
    setup()?;
    set_panic_hook();

    let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut infos = [Info::new(), Info::new(), Info::new()];

    loop {
        term.draw(|frame| {
            let constraints = Constraint::from_maxes(std::iter::repeat(3).take(infos.len()));
            let layouts = Layout::vertical(constraints).split(frame.area());

            for (info, rect) in infos.iter_mut().zip(layouts.into_iter()) {
                frame.render_stateful_widget(InfoView, rect.clone(), info);
            }
        })?;

        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => break,

            _ => {}
        }
    }

    teardown()?;
    reset_panic_hook();

    Ok(())
}

fn setup() -> io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)
}

fn teardown() -> io::Result<()> {
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, cursor::Show)
}

fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let _ = teardown();
        println!("{info}");
    }));
}

fn reset_panic_hook() {
    let _ = std::panic::take_hook();
}

struct Info {
    file: String,
    len: u64,
    completed: u64,
    transfered: u64,
    start: Instant,
}

impl Info {
    pub fn new() -> Self {
        Self {
            file: "hello.txt".to_string(),
            len: 1024,
            completed: 235,
            transfered: 100,
            start: Instant::now(),
        }
    }
}

struct InfoView;

impl StatefulWidget for InfoView {
    type State = Info;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let header = Span::from(format!("{}", state.file));
        let block = Block::bordered().title(header);
        let gauge = Gauge::default()
            .ratio(state.completed as f64 / state.len as f64)
            .block(block)
            .green();

        gauge.render(area, buf);
    }
}
