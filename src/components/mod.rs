pub mod counter;
pub mod input;
pub mod table;

use color_eyre::eyre::Result;
use futures::Future;
use ratatui::Frame;

use crate::tui::Event;

pub trait Component {
    fn handle_action(&mut self, action: crate::action::Action);
    fn get_action(&self, event: Event) -> crate::action::Action;
    fn draw(&mut self, f: &mut Frame);
}
