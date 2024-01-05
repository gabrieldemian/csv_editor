use crossterm::event::KeyEvent;

/// A new component to be rendered on the UI.
/// Used in conjunction with [`Action`]
#[derive(Clone, Copy)]
pub enum Component {
    Counter,
    Table,
    Input,
}

#[derive(Clone, Copy)]
pub enum Action {
    Tick,
    Key(KeyEvent),
    Quit,
    Render,
    None,
    /// Render another component on the UI
    ChangeComponent(Component),
}
