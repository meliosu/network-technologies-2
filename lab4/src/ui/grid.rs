#![allow(dead_code)]

use ratatui::{prelude::*, widgets::Paragraph};

pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<Vec<Color>>,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![vec![Color::Black; width]; height],
        }
    }

    pub fn set_cell(&mut self, (x, y): &(usize, usize), color: Color) {
        self.cells[*y][*x] = color;
    }
}

impl Widget for Grid {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if self.width > area.width as usize || self.height > area.height as usize * 2 {
            Paragraph::new("Not enough cells to render field\nPlease resize screen")
                .centered()
                .red()
                .render(area, buf);

            return;
        }

        for row in 0..self.height / 2 {
            for col in 0..self.width {
                let foreground = self.cells[row * 2][col];
                let background = self.cells[row * 2 + 1][col];

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
