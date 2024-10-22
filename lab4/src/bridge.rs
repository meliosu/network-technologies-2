use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

use crate::{
    game::{Game, Player, Snake},
    proto::{
        game_state::{snake::SnakeState, Coord, Snake as ProtoSnake},
        GameAnnouncement, GameConfig, GamePlayer, GamePlayers, GameState, NodeRole, PlayerType,
    },
};

impl From<(i32, i32)> for Coord {
    fn from((x, y): (i32, i32)) -> Self {
        Self {
            x: (x != 0).then_some(x),
            y: (y != 0).then_some(y),
        }
    }
}

impl From<(usize, usize)> for Coord {
    fn from((x, y): (usize, usize)) -> Self {
        (x as i32, y as i32).into()
    }
}

impl Snake {
    pub fn body_to_anchors(&self) -> Vec<Coord> {
        let mut shifts: Vec<(i32, i32)> = Vec::new();

        for w in self.body.windows(2) {
            let (x1, y1) = w[0];
            let (x2, y2) = w[1];

            let temp = ((x2 as i32 - x1 as i32), (y2 as i32 - y1 as i32));

            let shift = match temp {
                (1, 0) | (0, 1) | (-1, 0) | (0, -1) => temp,
                (dx, 0) if dx > 0 => (-1, 0),
                (dx, 0) if dx < 0 => (1, 0),
                (0, dy) if dy > 0 => (0, -1),
                (0, dy) if dy < 0 => (0, 1),
                _ => unreachable!(),
            };

            shifts.push(shift);
        }

        let mut anchors = Vec::new();

        let (head_x, head_y) = self.head();
        anchors.push(Coord {
            x: Some(head_x as i32),
            y: Some(head_y as i32),
        });

        let mut dx = 0;
        let mut dy = 0;

        for shift in shifts.iter() {
            if dx == 0 && dy == 0 {
                dx = shift.0;
                dy = shift.1;
                continue;
            }

            if dx == 0 && shift.0 != 0 || dy == 0 && shift.1 != 0 {
                anchors.push((dx, dy).into());
                dx = 0;
                dy = 0;
            }

            dx += shift.0;
            dy += shift.1;
        }

        if dx != 0 || dy != 0 {
            anchors.push((dx, dy).into());
        }

        anchors
    }

    pub fn body_from_anchors(
        anchors: Vec<Coord>,
        width: usize,
        height: usize,
    ) -> Vec<(usize, usize)> {
        let mut body = Vec::new();

        let (mut x, mut y) = (anchors[0].x() as usize, anchors[0].y() as usize);

        body.push((x, y));

        for shift in anchors.iter().skip(1) {
            let (cx, cy) = (shift.x(), shift.y());

            if cx != 0 {
                if cx > 0 {
                    for _ in 0..cx {
                        if x == width - 1 {
                            x = 0;
                        } else {
                            x += 1;
                        }

                        body.push((x, y));
                    }
                } else {
                    for _ in 0..-cx {
                        if x == 0 {
                            x = width - 1;
                        } else {
                            x -= 1;
                        }

                        body.push((x, y));
                    }
                }
            } else {
                if cy > 0 {
                    for _ in 0..cy {
                        if y == height - 1 {
                            y = 0;
                        } else {
                            y += 1;
                        }

                        body.push((x, y));
                    }
                } else {
                    for _ in 0..-cy {
                        if y == 0 {
                            y = height - 1;
                        } else {
                            y -= 1;
                        }

                        body.push((x, y));
                    }
                }
            }
        }

        body
    }
}

impl Game {
    pub fn update(&mut self, state: GameState) {
        self.turn = state.state_order as usize;
        self.food = state
            .foods
            .into_iter()
            .map(|coord| (coord.x() as usize, coord.y() as usize))
            .collect();
        self.snakes = state
            .snakes
            .into_iter()
            .map(|snake| Snake {
                id: snake.player_id,
                direction: snake.head_direction(),
                body: Snake::body_from_anchors(snake.points, self.width, self.height),
            })
            .collect();
        self.players = state
            .players
            .players
            .into_iter()
            .map(|player| {
                (
                    player.id,
                    Player {
                        score: player.score as usize,
                        name: player.name.clone(),
                        addr: SocketAddr::from_str(&format!(
                            "{}:{}",
                            player.ip_address(),
                            player.port()
                        ))
                        .unwrap_or(SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0)),
                        role: player.role(),
                    },
                )
            })
            .collect();
    }
}

impl From<&Game> for GameConfig {
    fn from(game: &Game) -> Self {
        Self {
            width: Some(game.width as i32),
            height: Some(game.height as i32),
            food_static: Some(game.food_const as i32),
            state_delay_ms: Some(game.delay as i32),
        }
    }
}

impl From<&Game> for GamePlayers {
    fn from(game: &Game) -> Self {
        Self {
            players: game
                .players
                .iter()
                .map(|(id, player)| GamePlayer {
                    name: player.name.clone(),
                    id: *id,
                    ip_address: Some(player.addr.ip().to_string()),
                    port: Some(player.addr.port() as i32),
                    role: player.role.into(),
                    r#type: Some(PlayerType::Human.into()),
                    score: player.score as i32,
                })
                .collect(),
        }
    }
}

impl From<&Game> for GameState {
    fn from(game: &Game) -> Self {
        Self {
            state_order: game.turn as i32,
            snakes: game
                .snakes
                .iter()
                .map(|snake| ProtoSnake {
                    player_id: snake.id,
                    points: snake.body_to_anchors(),
                    state: if game
                        .players
                        .get(&snake.id)
                        .is_some_and(|p| p.role != NodeRole::Viewer)
                    {
                        SnakeState::Alive.into()
                    } else {
                        SnakeState::Zombie.into()
                    },
                    head_direction: snake.direction.into(),
                })
                .collect(),
            foods: game.food.iter().map(|&p| p.into()).collect(),
            players: game.into(),
        }
    }
}

impl From<&Game> for GameAnnouncement {
    fn from(game: &Game) -> Self {
        Self {
            players: game.into(),
            config: game.into(),
            can_join: Some(true),
            game_name: game.name.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::proto::Direction;

    use super::*;

    #[test]
    fn body_from_acnhors() {
        let bodies = [
            vec![(0, 0), (0, 1)],
            vec![(0, 0), (1, 0)],
            vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
            vec![(0, 0), (0, 1), (1, 1), (1, 2), (2, 2)],
            vec![(0, 0), (0, 1), (1, 1), (1, 0)],
        ];

        for body in bodies {
            let snake = Snake {
                body: body.clone(),
                direction: Direction::Down,
                id: 0,
            };

            let anchors = snake.body_to_anchors();

            dbg!(&anchors);

            let new_body = Snake::body_from_anchors(anchors, 100, 100);

            assert_eq!(body.clone(), new_body);
        }
    }
}
