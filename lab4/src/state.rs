use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

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
    pub announcements: Vec<Announcement>,
}

pub struct Player {
    pub timestamp: Instant,
    pub player: GamePlayer,
    pub addr: SocketAddr,
}

pub struct Announcement {
    pub timestamp: Instant,
    pub addr: SocketAddr,
    pub announce: GameAnnouncement,
}

impl Player {
    pub fn new(addr: SocketAddr, player: GamePlayer) -> Self {
        Self {
            player,
            addr,
            timestamp: Instant::now(),
        }
    }
}

impl Announcement {
    pub fn new(addr: SocketAddr, announce: GameAnnouncement) -> Self {
        Self {
            addr,
            announce,
            timestamp: Instant::now(),
        }
    }

    pub fn refresh(&mut self) {
        self.timestamp = Instant::now();
    }

    pub fn elapsed(&self) -> Duration {
        self.timestamp.elapsed()
    }
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
