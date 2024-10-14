use std::{
    io,
    sync::{Arc, Mutex},
    thread,
};

use crossbeam::channel;

use lab4::comm::Communicator;
use lab4::state::State;
use lab4::threads;

fn main() -> io::Result<()> {
    let comm = Arc::new(Communicator::new("224.0.0.1:1337")?);
    let state = Arc::new(Mutex::new(State::new()));

    thread::spawn({
        let state = state.clone();
        move || threads::ui(state).unwrap()
    });

    thread::spawn({
        let state = state.clone();
        let comm = comm.clone();
        move || threads::announcement_receiver(state, comm)
    });

    thread::spawn({
        let state = state.clone();
        let comm = comm.clone();
        move || threads::announcement_sender(state, comm)
    });

    thread::spawn({
        let state = state.clone();
        move || threads::announcement_reaper(state)
    });

    Ok(())
}
