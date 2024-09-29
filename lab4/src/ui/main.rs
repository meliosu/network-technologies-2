#![allow(unused)]

use std::{fmt::format, net::SocketAddr};

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, Row, Table},
};

use crate::{
    logic::Game,
    proto::GameAnnouncement,
    state::{Player, State},
};

use super::grid::Grid;

pub fn ui(frame: &mut Frame, state: &State) {
    let constraints = if let Some(ref game) = state.game {
        [Constraint::Min(game.width as u16 + 4), Constraint::Fill(1)]
    } else {
        [Constraint::Percentage(60), Constraint::Percentage(40)]
    };

    let [left, right] = Layout::horizontal(constraints).areas(frame.area());

    let constraints = [Constraint::Fill(1), Constraint::Fill(2)];
    let [right_top, announcements] = Layout::vertical(constraints).areas(right);

    let constraints = [Constraint::Fill(2), Constraint::Fill(1)];
    let [leaderboard, info] = Layout::horizontal(constraints).areas(right_top);

    let buf = frame.buffer_mut();
    render_game(state.game.as_ref(), left, buf);
    render_announcements(&state.announcements, announcements, buf);
    render_leaderboard(&state.players, leaderboard, buf);
    render_info(state.game.as_ref(), info, buf);
}

fn default_block() -> Block<'static> {
    Block::new()
        .borders(Borders::all())
        .border_style(Style::new().cyan())
        .border_type(BorderType::Rounded)
}

fn render_announcements(
    announcements: &[(SocketAddr, GameAnnouncement)],
    area: Rect,
    buf: &mut Buffer,
) {
    let header =
        Row::new(vec!["Name", "Address", "#", "Size", "Delay", "Open"]).style(Style::new().bold());

    let widths = [
        Constraint::Fill(2),
        Constraint::Fill(2),
        Constraint::Max(3),
        Constraint::Max(7),
        Constraint::Max(7),
        Constraint::Max(6),
    ];

    let rows: Vec<_> = announcements
        .iter()
        .map(|(addr, announce)| {
            Row::new(vec![
                format!("{}", announce.game_name),
                format!("{}", announce.players.players.len()),
                format!("{}", addr),
                format!("{}x{}", announce.config.width(), announce.config.height()),
                format!("{}", announce.config.food_static()),
                format!("{}", if announce.can_join() { "✓" } else { "❌" }),
            ])
        })
        .collect();

    let table = Table::new(rows, widths)
        .block(default_block().title("Active Games"))
        .header(header);

    Widget::render(table, area, buf);
}

fn render_info(game: Option<&Game>, area: Rect, buf: &mut Buffer) {
    let block = default_block().title("Current Game");

    if let Some(game) = game {
        let items = vec![
            format!("width:  {}", game.width),
            format!("height: {}", game.height),
            format!("snakes: {}", game.snakes.len()),
        ];

        let list = List::new(items).block(block);

        Widget::render(list, area, buf);
    } else {
        Widget::render(block, area, buf);
    }
}

fn render_game(game: Option<&Game>, area: Rect, buf: &mut Buffer) {
    let block = default_block().title("Game");

    if let Some(game) = game {
        let mut grid = Grid::new(game.width, game.height);

        for snake in &game.snakes {
            for pos in &snake.body {
                grid.set_cell(pos, Color::Red);
            }
        }

        for pos in &game.food {
            grid.set_cell(pos, Color::Magenta);
        }

        let inner = block.inner(area);
        let grid_area = center_area(inner, grid.width as u16 + 2, grid.height as u16 / 2 + 2);

        let grid_block = Block::new()
            .borders(Borders::all())
            .style(Style::new().yellow())
            .border_type(BorderType::Thick);

        let inner_grid_block = grid_block.inner(grid_area);

        Widget::render(block, area, buf);
        Widget::render(grid_block, grid_area, buf);
        Widget::render(grid, inner_grid_block, buf);
    } else {
        Widget::render(block, area, buf);
    }
}

fn render_leaderboard(players: &[Player], area: Rect, buf: &mut Buffer) {
    let mut players: Vec<_> = players
        .iter()
        .map(|player| (&player.player.name, player.addr, player.player.score))
        .collect();

    players.sort_by_key(|(.., score)| *score);

    let rows: Vec<_> = players
        .iter()
        .map(|(name, addr, score)| {
            Row::new(vec![
                format!("{name}"),
                format!("{addr}"),
                format!("{score}"),
            ])
        })
        .collect();

    let widths = [Constraint::Fill(1), Constraint::Fill(1), Constraint::Max(7)];

    let header = Row::new(vec!["Name", "Address", "Score"]).style(Style::new().bold());

    let block = default_block().title("Leaderboard");

    let table = Table::new(rows, widths).block(block).header(header);

    Widget::render(table, area, buf);
}

fn center_area(area: Rect, width: u16, height: u16) -> Rect {
    let layout = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Min(width),
        Constraint::Fill(1),
    ]);

    let [_, mid, _] = layout.areas(area);

    let layout = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Min(height),
        Constraint::Fill(1),
    ]);

    let [_, target, _] = layout.areas(mid);

    target
}
