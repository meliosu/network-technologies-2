use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crossbeam::channel::{tick, Receiver, Sender};
use ratatui::{prelude::CrosstermBackend, Terminal};

use crate::{
    comm::Communicator,
    game::Game,
    proto::{
        game_message::{AnnouncementMsg, Type},
        GameMessage, NodeRole,
    },
    state::{Announcement, State},
    ui::{self, input::Input},
};

const SECOND: Duration = Duration::from_secs(1);

pub fn ui(state: Arc<Mutex<State>>) -> io::Result<()> {
    let mut term = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    ui::utils::set_panic_hook();
    ui::utils::setup()?;

    for _ in tick(SECOND / 50) {
        let state = state.lock().unwrap();

        if state.exited {
            break;
        }

        term.draw(|frame| ui::view::render(frame, &state))?;
    }

    ui::utils::reset_panic_hook();
    ui::utils::teardown()?;

    Ok(())
}

pub fn input(channel: Sender<Input>) -> io::Result<()> {
    loop {
        if let Some(input) = ui::input::read()? {
            channel.send(input).unwrap();
        }
    }
}

pub fn announcement_sender(state: Arc<Mutex<State>>, comm: Arc<Communicator>) -> io::Result<()> {
    for _ in tick(SECOND) {
        let announcement = {
            let state = state.lock().unwrap();

            if state.role != NodeRole::Master {
                continue;
            }

            AnnouncementMsg::new((&state.game).into(), 0)
        };

        comm.send_mcast(&announcement)?;
    }

    Ok(())
}

pub fn announcement_receiver(state: Arc<Mutex<State>>, comm: Arc<Communicator>) -> io::Result<()> {
    loop {
        let (msg, addr) = comm.recv_mcast()?;

        if let Some(Type::Announcement(announcements)) = msg.r#type {
            if let Some(announcement) = announcements.games.first() {
                let mut state = state.lock().unwrap();

                state.announcements.insert(
                    addr,
                    Announcement {
                        time: Instant::now(),
                        announcement: announcement.clone(),
                    },
                );
            }
        }
    }
}

pub fn announcement_reaper(state: Arc<Mutex<State>>) {
    for _ in tick(3 * SECOND) {
        let mut state = state.lock().unwrap();

        state
            .announcements
            .retain(|_, announcement| announcement.time.elapsed() < 3 * SECOND);
    }
}

pub fn sender(
    comm: Arc<Communicator>,
    channel: Receiver<(GameMessage, SocketAddr)>,
) -> io::Result<()> {
    loop {
        let (msg, addr) = channel.recv().unwrap();
        comm.send_ucast(addr, &msg)?;
    }
}

pub fn receiver(
    comm: Arc<Communicator>,
    channel: Sender<(GameMessage, SocketAddr)>,
) -> io::Result<()> {
    loop {
        let (msg, addr) = comm.recv_ucast()?;
        channel.send((msg, addr)).unwrap();
    }
}

pub fn master_sender(
    comm: Arc<Communicator>,
    state: Arc<Mutex<State>>,
    channel: Receiver<GameMessage>,
) -> io::Result<()> {
    Ok(())
}

pub fn main_thread(
    comm: Arc<Communicator>,
    state: Arc<Mutex<State>>,
    msg_channel: Receiver<(GameMessage, SocketAddr)>,
    input_channel: Receiver<Input>,
    master_channel: Sender<GameMessage>,
) -> io::Result<()> {
    let handle_message_master = |(msg, addr): (GameMessage, SocketAddr)| {
        if let Some(r#type) = msg.r#type {
            match r#type {
                Type::Ping(ping_msg) => {
                    let mut state = state.lock().unwrap();
                    state.active.insert(addr, Instant::now());
                }

                Type::Steer(steer_msg) => {
                    let mut state = state.lock().unwrap();

                    if let Some((id, _)) = state.game.player_by_addr(addr) {
                        if let Some(snake) = state.game.snake_by_id(id) {
                            snake.turn(steer_msg.direction());
                        }
                    }
                }

                Type::Ack(ack_msg) => todo!(),

                Type::State(state_msg) => {
                    // ignore on master
                }

                Type::Announcement(announcement_msg) => {
                    // ignore on ucast socket
                }

                Type::Join(join_msg) => {
                    let mut state = state.lock().unwrap();
                    let id = state.game.free_id();

                    if state.game.spawn_snake(id) {
                        // success

                        todo!()
                    } else {
                        // failure

                        todo!()
                    }
                }

                Type::Error(error_msg) => {
                    // ignore on master
                }

                Type::RoleChange(role_change_msg) => {
                    // ignore on alive master
                }

                Type::Discover(discover_msg) => {
                    todo!()
                }
            }
        }
    };

    let handle_input_master = |input: Input| match input {
        Input::Turn(direction) => {
            let mut state = state.lock().unwrap();
            let id = state.id;

            if let Some(snake) = state.game.snakes.iter_mut().find(|s| s.id == id) {
                snake.turn(direction);
            }
        }

        Input::Escape => {
            let mut state = state.lock().unwrap();
            state.exited = true;
        }

        Input::New => {
            let mut state = state.lock().unwrap();
            state.game = Game::from_config(&state.config);
            state.id = 0;
            state.game.spawn_snake(0);
        }

        Input::Join => {
            let mut state = state.lock().unwrap();
            *state = State::new();
            state.role = NodeRole::Normal;

            todo!();
        }

        Input::View => {
            let mut state = state.lock().unwrap();
            *state = State::new();
            state.role = NodeRole::Viewer;

            todo!();
        }
    };

    let handle_input_normal = |input: Input| match input {
        Input::Turn(direction) => {
            todo!()
        }

        Input::Escape => {
            let mut state = state.lock().unwrap();
            state.exited = true;
        }

        Input::New => {
            let mut state = state.lock().unwrap();
            state.game = Game::from_config(&state.config);
            state.id = 0;
            state.game.spawn_snake(0);
            state.role = NodeRole::Master;
        }

        Input::Join => {
            let mut state = state.lock().unwrap();
            *state = State::new();
            state.role = NodeRole::Normal;

            todo!();
        }

        Input::View => {
            let mut state = state.lock().unwrap();
            *state = State::new();
            state.role = NodeRole::Viewer;

            todo!();
        }
    };

    loop {
        let state = state.lock().unwrap();

        if state.role == NodeRole::Master {
            crossbeam::select! {
                recv(msg_channel) -> msg => handle_message_master(msg.unwrap()),
                recv(input_channel) -> input => handle_input_master(input.unwrap()),
            }
        } else {
            crossbeam::select! {
                recv(msg_channel) -> msg => handle_message_normal(msg.unwrap()),
                recv(input_channel) -> input => handle_input_normal(input.unwrap()),
            }
        }
    }
}
