use std::sync::{Arc, Mutex};

use crate::game::Game;
use crate::proto::{game_message::*, NodeRole};
use crate::ui::input::Input;
use crate::{comm::Communicator, node::Node, state::State};

pub struct Normal {
    state: Arc<Mutex<State>>,
    comm: Arc<Communicator>,
}

impl Normal {
    pub fn new(state: Arc<Mutex<State>>, comm: Arc<Communicator>) -> Self {
        Self { state, comm }
    }
}

impl Node for Normal {
    fn handle_input(&mut self, input: Input) {
        match input {
            Input::Turn(direction) => {
                todo!()
            }

            Input::Escape => {
                let mut state = self.state.lock().unwrap();
                state.exited = true;
            }

            Input::New => {
                let mut state = self.state.lock().unwrap();
                state.game = Game::from_config(&state.config);
                state.id = 0;
                state.game.spawn_snake(0);
                state.role = NodeRole::Master;
            }

            Input::Join => {
                let mut state = self.state.lock().unwrap();
                *state = State::new();
                state.role = NodeRole::Normal;

                todo!();
            }

            Input::View => {
                let mut state = self.state.lock().unwrap();
                *state = State::new();
                state.role = NodeRole::Viewer;

                todo!();
            }
        }
    }

    fn handle_ping(&mut self, ping: PingMsg, addr: std::net::SocketAddr, seq: i64) {
        todo!()
    }

    fn handle_steer(&mut self, steer: SteerMsg, addr: std::net::SocketAddr, seq: i64) {
        todo!()
    }

    fn handle_ack(
        &mut self,
        ack: AckMsg,
        addr: std::net::SocketAddr,
        sender: i32,
        receiver: i32,
        seq: i64,
    ) {
        todo!()
    }

    fn handle_state(&mut self, state: StateMsg, addr: std::net::SocketAddr, seq: i64) {
        todo!()
    }

    fn handle_announcement(&mut self, announcement: AnnouncementMsg, seq: i64) {
        todo!()
    }

    fn handle_join(&mut self, join: JoinMsg, seq: i64) {
        todo!()
    }

    fn handle_error(&mut self, error: ErrorMsg, seq: i64) {
        todo!()
    }

    fn handle_discover(&mut self, discover: DiscoverMsg, addr: std::net::SocketAddr, seq: i64) {
        todo!()
    }

    fn handle_role_change(
        &mut self,
        role_change: RoleChangeMsg,
        addr: std::net::SocketAddr,
        sender: i32,
        receiver: i32,
        seq: i64,
    ) {
        todo!()
    }
}
