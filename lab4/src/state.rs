use std::{collections::HashMap, net::SocketAddr, time::Instant};

use crate::{game::Game, proto::GameAnnouncement};

pub struct State {
    pub game: Game,
    pub announcements: HashMap<SocketAddr, Announcement>,
}

pub struct Announcement {
    pub time: Instant,
    pub announcement: GameAnnouncement,
}
