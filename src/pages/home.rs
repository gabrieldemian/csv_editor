use crossterm::event::KeyCode;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc;

use crate::{
    action::Action,
    components::{csv_table::CsvTable, Component, HandleActionResponse},
    tui::Event,
};

use super::Page;

pub struct Home<'a> {
    pub layout: Layout,
    pub csv_table: CsvTable<'a>,
    /// The component from components which is being focused
    pub focused: usize,
    pub tx: mpsc::UnboundedSender<Action>,
}

impl<'a> Home<'a> {
    pub fn new(tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            layout: Layout::new(
                Direction::Vertical,
                [
                    Constraint::Length(50), // cells
                    Constraint::Length(2),  // keybindings help
                    Constraint::Min(0),     // fills remaining space
                ],
            ),
            csv_table: CsvTable::new(tx.clone()),
            focused: 0,
            tx,
        }
    }
}

impl<'a> Page for Home<'a> {
    fn draw(&mut self, f: &mut Frame) {
        let areas = self.layout.split(f.size());

        let text = vec![Line::from(vec![
            "move: ".into(),
            "hjkl".bold().blue(),
            " edit: ".into(),
            "e".bold().blue(),
            " delete: ".into(),
            "d".bold().blue(),
            " quit: ".into(),
            "q".bold().blue(),
        ])];

        f.render_widget(Paragraph::new(text), areas[1]);
        self.csv_table.draw(f, areas[0]);
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
        if let HandleActionResponse::Handle =
            self.csv_table.handle_action(action)
        {
            match action {
                Action::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        self.tx.send(Action::Quit).unwrap();
                    }
                    _ => {}
                },
                _ => {}
            };
        }
    }

    fn focus_next(&mut self) {}

    fn focus_prev(&mut self) {}
}
