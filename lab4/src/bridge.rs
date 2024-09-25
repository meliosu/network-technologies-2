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

impl Snake {
    // FIXME: wrapping
    pub fn anchors(&self) -> Vec<Coord> {
        let mut anchors = Vec::new();
        anchors.push(self.tail());

        for w in self.body.windows(3) {
            if (w[1].0 == w[0].0 && w[1].0 != w[2].0) || (w[1].1 == w[0].1 && w[1].1 != w[2].1) {
                anchors.push(w[1]);
            }
        }

        anchors.push(self.head());
        anchors.reverse();

        let mut result = Vec::new();

        result.push(Coord {
            x: Some(anchors[0].0 as i32),
            y: Some(anchors[0].1 as i32),
        });

        for w in anchors.windows(2) {
            let (px, py) = w[0];
            let (cx, cy) = w[1];

            let dx = cx as i32 - px as i32;
            let dy = cy as i32 - py as i32;

            result.push(Coord {
                x: if dx != 0 { Some(dx) } else { None },
                y: if dy != 0 { Some(dy) } else { None },
            });
        }

        result
    }

    // FIXME: wrapping
    pub fn body_from_anchors(anchors: Vec<Coord>) -> Vec<(usize, usize)> {
        let mut body = Vec::new();

        let (mut px, mut py) = (anchors[0].x(), anchors[0].y());

        for coord in anchors.iter().skip(1) {
            for x in px..px + coord.x() {
                body.push((x as usize, py as usize));
            }

            for y in py..py + coord.y() {
                body.push((px as usize, y as usize));
            }

            px += coord.x();
            py += coord.y();
        }

        body.push((px as usize, py as usize));
        body.reverse();

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
