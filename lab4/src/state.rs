use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

use inner::Announcement;

use crate::proto::{
    game_message::AnnouncementMsg, Direction, GameAnnouncement, GameMessage, NodeRole,
};

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

    pub fn role(&self) -> NodeRole {
        let state = self.lock();
        state.role
    }

    pub fn turn_self(&self, direction: Direction) {
        let mut state = self.lock();
        let id = state.id;

        if let Some(snake) = state.game.snake_by_id(id) {
            snake.turn(direction);
        }
    }

    pub fn exit(&self) {
        let mut state = self.lock();
        state.exited = true;
    }

    pub fn is_exited(&self) -> bool {
        let state = self.lock();
        state.exited
    }

    pub fn nth_announcement(&self, n: usize) -> Option<(SocketAddr, GameAnnouncement)> {
        let state = self.lock();

        state
            .announcements
            .iter()
            .nth(n)
            .map(|(addr, announcement)| (*addr, announcement.announcement.clone()))
    }

    pub fn new_master(&self) {
        let mut state = self.lock();
        *state = inner::State::new();
        state.game.spawn_snake(0);
    }

    pub fn new_normal(&self) {
        let mut state = self.lock();
        *state = inner::State::new();
        state.role = NodeRole::Normal;
    }

    pub fn new_viewer(&self) {
        let mut state = self.lock();
        *state = inner::State::new();
        state.role = NodeRole::Viewer;
    }

    pub fn player_name(&self) -> String {
        let state = self.lock();
        state.config.name.clone()
    }

    pub fn master(&self) -> SocketAddr {
        let state = self.lock();
        state
            .game
            .players
            .iter()
            .find_map(|(_, p)| {
                if p.role == NodeRole::Master {
                    Some(p.addr)
                } else {
                    None
                }
            })
            .unwrap()
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
        pub exited: bool,
        pub game: Game,
        pub config: Config,
        pub role: NodeRole,
        pub id: i32,
        pub announcements: HashMap<SocketAddr, Announcement>,
    }

    #[derive(Clone)]
    pub struct Announcement {
        pub time: Instant,
        pub announcement: GameAnnouncement,
    }

    impl State {
        pub fn new() -> Self {
            Self {
                exited: false,
                config: Config::load("snakes.toml"),
                role: NodeRole::Master,
                id: 0,
                game: Game::from_config(&Config::load("snakes.toml")),
                announcements: HashMap::new(),
            }
        }
    }
}
