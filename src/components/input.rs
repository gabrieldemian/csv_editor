use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::*,
    symbols,
    text::{Line, Text},
    widgets::*,
    Frame,
};
use tokio::sync::mpsc;

use crate::{action, action::Action, tui::Event};

use super::Component;

#[derive(Default, Clone, Copy)]
enum Mode {
    #[default]
    Normal,
    Insert,
}

pub struct Input {
    value: String,
    cursor_position: usize,
    mode: Mode,
    messages: Vec<String>,
    tx: mpsc::UnboundedSender<Action>,
}

impl Input {
    pub fn new(tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            tx,
            value: "".into(),
            mode: Mode::default(),
            messages: Vec::new(),
            cursor_position: 0,
        }
    }
}

impl Input {
    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the
            // selected char. Reason: Using remove on String works
            // on bytes instead of the chars. Using remove would
            // require special care because of char boundaries.
            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete =
                self.value.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.value.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore
            // deleted.
            self.value =
                before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.value.len())
    }

    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }

    fn submit_message(&mut self) {
        self.messages.push(self.value.clone());
        self.value.clear();
        self.reset_cursor();
    }
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.value.insert(self.cursor_position, new_char);
        self.move_cursor_right();
    }
}

impl Component for Input {
    fn draw(&mut self, f: &mut Frame) {
        let area = f.size();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(3)])
            .split(area);

        let (msg, style) = match self.mode {
            Mode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "i".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            Mode::Insert => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to record the message".into(),
                ],
                Style::default(),
            ),
        };
        let mut text = Text::from(Line::from(msg));
        text.patch_style(style);
        let help_message = Paragraph::new(text);

        // render help message
        f.render_widget(help_message, chunks[0]);

        let input = Paragraph::new(self.value.as_str())
            .style(match self.mode {
                Mode::Normal => Style::default(),
                Mode::Insert => Style::default().fg(Color::Cyan),
            })
            .block(Block::default().borders(Borders::ALL).title("Input"));

        // render input
        f.render_widget(input, chunks[1]);
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
        match self.mode {
            Mode::Normal if let Action::Key(k) = action => match k.code {
                KeyCode::Char('i') => self.mode = Mode::Insert,
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.tx.send(Action::Quit).unwrap()
                }
                _ => {}
            },
            Mode::Insert if let Action::Key(k) = action => match k.code {
                KeyCode::Esc => self.mode = Mode::Normal,
                KeyCode::Char(c) => self.enter_char(c),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Enter => self.submit_message(),
                _ => {}
            },
            _ => {}
        }
    }
}
