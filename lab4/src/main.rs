#![allow(dead_code)]
#![allow(unused)]

use std::{
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use lab4::{
    config::Config,
    logic::Game,
    proto::NodeRole,
    state::State,
    ui::{self, input::Input},
};
use ratatui::{prelude::CrosstermBackend, Terminal};

const MULTIADDR: &'static str = "239.192.0.4:9192";
const CONFIG_PATH: &'static str = "snakes.toml";

fn main() -> io::Result<()> {
    let state = Arc::new(Mutex::new(State::new()));
    let mut config = Config::load(CONFIG_PATH).unwrap_or_default();

    config.field.width = 60;
    config.field.height = 40;

    ui::utils::set_panic_hook();
    ui::utils::setup()?;

    let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    loop {
        {
            let state = state.lock().unwrap();
            term.draw(|frame| ui::main::ui(frame, &state))?;
        }

        match ui::input::read(Some(Duration::from_millis(20)))? {
            Some(Input::Escape) => break,

            Some(Input::NewGame) => {
                let mut game_state = state.lock().unwrap();

                let mut game = Game::from_cfg(&config);

                if !game.spawn_snake(0) {
                    panic!("uwuwu");
                }

                game_state.game = Some(game);
                game_state.role = NodeRole::Master;

                thread::spawn({
                    let state = Arc::clone(&state);
                    move || loop {
                        {
                            let mut state = state.lock().unwrap();

                            if let Some(ref mut game) = state.game {
                                game.step();
                            }
                        }

                        thread::sleep(Duration::from_millis(100));
                    }
                });
            }

            Some(Input::Turn(direction)) => {
                if let Some(ref mut game) = state.lock().unwrap().game {
                    if let Some(snake) = game.snakes.iter_mut().find(|s| s.id == 0) {
                        snake.update_direction(direction);
                    }
                }
            }

            _ => {}
        }
    }

    ui::utils::reset_panic_hook();
    ui::utils::teardown()?;

    Ok(())
}
