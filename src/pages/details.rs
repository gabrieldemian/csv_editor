use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::*;
use tokio::sync::mpsc;

use crate::{
    action::Action,
    components::{input::Input, table::Table, Component, HandleActionResponse},
    tui::Event,
};

use super::Page;

pub struct Details {
    pub layout: Layout,
    pub components: Vec<Box<dyn Component>>,
    /// The component from components which is being focused
    pub focused: usize,
    pub tx: mpsc::UnboundedSender<Action>,
}

impl Details {
    pub fn new(tx: mpsc::UnboundedSender<Action>) -> Self {
        let mut table = Box::new(Table::new(tx.clone()));
        let input = Box::new(Input::new(tx.clone()));

        table.focus();

        Self {
            layout: Layout::new(
                Direction::Vertical,
                Constraint::from_percentages([50, 50]),
            ),
            components: vec![table, input],
            focused: 0,
            tx,
        }
    }
}

impl Page for Details {
    fn draw(&mut self, f: &mut Frame) {
        let areas = self.layout.split(f.size());

        for (component, rect) in
            &mut self.components.iter_mut().zip(areas.into_iter())
        {
            component.draw(f, *rect);
        }
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
        if let Some(component) = self.components.get_mut(self.focused) {
            if let HandleActionResponse::Handle =
                component.handle_action(action)
            {
                match action {
                    Action::Key(KeyEvent { code, modifiers, .. }) => match code
                    {
                        KeyCode::Char('q') => {
                            self.tx.send(Action::Quit).unwrap();
                        }
                        KeyCode::Char('j')
                            if modifiers == KeyModifiers::CONTROL =>
                        {
                            self.focus_next();
                        }
                        KeyCode::Char('k')
                            if modifiers == KeyModifiers::CONTROL =>
                        {
                            self.focus_prev();
                        }
                        _ => {}
                    },
                    _ => {}
                };
            }
        }
    }

    fn focus_next(&mut self) {
        if self.focused + 1 <= self.components.len() - 1 {
            let component = self.components.get_mut(self.focused).unwrap();
            component.unfocus();

            self.focused += 1;

            let component = self.components.get_mut(self.focused).unwrap();
            component.focus();
        }
    }

    fn focus_prev(&mut self) {
        if self.focused > 0 && !self.components.is_empty() {
            let component = self.components.get_mut(self.focused).unwrap();
            component.unfocus();

            self.focused -= 1;

            let component = self.components.get_mut(self.focused).unwrap();
            component.focus();
        }
    }
}
