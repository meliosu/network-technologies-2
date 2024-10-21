use std::{
    io,
    net::SocketAddr,
    thread,
    time::{Duration, Instant},
};

use crossbeam::channel::{tick, Sender};
use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::{
    comm::Communicator,
    proto::{game_message::Type, GameMessage},
    state::State,
    ui::{self, input::Input},
};

const SECOND: Duration = Duration::from_secs(1);

pub fn ui(state: State) -> io::Result<()> {
    let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    ui::utils::set_panic_hook();
    ui::utils::setup()?;

    for _ in tick(SECOND / 50) {
        let state = state.lock();

        if state.exited {
            break;
        }

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

pub fn announcement_sender(state: State, comm: Communicator) -> io::Result<()> {
    for _ in tick(SECOND) {
        if let Some(announcement) = state.get_announcement() {
            comm.send_mcast(&announcement)?;
        }
    }

    Ok(())
}

pub fn announcement_receiver(state: State, comm: Communicator) -> io::Result<()> {
    loop {
        let (msg, addr) = comm.recv_mcast()?;

        if let Some(Type::Announcement(announcement)) = msg.r#type {
            if let Some(announcement) = announcement.games.first() {
                state.add_announcement(announcement.clone(), addr);
            }
        }
    }
}

pub fn announcement_reaper(state: State) {
    for _ in tick(3 * SECOND) {
        state.remove_announcements();
    }
}

pub fn ucast_receiver(comm: Communicator, channel: Sender<(GameMessage, SocketAddr)>) {
    loop {
        let (msg, addr) = comm.recv_ucast().unwrap();
        channel.send((msg, addr)).unwrap();
    }
}

pub fn turn_tick(state: State, channel: Sender<Instant>) {
    loop {
        let delay = state.delay();
        thread::sleep(delay);
        channel.send(Instant::now()).unwrap();
    }
}

pub fn interval_tick(state: State, channel: Sender<Instant>) {
    loop {
        let delay = state.delay();
        thread::sleep(delay);
        channel.send(Instant::now()).unwrap();
    }
}
