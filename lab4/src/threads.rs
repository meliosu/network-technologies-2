use std::{
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crossbeam::channel::{tick, Receiver, Sender};
use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::{
    comm::Communicator,
    proto::{
        game_message::{AnnouncementMsg, Type},
        GameMessage, NodeRole,
    },
    state::{Announcement, State},
    ui::{self, input::Input},
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

pub fn input(channel: Sender<Input>) -> io::Result<()> {
    loop {
        if let Some(input) = ui::input::read()? {
            channel.send(input).unwrap();
        }
    }
}

pub fn announcement_sender(state: Arc<Mutex<State>>, comm: Arc<Communicator>) -> io::Result<()> {
    for _ in tick(SECOND) {
        let announcement = {
            let state = state.lock().unwrap();

            if state.role != NodeRole::Master {
                continue;
            }

            AnnouncementMsg::new((&state.game).into(), 0)
        };

        comm.send_mcast(&announcement)?;
    }

    Ok(())
}

pub fn announcement_receiver(state: Arc<Mutex<State>>, comm: Arc<Communicator>) -> io::Result<()> {
    loop {
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
}

pub fn announcement_reaper(state: Arc<Mutex<State>>) {
    for _ in tick(3 * SECOND) {
        let mut state = state.lock().unwrap();

        state
            .announcements
            .retain(|_, announcement| announcement.time.elapsed() < 3 * SECOND);
    }
}

pub fn ucast_sender(
    comm: Arc<Communicator>,
    channel: Receiver<(GameMessage, SocketAddr)>,
) -> io::Result<()> {
    loop {
        let (msg, addr) = channel.recv().unwrap();
        comm.send_ucast(addr, &msg)?;
    }
}

pub fn ucast_receiver(
    comm: Arc<Communicator>,
    channel: Sender<(GameMessage, SocketAddr)>,
) -> io::Result<()> {
    loop {
        let (msg, addr) = comm.recv_ucast()?;
        channel.send((msg, addr)).unwrap();
    }
}
