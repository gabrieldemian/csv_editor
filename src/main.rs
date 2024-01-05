#![feature(if_let_guard)]
mod tui;
mod app;
mod components;
mod action;

use app::App;
use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new();
    let result = app.run().await;

    result?;

    Ok(())
}
