use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Rect},
    style::Color,
    symbols,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc;

use crate::{action, action::Action};

use super::{Component, HandleActionResponse};

pub struct Counter {
    tx: mpsc::UnboundedSender<Action>,
    pub value: i64,
    pub focused: bool,
}

impl Counter {
    pub fn new(tx: mpsc::UnboundedSender<Action>) -> Self {
        Self { tx, value: 0, focused: false }
    }
}

impl Component for Counter {
    fn draw(&mut self, f: &mut Frame, rect: Rect) {
        f.render_widget(
            Paragraph::new(format!(
                "Press j or k to increment or decrement.\n\nCounter: {}",
                self.value,
            ))
            .block(
                Block::default()
                    .title("ratatui async counter app")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_set(symbols::border::ROUNDED),
            )
            .style(ratatui::style::Style::default().fg(Color::Cyan))
            .alignment(Alignment::Center),
            rect,
        );
    }

    fn handle_action(&mut self, action: Action) -> HandleActionResponse {
        match action {
            Action::Key(k) => match k.code {
                KeyCode::Char('j') => self.value -= 1,
                KeyCode::Char('k') => self.value += 1,
                KeyCode::Enter => self
                    .tx
                    .send(Action::ChangePage(action::Page::Details))
                    .unwrap(),
                _ => {}
            },
            _ => {}
        }
        HandleActionResponse::default()
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}
