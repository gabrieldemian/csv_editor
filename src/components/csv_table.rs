use std::{fs::OpenOptions, io::Write};

use color_eyre::eyre::{eyre, Result};
use crossterm::event::KeyCode;
use itertools::Itertools;
use ratatui::{prelude::*, widgets::*, Frame};
use tokio::{spawn, sync::mpsc, io::AsyncWriteExt};

use crate::{action::Action, utils::centered_rect};

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
    edit_input: Option<Input<'a>>,
    /// this is used to make 'edit_input' into None or Some.
    show_edit_popup: bool,
    show_delete_popup: bool,
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
            show_edit_popup: false,
            show_delete_popup: false,
            edit_input: None,
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

    /// Delete the focused cell and return it's value, if the deletion happened
    /// successfully
    pub fn delete_focused_cell(&mut self) -> Result<String> {
        if let Some(row) = self.matrix.get_mut(self.cell_focused.0) {
            let r = Ok(row.remove(self.cell_focused.1));
            self.sync_file()?;
            return r;
        }
        Err(eyre!("Could not delete cell"))
    }

    /// Synchronize the struct and write all data to the file in the disk.
    pub fn sync_file(&self) -> Result<()> {
        let mut r = String::new();
        // let mut r = vec![""; self.matrix.len()];

        for row in self.matrix.iter() {
            let mut line: String =
                row.iter().map(|s| format!("\"{s}\",")).collect();

            // remote , from the last item
            line.pop();

            r.push_str(&line);
            r.push_str("\n");
        }

        spawn(async move {
            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .read(false)
                .create(true)
                .open("testdata.csv")
                .await
                .unwrap();

            let _r = file.write_all(r.as_bytes()).await.unwrap();
        });

        Ok(())
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

        if !self.show_edit_popup {
            self.edit_input = None;
        }

        if self.show_delete_popup {
            let area = centered_rect(40, 10, rect);
            f.render_widget(Clear, area);

            let text = vec![
                "Delete Cell?".into(),
                "".into(),
                Line::from(vec!["[y]es".red(), " [n]o".green()]),
            ];

            f.render_widget(
                Paragraph::new(text)
                    .block(
                        Block::default()
                            .title("Warning")
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded),
                    )
                    .alignment(Alignment::Center),
                area,
            );
        }

        if let Some(input) = &mut self.edit_input {
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

        if let Some(input) = &mut self.edit_input {
            if let Action::Key(k) = action {
                if k.code == KeyCode::Enter {
                    *self.get_mut_focused_cell().unwrap() = input.value.clone();
                    let _ = self.sync_file();
                    self.show_edit_popup = false;
                } else {
                    input.handle_action(action);
                }
            }
        }

        if self.show_delete_popup {
            if let Action::Key(k) = action {
                match k.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        self.show_delete_popup = false;
                        let _ = self.delete_focused_cell();
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        self.show_delete_popup = false;
                    }
                    _ => {}
                }
            }
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
                    if self.edit_input.is_none() {
                        let input = Input::new(self.tx.clone())
                            .value(self.get_focused_cell().unwrap().to_owned())
                            .focused(true)
                            .mode(Mode::Insert);

                        self.edit_input = Some(input);
                        self.show_edit_popup = true;
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    if self.show_delete_popup {
                        self.show_delete_popup = false;
                        response = HandleActionResponse::Ignore;
                    }
                    if let Some(input) = &mut self.edit_input {
                        if input.mode == Mode::Normal {
                            self.show_edit_popup = false;
                            self.edit_input = None;
                        }
                        response = HandleActionResponse::Ignore;
                    }
                }
                KeyCode::Char('d') => {
                    if !self.show_edit_popup
                        && self.edit_input.is_none()
                        && !self.show_delete_popup
                    {
                        self.show_delete_popup = true;
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
