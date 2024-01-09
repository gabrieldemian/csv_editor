use crossterm::event::KeyCode;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*, Frame};
use tokio::sync::mpsc;

use crate::{action, action::Action, utils::centered_rect};

use super::{
    input::{Input, Mode},
    Component, HandleActionResponse,
};

pub struct CsvTable<'a> {
    tx: mpsc::UnboundedSender<Action>,
    /// which cell is currently focused. (row, coll)
    pub cell_focused: (usize, usize),
    /// if the component is focused
    pub focused: bool,
    /// Matrix of rows and cells
    pub matrix: Vec<Vec<String>>,
    /// If this is Some, a popup will be rendered ontop of the current UI.
    input: Option<Input<'a>>,
    show_popup: bool,
}

impl<'a> CsvTable<'a> {
    pub fn new(tx: mpsc::UnboundedSender<Action>) -> Self {
        let matrix: Vec<Vec<String>> = include_str!("../../testdata.csv")
            .trim_end()
            .split("\n")
            .into_iter()
            .map(|r| {
                r.replace("\"", "")
                    .split(",")
                    .filter_map(|v| v.parse().ok())
                    .collect()
            })
            .collect();

        Self {
            tx,
            focused: true,
            show_popup: false,
            input: None,
            cell_focused: (0, 0),
            matrix,
        }
    }

    pub fn get_focused_cell(&mut self) -> Option<&str> {
        self.matrix
            .get(self.cell_focused.0)
            .and_then(|r| r.get(self.cell_focused.1).map(|s| s.as_str()))
    }

    pub fn get_mut_focused_cell(&mut self) -> Option<&mut String> {
        self.matrix
            .get_mut(self.cell_focused.0)
            .and_then(|r| r.get_mut(self.cell_focused.1))
    }

    pub fn get_mut_focused_cell_coordinates(
        &mut self,
    ) -> Option<(&mut usize, &mut usize)> {
        self.matrix.get_mut(self.cell_focused.0).and_then(|r| {
            r.get_mut(self.cell_focused.1).and_then(|_| {
                Some((&mut self.cell_focused.0, &mut self.cell_focused.1))
            })
        })
    }

    pub fn get_focused_cell_coordinates(&mut self) -> Option<(usize, usize)> {
        self.matrix.get(self.cell_focused.0).and_then(|r| {
            r.get(self.cell_focused.1)
                .and_then(|_| Some((self.cell_focused.0, self.cell_focused.1)))
        })
    }

    pub fn matrix(mut self, matrix: Vec<Vec<String>>) -> Self {
        self.matrix = matrix;
        self
    }
}

impl<'a> Component for CsvTable<'a> {
    fn draw(&mut self, f: &mut Frame, rect: Rect) {
        let rows = Layout::new(
            Direction::Vertical,
            vec![Constraint::Length(2); self.matrix.len()],
        )
        .split(rect);

        let mut matrix_iter = self.matrix.iter().flatten();

        for ((row_i, layout_row), cell_row) in
            rows.iter().enumerate().zip(self.matrix.iter())
        {
            let areas: Vec<Constraint> = cell_row
                .iter()
                .map(|v| {
                    let w = v.chars().count();
                    Constraint::Length(w as u16 + 1)
                })
                .collect();

            let cols = Layout::new(Direction::Horizontal, areas)
                .split(*layout_row)
                .iter()
                .copied()
                .collect_vec();

            for ((col_i, rect), cell_text) in
                cols.into_iter().enumerate().zip(&mut matrix_iter)
            {
                let (x, y) = self.cell_focused;

                // if the cell of this loop is focused
                let is_selected = x == row_i && y == col_i;

                let mut text = Paragraph::new(cell_text.clone());

                if is_selected {
                    text = text.fg(Color::Red);
                }

                f.render_widget(text, rect);
            }
        }

        if !self.show_popup {
            self.input = None;
        }

        if let Some(input) = &mut self.input {
            let block =
                Block::default().title("Editing Cell").borders(Borders::ALL);

            input.block = block;

            let area = centered_rect(60, 20, rect);

            f.render_widget(Clear, area);
            input.draw(f, area);
        }
    }

    fn handle_action(&mut self, action: Action) -> HandleActionResponse {
        let mut response = HandleActionResponse::default();

        if let Some(input) = &mut self.input {
            response = HandleActionResponse::Ignore;
            if let Action::Key(k) = action {
                if k.code == KeyCode::Enter {
                    *self.get_mut_focused_cell().unwrap() = input.value.clone();
                    self.show_popup = false;
                } else {
                    input.handle_action(action);
                }
            }
            // return response;
        }

        match action {
            Action::Key(k) => match k.code {
                KeyCode::Char('j') => {
                    if self.matrix.get(self.cell_focused.0 + 1).is_some() {
                        self.cell_focused.0 += 1;
                    }
                }
                KeyCode::Char('k') => {
                    if self
                        .matrix
                        .get(self.cell_focused.0.overflowing_sub(1).0)
                        .is_some()
                    {
                        self.cell_focused.0 -= 1;
                    }
                }
                KeyCode::Char('h') => {
                    // focus the cell on the left of the current one, if it
                    // exists
                    if let Some((_row, col)) =
                        self.get_mut_focused_cell_coordinates()
                    {
                        *col = if *col > 0 { *col - 1 } else { 0 };
                    }
                }
                KeyCode::Char('l') => {
                    // focus the cell on the right of the current one, if it
                    // exists
                    let col = &mut self.cell_focused.1;
                    if let Some(cols) = self.matrix.get_mut(self.cell_focused.0)
                    {
                        *col = (*col + 1).min(cols.len() - 1);
                    }
                }
                // open a popup to edit the cell
                KeyCode::Char('e') | KeyCode::Enter => {
                    if self.input.is_none() {
                        let input = Input::new(self.tx.clone())
                            .value(self.get_focused_cell().unwrap().to_owned())
                            .focused(true)
                            .mode(Mode::Insert);

                        self.input = Some(input);
                        self.show_popup = true;
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    if let Some(input) = &mut self.input {
                        if input.mode == Mode::Normal {
                            self.show_popup = false;
                            self.input = None;
                        }
                        response = HandleActionResponse::Ignore;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        response
    }

    fn focus(&mut self) {
        self.focused = true;
    }

    fn unfocus(&mut self) {
        self.focused = false;
    }
}
