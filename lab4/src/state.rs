use std::net::SocketAddr;

use crate::{
    config::Config,
    logic::Game,
    proto::{GameAnnouncement, GamePlayer, NodeRole},
};

pub struct State {
    pub exited: bool,
    pub config: Config,
    pub role: NodeRole,
    pub game: Option<Game>,
    pub players: Vec<Player>,
    pub announcements: Vec<(SocketAddr, GameAnnouncement)>,
}

pub struct Player {
    pub player: GamePlayer,
    pub addr: SocketAddr,
}

impl State {
    pub fn new() -> Self {
        Self {
            exited: false,
            config: Config::load("snakes.toml").unwrap_or_default(),
            role: NodeRole::Master,
            game: None,
            players: Vec::new(),
            announcements: Vec::new(),
        }
    }
}
