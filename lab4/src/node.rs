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
    active: HashMap<SocketAddr, (Instant, Instant)>, // send, receive
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
        let receiver_id = msg.receiver_id();

        let Some(r#type) = msg.r#type else {
            return;
        };

        if let Some(active) = self.active.get_mut(&addr) {
            active.1 = Instant::now();
        } else {
            self.active.insert(addr, (Instant::now(), Instant::now()));
        }

        let role = self.state.role();

        match r#type {
            Type::Steer(steer_msg) => {
                if role == NodeRole::Master {
                    self.state.turn_snake_by_addr(addr, steer_msg.direction());
                    self.oneshot_send(AckMsg::new(None, None, msg.msg_seq), addr);
                }
            }

            Type::Ack(_) => {
                if role != NodeRole::Master {
                    if receiver_id != 0 {
                        let mut state = self.state.lock();
                        state.id = receiver_id;
                    }
                }

                self.ack(msg.msg_seq);
            }

            Type::State(state_msg) => {
                if role != NodeRole::Master {
                    {
                        let mut state = self.state.lock();

                        if state.master.is_some_and(|m| m == addr) || state.master.is_none() {
                            state.master = Some(addr);
                        }
                    }

                    self.state.update(state_msg.state, self.comm.ucast_addr());
                }
            }

            Type::RoleChange(role_change_msg) => {
                if role == NodeRole::Master {
                    self.state.change_role(role_change_msg, addr);
                    self.send_to_master(AckMsg::new(None, None, msg.msg_seq));
                }
            }

            Type::Join(join_msg) => {
                if role == NodeRole::Master {
                    if join_msg.requested_role() == NodeRole::Viewer {
                        let id = self.state.add_viewer(join_msg, addr);
                        self.oneshot_send(AckMsg::new(None, Some(id), msg.msg_seq), addr);
                    } else {
                        let Some(id) = self.state.add_normal(join_msg, addr) else {
                            unimplemented!()
                        };

                        self.oneshot_send(AckMsg::new(None, Some(id), msg.msg_seq), addr);
                    }
                }
            }

            Type::Ping(_) => {}
            Type::Announcement(_) => {}
            Type::Error(_) => {}
            Type::Discover(_) => {}
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
                    self.state.new_normal(announcement.clone());
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
                    self.state.new_viewer(announcement.clone());
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
            //if msg.time.elapsed() > interval * 8 {
            //    continue;
            //}
            //
            //if msg.time.elapsed() > interval {
            //    self.oneshot_send(msg.msg.clone(), msg.addr);
            //}
        }

        if let Some(master) = self.state.master() {
            for msg in &self.master_msgs {
                //if msg.time.elapsed() > interval * 8 {
                //    continue;
                //}
                //
                //if msg.time.elapsed() > interval {
                //    self.oneshot_send(msg.msg.clone(), master);
                //}
            }
        }

        let to_ping: Vec<_> = self
            .active
            .iter()
            .filter_map(|(addr, (send, _))| (send.elapsed() > interval / 10).then_some(*addr))
            .collect();

        for addr in to_ping {
            let seq = self.free_seq();
            self.send_to_addr(PingMsg::new(seq), addr);
        }

        self.check_dead_nodes(interval * 8);
    }

    fn send_to_master(&mut self, mut msg: GameMessage) {
        msg.sender_id = Some(self.state.id());

        if let Some(master) = self.state.master() {
            self.comm.send_ucast(master, &msg).unwrap();
            self.master_msgs.push(MasterMessage {
                time: Instant::now(),
                msg,
            });
        }
    }

    fn send_to_addr(&mut self, mut msg: GameMessage, addr: SocketAddr) {
        msg.sender_id = Some(self.state.id());

        self.comm.send_ucast(addr, &msg).unwrap();
        self.peer_msgs.push(PeerMessage {
            time: Instant::now(),
            addr,
            msg,
        });
    }

    fn oneshot_send(&self, mut msg: GameMessage, addr: SocketAddr) {
        msg.sender_id = Some(self.state.id());
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

    fn check_dead_nodes(&mut self, interval: Duration) {
        let role = self.state.role();

        let mut master_dead = false;

        for (addr, (send, receive)) in &self.active {
            if receive.elapsed() > interval {
                let mut state = self.state.lock();

                if state.master.is_some_and(|a| a == *addr) {
                    state.master = None;
                    master_dead = true;
                }

                if let Some(id) = state.game.players.iter().find_map(|(id, player)| {
                    if player.addr == *addr {
                        Some(*id)
                    } else {
                        None
                    }
                }) {
                    state.game.players.remove(&id);
                }

                if let Some((_, player)) = state.game.player_by_addr(*addr) {
                    player.role = NodeRole::Viewer;
                }
            }
        }

        self.active
            .retain(|_, (send, receive)| receive.elapsed() < interval);

        match role {
            NodeRole::Master => {
                if self.state.deputy().is_none() {
                    if let Some(deputy_addr) = self.state.choose_deputy() {
                        let seq = self.free_seq();

                        self.send_to_addr(
                            RoleChangeMsg::new(0, None, 0, Some(NodeRole::Deputy), seq),
                            deputy_addr,
                        );
                    }
                }
            }

            NodeRole::Deputy => {
                if master_dead {
                    let mut state = self.state.lock();
                    state.role = NodeRole::Master;

                    if let Some((_, player)) = state.game.player_by_addr(self.comm.ucast_addr()) {
                        player.role = NodeRole::Master;
                    }

                    return;
                }

                let Some(master) = self.state.master() else {
                    let mut state = self.state.lock();
                    state.role = NodeRole::Master;

                    if let Some((_, player)) = state.game.player_by_addr(self.comm.ucast_addr()) {
                        player.role = NodeRole::Master;
                    }

                    return;
                };

                if self
                    .active
                    .get(&master)
                    .is_some_and(|(send, receive)| receive.elapsed() > interval)
                {
                    let mut state = self.state.lock();
                    state.role = NodeRole::Master;
                }
            }

            NodeRole::Normal => {}
            NodeRole::Viewer => {}
        }
    }
}
