use crate::config::Config;
use crate::logic::Snake;
use crate::proto::game_state::Coord;
use crate::proto::GameConfig;

impl From<(usize, usize)> for Coord {
    fn from((x, y): (usize, usize)) -> Self {
        Self {
            x: Some(x as i32),
            y: Some(y as i32),
        }
    }
}

impl From<(i32, i32)> for Coord {
    fn from((x, y): (i32, i32)) -> Self {
        Self {
            x: (x != 0).then_some(x),
            y: (y != 0).then_some(y),
        }
    }
}

impl Snake {
    pub fn anchors(&self) -> Vec<Coord> {
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
        let head = self.head();
        anchors.push(head.into());

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
                for _ in 0..cx {
                    if cx > 0 {
                        if x == width - 1 {
                            x = 0;
                        } else {
                            x += 1;
                        }
                    } else {
                        if x == 0 {
                            x = width - 1;
                        } else {
                            x -= 1;
                        }
                    }

                    body.push((x, y));
                }
            } else {
                for _ in 0..cy {
                    if cy > 0 {
                        if y == height - 1 {
                            y = 0;
                        } else {
                            y += 1;
                        }
                    } else {
                        if y == 0 {
                            y = height - 1;
                        } else {
                            y -= 1;
                        }
                    }

                    body.push((x, y));
                }
            }
        }

        body
    }
}

impl From<&Config> for GameConfig {
    fn from(config: &Config) -> Self {
        Self {
            width: Some(config.field.width as i32),
            height: Some(config.field.height as i32),
            food_static: Some(config.food as i32),
            state_delay_ms: Some(config.delay as i32),
        }
    }
}
