use std::net::SocketAddr;

use crate::{
    logic::Game,
    proto::{GameAnnouncement, GamePlayer, NodeRole},
};

pub struct State {
    pub role: NodeRole,
    pub game: Option<Game>,
    pub players: Vec<Player>,
    pub announcements: Vec<GameAnnouncement>,
}

pub struct Player {
    pub player: GamePlayer,
    pub addr: SocketAddr,
}

impl State {
    pub fn new() -> Self {
        Self {
            role: NodeRole::Master,
            game: None,
            players: Vec::new(),
            announcements: Vec::new(),
        }
    }
}
