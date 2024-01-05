use crossterm::event::KeyCode;
use ratatui::{
    layout::Alignment,
    style::Color,
    symbols,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tokio::sync::mpsc;

use crate::{action, action::Action, tui::Event};

use super::Component;

pub struct Counter {
    value: i64,
    tx: mpsc::UnboundedSender<Action>,
}

impl Counter {
    pub fn new(tx: mpsc::UnboundedSender<Action>) -> Self {
        Self { tx, value: 0 }
    }
}

impl Component for Counter {
    fn draw(&mut self, f: &mut Frame) {
        let area = f.size();

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
            area,
        );
    }

    fn get_action(&self, event: Event) -> Action {
        match event {
            Event::Error => Action::None,
            Event::Tick => Action::Tick,
            Event::Render => Action::Render,
            Event::Key(key) => Action::Key(key),
            Event::Quit => Action::Quit,
            _ => Action::None,
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::Key(k) => match k.code {
                KeyCode::Char('j') => self.value -= 1,
                KeyCode::Char('k') => self.value += 1,
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.tx.send(Action::Quit).unwrap()
                }
                KeyCode::Enter => self
                    .tx
                    .send(Action::ChangeComponent(action::Component::Input))
                    .unwrap(),
                _ => {}
            },
            _ => {}
        }
    }
}
