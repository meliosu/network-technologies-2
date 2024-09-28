#![allow(unused)]

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders},
};

use crate::{
    logic::Game,
    proto::GameAnnouncement,
    state::{Player, State},
};

pub fn ui(frame: &mut Frame, state: &State) {
    let constraints = if let Some(ref game) = state.game {
        [Constraint::Min(game.width as u16), Constraint::Fill(1)]
    } else {
        [Constraint::Percentage(50), Constraint::Percentage(50)]
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

fn render_announcements(announcements: &[GameAnnouncement], area: Rect, buf: &mut Buffer) {
    todo!()
}

fn render_info(game: Option<&Game>, area: Rect, buf: &mut Buffer) {
    todo!()
}

fn render_game(game: Option<&Game>, area: Rect, buf: &mut Buffer) {
    todo!()
}

fn render_leaderboard(players: &[Player], area: Rect, buf: &mut Buffer) {
    todo!()
}
