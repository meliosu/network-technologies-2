use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use crossbeam::channel::Receiver;

use crate::{comm::Communicator, proto::GameMessage, state::State, ui::input::Input};

pub struct Node {
    state: State,
    comm: Communicator,
    message_channel: Receiver<(GameMessage, SocketAddr)>,
    input_channel: Receiver<Input>,
    turn_channel: Receiver<Instant>,
    interval_channel: Receiver<Instant>,
}

impl Node {
    pub fn new(
        state: State,
        comm: Communicator,
        message_channel: Receiver<(GameMessage, SocketAddr)>,
        input_channel: Receiver<Input>,
        turn_channel: Receiver<Instant>,
        interval_channel: Receiver<Instant>,
    ) -> Self {
        Self {
            state,
            comm,
            message_channel,
            input_channel,
            turn_channel,
            interval_channel,
        }
    }

    pub fn run(&self) {
        loop {
            crossbeam::select! {
                recv(self.message_channel) -> msg => self.handle_message(msg.unwrap()),
                recv(self.input_channel) -> input => self.handle_input(input.unwrap()),
                recv(self.turn_channel) -> _ => self.on_turn(),
                recv(self.interval_channel) -> _ => self.on_interval(),
            }
        }
    }

    fn handle_message(&self, (msg, addr): (GameMessage, SocketAddr)) {
        todo!()
    }

    fn handle_input(&self, input: Input) {
        todo!()
    }

    fn on_turn(&self) {
        todo!()
    }

    fn on_interval(&self) {
        todo!()
    }
}
