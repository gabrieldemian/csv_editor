use color_eyre::eyre::Result;
use tokio::sync::mpsc::{
    unbounded_channel, UnboundedReceiver, UnboundedSender,
};

use crate::{
    action::{self, Action},
    pages::{home::Home, Page, details::Details},
    tui::Tui,
};

pub struct App {
    should_quit: bool,
    tx: UnboundedSender<Action>,
    rx: Option<UnboundedReceiver<Action>>,
    page: Box<dyn Page>,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        let page = Box::new(Home::new(tx.clone()));
        App { should_quit: false, tx, rx: Some(rx), page }
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
            let a = self.page.get_action(e);
            tx.send(a)?;

            while let Ok(action) = rx.try_recv() {
                self.page.handle_action(action);

                if let Action::Render = action {
                    tui.draw(|f| {
                        self.page.draw(f);
                    })?;
                }

                if let Action::Quit = action {
                    self.should_quit = true;
                }

                if let Action::ChangePage(component) = action {
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
    /// receiving an [`Action::ChangePage`]
    fn handle_change_component(&mut self, page: action::Page) -> Result<()> {
        self.page = match page {
            action::Page::Home => Box::new(Home::new(self.tx.clone())),
            action::Page::Details => Box::new(Details::new(self.tx.clone())),
        };
        Ok(())
    }
}
