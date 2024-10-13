use std::{
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::{
    net::Communicator,
    state::{Announcement, State},
    ui::input::Input,
};
use crate::{proto::game_message::Type, ui};

pub fn ui_thread(state: Arc<Mutex<State>>) -> io::Result<()> {
    ui::utils::set_panic_hook();
    ui::utils::setup()?;
    let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    loop {
        thread::sleep(Duration::from_millis(20));

        let state = state.lock().unwrap();
        if state.exited {
            break;
        }

        term.draw(|frame| ui::main::ui(frame, &state)).unwrap();
    }

    ui::utils::reset_panic_hook();
    ui::utils::teardown()?;

    Ok(())
}

pub fn announcement_reaper_thread(state: Arc<Mutex<State>>) {
    loop {
        thread::sleep(Duration::from_secs(3));

        let mut state = state.lock().unwrap();
        state
            .announcements
            .retain(|a| a.elapsed() < Duration::from_secs(3));
    }
}

pub fn announcement_monitor_thread(state: Arc<Mutex<State>>, comm: Arc<Communicator>) {
    loop {
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
}

pub fn input_thread(state: Arc<Mutex<State>>) {
    loop {
        match ui::input::read(None).unwrap() {
            Some(Input::Escape) => {
                let mut state = state.lock().unwrap();
                state.exited = true;
                break;
            }

            _ => {}
        }
    }
}
