use std::net::SocketAddr;

use crate::{
    proto::{game_message::*, Direction, GameMessage},
    ui::input::Input,
};

pub trait Node {
    fn handle_input(&mut self, input: Input);

    fn handle_ping(&mut self, ping: PingMsg, addr: SocketAddr, seq: i64);
    fn handle_steer(&mut self, steer: SteerMsg, addr: SocketAddr, seq: i64);
    fn handle_ack(&mut self, ack: AckMsg, addr: SocketAddr, sender: i32, receiver: i32, seq: i64);
    fn handle_state(&mut self, state: StateMsg, addr: SocketAddr, seq: i64);
    fn handle_announcement(&mut self, announcement: AnnouncementMsg, seq: i64);
    fn handle_join(&mut self, join: JoinMsg, seq: i64);
    fn handle_error(&mut self, error: ErrorMsg, seq: i64);
    fn handle_discover(&mut self, discover: DiscoverMsg, addr: SocketAddr, seq: i64);
    fn handle_role_change(
        &mut self,
        role_change: RoleChangeMsg,
        addr: SocketAddr,
        sender: i32,
        receiver: i32,
        seq: i64,
    );

    fn handle_message(&mut self, (msg, addr): (GameMessage, SocketAddr)) {
        let seq = msg.msg_seq;
        let sender = msg.sender_id();
        let receiver = msg.receiver_id();

        let Some(r#type) = msg.r#type else {
            return;
        };

        match r#type {
            Type::Ping(ping_msg) => self.handle_ping(ping_msg, addr, seq),
            Type::Steer(steer_msg) => self.handle_steer(steer_msg, addr, seq),
            Type::Ack(ack_msg) => self.handle_ack(ack_msg, addr, sender, receiver, seq),
            Type::State(state_msg) => self.handle_state(state_msg, addr, seq),
            Type::Announcement(announcement_msg) => self.handle_announcement(announcement_msg, seq),
            Type::Join(join_msg) => self.handle_join(join_msg, seq),
            Type::Error(error_msg) => self.handle_error(error_msg, seq),
            Type::RoleChange(role_change_msg) => {
                self.handle_role_change(role_change_msg, addr, sender, receiver, seq)
            }

            Type::Discover(discover_msg) => self.handle_discover(discover_msg, addr, seq),
        }
    }
}
