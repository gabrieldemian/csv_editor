use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Row, Table as TUITable, TableState},
    Frame,
};
use tokio::sync::mpsc;

use crate::{action, action::Action};

use super::{Component, HandleActionResponse};

pub struct Table<'a> {
    tx: mpsc::UnboundedSender<Action>,
    pub focused: bool,
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
        Self { tx, state, rows, widths, focused: false }
    }
}

impl<'a> Component for Table<'a> {
    fn draw(&mut self, f: &mut Frame, rect: Rect) {
        let mut border_style = Style::default().fg(Color::Gray);
        let mut highlight_style = Style::default();

        if self.focused {
            border_style = border_style.fg(Color::Cyan);
            highlight_style = highlight_style.fg(Color::Black).bg(Color::Cyan);
        }

        f.render_stateful_widget(
            TUITable::new(self.rows.clone(), self.widths.clone())
                .block(
                    Block::default()
                        .title("ratatui table")
                        .title_alignment(Alignment::Center)
                        .borders(Borders::ALL)
                        .border_style(border_style)
                        .border_type(BorderType::Rounded),
                )
                .highlight_style(highlight_style)
                .style(Style::default().fg(Color::Cyan)),
            rect,
            &mut self.state,
        );
    }

    fn handle_action(&mut self, action: Action) -> HandleActionResponse {
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
                KeyCode::Enter => self
                    .tx
                    .send(Action::ChangePage(action::Page::Home))
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
