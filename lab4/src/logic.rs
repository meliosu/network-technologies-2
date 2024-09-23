#![allow(dead_code)]

use std::collections::HashMap;

use rand::seq::{IteratorRandom, SliceRandom};

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
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

#[derive(Debug, Clone, Copy)]
pub struct Ends {
    head: (usize, usize),
    tail: (usize, usize),
}

#[derive(Debug)]
pub struct Game {
    width: usize,
    height: usize,
    ends: HashMap<i32, Ends>,
    cells: Vec<Vec<Cell>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Cell {
    Empty,
    Food,
    Snake { id: i32, direction: Direction },
}

impl Cell {
    pub fn is_empty(&self) -> bool {
        match self {
            Cell::Empty => true,
            _ => false,
        }
    }

    pub fn is_snake(&self) -> bool {
        match self {
            Cell::Snake { .. } => true,
            _ => false,
        }
    }

    pub fn is_food(&self) -> bool {
        match self {
            Cell::Food => true,
            _ => false,
        }
    }
}

impl Game {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            ends: HashMap::new(),
            cells: vec![vec![Cell::Empty; width]; height],
        }
    }

    pub fn free_cells(&mut self) -> Vec<&mut Cell> {
        let mut result = Vec::new();

        for row in self.cells.iter_mut() {
            for cell in row.iter_mut() {
                if cell.is_empty() {
                    result.push(cell);
                }
            }
        }

        result
    }

    pub fn spawn_food(&mut self, count: usize) {
        let mut free = self.free_cells();

        for idx in (0..free.len()).choose_multiple(&mut rand::thread_rng(), count) {
            *free[idx] = Cell::Food;
        }
    }

    pub fn free_spawn_positions(&self) -> Vec<((usize, usize), Direction)> {
        let mut result = Vec::new();

        for x in 2..self.width - 2 {
            'outer: for y in 2..self.height - 2 {
                for cx in x - 2..x + 2 {
                    for cy in y - 2..y + 2 {
                        if self.cells[cy][cx].is_snake() {
                            continue 'outer;
                        }
                    }
                }

                if !self.cells[y][x].is_empty() {
                    continue 'outer;
                }

                if !self.cells[y - 1][x].is_empty() {
                    result.push(((x, y), Direction::Down));
                }

                if !self.cells[y + 1][x].is_empty() {
                    result.push(((x, y), Direction::Up));
                }

                if !self.cells[y][x - 1].is_empty() {
                    result.push(((x, y), Direction::Right));
                }

                if !self.cells[y][x + 1].is_empty() {
                    result.push(((x, y), Direction::Left));
                }
            }
        }

        result
    }

    pub fn spawn_snake(&mut self, id: i32) -> bool {
        let free = self.free_spawn_positions();

        let Some(&((x, y), direction)) = free.choose(&mut rand::thread_rng()) else {
            return false;
        };

        self.cells[y][x] = Cell::Snake { id, direction };

        let (dx, dy) = direction.opposite().dxdy();
        let (x, y) = self.offset(x, y, dx, dy);

        self.cells[y][x] = Cell::Snake { id, direction };
        true
    }

    pub fn offset(&self, x: usize, y: usize, dx: isize, dy: isize) -> (usize, usize) {
        let real_x = if x as isize + dx > 0 {
            (x as isize + dx) as usize % self.width
        } else {
            ((self.width + x) as isize + dx) as usize
        };

        let real_y = if y as isize + dy > 0 {
            (y as isize + dy) as usize % self.height
        } else {
            ((self.width + y) as isize + dy) as usize
        };

        (real_x, real_y)
    }

    pub fn step(&mut self) {}
}
