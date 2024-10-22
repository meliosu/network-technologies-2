use std::{
    net::{Ipv4Addr, SocketAddr},
    os::linux::raw::stat,
    str::FromStr,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

use inner::Announcement;
use socket2::Socket;

use crate::{
    game::Player,
    proto::{
        game_message::{AnnouncementMsg, JoinMsg, RoleChangeMsg},
        Direction, GameAnnouncement, GameMessage, GameState, NodeRole,
    },
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

    pub fn new_master(&self, addr: SocketAddr) {
        let mut state = self.lock();
        *state = inner::State::new();
        let name = state.config.nickname.clone();
        state.game.spawn_snake(0);
        state.game.players.insert(
            0,
            Player {
                score: 0,
                name,
                addr,
                role: NodeRole::Master,
            },
        );
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
        state.config.nickname.clone()
    }

    pub fn master(&self) -> Option<SocketAddr> {
        let state = self.lock();
        state.game.players.iter().find_map(|(_, p)| {
            if p.role == NodeRole::Master {
                Some(p.addr)
            } else {
                None
            }
        })
    }

    pub fn turn_snake_by_addr(&self, addr: SocketAddr, direction: Direction) {
        let mut state = self.lock();

        let Some((id, _)) = state.game.player_by_addr(addr) else {
            println!("didn't find player with addr {addr}");
            return;
        };

        let Some(snake) = state.game.snake_by_id(id) else {
            println!("didn't find snake");
            return;
        };

        snake.turn(direction);
    }

    pub fn add_normal(&self, msg: JoinMsg, addr: SocketAddr) -> Option<i32> {
        let mut state = self.lock();
        let id = state.game.free_id();

        if state.game.spawn_snake(id) {
            let role = if state
                .game
                .players
                .iter()
                .any(|(_, p)| p.role == NodeRole::Deputy)
            {
                NodeRole::Normal
            } else {
                NodeRole::Deputy
            };

            state.game.players.insert(
                id,
                Player {
                    score: 0,
                    name: msg.player_name,
                    addr,
                    role,
                },
            );

            Some(id)
        } else {
            None
        }
    }

    pub fn add_viewer(&self, msg: JoinMsg, addr: SocketAddr) -> i32 {
        let mut state = self.lock();
        let id = state.game.free_id();

        state.game.players.insert(
            id,
            Player {
                score: 0,
                name: msg.player_name,
                addr,
                role: NodeRole::Viewer,
            },
        );

        id
    }

    pub fn update(&self, state_msg: GameState, addr: SocketAddr) {
        let mut state = self.lock();
        state.game.update(state_msg);

        if let Some((_, player)) = state.game.player_by_addr(addr) {
            state.role = player.role;
        }
    }

    pub fn get_game_state(&self) -> GameState {
        let state = self.lock();
        GameState::from(&state.game)
    }

    pub fn get_addresses(&self) -> Vec<SocketAddr> {
        let state = self.lock();
        state
            .game
            .players
            .values()
            .filter_map(|p| {
                if p.role != NodeRole::Master {
                    Some(p.addr)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn step(&self) {
        let mut state = self.lock();
        state.game.step();
    }

    pub fn deputy(&self) -> Option<SocketAddr> {
        let state = self.lock();

        state.game.players.iter().find_map(|(_, player)| {
            if player.role == NodeRole::Deputy {
                Some(player.addr)
            } else {
                None
            }
        })
    }

    pub fn choose_deputy(&self) -> Option<SocketAddr> {
        let mut state = self.lock();

        state.game.players.iter_mut().find_map(|(_id, player)| {
            if player.role == NodeRole::Viewer || player.role == NodeRole::Normal {
                player.role = NodeRole::Deputy;
                Some(player.addr)
            } else {
                None
            }
        })
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
