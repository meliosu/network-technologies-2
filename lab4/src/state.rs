use std::{collections::HashMap, net::SocketAddr, time::Instant};

use crate::{
    config::Config,
    game::Game,
    proto::{GameAnnouncement, NodeRole},
};

pub struct State {
    pub config: Config,
    pub exited: bool,
    pub role: NodeRole,
    pub id: i32,
    pub game: Game,
    pub announcements: HashMap<SocketAddr, Announcement>,
    pub active: HashMap<SocketAddr, Instant>,
}

pub struct Announcement {
    pub time: Instant,
    pub announcement: GameAnnouncement,
}

impl State {
    pub fn new() -> Self {
        Self {
            config: Config::load("snakes.toml"),
            exited: false,
            role: NodeRole::Master,
            id: 0,
            game: Game::from_config(&Config::load("snakes.toml")),
            announcements: HashMap::new(),
            active: HashMap::new(),
        }
    }
}
