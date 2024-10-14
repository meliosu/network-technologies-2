use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::{Duration, Instant},
};

use crate::{
    config::Config,
    id,
    logic::Game,
    proto::{GameAnnouncement, GamePlayer, NodeRole},
};

pub struct State {
    pub msg_seq_gen: id::Generator,
    pub my_id: i32,
    pub master: SocketAddr,
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
            msg_seq_gen: id::Generator::new(),
            my_id: 0,
            master: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
            exited: false,
            config: Config::load("snakes.toml").unwrap_or_default(),
            role: NodeRole::Master,
            game: None,
            players: Vec::new(),
            announcements: Vec::new(),
        }
    }
}
