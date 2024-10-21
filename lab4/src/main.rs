use std::thread;

use lab4::{comm::Communicator, node::Node, state::State, threads};

fn main() {
    let state = State::new();
    let comm = Communicator::new("240.0.0.1").unwrap();

    let (ucast_tx, ucast_rx) = crossbeam::channel::unbounded();
    let (input_tx, input_rx) = crossbeam::channel::unbounded();
    let (turn_tx, turn_rx) = crossbeam::channel::unbounded();
    let (interval_tx, interval_rx) = crossbeam::channel::unbounded();

    let ui_handle = thread::spawn({
        let state = state.clone();
        move || threads::ui(state)
    });

    thread::spawn(move || threads::input(input_tx));

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

    thread::spawn({
        let comm = comm.clone();
        move || threads::ucast_receiver(comm, ucast_tx)
    });

    thread::spawn({
        let state = state.clone();
        move || threads::turn_tick(state, turn_tx)
    });

    thread::spawn({
        let state = state.clone();
        move || threads::interval_tick(state, interval_tx)
    });

    thread::spawn({
        let state = state.clone();
        let comm = comm.clone();
        move || {
            Node::new(state, comm, ucast_rx, input_rx, turn_rx, interval_rx).run();
        }
    });

    ui_handle.join().unwrap().unwrap();
}
