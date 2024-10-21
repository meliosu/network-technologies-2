use ratatui::prelude::*;

pub struct Grid {
    pub cells: Vec<Vec<Color>>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![vec![Color::Black; width]; height],
        }
    }

    pub fn width(&self) -> usize {
        self.cells[0].len()
    }

    pub fn height(&self) -> usize {
        self.cells.len()
    }

    pub fn set(&mut self, (x, y): (usize, usize), color: Color) {
        self.cells[y][x] = color;
    }

    pub fn get(&self, (x, y): (usize, usize)) -> Color {
        self.cells[y][x]
    }
}

impl Widget for Grid {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if self.width() > area.width as usize || self.height() > area.height as usize * 2 {
            ratatui::widgets::Paragraph::new("Not enough cells to render field, resize screen")
                .centered()
                .red()
                .render(area, buf);
        } else {
            for row in 0..self.height() / 2 {
                for col in 0..self.width() {
                    let foreground = self.get((col, row * 2));
                    let background = self.get((col, row * 2 + 1));

                    let style = Style {
                        fg: Some(foreground),
                        bg: Some(background),
                        ..Default::default()
                    };

                    if let Some(cell) = buf.cell_mut((area.x + col as u16, area.y + row as u16)) {
                        cell.set_style(style);
                        cell.set_char('▀');
                    }
                }
            }
        }
    }
}
