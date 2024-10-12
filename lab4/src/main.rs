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
    logic::Game,
    net::Communicator,
    proto::{
        game_message::{AnnouncementMsg, Type},
        NodeRole,
    },
    state::{Announcement, State},
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
        let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        move || {
            ui::utils::set_panic_hook();
            ui::utils::setup().unwrap();

            loop {
                thread::sleep(Duration::from_millis(20));

                let state = state.lock().unwrap();
                if state.exited {
                    break;
                }

                term.draw(|frame| ui::main::ui(frame, &state)).unwrap();
            }

            ui::utils::reset_panic_hook();
            ui::utils::teardown().unwrap();
        }
    });

    thread::spawn({
        let state = Arc::clone(&state);
        let comm = Arc::clone(&comm);
        move || loop {
            match ui::input::read(None).unwrap() {
                Some(Input::Escape) => {
                    let mut state = state.lock().unwrap();
                    state.exited = true;
                    break;
                }

                _ => {}
            }
        }
    });

    // ok
    thread::spawn({
        let state = Arc::clone(&state);
        move || loop {
            thread::sleep(Duration::from_secs(3));

            let mut state = state.lock().unwrap();
            state
                .announcements
                .retain(|a| a.elapsed() < Duration::from_secs(3));
        }
    });

    // ok
    thread::spawn({
        let state = Arc::clone(&state);
        let comm = Arc::clone(&comm);
        move || loop {
            let (msg, addr) = comm.recv_multicast().unwrap();

            let announces = match msg.r#type {
                Some(Type::Announcement(announce)) => announce.games,
                _ => continue,
            };

            let mut state = state.lock().unwrap();

            for announcement in &mut state.announcements {
                if announcement.addr == addr {
                    announcement.refresh();
                }
            }

            if state
                .announcements
                .iter()
                .find(|Announcement { addr: a, .. }| *a == addr)
                .is_none()
            {
                state
                    .announcements
                    .extend(announces.into_iter().map(|a| Announcement::new(addr, a)));
            }
        }
    });

    ui_handle.join().unwrap();

    Ok(())
}
