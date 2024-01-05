use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Row, Table as TUITable, TableState},
    Frame,
};
use tokio::sync::mpsc;

use crate::{action, action::Action, tui::Event};

use super::Component;

pub struct Table<'a> {
    tx: mpsc::UnboundedSender<Action>,
    pub state: TableState,
    pub rows: Vec<Row<'a>>,
    pub widths: Vec<Constraint>,
}

impl<'a> Table<'a> {
    pub fn new(tx: mpsc::UnboundedSender<Action>) -> Self {
        let state = TableState::default().with_selected(Some(0));
        let rows = vec![
            Row::new(vec!["Cell1", "Cell2"]),
            Row::new(vec!["Cell3", "Cell4"]),
        ];
        let widths = vec![Constraint::Length(5), Constraint::Length(5)];
        Self { tx, state, rows, widths }
    }
}

impl<'a> Component for Table<'a> {
    fn draw(&mut self, f: &mut Frame) {
        let area = f.size();
        let areas = Layout::new(
            Direction::Vertical,
            Constraint::from_percentages([50, 50]),
        )
        .split(area);

        f.render_stateful_widget(
            TUITable::new(self.rows.clone(), self.widths.clone())
                .block(
                    Block::default()
                        .title("ratatui table")
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                )
                .highlight_style(
                    Style::default().bg(Color::Cyan).fg(Color::Black),
                )
                .style(Style::default().fg(Color::Cyan)),
            areas[0],
            &mut self.state,
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
        let rows = self.rows.len();
        let current = self.state.selected();

        match action {
            Action::Key(k) => match k.code {
                KeyCode::Char('j') => {
                    self.state.select(current.map(|v| (v + 1) % rows))
                }
                KeyCode::Char('k') => self.state.select(current.map(|v| {
                    if v == 0 {
                        rows - 1
                    } else {
                        v - 1
                    }
                })),
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.tx.send(Action::Quit).unwrap()
                }
                KeyCode::Enter => self
                    .tx
                    .send(Action::ChangeComponent(action::Component::Counter))
                    .unwrap(),
                _ => {}
            },
            _ => {}
        }
    }
}
