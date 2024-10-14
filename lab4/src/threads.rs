use std::{
    io,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crossbeam::channel::tick;
use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::{
    comm::Communicator,
    proto::game_message::{AnnouncementMsg, Type},
    state::{Announcement, State},
    ui,
};

const SECOND: Duration = Duration::from_secs(1);

pub fn ui(state: Arc<Mutex<State>>) -> io::Result<()> {
    let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    ui::utils::set_panic_hook();
    ui::utils::setup()?;

    for _ in tick(SECOND / 50) {
        let state = state.lock().unwrap();
        term.draw(|frame| ui::view::render(frame, &state))?;
    }

    ui::utils::reset_panic_hook();
    ui::utils::teardown()?;

    Ok(())
}

pub fn announcement_sender(state: Arc<Mutex<State>>, comm: Arc<Communicator>) -> io::Result<()> {
    for _ in tick(SECOND) {
        let announcement = {
            let state = state.lock().unwrap();
            AnnouncementMsg::new((&state.game).into(), 0)
        };

        comm.send_mcast(&announcement)?;
    }

    Ok(())
}

pub fn announcement_receiver(state: Arc<Mutex<State>>, comm: Arc<Communicator>) -> io::Result<()> {
    for _ in tick(SECOND) {
        let (msg, addr) = comm.recv_mcast()?;

        if let Some(Type::Announcement(announcements)) = msg.r#type {
            if let Some(announcement) = announcements.games.first() {
                let mut state = state.lock().unwrap();

                state.announcements.insert(
                    addr,
                    Announcement {
                        time: Instant::now(),
                        announcement: announcement.clone(),
                    },
                );
            }
        }
    }

    Ok(())
}

pub fn announcement_reaper(state: Arc<Mutex<State>>) {
    for _ in tick(3 * SECOND) {
        let mut state = state.lock().unwrap();

        state
            .announcements
            .retain(|_, announcement| announcement.time.elapsed() < 3 * SECOND);
    }
}
