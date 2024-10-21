use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use crossbeam::channel::Receiver;

use crate::{
    comm::Communicator,
    id::Generator,
    proto::{game_message::*, GameMessage, NodeRole},
    state::State,
    ui::input::Input,
};

pub struct PeerMessage {
    time: Instant,
    msg: GameMessage,
    addr: SocketAddr,
}

pub struct MasterMessage {
    time: Instant,
    msg: GameMessage,
}

pub struct Node {
    state: State,
    comm: Communicator,
    message_channel: Receiver<(GameMessage, SocketAddr)>,
    input_channel: Receiver<Input>,
    turn_channel: Receiver<Duration>,
    interval_channel: Receiver<Duration>,
    seq_gen: Generator,
    peer_msgs: Vec<PeerMessage>,
    master_msgs: Vec<MasterMessage>,
}

impl Node {
    pub fn new(
        state: State,
        comm: Communicator,
        message_channel: Receiver<(GameMessage, SocketAddr)>,
        input_channel: Receiver<Input>,
        turn_channel: Receiver<Duration>,
        interval_channel: Receiver<Duration>,
    ) -> Self {
        Self {
            state,
            comm,
            message_channel,
            input_channel,
            turn_channel,
            interval_channel,
            seq_gen: Generator::new(),
            peer_msgs: Vec::new(),
            master_msgs: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            crossbeam::select! {
                recv(self.message_channel) -> msg => self.handle_message(msg.unwrap()),
                recv(self.input_channel) -> input => self.handle_input(input.unwrap()),
                recv(self.turn_channel) -> turn => self.on_turn(turn.unwrap()),
                recv(self.interval_channel) -> interval => self.on_interval(interval.unwrap()),
            }
        }
    }

    fn handle_message(&mut self, (msg, addr): (GameMessage, SocketAddr)) {
        let Some(r#type) = msg.r#type else {
            return;
        };

        let role = self.state.role();

        match r#type {
            Type::Ping(ping_msg) => todo!(),

            Type::Steer(steer_msg) => if role == NodeRole::Master {},

            Type::Ack(ack_msg) => todo!(),

            Type::State(state_msg) => {
                if role != NodeRole::Master {
                    todo!()
                }
            }

            Type::RoleChange(role_change_msg) => {
                if role == NodeRole::Master {
                    todo!()
                }
            }

            Type::Join(_join_msg) => {
                if role == NodeRole::Master {
                    todo!()
                }
            }

            Type::Announcement(announcement_msg) => {
                // ignore
            }

            Type::Error(_) => {
                // ignore
            }

            Type::Discover(_) => {
                // ignore
            }
        }
    }

    fn handle_input(&mut self, input: Input) {
        let role = self.state.role();

        match input {
            Input::Turn(direction) => {
                if role == NodeRole::Master {
                    self.state.turn_self(direction);
                } else {
                    let steer = SteerMsg::new(direction, self.free_seq());
                    self.send_to_master(steer);
                }
            }

            Input::Escape => {
                self.state.exit();
            }

            Input::New => {
                self.state.new_master();
                self.clean_messages();
            }

            Input::Join(idx) => {
                if let Some((addr, announcement)) = self.state.nth_announcement(idx) {
                    self.clean_messages();
                    self.state.new_normal();
                    let name = self.state.player_name();
                    let join = JoinMsg::new(name, announcement.game_name, NodeRole::Normal, 0);
                    self.send_to_addr(join, addr);
                }
            }

            Input::View(idx) => {
                if let Some((addr, announcement)) = self.state.nth_announcement(idx) {
                    self.clean_messages();
                    self.state.new_viewer();
                    let name = self.state.player_name();
                    let join = JoinMsg::new(name, announcement.game_name, NodeRole::Normal, 0);
                    self.send_to_addr(join, addr);
                }
            }
        }
    }

    fn on_turn(&mut self, turn: Duration) {
        todo!()
    }

    fn on_interval(&mut self, interval: Duration) {
        todo!()
    }

    fn send_to_master(&mut self, msg: GameMessage) {
        let master = self.state.master();
        self.comm.send_ucast(master, &msg).unwrap();
        self.master_msgs.push(MasterMessage {
            time: Instant::now(),
            msg,
        });
    }

    fn send_to_addr(&mut self, msg: GameMessage, addr: SocketAddr) {
        self.comm.send_ucast(addr, &msg).unwrap();
        self.peer_msgs.push(PeerMessage {
            time: Instant::now(),
            addr,
            msg,
        });
    }

    fn free_seq(&mut self) -> i64 {
        self.seq_gen.next()
    }

    fn clean_messages(&mut self) {
        self.peer_msgs = Vec::new();
        self.master_msgs = Vec::new();
    }
}
