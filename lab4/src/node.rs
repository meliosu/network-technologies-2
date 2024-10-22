use std::{
    collections::HashMap,
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
    active: HashMap<SocketAddr, Instant>,
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
            active: HashMap::new(),
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

        self.active.insert(addr, Instant::now());

        let role = self.state.role();

        match r#type {
            Type::Steer(steer_msg) => {
                if role == NodeRole::Master {
                    self.state.turn_snake_by_addr(addr, steer_msg.direction());
                }
            }

            Type::Ack(_) => {
                self.ack(msg.msg_seq);
            }

            Type::State(state_msg) => {
                if role != NodeRole::Master {
                    self.state.update(state_msg.state);
                }
            }

            Type::RoleChange(role_change_msg) => {
                if role == NodeRole::Master {
                    self.state.change_role(role_change_msg, addr);
                }
            }

            Type::Join(join_msg) => {
                if role == NodeRole::Master {
                    if join_msg.requested_role() == NodeRole::Viewer {
                        let id = self.state.add_viewer(join_msg, addr);
                        self.oneshot_send(AckMsg::new(None, Some(id), msg.msg_seq), addr);
                    } else {
                        if let Some(id) = self.state.add_normal(join_msg, addr) {
                            self.oneshot_send(AckMsg::new(None, Some(id), msg.msg_seq), addr);
                        } else {
                            unimplemented!()
                        }
                    }
                }
            }

            Type::Ping(_) => {
                // ignore
            }

            Type::Announcement(_) => {
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
                let addr = self.comm.ucast_addr();
                self.state.new_master(addr);
                self.clean_messages();
            }

            Input::Join(idx) => {
                if let Some((addr, announcement)) = self.state.nth_announcement(idx) {
                    self.clean_messages();
                    self.state.new_normal();
                    let name = self.state.player_name();
                    let join = JoinMsg::new(
                        name,
                        announcement.game_name,
                        NodeRole::Normal,
                        self.free_seq(),
                    );

                    self.send_to_addr(join, addr);
                }
            }

            Input::View(idx) => {
                if let Some((addr, announcement)) = self.state.nth_announcement(idx) {
                    self.clean_messages();
                    self.state.new_viewer();
                    let name = self.state.player_name();
                    let join = JoinMsg::new(
                        name,
                        announcement.game_name,
                        NodeRole::Normal,
                        self.free_seq(),
                    );

                    self.send_to_addr(join, addr);
                }
            }
        }
    }

    fn on_turn(&mut self, _turn: Duration) {
        let role = self.state.role();

        if role == NodeRole::Master {
            self.state.step();
            let game_state = self.state.get_game_state();
            let seq = self.free_seq();
            self.broadcast(StateMsg::new(game_state, seq));
        }
    }

    fn on_interval(&mut self, interval: Duration) {
        for msg in &self.peer_msgs {
            if msg.time.elapsed() > 8 * interval {
                // todo
            } else if msg.time.elapsed() > interval {
                self.oneshot_send(msg.msg.clone(), msg.addr);
            }
        }

        if let Some(master) = self.state.master() {
            for msg in &self.master_msgs {
                if msg.time.elapsed() > 8 * interval {
                    // todo
                } else if msg.time.elapsed() > interval {
                    self.oneshot_send(msg.msg.clone(), master);
                }
            }
        }

        self.check_dead_nodes();
    }

    fn send_to_master(&mut self, msg: GameMessage) {
        if let Some(master) = self.state.master() {
            self.comm.send_ucast(master, &msg).unwrap();
            self.master_msgs.push(MasterMessage {
                time: Instant::now(),
                msg,
            });
        }
    }

    fn send_to_addr(&mut self, msg: GameMessage, addr: SocketAddr) {
        self.comm.send_ucast(addr, &msg).unwrap();
        self.peer_msgs.push(PeerMessage {
            time: Instant::now(),
            addr,
            msg,
        });
    }

    fn oneshot_send(&self, msg: GameMessage, addr: SocketAddr) {
        self.comm.send_ucast(addr, &msg).unwrap();
    }

    fn broadcast(&mut self, msg: GameMessage) {
        for addr in self.state.get_addresses() {
            self.send_to_addr(msg.clone(), addr);
        }
    }

    fn free_seq(&mut self) -> i64 {
        self.seq_gen.next()
    }

    fn clean_messages(&mut self) {
        self.peer_msgs = Vec::new();
        self.master_msgs = Vec::new();
        self.active = HashMap::new();
    }

    fn ack(&mut self, seq: i64) {
        if let Some(idx) = self.peer_msgs.iter().position(|msg| msg.msg.msg_seq == seq) {
            self.peer_msgs.remove(idx);
        }

        if let Some(idx) = self
            .master_msgs
            .iter()
            .position(|msg| msg.msg.msg_seq == seq)
        {
            self.master_msgs.remove(idx);
        }
    }

    fn check_dead_nodes(&mut self) {
        // todo

        let role = self.state.role();

        match role {
            NodeRole::Master => {
                // todo
            }

            NodeRole::Normal => {
                // todo
            }

            NodeRole::Deputy => {
                // todo
            }

            NodeRole::Viewer => {}
        }
    }
}
