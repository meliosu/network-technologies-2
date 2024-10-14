use std::{collections::HashMap, net::SocketAddr, time::Instant};

use crate::{
    config::Config,
    game::Game,
    proto::{GameAnnouncement, NodeRole},
};

pub struct State {
    pub role: NodeRole,
    pub game: Game,
    pub announcements: HashMap<SocketAddr, Announcement>,
}

pub struct Announcement {
    pub time: Instant,
    pub announcement: GameAnnouncement,
}

impl State {
    pub fn new() -> Self {
        Self {
            role: NodeRole::Master,
            game: Game::from_config(&Config::load("snakes.toml")),
            announcements: HashMap::new(),
        }
    }
}
