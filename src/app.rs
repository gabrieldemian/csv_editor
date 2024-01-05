use color_eyre::eyre::Result;
use tokio::sync::mpsc::{
    unbounded_channel, UnboundedReceiver, UnboundedSender,
};

use crate::{
    action::{self, Action},
    components::{counter::Counter, input::Input, table::Table, Component},
    tui::Tui,
};

pub struct App {
    should_quit: bool,
    tx: UnboundedSender<Action>,
    rx: Option<UnboundedReceiver<Action>>,
    component: Box<dyn Component>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        let component = Box::new(Counter::new(tx.clone()));
        App { should_quit: false, tx, rx: Some(rx), component }
    }

    pub async fn run(&mut self) -> Result<()> {
        // ratatui terminal
        let mut tui = Tui::new()?.tick_rate(4.0).frame_rate(60.0);
        tui.run()?;

        let tx = self.tx.clone();
        let mut rx = std::mem::take(&mut self.rx).unwrap();

        loop {
            // block until the next event
            let e = tui.next().await?;
            let a = self.component.get_action(e);
            tx.send(a)?;

            while let Ok(action) = rx.try_recv() {
                self.component.handle_action(action);

                if let Action::Render = action {
                    tui.draw(|f| {
                        self.component.draw(f);
                    })?;
                }

                if let Action::Quit = action {
                    self.should_quit = true;
                }

                if let Action::ChangeComponent(component) = action {
                    self.handle_change_component(component)?
                }
            }

            if self.should_quit {
                break;
            }
        }
        tui.exit()?;

        Ok(())
    }

    /// Handle the logic to render another component on the screen, after
    /// receiving an [`Action::ChangeComponent`]
    fn handle_change_component(
        &mut self,
        component: action::Component,
    ) -> Result<()> {
        self.component = match component {
            action::Component::Table => Box::new(Table::new(self.tx.clone())),
            action::Component::Counter => {
                Box::new(Counter::new(self.tx.clone()))
            }
            action::Component::Input => Box::new(Input::new(self.tx.clone())),
        };
        Ok(())
    }
}
