use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

use inner::Announcement;

use crate::proto::{game_message::AnnouncementMsg, GameAnnouncement, GameMessage, NodeRole};

#[derive(Clone)]
pub struct State {
    inner: Arc<Mutex<inner::State>>,
}

impl State {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(inner::State::new())),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, inner::State> {
        self.inner.lock().unwrap()
    }

    pub fn get_announcement(&self) -> Option<GameMessage> {
        let state = self.lock();

        if state.role != NodeRole::Master {
            None
        } else {
            Some(AnnouncementMsg::new((&state.game).into(), 0))
        }
    }

    pub fn add_announcement(&self, announcement: GameAnnouncement, addr: SocketAddr) {
        let mut state = self.lock();

        state.announcements.insert(
            addr,
            Announcement {
                announcement,
                time: Instant::now(),
            },
        );
    }

    pub fn remove_announcements(&self) {
        let mut state = self.lock();

        state
            .announcements
            .retain(|_, announcement| announcement.time.elapsed() < Duration::from_secs(3));
    }

    pub fn delay(&self) -> Duration {
        let state = self.lock();
        Duration::from_millis(state.game.delay as u64)
    }
}

pub mod inner {
    use std::{collections::HashMap, net::SocketAddr, time::Instant};

    use crate::{
        config::Config,
        game::Game,
        proto::{GameAnnouncement, NodeRole},
    };

    pub struct State {
        pub game: Game,
        pub config: Config,
        pub role: NodeRole,
        pub id: i32,
        pub announcements: HashMap<SocketAddr, Announcement>,
    }

    pub struct Announcement {
        pub time: Instant,
        pub announcement: GameAnnouncement,
    }

    impl State {
        pub fn new() -> Self {
            Self {
                config: Config::load("snakes.toml"),
                role: NodeRole::Master,
                id: 0,
                game: Game::from_config(&Config::load("snakes.toml")),
                announcements: HashMap::new(),
            }
        }
    }
}
