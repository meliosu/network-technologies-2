use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent};

use crate::proto::Direction;

pub enum Input {
    Turn(Direction),
    Escape,
    Enter,
    NewGame,
}

pub fn read(timeout: Option<Duration>) -> io::Result<Option<Input>> {
    if let Some(timeout) = timeout {
        if !event::poll(timeout)? {
            return Ok(None);
        }
    }

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
            code: KeyCode::Enter,
            ..
        }) => Some(Input::Enter),

        Event::Key(KeyEvent {
            code: KeyCode::Char('n'),
            ..
        }) => Some(Input::NewGame),

        _ => None,
    };

    Ok(event)
}
