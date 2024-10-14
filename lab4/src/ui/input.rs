use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent};

use crate::proto::Direction;

pub enum Input {
    Turn(Direction),
    Escape,
    New,
    Join,
    View,
}

pub fn read() -> io::Result<Option<Input>> {
    let event = match event::read()? {
        Event::Key(KeyEvent {
            code: KeyCode::Up | KeyCode::Char('w'),
            ..
        }) => Some(Input::Turn(Direction::Up)),

        Event::Key(KeyEvent {
            code: KeyCode::Down | KeyCode::Char('s'),
            ..
        }) => Some(Input::Turn(Direction::Down)),

        Event::Key(KeyEvent {
            code: KeyCode::Left | KeyCode::Char('a'),
            ..
        }) => Some(Input::Turn(Direction::Left)),

        Event::Key(KeyEvent {
            code: KeyCode::Right | KeyCode::Char('d'),
            ..
        }) => Some(Input::Turn(Direction::Right)),

        Event::Key(KeyEvent {
            code: KeyCode::Esc, ..
        }) => Some(Input::Escape),

        Event::Key(KeyEvent {
            code: KeyCode::Char('n'),
            ..
        }) => Some(Input::New),

        Event::Key(KeyEvent {
            code: KeyCode::Char('j'),
            ..
        }) => Some(Input::Join),

        Event::Key(KeyEvent {
            code: KeyCode::Char('v'),
            ..
        }) => Some(Input::View),

        _ => None,
    };

    Ok(event)
}
