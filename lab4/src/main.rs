#![allow(dead_code)]
#![allow(unused)]

use std::{
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{prelude::CrosstermBackend, Terminal};

use lab4::{
    config::Config,
    logic::{Game, Snake},
    net::Communicator,
    proto::{
        game_message::{AnnouncementMsg, Type},
        Direction, NodeRole,
    },
    state::{Announcement, State},
    threads::{announcement_monitor_thread, announcement_reaper_thread, input_thread, ui_thread},
    ui::{self, input::Input},
};

const MULTIADDR: &'static str = "239.192.0.4:9192";

fn main() -> io::Result<()> {
    // ok
    let state = Arc::new(Mutex::new(State::new()));
    let comm = Arc::new(Communicator::new(MULTIADDR)?);

    // ok
    let ui_handle = thread::spawn({
        let state = Arc::clone(&state);
        move || {
            ui_thread(state).unwrap();
        }
    });

    thread::spawn({
        let state = Arc::clone(&state);
        move || {
            input_thread(state);
        }
    });

    // ok
    thread::spawn({
        let state = Arc::clone(&state);
        move || {
            announcement_reaper_thread(state);
        }
    });

    // ok
    thread::spawn({
        let state = Arc::clone(&state);
        let comm = Arc::clone(&comm);
        move || {
            announcement_monitor_thread(state, comm);
        }
    });

    ui_handle.join().unwrap();

    Ok(())
}
