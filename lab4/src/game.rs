use std::{collections::HashMap, net::SocketAddr, time::Instant};

use rand::seq::SliceRandom;

use crate::{
    config::Config,
    proto::{Direction, NodeRole},
};

pub struct Game {
    pub name: String,
    pub delay: usize,
    pub width: usize,
    pub height: usize,
    pub food_const: usize,
    pub food: Vec<(usize, usize)>,
    pub snakes: Vec<Snake>,
    pub players: HashMap<i32, Player>,
    pub turn: usize,
}

#[derive(Clone)]
pub struct Snake {
    pub id: i32,
    pub direction: Direction,
    pub body: Vec<(usize, usize)>,
}

pub struct Player {
    pub score: usize,
    pub name: String,
    pub addr: SocketAddr,
    pub role: NodeRole,
}

impl Direction {
    pub fn dxdy(&self) -> (isize, isize) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

impl Snake {
    pub fn head(&self) -> (usize, usize) {
        self.body[0]
    }

    pub fn tail(&self) -> (usize, usize) {
        *self.body.last().unwrap()
    }

    pub fn contains(&self, pos: (usize, usize)) -> bool {
        self.body.contains(&pos)
    }

    pub fn turn(&mut self, direction: Direction) {
        if direction != self.direction {
            self.direction = direction;
        }
    }
}

impl Game {
    pub fn player_by_addr(&mut self, addr: SocketAddr) -> Option<(i32, &mut Player)> {
        self.players
            .iter_mut()
            .find_map(|(id, p)| if p.addr == addr { Some((*id, p)) } else { None })
    }

    pub fn snake_by_id(&mut self, id: i32) -> Option<&mut Snake> {
        self.snakes.iter_mut().find(|s| s.id == id)
    }

    pub fn free_id(&self) -> i32 {
        (0..)
            .find(|id| {
                !self.players.contains_key(id)
                    && !self.snakes.iter().find(|s| s.id == *id).is_none()
            })
            .unwrap()
    }

    pub fn from_config(config: &Config) -> Self {
        Self {
            name: config.name.clone(),
            delay: config.delay,
            width: config.field.width,
            height: config.field.height,
            food_const: config.food,
            food: Vec::new(),
            snakes: Vec::new(),
            players: HashMap::new(),
            turn: 0,
        }
    }

    pub fn has_snake_at(&self, pos: (usize, usize)) -> bool {
        self.snakes.iter().any(|s| s.contains(pos))
    }

    pub fn has_food_at(&self, pos: (usize, usize)) -> bool {
        self.food.contains(&pos)
    }

    pub fn free_cells(&self) -> Vec<(usize, usize)> {
        let mut results = Vec::new();

        for x in 0..self.width {
            for y in 0..self.height {
                if !self.has_food_at((x, y)) && !self.has_snake_at((x, y)) {
                    results.push((x, y));
                }
            }
        }

        results
    }

    pub fn spawn_food(&mut self, count: usize) {
        self.free_cells()
            .choose_multiple(&mut rand::thread_rng(), count)
            .for_each(|pos| self.food.push(*pos));
    }

    pub fn offset(&self, x: usize, y: usize, dx: isize, dy: isize) -> (usize, usize) {
        let real_x = if x as isize + dx >= 0 {
            (x as isize + dx) as usize % self.width
        } else {
            ((self.width + x) as isize + dx) as usize
        };

        let real_y = if y as isize + dy >= 0 {
            (y as isize + dy) as usize % self.height
        } else {
            ((self.height + y) as isize + dy) as usize
        };

        (real_x, real_y)
    }

    pub fn free_spawn_points(&self) -> Vec<Snake> {
        let mut results = Vec::new();

        for x in 0..self.width {
            'outer: for y in 0..self.height {
                for dx in -2..=2 {
                    for dy in -2..=2 {
                        let (x, y) = self.offset(x, y, dx, dy);

                        if self.has_snake_at((x, y)) {
                            continue 'outer;
                        }
                    }
                }

                if self.has_food_at((x, y)) {
                    continue 'outer;
                }

                for direction in [
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ] {
                    let (dx, dy) = direction.dxdy();
                    let (tail_x, tail_y) = self.offset(x, y, dx, dy);

                    if self.has_food_at((tail_x, tail_y)) {
                        continue;
                    }

                    results.push(Snake {
                        id: 0,
                        direction: direction.opposite(),
                        body: vec![(tail_x, tail_y), (x, y)],
                    });
                }
            }
        }

        results
    }

    pub fn spawn_snake(&mut self, id: i32) -> bool {
        if let Some(snake) = self.free_spawn_points().choose(&mut rand::thread_rng()) {
            self.snakes.push(Snake {
                id,
                ..snake.clone()
            });

            true
        } else {
            false
        }
    }

    pub fn step(&mut self) {
        let mut eaten = Vec::new();

        let mut moved: Vec<Snake> = self
            .snakes
            .clone()
            .into_iter()
            .map(|mut snake| {
                let (head_x, head_y) = snake.head();
                let (dx, dy) = snake.direction.dxdy();
                let (next_x, next_y) = self.offset(head_x, head_y, dx, dy);

                snake.body.insert(0, (next_x, next_y));

                if !self.has_food_at((next_x, next_y)) {
                    snake.body.pop();
                } else {
                    eaten.push((next_x, next_y));
                }

                snake
            })
            .collect();

        let mut kills = Vec::new();

        for (i, first) in moved.iter().enumerate() {
            for second in &moved[i..] {
                if first.id == second.id {
                    if first.body.iter().filter(|&&p| p == first.head()).count() > 1 {
                        kills.push((first.id, second.id));
                    }
                } else if second.head() == first.head() {
                    kills.push((first.id, second.id));
                } else if second.body.contains(&first.head()) {
                    kills.push((first.id, second.id));
                } else if first.body.contains(&second.head()) {
                    kills.push((second.id, first.id));
                }
            }
        }

        moved.retain(|snake| {
            if kills.iter().any(|(_, id)| *id == snake.id) {
                for &pos in &snake.body {
                    if pos != snake.head() && rand::random() {
                        self.food.push(pos);
                    }
                }

                false
            } else {
                true
            }
        });

        for (killer, _) in kills {
            if let Some(player) = self.players.get_mut(&killer) {
                player.score += 1;
            }
        }

        self.snakes = moved;
        self.food.retain(|pos| !eaten.contains(pos));
        self.spawn_food((self.snakes.len() + self.food_const).saturating_sub(self.food.len()));
        self.turn += 1;
    }
}
