use crossterm::event::KeyEvent;

/// A new component to be rendered on the UI.
/// Used in conjunction with [`Action`]
#[derive(Clone, Copy)]
pub enum Page {
    Home,
    Details,
}

#[derive(Clone, Copy)]
pub enum Action {
    Tick,
    Key(KeyEvent),
    Quit,
    Render,
    None,
    /// Render another page on the UI
    ChangePage(Page),
}
